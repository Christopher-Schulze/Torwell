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
fn init_hsm_loads_keypair_from_softhsm() {
    let (_dir, _slot) = setup_softhsm("../tests_data/ca.key", "../tests_data/ca.pem");

    let (ctx, pair) = torwell84::secure_http::init_hsm().expect("init_hsm failed");
    let pair = pair.expect("no key pair returned");
    assert!(!pair.key.is_empty() && !pair.cert.is_empty());
    torwell84::secure_http::finalize_hsm(ctx);
}
