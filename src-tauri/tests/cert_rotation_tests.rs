use httpmock::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use tokio::sync::Mutex;

use torwell84::secure_http::SecureHttpClient;
use torwell84::session::SessionManager;
use torwell84::state::AppState;
use torwell84::tor_manager::TorManager;

const CA_PEM: &str = include_str!("../tests_data/ca.pem");
const NEW_CERT: &str = include_str!("../tests_data/new_cert.pem");

#[tokio::test]
async fn schedule_updates_rotates_certificate() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cert.pem");
            then.status(200).body(NEW_CERT);
        })
        .await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/hello");
            then.status(200).body("ok");
        })
        .await;

    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, CA_PEM).unwrap();

    let client = Arc::new(SecureHttpClient::new(&cert_path).unwrap());
    assert!(client.get_text(&server.url("/hello")).await.is_ok());

    client
        .clone()
        .schedule_updates(vec![server.url("/cert.pem")], Duration::from_millis(50))
        .await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    let res = client.get_text(&server.url("/hello")).await;
    assert!(res.is_err());

    let updated = fs::read_to_string(&cert_path).unwrap();
    assert_eq!(updated, NEW_CERT);
}

#[tokio::test]
async fn tray_warning_after_failed_updates() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cert.pem");
            then.status(500);
        })
        .await;

    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, CA_PEM).unwrap();
    let client = Arc::new(SecureHttpClient::new(&cert_path).unwrap());

    for _ in 0..3 {
        let _ = client
            .update_certificates_from(&[server.url("/cert.pem")])
            .await;
    }

    let mut app = tauri::test::mock_app();
    let state = torwell84::state::AppState {
        tor_manager: Arc::new(torwell84::tor_manager::TorManager::new()),
        http_client: client.clone(),
        log_file: cert_path.clone(),
        log_lock: Arc::new(tokio::sync::Mutex::new(())),
        retry_counter: Arc::new(tokio::sync::Mutex::new(0)),
        max_log_lines: Arc::new(tokio::sync::Mutex::new(1000)),
        memory_usage: Arc::new(tokio::sync::Mutex::new(0)),
        circuit_count: Arc::new(tokio::sync::Mutex::new(0)),
        oldest_circuit_age: Arc::new(tokio::sync::Mutex::new(0)),
        latency_ms: Arc::new(tokio::sync::Mutex::new(0)),
        cpu_usage: Arc::new(tokio::sync::Mutex::new(0.0)),
        network_throughput: Arc::new(tokio::sync::Mutex::new(0)),
        max_memory_mb: 1024,
        max_circuits: 20,
        session: torwell84::session::SessionManager::new(Duration::from_secs(60)),
        app_handle: Arc::new(tokio::sync::Mutex::new(None)),
        tray_warning: Arc::new(tokio::sync::Mutex::new(None)),
    };
    app.manage(state);
    let state = app.state::<torwell84::state::AppState>();
    state.register_handle(app.handle()).await;

    tokio::time::pause();
    state.clone().start_metrics_task(app.handle());
    tokio::time::advance(Duration::from_secs(31)).await;

    assert!(state
        .tray_warning
        .lock()
        .await
        .as_ref()
        .unwrap()
        .contains("certificate update"));
}
const INVALID_PEM: &str = "-----BEGIN CERTIFICATE-----\ninvalid\n-----END CERTIFICATE-----\n";

#[tokio::test]
async fn schedule_updates_recovers_from_invalid_pem() {
    let invalid = MockServer::start_async().await;
    invalid
        .mock_async(|when, then| {
            when.method(GET).path("/cert.pem");
            then.status(200).body(INVALID_PEM);
        })
        .await;

    let valid = MockServer::start_async().await;
    valid
        .mock_async(|when, then| {
            when.method(GET).path("/cert.pem");
            then.status(200).body(NEW_CERT);
        })
        .await;
    valid
        .mock_async(|when, then| {
            when.method(GET).path("/hello");
            then.status(200).body("ok");
        })
        .await;

    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, CA_PEM).unwrap();
    let client = Arc::new(SecureHttpClient::new(&cert_path).unwrap());

    tokio::time::pause();
    client
        .clone()
        .schedule_updates(vec![invalid.url("/cert.pem")], Duration::from_millis(20))
        .await;
    tokio::time::advance(Duration::from_millis(25)).await;
    assert_eq!(*client.update_failures.lock().await, 1);

    client
        .clone()
        .schedule_updates(vec![valid.url("/cert.pem")], Duration::from_millis(20))
        .await;
    tokio::time::advance(Duration::from_millis(25)).await;

    assert!(client.get_text(&valid.url("/hello")).await.is_ok());
    let updated = fs::read_to_string(&cert_path).unwrap();
    assert_eq!(updated, NEW_CERT);
}
