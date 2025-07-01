use httpmock::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use torwell84::secure_http::{SecureHttpClient, DEFAULT_CONFIG_PATH};

const CA_PEM: &str = include_str!("../tests_data/ca.pem");
const NEW_CERT: &str = include_str!("../tests_data/new_cert.pem");

#[tokio::test]
async fn init_fetches_new_certificate() {
    // Start https mock server using built-in CA
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cert.pem");
            then.status(200).body(NEW_CERT);
        })
        .await;

    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, CA_PEM).unwrap();

    let config_path = dir.path().join("config.json");
    let config = serde_json::json!({
        "cert_path": cert_path.to_string_lossy(),
        "cert_url": server.url("/cert.pem")
    });
    fs::write(&config_path, config.to_string()).unwrap();

    let _client = SecureHttpClient::init(&config_path, None, None, None)
        .await
        .unwrap();

    let updated = fs::read_to_string(&cert_path).unwrap();
    assert_eq!(updated, NEW_CERT);
}

#[tokio::test]
async fn update_certificates_replaces_file() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cert.pem");
            then.status(200).body(NEW_CERT);
        })
        .await;

    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, CA_PEM).unwrap();

    let client = SecureHttpClient::new(&cert_path).unwrap();
    client
        .update_certificates(&server.url("/cert.pem"))
        .await
        .unwrap();

    let updated = fs::read_to_string(&cert_path).unwrap();
    assert_eq!(updated, NEW_CERT);
}

#[tokio::test]
async fn reload_certificates_applies_new_file() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/hello");
            then.status(200).body("ok");
        })
        .await;

    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, CA_PEM).unwrap();

    let client = SecureHttpClient::new(&cert_path).unwrap();
    assert!(client.get_text(&server.url("/hello")).await.is_ok());

    fs::write(&cert_path, NEW_CERT).unwrap();
    client.reload_certificates().await.unwrap();

    let res = client.get_text(&server.url("/hello")).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn schedule_updates_fetches_certificate() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cert.pem");
            then.status(200).body(NEW_CERT);
        })
        .await;

    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, CA_PEM).unwrap();

    let client = Arc::new(SecureHttpClient::new(&cert_path).unwrap());
    client
        .clone()
        .schedule_updates(server.url("/cert.pem"), Duration::from_millis(50));

    tokio::time::sleep(Duration::from_millis(100)).await;

    let updated = fs::read_to_string(&cert_path).unwrap();
    assert_eq!(updated, NEW_CERT);
}
#[tokio::test]
async fn init_overrides_config_values() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cert.pem");
            then.status(200).body(NEW_CERT);
        })
        .await;

    let dir = tempdir().unwrap();
    let cfg_cert_path = dir.path().join("cfg.pem");
    fs::write(&cfg_cert_path, CA_PEM).unwrap();

    let config_path = dir.path().join("config.json");
    let config = serde_json::json!({
        "cert_path": cfg_cert_path.to_string_lossy(),
        "cert_url": "https://invalid.example/cert.pem"
    });
    fs::write(&config_path, config.to_string()).unwrap();

    let override_path = dir.path().join("override.pem");
    fs::write(&override_path, CA_PEM).unwrap();

    let _client = SecureHttpClient::init(
        &config_path,
        Some(override_path.to_string_lossy().to_string()),
        Some(server.url("/cert.pem")),
        None,
    )
    .await
    .unwrap();

    let updated = fs::read_to_string(&override_path).unwrap();
    assert_eq!(updated, NEW_CERT);
    let original = fs::read_to_string(&cfg_cert_path).unwrap();
    assert_eq!(original, CA_PEM);
}

#[tokio::test]
async fn init_with_repo_config() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cert.pem");
            then.status(200).body(NEW_CERT);
        })
        .await;

    let cfg_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("certs/cert_config.json");
    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, CA_PEM).unwrap();

    let _client = SecureHttpClient::init(
        &cfg_path,
        Some(cert_path.to_string_lossy().to_string()),
        Some(server.url("/cert.pem")),
        None,
    )
    .await
    .unwrap();

    let updated = fs::read_to_string(&cert_path).unwrap();
    assert_eq!(updated, NEW_CERT);
}

#[tokio::test]
async fn init_with_missing_config() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cert.pem");
            then.status(200).body(NEW_CERT);
        })
        .await;

    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, CA_PEM).unwrap();

    let cfg_path = dir.path().join("missing.json");

    let _client = SecureHttpClient::init(
        &cfg_path,
        Some(cert_path.to_string_lossy().to_string()),
        Some(server.url("/cert.pem")),
        None,
    )
    .await
    .unwrap();

    let updated = fs::read_to_string(&cert_path).unwrap();
    assert_eq!(updated, NEW_CERT);
}
#[tokio::test]
async fn init_using_default_constant() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cert.pem");
            then.status(200).body(NEW_CERT);
        })
        .await;

    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, CA_PEM).unwrap();

    let _client = SecureHttpClient::init(
        DEFAULT_CONFIG_PATH,
        Some(cert_path.to_string_lossy().to_string()),
        Some(server.url("/cert.pem")),
        None,
    )
    .await
    .unwrap();

    let updated = fs::read_to_string(&cert_path).unwrap();
    assert_eq!(updated, NEW_CERT);
}
