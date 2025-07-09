use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use httpmock::prelude::*;
use urlencoding::encode;

use torwell84::commands;
use torwell84::secure_http::SecureHttpClient;
use torwell84::session::SessionManager;
use torwell84::state::AppState;
use torwell84::tor_manager::{TorClientBehavior, TorClientConfig, TorManager};

#[derive(Clone, Default)]
struct DummyClient;

#[async_trait::async_trait]
impl TorClientBehavior for DummyClient {
    async fn create_bootstrapped(_config: TorClientConfig) -> std::result::Result<Self, String> {
        Ok(Self)
    }

    async fn create_bootstrapped_with_progress<P>(
        _config: TorClientConfig,
        _progress: &mut P,
    ) -> std::result::Result<Self, String>
    where
        P: FnMut(u8, String) + Send,
    {
        Ok(Self)
    }

    fn reconfigure(&self, _config: &TorClientConfig) -> std::result::Result<(), String> {
        Ok(())
    }

    fn retire_all_circs(&self) {}

    async fn build_new_circuit(&self) -> std::result::Result<(), String> {
        Ok(())
    }
}

fn mock_state() -> AppState<DummyClient> {
    AppState {
        tor_manager: Arc::new(TorManager::new()),
        http_client: Arc::new(SecureHttpClient::new_default().unwrap()),
        log_file: PathBuf::from("worker.log"),
        log_lock: Arc::new(Mutex::new(())),
        retry_counter: Arc::new(Mutex::new(0)),
        max_log_lines: Arc::new(Mutex::new(1000)),
        memory_usage: Arc::new(Mutex::new(0)),
        circuit_count: Arc::new(Mutex::new(0)),
        oldest_circuit_age: Arc::new(Mutex::new(0)),
        latency_ms: Arc::new(Mutex::new(0)),
        cpu_usage: Arc::new(Mutex::new(0.0)),
        network_throughput: Arc::new(Mutex::new(0)),
        prev_traffic: Arc::new(Mutex::new(0)),
        max_memory_mb: 1024,
        max_circuits: 20,
        session: SessionManager::new(std::time::Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    }
}

#[tokio::test]
async fn validate_worker_token_success() {
    let server = MockServer::start_async().await;
    let encoded = encode("https://example.com");
    server
        .mock_async(|when, then| {
            when.method(GET)
                .path(format!("/proxy?url={}", encoded))
                .header("X-Proxy-Token", "secret");
            then.status(200).body("ok");
        })
        .await;

    let state = mock_state();
    state
        .http_client
        .set_worker_config(vec![server.url("/proxy")], Some("secret".into()))
        .await;

    let valid = commands::validate_worker_token(state).await.unwrap();
    assert!(valid);
}

#[tokio::test]
async fn validate_worker_token_failure() {
    let server = MockServer::start_async().await;
    let encoded = encode("https://example.com");
    server
        .mock_async(|when, then| {
            when.method(GET)
                .path(format!("/proxy?url={}", encoded))
                .header("X-Proxy-Token", "right");
            then.status(200).body("ok");
        })
        .await;

    let state = mock_state();
    state
        .http_client
        .set_worker_config(vec![server.url("/proxy")], Some("wrong".into()))
        .await;

    let valid = commands::validate_worker_token(state).await.unwrap();
    assert!(!valid);
}
