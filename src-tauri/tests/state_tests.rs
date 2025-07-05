use async_trait::async_trait;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::sync::Mutex;
use log::Level;
use regex::Regex;

use torwell84::secure_http::SecureHttpClient;
use torwell84::session::SessionManager;
use torwell84::state::AppState;
use torwell84::tor_manager::{TorClientBehavior, TorClientConfig, TorManager};

#[derive(Clone)]
struct DummyClient {
    retired: Arc<StdMutex<bool>>,
}

static CLIENTS: Lazy<StdMutex<VecDeque<DummyClient>>> =
    Lazy::new(|| StdMutex::new(VecDeque::new()));

impl DummyClient {
    fn new(flag: Arc<StdMutex<bool>>) -> Self {
        Self { retired: flag }
    }

    fn push(client: DummyClient) {
        CLIENTS.lock().unwrap().push_back(client);
    }
}

#[async_trait]
impl TorClientBehavior for DummyClient {
    async fn create_bootstrapped(_c: TorClientConfig) -> std::result::Result<Self, String> {
        Ok(CLIENTS.lock().unwrap().pop_front().expect("no client"))
    }

    async fn create_bootstrapped_with_progress<P>(
        c: TorClientConfig,
        _p: &mut P,
    ) -> std::result::Result<Self, String>
    where
        P: FnMut(u8, String) + Send,
    {
        Self::create_bootstrapped(c).await
    }

    fn reconfigure(&self, _c: &TorClientConfig) -> std::result::Result<(), String> {
        Ok(())
    }

    fn retire_all_circs(&self) {
        *self.retired.lock().unwrap() = true;
    }

    async fn build_new_circuit(&self) -> std::result::Result<(), String> {
        Ok(())
    }
}

#[tokio::test]
async fn update_metrics_closes_circuits_on_limit() {
    let flag = Arc::new(StdMutex::new(false));
    DummyClient::push(DummyClient::new(flag.clone()));
    let manager: TorManager<DummyClient> = TorManager::new();
    manager.connect().await.unwrap();

    let state = AppState {
        tor_manager: Arc::new(manager),
        http_client: Arc::new(SecureHttpClient::new_default().unwrap()),
        log_file: PathBuf::from("state.log"),
        log_lock: Arc::new(Mutex::new(())),
        retry_counter: Arc::new(Mutex::new(0)),
        max_log_lines: Arc::new(Mutex::new(1000)),
        memory_usage: Arc::new(Mutex::new(0)),
        circuit_count: Arc::new(Mutex::new(0)),
        oldest_circuit_age: Arc::new(Mutex::new(0)),
        latency_ms: Arc::new(Mutex::new(0)),
        max_memory_mb: 1,
        max_circuits: 1,
        session: SessionManager::new(Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    };
    let _ = tokio::fs::remove_file("state.log").await;
    state.update_metrics(2 * 1024 * 1024, 2, 0).await;

    assert!(*flag.lock().unwrap());
    let logs = state.read_logs().await.unwrap();
    assert!(!logs.is_empty());
}

#[tokio::test]
async fn tray_warning_on_memory_limit() {
    let flag = Arc::new(StdMutex::new(false));
    DummyClient::push(DummyClient::new(flag));
    let manager: TorManager<DummyClient> = TorManager::new();
    manager.connect().await.unwrap();

    let state = AppState {
        tor_manager: Arc::new(manager),
        http_client: Arc::new(SecureHttpClient::new_default().unwrap()),
        log_file: PathBuf::from("mem.log"),
        log_lock: Arc::new(Mutex::new(())),
        retry_counter: Arc::new(Mutex::new(0)),
        max_log_lines: Arc::new(Mutex::new(1000)),
        memory_usage: Arc::new(Mutex::new(0)),
        circuit_count: Arc::new(Mutex::new(0)),
        oldest_circuit_age: Arc::new(Mutex::new(0)),
        latency_ms: Arc::new(Mutex::new(0)),
        max_memory_mb: 1,
        max_circuits: 10,
        session: SessionManager::new(Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    };
    let _ = tokio::fs::remove_file("mem.log").await;
    state.update_metrics(2 * 1024 * 1024, 0, 0).await;
    assert!(state.tray_warning.lock().await.as_ref().unwrap().contains("memory"));
}

#[tokio::test]
async fn tray_warning_on_circuit_limit() {
    let flag = Arc::new(StdMutex::new(false));
    DummyClient::push(DummyClient::new(flag));
    let manager: TorManager<DummyClient> = TorManager::new();
    manager.connect().await.unwrap();

    let state = AppState {
        tor_manager: Arc::new(manager),
        http_client: Arc::new(SecureHttpClient::new_default().unwrap()),
        log_file: PathBuf::from("circ.log"),
        log_lock: Arc::new(Mutex::new(())),
        retry_counter: Arc::new(Mutex::new(0)),
        max_log_lines: Arc::new(Mutex::new(1000)),
        memory_usage: Arc::new(Mutex::new(0)),
        circuit_count: Arc::new(Mutex::new(0)),
        oldest_circuit_age: Arc::new(Mutex::new(0)),
        latency_ms: Arc::new(Mutex::new(0)),
        max_memory_mb: 1024,
        max_circuits: 1,
        session: SessionManager::new(Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    };
    let _ = tokio::fs::remove_file("circ.log").await;
    state.update_metrics(0, 2, 0).await;
    assert!(state.tray_warning.lock().await.as_ref().unwrap().contains("circuit"));
}

#[tokio::test]
async fn log_rotation_creates_archive() {
    let manager: TorManager<DummyClient> = TorManager::new();

    let state = AppState {
        tor_manager: Arc::new(manager),
        http_client: Arc::new(SecureHttpClient::new_default().unwrap()),
        log_file: PathBuf::from("rotate.log"),
        log_lock: Arc::new(Mutex::new(())),
        retry_counter: Arc::new(Mutex::new(0)),
        max_log_lines: Arc::new(Mutex::new(1000)),
        memory_usage: Arc::new(Mutex::new(0)),
        circuit_count: Arc::new(Mutex::new(0)),
        oldest_circuit_age: Arc::new(Mutex::new(0)),
        latency_ms: Arc::new(Mutex::new(0)),
        max_memory_mb: 1,
        max_circuits: 1,
        session: SessionManager::new(Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    };

    let _ = tokio::fs::remove_file("rotate.log").await;
    let _ = tokio::fs::remove_dir_all("archive").await;

    state.set_max_log_lines(2).await.unwrap();

    for i in 0..3 {
        state
            .add_log(Level::Info, format!("line{}", i), None)
            .await
            .unwrap();
    }

    let mut dir = tokio::fs::read_dir("archive").await.unwrap();
    let mut has_file = false;
    while let Some(_) = dir.next_entry().await.unwrap() {
        has_file = true;
        break;
    }
    assert!(has_file);

    let logs = state.read_logs().await.unwrap();
    assert_eq!(logs.len(), 2);
    assert!(Regex::new("line1").unwrap().is_match(&logs[0].message));
    assert!(Regex::new("line2").unwrap().is_match(&logs[1].message));
}
