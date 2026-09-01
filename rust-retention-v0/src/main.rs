use pulpo_retention_v0::{build_app, AppState};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let state = AppState::new(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_secs()
    });

    // V0 intentionally starts with no evidence objects and no authority source.
    // A real deployment must inject already-authorized execution state through
    // a separately governed Pulpo -> Keel boundary.
    let app = build_app(state);
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("bind retention listener");
    axum::serve(listener, app)
        .await
        .expect("serve retention api");
}
