use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tauri::Manager;
use tokio::sync::Mutex;
use tokio::time::{advance, Duration};

use log::Level;
use regex::Regex;
use torwell84::commands;
use torwell84::error::Error;
use torwell84::secure_http::SecureHttpClient;
use torwell84::session::SessionManager;
use torwell84::state::{AppState, LogEntry};
use torwell84::tor_manager::{TorClientBehavior, TorClientConfig, TorManager};

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
        CONNECT_RESULTS
            .lock()
            .unwrap()
            .pop_front()
            .expect("no result")
    }
    async fn create_bootstrapped_with_progress<P>(
        config: TorClientConfig,
        progress: &mut P,
    ) -> std::result::Result<Self, String>
    where
        P: FnMut(u8, String) + Send,
    {
        let res = Self::create_bootstrapped(config).await;
        if res.is_ok() {
            progress(100, "done".into());
        }
        res
    }
    fn reconfigure(&self, _config: &TorClientConfig) -> std::result::Result<(), String> {
        if self.reconfigure_ok {
            Ok(())
        } else {
            Err("reconf".into())
        }
    }
    fn retire_all_circs(&self) {}
    async fn build_new_circuit(&self) -> std::result::Result<(), String> {
        if self.build_ok {
            Ok(())
        } else {
            Err("build".into())
        }
    }
}

