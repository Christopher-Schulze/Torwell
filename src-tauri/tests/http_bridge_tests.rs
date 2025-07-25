use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

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
        log_file: PathBuf::from("bridge.log"),
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
        session: SessionManager::new(Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    }
}

#[cfg(feature = "mobile")]
#[tokio::test]
async fn http_bridge_reports_disconnect() {
    let state = mock_state();
    torwell84::http_bridge::start(state.clone());
    tokio::time::sleep(Duration::from_millis(100)).await;

    let body: String = reqwest::get("http://127.0.0.1:1421/status")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body, "DISCONNECTED");
}

#[cfg(feature = "mobile")]
#[tokio::test]
async fn http_bridge_reports_connect() {
    let state = mock_state();
    state.tor_manager.connect().await.unwrap();
    torwell84::http_bridge::start(state.clone());
    tokio::time::sleep(Duration::from_millis(100)).await;

    let body: String = reqwest::get("http://127.0.0.1:1421/status")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body, "CONNECTED");
}

#[cfg(feature = "mobile")]
#[tokio::test]
async fn http_bridge_sets_workers() {
    let state = mock_state();
    torwell84::http_bridge::start(state.clone());
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let res = client
        .post("http://127.0.0.1:1421/workers")
        .json(&serde_json::json!({
            "workers": ["https://example.com"],
            "token": "abc"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::NO_CONTENT);
    let urls = state.http_client.worker_urls.lock().await;
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0], "https://example.com");
}

#[cfg(feature = "mobile")]
#[tokio::test]
async fn http_bridge_validates_worker_token() {
    use httpmock::prelude::*;
    use urlencoding::encode;

    let worker = MockServer::start_async().await;
    let target = "https://example.com/hello";
    let encoded = encode(target);
    worker
        .mock_async(|when, then| {
            when.method(GET)
                .path(format!("/proxy?url={}", encoded))
                .header("X-Proxy-Token", "secret");
            then.status(200).body("ok");
        })
        .await;

    let state = mock_state();
    torwell84::http_bridge::start(state.clone());
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    client
        .post("http://127.0.0.1:1421/workers")
        .json(&serde_json::json!({
            "workers": [worker.url("/proxy")],
            "token": "secret"
        }))
        .send()
        .await
        .unwrap();

    let valid: bool = client
        .get("http://127.0.0.1:1421/validate")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(valid);
}

#[cfg(feature = "mobile")]
#[tokio::test]
async fn http_bridge_validate_token_fails() {
    use httpmock::prelude::*;
    use urlencoding::encode;

    let worker = MockServer::start_async().await;
    let target = "https://example.com/hello";
    let encoded = encode(target);
    worker
        .mock_async(|when, then| {
            when.method(GET)
                .path(format!("/proxy?url={}", encoded))
                .header("X-Proxy-Token", "right");
            then.status(200).body("ok");
        })
        .await;

    let state = mock_state();
    torwell84::http_bridge::start(state.clone());
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    client
        .post("http://127.0.0.1:1421/workers")
        .json(&serde_json::json!({
            "workers": [worker.url("/proxy")],
            "token": "wrong"
        }))
        .send()
        .await
        .unwrap();

    let valid: bool = client
        .get("http://127.0.0.1:1421/validate")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(!valid);
}
