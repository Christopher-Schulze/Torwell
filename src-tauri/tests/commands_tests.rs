use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::Mutex as StdMutex;

use torwell84::commands;
use torwell84::state::AppState;
use torwell84::tor_manager::{TorManager, TorClientBehavior, TorClientConfig};
use torwell84::error::Error;

#[derive(Clone, Default)]
struct MockTorClient {
    reconfigure_ok: bool,
    build_ok: bool,
}

static CONNECT_RESULTS: Lazy<StdMutex<VecDeque<Result<MockTorClient, String>>>> =
    Lazy::new(|| StdMutex::new(VecDeque::new()));

impl MockTorClient {
    fn push_result(res: Result<MockTorClient, String>) {
        CONNECT_RESULTS.lock().unwrap().push_back(res);
    }
}

#[async_trait::async_trait]
impl TorClientBehavior for MockTorClient {
    async fn create_bootstrapped(_config: TorClientConfig) -> std::result::Result<Self, String> {
        CONNECT_RESULTS.lock().unwrap().pop_front().expect("no result")
    }
    fn reconfigure(&self, _config: &TorClientConfig) -> std::result::Result<(), String> {
        if self.reconfigure_ok { Ok(()) } else { Err("reconf".into()) }
    }
    fn retire_all_circs(&self) {}
    async fn build_new_circuit(&self) -> std::result::Result<(), String> {
        if self.build_ok { Ok(()) } else { Err("build".into()) }
    }
}

fn mock_state() -> AppState<MockTorClient> {
    AppState {
        tor_manager: Arc::new(TorManager::new()),
        log_file: PathBuf::from("test.log"),
        log_lock: Arc::new(Mutex::new(())),
        max_log_lines: 1000,
    }
}

#[tokio::test]
async fn command_get_status() {
    let mut app = tauri::test::mock_app();
    app.manage(mock_state());
    let state = app.state::<AppState<MockTorClient>>();
    let status = commands::get_status(state).await.unwrap();
    assert_eq!(status, "DISCONNECTED");
}

#[tokio::test]
async fn command_disconnect_not_connected() {
    let mut app = tauri::test::mock_app();
    app.manage(mock_state());
    let received = Arc::new(StdMutex::new(Vec::new()));
    let recv_clone = received.clone();
    let _handler = app.listen_global("tor-status-update", move |event| {
        if let Some(p) = event.payload() {
            recv_clone.lock().unwrap().push(p.to_string());
        }
    });
    let state = app.state::<AppState<MockTorClient>>();
    let res = commands::disconnect(app.handle(), state).await;
    assert!(matches!(res, Err(Error::NotConnected)));
    let events = received.lock().unwrap();
    assert_eq!(events.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&events[0]).unwrap();
    assert_eq!(payload["status"], "DISCONNECTING");
}

#[tokio::test]
async fn command_disconnect_connected() {
    MockTorClient::push_result(Ok(MockTorClient::default()));
    let mut app = tauri::test::mock_app();
    let state = mock_state();
    state.tor_manager.connect().await.unwrap();
    app.manage(state);
    let received = Arc::new(StdMutex::new(Vec::new()));
    let recv_clone = received.clone();
    let _handler = app.listen_global("tor-status-update", move |event| {
        if let Some(p) = event.payload() {
            recv_clone.lock().unwrap().push(p.to_string());
        }
    });
    let state = app.state::<AppState<MockTorClient>>();
    commands::disconnect(app.handle(), state).await.unwrap();
    let events = received.lock().unwrap();
    assert_eq!(events.len(), 2);
    let first: serde_json::Value = serde_json::from_str(&events[0]).unwrap();
    let second: serde_json::Value = serde_json::from_str(&events[1]).unwrap();
    assert_eq!(first["status"], "DISCONNECTING");
    assert_eq!(second["status"], "DISCONNECTED");
}

#[tokio::test]
async fn command_new_identity_success() {
    MockTorClient::push_result(Ok(MockTorClient { reconfigure_ok: true, build_ok: true }));
    let mut app = tauri::test::mock_app();
    let state = mock_state();
    state.tor_manager.connect().await.unwrap();
    app.manage(state);
    let received = Arc::new(StdMutex::new(Vec::new()));
    let recv_clone = received.clone();
    let _handler = app.listen_global("tor-status-update", move |event| {
        if let Some(p) = event.payload() {
            recv_clone.lock().unwrap().push(p.to_string());
        }
    });
    let state = app.state::<AppState<MockTorClient>>();
    commands::new_identity(app.handle(), state).await.unwrap();
    let events = received.lock().unwrap();
    assert_eq!(events.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&events[0]).unwrap();
    assert_eq!(payload["status"], "NEW_IDENTITY");
}

#[tokio::test]
async fn command_new_identity_not_connected() {
    let mut app = tauri::test::mock_app();
    app.manage(mock_state());
    let received = Arc::new(StdMutex::new(Vec::new()));
    let recv_clone = received.clone();
    let _handler = app.listen_global("tor-status-update", move |event| {
        if let Some(p) = event.payload() {
            recv_clone.lock().unwrap().push(p.to_string());
        }
    });
    let state = app.state::<AppState<MockTorClient>>();
    let res = commands::new_identity(app.handle(), state).await;
    assert!(matches!(res, Err(Error::NotConnected)));
    assert!(received.lock().unwrap().is_empty());
}

#[tokio::test]
async fn command_log_retrieval() {
    let mut app = tauri::test::mock_app();
    let state = mock_state();
    let _ = tokio::fs::remove_file(&state.log_file).await;
    app.manage(state);
    let state = app.state::<AppState<MockTorClient>>();
    state.add_log("line1".into()).await.unwrap();
    state.add_log("line2".into()).await.unwrap();
    let logs = commands::get_logs(state).await.unwrap();
    assert_eq!(logs, vec!["line1", "line2"]);
    commands::clear_logs(state).await.unwrap();
    let logs = commands::get_logs(state).await.unwrap();
    assert!(logs.is_empty());
    let path = commands::get_log_file_path(state).await.unwrap();
    assert!(path.ends_with("test.log"));
}
