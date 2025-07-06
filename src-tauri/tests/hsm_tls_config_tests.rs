#[cfg(feature = "hsm")]
#[test]
fn tls_config_with_min_tls_uses_hsm_keys() {
    use base64::{engine::general_purpose, Engine as _};
    use rustls::client::ResolvesClientCert;
    use reqwest::tls::Version;
    use std::fs;
    use tempfile::tempdir;

    // Provide SoftHSM library so no real hardware is required.
    std::env::set_var("TORWELL_HSM_LIB", "/usr/lib/softhsm/libsofthsm2.so");
    let key_b64 = general_purpose::STANDARD.encode(include_bytes!("../tests_data/ca.key"));
    let cert_b64 = general_purpose::STANDARD.encode(include_bytes!("../tests_data/ca.pem"));
    std::env::set_var("TORWELL_HSM_MOCK_KEY", &key_b64);
    std::env::set_var("TORWELL_HSM_MOCK_CERT", &cert_b64);

    const CA_PEM: &str = include_str!("../tests_data/ca.pem");
    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, CA_PEM).unwrap();
    let cfg = torwell84::secure_http::SecureHttpClient::build_tls_config_with_min_tls(
        &cert_path,
        Version::TLS_1_3,
    )
    .unwrap();
    assert!(cfg.client_auth_cert_resolver.has_certs());

    std::env::remove_var("TORWELL_HSM_MOCK_KEY");
    std::env::remove_var("TORWELL_HSM_MOCK_CERT");
}

#[cfg(feature = "hsm")]
#[tokio::test]
async fn init_signs_https_requests_with_hsm() {
    use base64::{engine::general_purpose, Engine as _};
    use hyper::{server::conn::http1, service::service_fn, Body, Response};
    use rustls::RootCertStore;
    use rustls::server::WebPkiClientVerifier;
    use std::fs;
    use std::io::BufReader;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio_rustls::TlsAcceptor;

    const CA_PEM: &str = include_str!("../tests_data/ca.pem");
    const CA_KEY: &[u8] = include_bytes!("../tests_data/ca.key");

    std::env::set_var("TORWELL_HSM_LIB", "/usr/lib/softhsm/libsofthsm2.so");
    std::env::set_var("TORWELL_HSM_SLOT", "0");
    std::env::set_var("TORWELL_HSM_PIN", "1234");
    std::env::set_var("TORWELL_HSM_KEY_LABEL", "tls-key");
    std::env::set_var("TORWELL_HSM_CERT_LABEL", "tls-cert");

    let key_b64 = general_purpose::STANDARD.encode(CA_KEY);
    let cert_b64 = general_purpose::STANDARD.encode(CA_PEM.as_bytes());
    std::env::set_var("TORWELL_HSM_MOCK_KEY", &key_b64);
    std::env::set_var("TORWELL_HSM_MOCK_CERT", &cert_b64);

    let mut reader = BufReader::new(CA_PEM.as_bytes());
    let certs: Vec<_> = rustls_pemfile::certs(&mut reader).collect::<Result<_, _>>().unwrap();
    let mut reader = BufReader::new(CA_KEY);
    let key = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .next()
        .unwrap()
        .unwrap();
    let key = rustls::pki_types::PrivateKeyDer::from(key);
    let certs_der: Vec<_> = certs
        .into_iter()
        .map(rustls::pki_types::CertificateDer::from)
        .collect();

    let mut roots = RootCertStore::empty();
    for c in &certs_der {
        roots.add(c.clone()).unwrap();
    }
    let verifier = WebPkiClientVerifier::builder(Arc::new(roots))
        .build()
        .unwrap();

    let server_config = rustls::ServerConfig::builder()
        .with_client_cert_verifier(verifier)
        .with_single_cert(certs_der.clone(), key)
        .unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(server_config));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let tls = acceptor.accept(stream).await.unwrap();
            let service = service_fn(|_req| async {
                Ok::<_, hyper::Error>(Response::new(Body::from("ok")))
            });
            let _ = http1::Builder::new()
                .serve_connection(tls, service)
                .await;
        }
    });

    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, CA_PEM).unwrap();
    let cfg_path = dir.path().join("config.json");
    let cfg = serde_json::json!({
        "cert_path": cert_path.to_string_lossy(),
        "cert_url": "https://invalid.example/cert.pem"
    });
    fs::write(&cfg_path, cfg.to_string()).unwrap();

    let client = torwell84::secure_http::SecureHttpClient::init(&cfg_path, None, None, None, None)
        .await
        .unwrap();

    let body = client.get_text(&format!("https://{}", addr)).await.unwrap();
    assert_eq!(body, "ok");

    server.await.unwrap();

    std::env::remove_var("TORWELL_HSM_MOCK_KEY");
    std::env::remove_var("TORWELL_HSM_MOCK_CERT");
}

