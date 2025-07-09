use async_trait::async_trait;
use log::Level;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::sync::Mutex;

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
        cpu_usage: Arc::new(Mutex::new(0.0)),
        network_throughput: Arc::new(Mutex::new(0)),
        prev_traffic: Arc::new(Mutex::new(0)),
        max_memory_mb: 1,
        max_circuits: 1,
        session: SessionManager::new(Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    };
    let _ = tokio::fs::remove_file("state.log").await;
    state
        .update_metrics(2 * 1024 * 1024, 2, 0, 0.0, 0, 30)
        .await;

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
        cpu_usage: Arc::new(Mutex::new(0.0)),
        network_throughput: Arc::new(Mutex::new(0)),
        prev_traffic: Arc::new(Mutex::new(0)),
        max_memory_mb: 1,
        max_circuits: 10,
        session: SessionManager::new(Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    };
    let _ = tokio::fs::remove_file("mem.log").await;
    state
        .update_metrics(2 * 1024 * 1024, 0, 0, 0.0, 0, 30)
        .await;
    assert!(state
        .tray_warning
        .lock()
        .await
        .as_ref()
        .unwrap()
        .contains("memory"));
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
        cpu_usage: Arc::new(Mutex::new(0.0)),
        network_throughput: Arc::new(Mutex::new(0)),
        prev_traffic: Arc::new(Mutex::new(0)),
        max_memory_mb: 1024,
        max_circuits: 1,
        session: SessionManager::new(Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    };
    let _ = tokio::fs::remove_file("circ.log").await;
    state.update_metrics(0, 2, 0, 0.0, 0, 30).await;
    assert!(state
        .tray_warning
        .lock()
        .await
        .as_ref()
        .unwrap()
        .contains("circuit"));
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
        cpu_usage: Arc::new(Mutex::new(0.0)),
        network_throughput: Arc::new(Mutex::new(0)),
        prev_traffic: Arc::new(Mutex::new(0)),
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

#[tokio::test]
async fn metrics_rotation_creates_archive() {
    let mut state = AppState::<DummyClient>::default();
    state.log_file = PathBuf::from("metrics.log");
    state.metrics_file = Some(PathBuf::from("metrics.json"));

    let _ = tokio::fs::remove_file("metrics.json").await;
    let _ = tokio::fs::remove_dir_all("archive").await;

    for _ in 0..=torwell84::state::DEFAULT_MAX_METRIC_LINES {
        let point = torwell84::state::MetricPoint {
            time: 0,
            memory_mb: 0,
            circuit_count: 0,
            latency_ms: 0,
            oldest_age: 0,
            avg_create_ms: 0,
            failed_attempts: 0,
            cpu_percent: 0.0,
            network_bytes: 0,
            network_total: 0,
            complete: false,
        };
        state.append_metric(&point).await.unwrap();
    }

    let mut dir = tokio::fs::read_dir("archive").await.unwrap();
    let mut has_file = false;
    while let Some(_) = dir.next_entry().await.unwrap() {
        has_file = true;
        break;
    }
    assert!(has_file);

    let metrics = state.load_metrics().await.unwrap();
    assert_eq!(metrics.len(), torwell84::state::DEFAULT_MAX_METRIC_LINES);
}

#[tokio::test]
async fn log_trim_handles_large_file() {
    let mut state = AppState::<DummyClient>::default();
    state.log_file = PathBuf::from("large.log");
    let _ = tokio::fs::remove_file("large.log").await;
    let _ = tokio::fs::remove_dir_all("archive").await;

    for _ in 0..=(torwell84::state::DEFAULT_MAX_LOG_LINES * 2) {
        state
            .add_log(Level::Info, "big".into(), None)
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
    assert_eq!(logs.len(), torwell84::state::DEFAULT_MAX_LOG_LINES);
}

#[tokio::test]
async fn metrics_limit_from_env_shows_warning() {
    std::env::set_var("TORWELL_MAX_METRIC_LINES", "5");
    std::env::set_var("TORWELL_MAX_METRIC_MB", "1");

    let mut state = AppState::<DummyClient>::default();
    state.log_file = PathBuf::from("metric_warn.log");
    state.metrics_file = Some(PathBuf::from("metric_warn.json"));

    let _ = tokio::fs::remove_file("metric_warn.json").await;
    let _ = tokio::fs::remove_dir_all("archive").await;

    let point = torwell84::state::MetricPoint {
        time: 0,
        memory_mb: 0,
        circuit_count: 0,
        latency_ms: 0,
        oldest_age: 0,
        avg_create_ms: 0,
        failed_attempts: 0,
        cpu_percent: 0.0,
        network_bytes: 0,
        network_total: 0,
        complete: false,
    };

    // create existing large file
    let mut data = String::new();
    let line = format!("{}\n", serde_json::to_string(&point).unwrap());
    for _ in 0..10 {
        data.push_str(&line);
    }
    tokio::fs::write("metric_warn.json", data).await.unwrap();

    state.append_metric(&point).await.unwrap();

    let metrics = state.load_metrics().await.unwrap();
    assert_eq!(metrics.len(), 5);
    assert!(state
        .tray_warning
        .lock()
        .await
        .as_ref()
        .unwrap()
        .contains("metric"));

    std::env::remove_var("TORWELL_MAX_METRIC_LINES");
    std::env::remove_var("TORWELL_MAX_METRIC_MB");
}

#[tokio::test]
async fn security_warning_emits_event() {
    let mut app = tauri::test::mock_app();
    let state = AppState {
        tor_manager: Arc::new(TorManager::new()),
        http_client: Arc::new(SecureHttpClient::new_default().unwrap()),
        log_file: PathBuf::from("warn.log"),
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
    app.manage(state);
    let state = app.state::<AppState<DummyClient>>();
    state.register_handle(app.handle()).await;

    let received = Arc::new(StdMutex::new(Vec::new()));
    let recv_clone = received.clone();
    let _handler = app.listen_global("security-warning", move |event| {
        if let Some(p) = event.payload() {
            recv_clone.lock().unwrap().push(p.to_string());
        }
    });

    state.emit_security_warning("alert".into()).await;

    let events = received.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], "alert");
}

