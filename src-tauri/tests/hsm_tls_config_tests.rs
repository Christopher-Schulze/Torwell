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
