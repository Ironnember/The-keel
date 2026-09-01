use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use pulpo_retention_v0::{
    authorization_object_hash, build_app, sha256_hex, AppState, DeletionAuthorization,
};
use serde_json::{json, Value};
use tower::ServiceExt;

const NOW: u64 = 1_800_000_000;

fn fixture() -> (AppState, DeletionAuthorization) {
    let state = AppState::new(|| NOW);
    let evidence = b"sensitive-evidence-v0".to_vec();
    let evidence_hash = sha256_hex(&evidence);
    state.seed_evidence("ev_123", evidence);

    let mut auth = DeletionAuthorization {
        permit_id: "permit_delete_123".to_string(),
        actor_id: "executor:keel-retention-v0".to_string(),
        action: "delete_evidence".to_string(),
        evidence_id: "ev_123".to_string(),
        evidence_hash,
        policy_hash: "policy_hash_abc".to_string(),
        expires_at_unix: NOW + 60,
        authority_effect: "none".to_string(),
        object_hash: String::new(),
    };
    auth.object_hash = authorization_object_hash(&auth);
    (state, auth)
}

async fn post_json(app: axum::Router, uri: &str, payload: Value) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap(),
    )
    .await
    .unwrap()
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn valid_exact_deletion_removes_evidence_and_returns_nonreconciled_manifest() {
    let (state, auth) = fixture();
    let response = post_json(
        build_app(state.clone()),
        "/evidence/ev_123/retention/delete",
        json!({"authorization": auth}),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(
        body["custody_events"],
        json!(["DeletionRequested", "DeletionExecuted"])
    );
    assert_eq!(body["reconciled"], false);
    assert_eq!(body["authority_effect"], "none");
    assert_eq!(body["merkle_root"].as_str().unwrap().len(), 64);
    assert!(!state.contains_evidence("ev_123"));
}

#[tokio::test]
async fn evidence_id_substitution_fails_closed_and_preserves_bytes() {
    let (state, auth) = fixture();
    let response = post_json(
        build_app(state.clone()),
        "/evidence/ev_attacker/retention/delete",
        json!({"authorization": auth}),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert!(state.contains_evidence("ev_123"));
}

#[tokio::test]
async fn evidence_hash_substitution_fails_closed_and_preserves_bytes() {
    let (state, mut auth) = fixture();
    auth.evidence_hash = "0".repeat(64);
    auth.object_hash = authorization_object_hash(&auth);

    let response = post_json(
        build_app(state.clone()),
        "/evidence/ev_123/retention/delete",
        json!({"authorization": auth}),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert!(state.contains_evidence("ev_123"));
}

#[tokio::test]
async fn authorization_object_hash_substitution_fails_closed_and_preserves_bytes() {
    let (state, mut auth) = fixture();
    auth.actor_id = "executor:attacker".to_string();
    // Deliberately retain the original object hash after actor substitution.

    let response = post_json(
        build_app(state.clone()),
        "/evidence/ev_123/retention/delete",
        json!({"authorization": auth}),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert!(state.contains_evidence("ev_123"));
}

#[tokio::test]
async fn expired_authorization_fails_closed_and_preserves_bytes() {
    let (state, mut auth) = fixture();
    auth.expires_at_unix = NOW - 1;
    auth.object_hash = authorization_object_hash(&auth);

    let response = post_json(
        build_app(state.clone()),
        "/evidence/ev_123/retention/delete",
        json!({"authorization": auth}),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(state.contains_evidence("ev_123"));
}

#[tokio::test]
async fn authority_expansion_is_denied() {
    let (state, mut auth) = fixture();
    auth.authority_effect = "expand".to_string();
    auth.object_hash = authorization_object_hash(&auth);

    let response = post_json(
        build_app(state.clone()),
        "/evidence/ev_123/retention/delete",
        json!({"authorization": auth}),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(state.contains_evidence("ev_123"));
}

#[tokio::test]
async fn replay_cannot_execute_a_second_deletion() {
    let (state, auth) = fixture();
    let payload = json!({"authorization": auth});

    let first = post_json(
        build_app(state.clone()),
        "/evidence/ev_123/retention/delete",
        payload.clone(),
    )
    .await;
    assert_eq!(first.status(), StatusCode::OK);

    // Re-seeding the same bytes simulates a hostile attempt to replay the spent
    // permit against a newly reappearing object. Permit consumption still wins.
    state.seed_evidence("ev_123", b"sensitive-evidence-v0".to_vec());
    let second = post_json(
        build_app(state.clone()),
        "/evidence/ev_123/retention/delete",
        payload,
    )
    .await;
    assert_eq!(second.status(), StatusCode::CONFLICT);
    assert!(state.contains_evidence("ev_123"));
}

#[tokio::test]
async fn check_endpoint_is_nonmutating_and_requires_exact_authorization() {
    let (state, auth) = fixture();
    let response = post_json(
        build_app(state.clone()),
        "/evidence/ev_123/retention/check",
        json!({"authorization": auth}),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["exact_evidence_present"], true);
    assert_eq!(body["governed_effect"], "evidence_deletion");
    assert!(state.contains_evidence("ev_123"));
}
