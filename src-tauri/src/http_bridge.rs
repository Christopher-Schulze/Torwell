use crate::state::AppState;
use axum::{extract::Extension, routing::get, Json, Router};
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

pub fn start(state: AppState) {
    let router = Router::new()
        .route("/status", get(status))
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
