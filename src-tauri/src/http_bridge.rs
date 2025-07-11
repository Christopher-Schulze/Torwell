use crate::state::AppState;
use axum::http::StatusCode;
use axum::{
    extract::Extension,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};

async fn status(Extension(state): Extension<Arc<AppState>>) -> Json<&'static str> {
    let s = if {
        let mgr = state.tor_manager.read().await.clone();
        mgr.is_connected().await
    } {
        "CONNECTED"
    } else {
        "DISCONNECTED"
    };
    Json(s)
}

#[derive(Deserialize)]
struct WorkerPayload {
    workers: Vec<String>,
    #[serde(default)]
    token: Option<String>,
}

async fn set_workers(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<WorkerPayload>,
) -> StatusCode {
    state
        .http_client
        .set_worker_config(payload.workers, payload.token)
        .await;
    StatusCode::NO_CONTENT
}

async fn validate_worker(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<bool> {
    let res = state
        .http_client
        .get_text("https://example.com")
        .await
        .is_ok();
    Json(res)
}

pub fn start(state: AppState) {
    let router = Router::new()
        .route("/status", get(status))
        .route("/workers", post(set_workers))
        .route("/validate", get(validate_worker))
        .layer(Extension(Arc::new(state)));

    tauri::async_runtime::spawn(async move {
        let addr = SocketAddr::from(([127, 0, 0, 1], 1421));
        if let Err(e) = axum::Server::bind(&addr)
            .serve(router.into_make_service())
            .await
        {
            eprintln!("HTTP bridge failed: {e}");
        }
    });
}
