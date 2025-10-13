#[cfg(feature = "hsm")]
fn setup_softhsm(key: &str, cert: &str) -> (tempfile::TempDir, u64) {
    use std::fs;
    use std::process::Command;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let conf = dir.path().join("softhsm2.conf");
    let tokens = dir.path().join("tokens");
    fs::create_dir_all(&tokens).unwrap();
    fs::write(
        &conf,
        format!("directories.tokendir = {}\n", tokens.display()),
    )
    .unwrap();
    std::env::set_var("SOFTHSM2_CONF", &conf);

    Command::new("softhsm2-util")
        .args([
            "--init-token",
            "--slot",
            "0",
            "--label",
            "torwell",
            "--so-pin",
            "0102030405060708",
            "--pin",
            "1234",
        ])
        .status()
        .unwrap();
    Command::new("softhsm2-util")
        .args([
            "--import", key, "--token", "torwell", "--label", "tls-key", "--id", "01", "--pin",
            "1234",
        ])
        .status()
        .unwrap();

    let out = Command::new("softhsm2-util")
        .arg("--show-slots")
        .output()
        .unwrap();
    let txt = String::from_utf8(out.stdout).unwrap();
    let slot = txt
        .split("Slot ")
        .filter_map(|s| {
            if s.contains("Label:            torwell") {
                s.lines().next().and_then(|l| l.trim().parse::<u64>().ok())
            } else {
                None
            }
        })
        .expect("slot not found");
    Command::new("pkcs11-tool")
        .args([
            "--module",
            "/usr/lib/softhsm/libsofthsm2.so",
            "--slot",
            &slot.to_string(),
            "--pin",
            "1234",
            "-w",
            cert,
            "-y",
            "cert",
            "-d",
            "01",
            "-a",
            "tls-cert",
        ])
        .status()
        .unwrap();

    std::env::set_var("TORWELL_HSM_LIB", "/usr/lib/softhsm/libsofthsm2.so");
    std::env::set_var("TORWELL_HSM_SLOT", slot.to_string());
    std::env::set_var("TORWELL_HSM_PIN", "1234");
    std::env::set_var("TORWELL_HSM_KEY_LABEL", "tls-key");
    std::env::set_var("TORWELL_HSM_CERT_LABEL", "tls-cert");

    (dir, slot)
}

#[cfg(feature = "hsm")]
#[test]
fn tls_config_with_min_tls_uses_hsm_keys() {
    use reqwest::tls::Version;
    use rustls::client::ResolvesClientCert;
    use std::fs;
    use tempfile::tempdir;

    const KEY: &str = "../tests_data/ca.key";
    const CERT: &str = "../tests_data/ca.pem";

    let (_dir, _slot) = setup_softhsm(KEY, CERT);

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
}

#[cfg(feature = "hsm")]
#[tokio::test]
async fn init_signs_https_requests_with_hsm() {
    use hyper::{server::conn::http1, service::service_fn, Body, Response};
    use rustls::server::WebPkiClientVerifier;
    use rustls::RootCertStore;
    use std::fs;
    use std::io::BufReader;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio_rustls::TlsAcceptor;

    const CA_PEM: &str = include_str!("../tests_data/ca.pem");
    const CA_KEY: &[u8] = include_bytes!("../tests_data/ca.key");

    let (_dir, _slot) = setup_softhsm("../tests_data/ca.key", "../tests_data/ca.pem");

    let mut reader = BufReader::new(CA_PEM.as_bytes());
    let certs: Vec<_> = rustls_pemfile::certs(&mut reader)
        .collect::<Result<_, _>>()
        .unwrap();
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
            let service =
                service_fn(|_req| async { Ok::<_, hyper::Error>(Response::new(Body::from("ok"))) });
            let _ = http1::Builder::new().serve_connection(tls, service).await;
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
}

#[cfg(feature = "hsm")]
#[tokio::test]
async fn hsm_client_establishes_tls_connection() {
    use hyper::{server::conn::http1, service::service_fn, Body, Response};
    use rustls::server::WebPkiClientVerifier;
    use rustls::RootCertStore;
    use std::fs;
    use std::io::BufReader;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio_rustls::TlsAcceptor;

    const PEM: &str = include_str!("../tests_data/ca.pem");

    let (_dir, _slot) = setup_softhsm("../tests_data/ca.key", "../tests_data/ca.pem");

    let mut reader = BufReader::new(PEM.as_bytes());
    let certs: Vec<_> = rustls_pemfile::certs(&mut reader)
        .collect::<Result<_, _>>()
        .unwrap();
    let mut reader = BufReader::new(include_bytes!("../tests_data/ca.key").as_slice());
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
            let service =
                service_fn(|_req| async { Ok::<_, hyper::Error>(Response::new(Body::from("ok"))) });
            let _ = http1::Builder::new().serve_connection(tls, service).await;
        }
    });

    let dir = tempdir().unwrap();
    let cert_path = dir.path().join("pinned.pem");
    fs::write(&cert_path, PEM).unwrap();
    let client = torwell84::secure_http::SecureHttpClient::new(&cert_path).unwrap();

    let body = client.get_text(&format!("https://{}", addr)).await.unwrap();
    assert_eq!(body, "ok");

    server.await.unwrap();
}
