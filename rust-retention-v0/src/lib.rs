use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<InnerState>>,
    clock: Arc<dyn Fn() -> u64 + Send + Sync>,
}

#[derive(Default)]
struct InnerState {
    evidence: HashMap<String, Vec<u8>>,
    spent_permits: HashSet<String>,
}

impl AppState {
    pub fn new(clock: impl Fn() -> u64 + Send + Sync + 'static) -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerState::default())),
            clock: Arc::new(clock),
        }
    }

    pub fn seed_evidence(&self, evidence_id: impl Into<String>, bytes: Vec<u8>) {
        self.inner
            .lock()
            .expect("retention state lock poisoned")
            .evidence
            .insert(evidence_id.into(), bytes);
    }

    pub fn contains_evidence(&self, evidence_id: &str) -> bool {
        self.inner
            .lock()
            .expect("retention state lock poisoned")
            .evidence
            .contains_key(evidence_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionAuthorization {
    pub permit_id: String,
    pub actor_id: String,
    pub action: String,
    pub evidence_id: String,
    pub evidence_hash: String,
    pub policy_hash: String,
    pub expires_at_unix: u64,
    pub authority_effect: String,
    pub object_hash: String,
}

#[derive(Debug, Serialize)]
struct AuthorizationBinding<'a> {
    permit_id: &'a str,
    actor_id: &'a str,
    action: &'a str,
    evidence_id: &'a str,
    evidence_hash: &'a str,
    policy_hash: &'a str,
    expires_at_unix: u64,
    authority_effect: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct DeletionRequest {
    pub authorization: DeletionAuthorization,
}

#[derive(Debug, Serialize)]
pub struct RetentionCheck {
    pub evidence_id: String,
    pub permit_id: String,
    pub authorization_object_hash: String,
    pub exact_evidence_present: bool,
    pub governed_effect: &'static str,
    pub authority_effect: &'static str,
}

#[derive(Debug, Serialize)]
pub struct DeletionManifest {
    pub manifest_id: String,
    pub permit_id: String,
    pub evidence_id: String,
    pub evidence_hash: String,
    pub policy_hash: String,
    pub authorization_object_hash: String,
    pub deletion_requested_hash: String,
    pub deletion_executed_hash: String,
    pub merkle_root: String,
    pub custody_events: Vec<&'static str>,
    pub reconciled: bool,
    pub authority_effect: &'static str,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
}

impl ApiError {
    fn bad_request(code: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
        }
    }

    fn forbidden(code: &'static str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code,
        }
    }

    fn conflict(code: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    authority_effect: &'static str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.code,
                authority_effect: "none",
            }),
        )
            .into_response()
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn authorization_object_hash(auth: &DeletionAuthorization) -> String {
    let binding = AuthorizationBinding {
        permit_id: &auth.permit_id,
        actor_id: &auth.actor_id,
        action: &auth.action,
        evidence_id: &auth.evidence_id,
        evidence_hash: &auth.evidence_hash,
        policy_hash: &auth.policy_hash,
        expires_at_unix: auth.expires_at_unix,
        authority_effect: &auth.authority_effect,
    };
    let canonical = serde_json::to_vec(&binding).expect("authorization binding serialization");
    sha256_hex(&canonical)
}

fn validate<'a>(
    path_id: &str,
    auth: &'a DeletionAuthorization,
    now_unix: u64,
    inner: &'a InnerState,
) -> Result<&'a [u8], ApiError> {
    if auth.action != "delete_evidence" {
        return Err(ApiError::bad_request("action_mismatch"));
    }
    if auth.authority_effect != "none" {
        return Err(ApiError::forbidden("authority_expansion_denied"));
    }
    if path_id != auth.evidence_id {
        return Err(ApiError::conflict("evidence_id_substitution"));
    }
    if now_unix > auth.expires_at_unix {
        return Err(ApiError::forbidden("authorization_expired"));
    }
    if inner.spent_permits.contains(&auth.permit_id) {
        return Err(ApiError::conflict("permit_replay_denied"));
    }
    let expected_object_hash = authorization_object_hash(auth);
    if expected_object_hash != auth.object_hash {
        return Err(ApiError::conflict("authorization_object_hash_mismatch"));
    }
    let evidence = inner
        .evidence
        .get(path_id)
        .ok_or_else(|| ApiError::conflict("evidence_not_present"))?;
    if sha256_hex(evidence) != auth.evidence_hash {
        return Err(ApiError::conflict("evidence_hash_mismatch"));
    }
    Ok(evidence)
}

async fn check_deletion(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<DeletionRequest>,
) -> Result<Json<RetentionCheck>, ApiError> {
    let now = (state.clock)();
    let inner = state.inner.lock().expect("retention state lock poisoned");
    validate(&id, &request.authorization, now, &inner)?;
    Ok(Json(RetentionCheck {
        evidence_id: id,
        permit_id: request.authorization.permit_id.clone(),
        authorization_object_hash: request.authorization.object_hash.clone(),
        exact_evidence_present: true,
        governed_effect: "evidence_deletion",
        authority_effect: "none",
    }))
}

async fn delete_evidence(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<DeletionRequest>,
) -> Result<Json<DeletionManifest>, ApiError> {
    let now = (state.clock)();
    let mut inner = state.inner.lock().expect("retention state lock poisoned");
    validate(&id, &request.authorization, now, &inner)?;

    let request_event = format!(
        "DeletionRequested|{}|{}|{}|{}",
        request.authorization.permit_id,
        id,
        request.authorization.evidence_hash,
        request.authorization.object_hash
    );
    let deletion_requested_hash = sha256_hex(request_event.as_bytes());

    let removed = inner
        .evidence
        .remove(&id)
        .ok_or_else(|| ApiError::conflict("evidence_not_present"))?;
    if sha256_hex(&removed) != request.authorization.evidence_hash {
        inner.evidence.insert(id.clone(), removed);
        return Err(ApiError::conflict("evidence_hash_changed_during_delete"));
    }

    inner
        .spent_permits
        .insert(request.authorization.permit_id.clone());

    let executed_event = format!(
        "DeletionExecuted|{}|{}|{}|{}",
        request.authorization.permit_id,
        id,
        request.authorization.evidence_hash,
        request.authorization.object_hash
    );
    let deletion_executed_hash = sha256_hex(executed_event.as_bytes());
    let merkle_root =
        sha256_hex(format!("{}{}", deletion_requested_hash, deletion_executed_hash).as_bytes());
    let manifest_id = format!("mnf_{}", &merkle_root[..16]);

    Ok(Json(DeletionManifest {
        manifest_id,
        permit_id: request.authorization.permit_id.clone(),
        evidence_id: id,
        evidence_hash: request.authorization.evidence_hash.clone(),
        policy_hash: request.authorization.policy_hash.clone(),
        authorization_object_hash: request.authorization.object_hash.clone(),
        deletion_requested_hash,
        deletion_executed_hash,
        merkle_root,
        custody_events: vec!["DeletionRequested", "DeletionExecuted"],
        reconciled: false,
        authority_effect: "none",
    }))
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/evidence/:id/retention/check", post(check_deletion))
        .route("/evidence/:id/retention/delete", post(delete_evidence))
        .with_state(state)
}
