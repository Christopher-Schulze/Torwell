#[cfg(feature = "hsm")]
fn setup_softhsm(dir: &tempfile::TempDir, key_pem: &[u8], cert_pem: &[u8]) {
    use pkcs11::types::*;
    use pkcs11::Ctx;
    use std::io::BufReader;
    use std::process::Command;

    let conf_path = dir.path().join("softhsm2.conf");
    let token_dir = dir.path().join("tokens");
    std::fs::create_dir_all(&token_dir).unwrap();
    std::fs::write(
        &conf_path,
        format!("directories.tokendir = {}\n", token_dir.display()),
    )
    .unwrap();
    std::env::set_var("SOFTHSM2_CONF", &conf_path);

    let status = Command::new("softhsm2-util")
        .args([
            "--init-token",
            "--slot",
            "0",
            "--label",
            "torwell",
            "--so-pin",
            "0000",
            "--pin",
            "1234",
        ])
        .status()
        .expect("softhsm2-util not found");
    assert!(status.success());

    let mut ctx = Ctx::new("/usr/lib/softhsm/libsofthsm2.so").unwrap();
    ctx.initialize(None).unwrap();
    let session = ctx
        .open_session(0, CKF_SERIAL_SESSION | CKF_RW_SESSION, None, None)
        .unwrap();
    ctx.login(session, CKU_USER, Some("1234")).unwrap();

    let mut reader = BufReader::new(key_pem);
    let key = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .next()
        .unwrap()
        .unwrap();
    let mut reader = BufReader::new(cert_pem);
    let cert = rustls_pemfile::certs(&mut reader).next().unwrap().unwrap();

    let key_tpl = vec![
        CK_ATTRIBUTE::new(CKA_CLASS).with_ck_ulong(&CKO_PRIVATE_KEY),
        CK_ATTRIBUTE::new(CKA_KEY_TYPE).with_ck_ulong(&CKK_RSA),
        CK_ATTRIBUTE::new(CKA_TOKEN).with_bool(&CK_TRUE),
        CK_ATTRIBUTE::new(CKA_LABEL).with_string("tls-key"),
        CK_ATTRIBUTE::new(CKA_VALUE).with_bytes(&key),
    ];
    ctx.create_object(session, &key_tpl).unwrap();

    let cert_tpl = vec![
        CK_ATTRIBUTE::new(CKA_CLASS).with_ck_ulong(&CKO_CERTIFICATE),
        CK_ATTRIBUTE::new(CKA_CERTIFICATE_TYPE).with_ck_ulong(&CKC_X_509),
        CK_ATTRIBUTE::new(CKA_TOKEN).with_bool(&CK_TRUE),
        CK_ATTRIBUTE::new(CKA_LABEL).with_string("tls-cert"),
        CK_ATTRIBUTE::new(CKA_VALUE).with_bytes(&cert),
    ];
    ctx.create_object(session, &cert_tpl).unwrap();

    ctx.logout(session).unwrap();
    ctx.close_session(session).unwrap();
    ctx.finalize().unwrap();
}

#[cfg(feature = "hsm")]
#[test]
fn tls_config_with_min_tls_uses_hsm_keys() {
    use reqwest::tls::Version;
    use rustls::client::ResolvesClientCert;
    use std::fs;
    use tempfile::tempdir;

    std::env::set_var("TORWELL_HSM_LIB", "/usr/lib/softhsm/libsofthsm2.so");

    const CA_PEM: &str = include_str!("../tests_data/ca.pem");
    const CA_KEY: &[u8] = include_bytes!("../tests_data/ca.key");

    let dir = tempdir().unwrap();
    setup_softhsm(&dir, CA_KEY, CA_PEM.as_bytes());

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

    std::env::set_var("TORWELL_HSM_LIB", "/usr/lib/softhsm/libsofthsm2.so");
    std::env::set_var("TORWELL_HSM_SLOT", "0");
    std::env::set_var("TORWELL_HSM_PIN", "1234");
    std::env::set_var("TORWELL_HSM_KEY_LABEL", "tls-key");
    std::env::set_var("TORWELL_HSM_CERT_LABEL", "tls-cert");

    let temp = tempdir().unwrap();
    setup_softhsm(&temp, CA_KEY, CA_PEM.as_bytes());

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
