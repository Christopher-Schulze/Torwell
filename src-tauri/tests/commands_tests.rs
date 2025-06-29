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
