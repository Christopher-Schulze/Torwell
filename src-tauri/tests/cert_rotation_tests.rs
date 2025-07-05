use httpmock::prelude::*;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

use torwell84::secure_http::SecureHttpClient;

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

    client.clone().schedule_updates(vec![server.url("/cert.pem")], Duration::from_millis(50));

    tokio::time::sleep(Duration::from_millis(100)).await;

    let res = client.get_text(&server.url("/hello")).await;
    assert!(res.is_err());

    let updated = fs::read_to_string(&cert_path).unwrap();
    assert_eq!(updated, NEW_CERT);
}