fn mock_state() -> AppState<MockTorClient> {
    AppState {
        tor_manager: Arc::new(TorManager::new()),
        http_client: Arc::new(SecureHttpClient::new_default().unwrap()),
        log_file: PathBuf::from("test.log"),
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
    MockTorClient::push_result(Ok(MockTorClient {
        reconfigure_ok: true,
        build_ok: true,
    }));
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
    state
        .add_log(Level::Info, "line1".into(), None)
        .await
        .unwrap();
    state
        .add_log(Level::Warn, "line2".into(), None)
        .await
        .unwrap();
    let logs = commands::get_logs(state).await.unwrap();
    assert_eq!(logs.len(), 2);
    assert!(Regex::new("line1").unwrap().is_match(&logs[0].message));
    assert_eq!(logs[0].level, "INFO");
    assert!(Regex::new("line2").unwrap().is_match(&logs[1].message));
    assert_eq!(logs[1].level, "WARN");
    commands::clear_logs(state).await.unwrap();
    let logs = commands::get_logs(state).await.unwrap();
    assert!(logs.is_empty());
    let path = commands::get_log_file_path(state).await.unwrap();
    assert_eq!(path, state.log_file_path());
}

#[tokio::test]
async fn command_set_log_limit_trims_logs() {
    let mut app = tauri::test::mock_app();
    let state = mock_state();
    let _ = tokio::fs::remove_file(&state.log_file).await;
    app.manage(state);
    let state = app.state::<AppState<MockTorClient>>();

    commands::set_log_limit(state, 2).await.unwrap();
    state
        .add_log(Level::Info, "one".into(), None)
        .await
        .unwrap();
    state
        .add_log(Level::Info, "two".into(), None)
        .await
        .unwrap();
    state
        .add_log(Level::Info, "three".into(), None)
        .await
        .unwrap();

    let logs = commands::get_logs(state).await.unwrap();
    assert_eq!(logs.len(), 2);
    assert!(Regex::new("two").unwrap().is_match(&logs[0].message));
    assert!(Regex::new("three").unwrap().is_match(&logs[1].message));
}

#[tokio::test]
async fn command_set_exit_country() {
    let mut app = tauri::test::mock_app();
    let state = mock_state();
    app.manage(state);
    let state = app.state::<AppState<MockTorClient>>();

    commands::set_exit_country(state, Some("us".into()))
        .await
        .unwrap();
    assert_eq!(
        state.tor_manager.get_exit_country().await.as_deref(),
        Some("US")
    );

    commands::set_exit_country(state, None).await.unwrap();
    assert!(state.tor_manager.get_exit_country().await.is_none());
}

#[tokio::test]
async fn command_set_exit_country_invalid() {
    let mut app = tauri::test::mock_app();
    app.manage(mock_state());
    let state = app.state::<AppState<MockTorClient>>();
    let res = commands::set_exit_country(state, Some("zzz".into())).await;
    match res {
        Err(Error::ConfigError { step, .. }) => assert_eq!(step, "set_exit_country"),
        _ => panic!("expected config error"),
    }
}

#[tokio::test]
async fn command_set_bridges() {
    let mut app = tauri::test::mock_app();
    let state = mock_state();
    app.manage(state);
    let state = app.state::<AppState<MockTorClient>>();

    let bridges = vec!["obfs4 1.2.3.4:80 key".to_string()];
    commands::set_bridges(state, bridges.clone()).await.unwrap();
    assert_eq!(state.tor_manager.get_bridges().await, bridges);
}
#[tokio::test]
async fn command_set_exit_country_mixed_case() {
    let mut app = tauri::test::mock_app();
    let state = mock_state();
    app.manage(state);
    let state = app.state::<AppState<MockTorClient>>();

    commands::set_exit_country(state, Some("dE".into()))
        .await
        .unwrap();
    assert_eq!(
        state.tor_manager.get_exit_country().await.as_deref(),
        Some("DE")
    );
}

#[tokio::test]
async fn command_clear_bridges() {
    let mut app = tauri::test::mock_app();
    let state = mock_state();
    app.manage(state);
    let state = app.state::<AppState<MockTorClient>>();

    let bridges = vec!["obfs4 2.3.4.5:443 key".to_string()];
    commands::set_bridges(state, bridges).await.unwrap();
    commands::set_bridges(state, Vec::new()).await.unwrap();
    assert!(state.tor_manager.get_bridges().await.is_empty());
}

#[tokio::test]
async fn ping_host_invalid_hostname() {
    let res = commands::ping_host(Some("bad host$".into()), Some(1)).await;
    assert!(matches!(res, Err(Error::Io(_))));
}

#[tokio::test]
async fn ping_host_count_capped() {
    // should succeed using localhost even when count exceeds limit
    let res = commands::ping_host(Some("127.0.0.1".into()), Some(100)).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn ping_host_rate_limited() {
    // repeatedly call ping_host to exceed the API rate limit
    let mut last = Ok(0u64);
    for _ in 0..65 {
        last = commands::ping_host(Some("127.0.0.1".into()), Some(1)).await;
    }
    assert!(matches!(last, Err(Error::RateLimitExceeded(_))));
}

#[tokio::test]
async fn tray_warning_set_and_cleared() {
    let mut app = tauri::test::mock_app();
    let state = mock_state();
    app.manage(state);
    let state = app.state::<AppState<MockTorClient>>();

    // trigger warning
    state.update_metrics(2 * 1024 * 1024, 0).await;
    assert!(state.tray_warning.lock().await.is_some());

    // clear warning
    state.clear_tray_warning().await;
    assert!(state.tray_warning.lock().await.is_none());
}

#[tokio::test]
async fn command_lookup_country() {
    let mut app = tauri::test::mock_app();
    app.manage(mock_state());
    let state = app.state::<AppState<MockTorClient>>();
    let code = commands::lookup_country(state, "8.8.8.8".into())
        .await
        .unwrap();
    assert!(!code.is_empty());
}

#[tokio::test(start_paused = true)]
async fn command_connect_error_propagates_details() {
    for _ in 0..6 {
        MockTorClient::push_result(Err("boot".into()));
    }
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
    commands::connect(app.handle(), state).await.unwrap();

    advance(Duration::from_secs(60)).await;
    tokio::task::yield_now().await;

    let events = received.lock().unwrap();
    assert!(!events.is_empty());
    let last: serde_json::Value = serde_json::from_str(&events[events.len() - 1]).unwrap();
    assert_eq!(last["status"], "ERROR");
    assert_eq!(last["errorStep"], "retries_exceeded");
    assert!(last["errorSource"].as_str().unwrap().contains("bootstrap"));
}