#[tokio::test]
async fn tray_menu_contains_metrics_items() {
    let mut app = tauri::test::mock_app();
    let state = AppState {
        tor_manager: Arc::new(TorManager::new()),
        http_client: Arc::new(SecureHttpClient::new_default().unwrap()),
        log_file: PathBuf::from("tray.log"),
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
    };
    app.manage(state);
    let state = app.state::<AppState<DummyClient>>();
    state.register_handle(app.handle()).await;
    state
        .update_metrics(10 * 1024 * 1024, 3, 0, 0.0, 0, 30)
        .await;
    state.update_tray_menu().await;

    let tray = app.tray_handle();
    assert!(tray.try_get_item("memory").is_some());
    assert!(tray.try_get_item("circuits").is_some());
}

#[tokio::test]
async fn update_metrics_emits_security_warning() {
    let mut app = tauri::test::mock_app();
    let state = AppState {
        tor_manager: Arc::new(TorManager::new()),
        http_client: Arc::new(SecureHttpClient::new_default().unwrap()),
        log_file: PathBuf::from("warn.log"),
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
        max_memory_mb: 1,
        max_circuits: 20,
        session: SessionManager::new(Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    };
    app.manage(state);
    let state = app.state::<AppState<DummyClient>>();
    state.register_handle(app.handle()).await;

    let received = Arc::new(StdMutex::new(Vec::new()));
    let recv_clone = received.clone();
    let _handler = app.listen_global("security-warning", move |event| {
        if let Some(p) = event.payload() {
            recv_clone.lock().unwrap().push(p.to_string());
        }
    });

    state
        .update_metrics(2 * 1024 * 1024, 0, 0, 0.0, 0, 30)
        .await;

    let events = received.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert!(events[0].contains("memory"));
}

#[tokio::test]
async fn tray_warning_cycle() {
    let mut app = tauri::test::mock_app();
    let state = AppState {
        tor_manager: Arc::new(TorManager::new()),
        http_client: Arc::new(SecureHttpClient::new_default().unwrap()),
        log_file: PathBuf::from("cycle.log"),
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
        max_memory_mb: 1,
        max_circuits: 20,
        session: SessionManager::new(Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    };
    app.manage(state);
    let state = app.state::<AppState<DummyClient>>();
    state.register_handle(app.handle()).await;

    // trigger warning by exceeding memory limit
    state
        .update_metrics(2 * 1024 * 1024, 0, 0, 0.0, 0, 30)
        .await;
    let tray = app.tray_handle();
    assert!(tray.try_get_item("warning").is_some());

    // clear warning and ensure menu item removed
    state.clear_tray_warning().await;
    let tray = app.tray_handle();
    assert!(tray.try_get_item("warning").is_none());
}

#[tokio::test]
async fn warning_set_clear_rebuilds_menu() {
    let mut app = tauri::test::mock_app();
    let state = AppState {
        tor_manager: Arc::new(TorManager::new()),
        http_client: Arc::new(SecureHttpClient::new_default().unwrap()),
        log_file: PathBuf::from("cycle2.log"),
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
        max_memory_mb: 1,
        max_circuits: 20,
        session: SessionManager::new(Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    };
    app.manage(state);
    let state = app.state::<AppState<DummyClient>>();
    state.register_handle(app.handle()).await;

    // trigger warning
    state
        .update_metrics(2 * 1024 * 1024, 0, 0, 0.0, 0, 30)
        .await;
    let tray = app.tray_handle();
    assert!(tray.try_get_item("warning").is_some());

    // clear warning
    state.clear_tray_warning().await;
    let tray = app.tray_handle();
    assert!(tray.try_get_item("warning").is_none());

    // rebuilding the menu should not re-add the warning
    state.update_tray_menu().await;
    let tray = app.tray_handle();
    assert!(tray.try_get_item("warning").is_none());
}

#[tokio::test]
async fn tray_menu_after_cert_failure() {
    let mut app = tauri::test::mock_app();
    let client = Arc::new(SecureHttpClient::new_default().unwrap());
    *client.update_failures.lock().await = 3;

    let mut state = AppState::<DummyClient>::default();
    state.http_client = client;
    state.metric_interval_secs = 1;
    app.manage(state);
    let state = app.state::<AppState<DummyClient>>();
    state.register_handle(app.handle()).await;

    tokio::time::pause();
    state.clone().start_metrics_task(app.handle());
    tokio::time::advance(Duration::from_secs(2)).await;

    let tray = app.tray_handle();
    assert!(tray.try_get_item("warning").is_some());
}
