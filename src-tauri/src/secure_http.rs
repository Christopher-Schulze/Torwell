use reqwest::{Client, ClientBuilder, Url};
use rustls::crypto::ring::{self, cipher_suite};
use rustls::crypto::{self, CryptoProvider};
use rustls::pki_types::CertificateDer;
use rustls::suites::CipherSuite;
use rustls::version::{TLS12, TLS13};
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile as pemfile;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Location of the pinned server certificate. The path is relative to the
/// repository root so updates persist across runs.
pub const DEFAULT_CERT_PATH: &str = "src-tauri/certs/server.pem";

/// Default URL for retrieving updated certificates
pub const DEFAULT_CERT_URL: &str = "https://example.com/certs/server.pem";

/// Default location of the certificate configuration file
pub const DEFAULT_CONFIG_PATH: &str = "src-tauri/certs/cert_config.json";

#[derive(Deserialize)]
struct CertConfig {
    #[serde(default = "default_cert_path")]
    cert_path: String,
    #[serde(default = "default_cert_url")]
    cert_url: String,
    #[serde(default = "default_min_tls_version")]
    min_tls_version: Option<String>,
}

fn default_cert_path() -> String {
    DEFAULT_CERT_PATH.to_string()
}

fn default_cert_url() -> String {
    DEFAULT_CERT_URL.to_string()
}

fn default_min_tls_version() -> Option<String> {
    Some("1.2".to_string())
}

impl CertConfig {
    fn load<P: AsRef<Path>>(path: P) -> Self {
        let data = std::fs::read_to_string(path).ok();
        if let Some(data) = data {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}

impl Default for CertConfig {
    fn default() -> Self {
        Self {
            cert_path: DEFAULT_CERT_PATH.to_string(),
            cert_url: DEFAULT_CERT_URL.to_string(),
            min_tls_version: default_min_tls_version(),
        }
    }
}

fn parse_max_age(header: &str) -> Option<u64> {
    header.split(';').find_map(|part| {
        let part = part.trim();
        part.strip_prefix("max-age=")?.parse().ok()
    })
}

fn parse_tls_version(v: Option<&str>) -> reqwest::tls::Version {
    match v.unwrap_or("1.2") {
        "1.3" => reqwest::tls::Version::TLS_1_3,
        _ => reqwest::tls::Version::TLS_1_2,
    }
}

fn strong_provider(min_tls: reqwest::tls::Version) -> CryptoProvider {
    let mut provider = ring::default_provider();
    provider.cipher_suites.retain(|suite| {
        let allowed = matches!(
            suite.common().suite,
            CipherSuite::TLS13_AES_256_GCM_SHA384
                | CipherSuite::TLS13_CHACHA20_POLY1305_SHA256
                | CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
                | CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
                | CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
                | CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
        );
        if !allowed {
            return false;
        }
        if matches!(min_tls, reqwest::tls::Version::TLS_1_3) {
            suite.common().versions.contains(&TLS13)
        } else {
            true
        }
    });
    provider
}

pub struct SecureHttpClient {
    client: Arc<Mutex<Client>>,
    cert_path: String,
    hsts: Arc<Mutex<HashMap<String, Instant>>>,
    min_tls: reqwest::tls::Version,
}

impl Clone for SecureHttpClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            cert_path: self.cert_path.clone(),
            hsts: self.hsts.clone(),
            min_tls: self.min_tls,
        }
    }
}

impl SecureHttpClient {
    #[cfg(test)]
    pub(crate) fn build_tls_config<P: AsRef<Path>>(path: P) -> anyhow::Result<ClientConfig> {
        Self::build_tls_config_with_min_tls(path, reqwest::tls::Version::TLS_1_2)
    }

    fn build_tls_config_with_min_tls<P: AsRef<Path>>(
        path: P,
        min_tls: reqwest::tls::Version,
    ) -> anyhow::Result<ClientConfig> {
        let mut store = RootCertStore::empty();
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let certs: Vec<CertificateDer<'static>> =
            pemfile::certs(&mut reader).collect::<Result<_, _>>()?;
        store.add_parsable_certificates(certs);

        let builder = ClientConfig::builder_with_provider(Arc::new(strong_provider(min_tls)));
        let builder = if matches!(min_tls, reqwest::tls::Version::TLS_1_3) {
            builder.with_protocol_versions(&[&TLS13])?
        } else {
            builder.with_protocol_versions(&[&TLS12, &TLS13])?
        };

        let mut config = builder.with_root_certificates(store).with_no_client_auth();

        config.enable_ocsp_stapling = true;
        Ok(config)
    }

    fn build_client<P: AsRef<Path>>(
        path: P,
        min_tls: reqwest::tls::Version,
    ) -> anyhow::Result<Client> {
        let config = Self::build_tls_config_with_min_tls(&path, min_tls)?;

        let client = ClientBuilder::new()
            .use_preconfigured_tls(config)
            .https_only(true)
            .min_tls_version(min_tls)
            .build()?;
        Ok(client)
    }

    pub fn new<P: AsRef<Path>>(cert_path: P) -> anyhow::Result<Self> {
        Self::new_with_min_tls(cert_path, reqwest::tls::Version::TLS_1_2)
    }

    pub fn new_with_min_tls<P: AsRef<Path>>(
        cert_path: P,
        min_tls: reqwest::tls::Version,
    ) -> anyhow::Result<Self> {
        let path = cert_path.as_ref().to_owned();
        let client = Self::build_client(&path, min_tls)?;
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            cert_path: path.to_string_lossy().to_string(),
            hsts: Arc::new(Mutex::new(HashMap::new())),
            min_tls,
        })
    }

    pub fn new_default() -> anyhow::Result<Self> {
        Self::new(DEFAULT_CERT_PATH)
    }

    /// Initialize a client using settings from a configuration file and
    /// optionally start periodic updates.
    pub async fn init<P: AsRef<Path>>(
        config_path: P,
        cert_path: Option<String>,
        cert_url: Option<String>,
        interval: Option<Duration>,
    ) -> anyhow::Result<Arc<Self>> {
        let mut cfg = CertConfig::load(config_path);

        if let Ok(env_url) = std::env::var("TORWELL_CERT_URL") {
            cfg.cert_url = env_url;
        }

        if let Some(path) = cert_path {
            cfg.cert_path = path;
        }
        if let Some(url) = cert_url {
            cfg.cert_url = url;
        }

        if cfg.cert_url.contains("example.com") {
            log::warn!(
                "certificate update URL still points to example.com; update cert_url in cert_config.json"
            );
        }
        let min_tls = parse_tls_version(cfg.min_tls_version.as_deref());
        let client = Arc::new(Self::new_with_min_tls(&cfg.cert_path, min_tls)?);

        // Always try to refresh certificates on startup using the currently
        // pinned certificate for validation.
        let url = cfg.cert_url.clone();
        if let Err(e) = client.update_certificates(&url).await {
            log::error!("initial certificate update failed: {}", e);
        }

        if let Some(int) = interval {
            client.clone().schedule_updates(url, int);
        }
        Ok(client)
    }

    async fn get_with_hsts_check(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        let mut parsed = Url::parse(url)?;

        if parsed.scheme() == "http" {
            if let Some(host) = parsed.host_str() {
                let upgrade = {
                    let map = self.hsts.lock().await;
                    map.get(host)
                        .map(|exp| *exp > Instant::now())
                        .unwrap_or(false)
                };
                if upgrade {
                    parsed.set_scheme("https").ok();
                }
            }
        }

        // clone the client to avoid holding the lock while awaiting network I/O
        let client = {
            let guard = self.client.lock().await;
            guard.clone()
        };

        let resp = client.get(parsed.clone()).send().await?;

        if let Some(hsts) = resp.headers().get("strict-transport-security") {
            if let Ok(val) = hsts.to_str() {
                if let Some(max_age) = parse_max_age(val) {
                    if let Some(host) = resp.url().host_str() {
                        let expiry = Instant::now() + Duration::from_secs(max_age);
                        let mut map = self.hsts.lock().await;
                        map.insert(host.to_string(), expiry);
                    }
                }
            }
        } else {
            log::warn!("HSTS header missing for {}", resp.url());
        }
        Ok(resp)
    }

    pub async fn get_text(&self, url: &str) -> reqwest::Result<String> {
        let resp = self.get_with_hsts_check(url).await?;
        resp.text().await
    }

    pub async fn reload_certificates(&self) -> anyhow::Result<()> {
        let client = Self::build_client(&self.cert_path, self.min_tls)?;
        let mut guard = self.client.lock().await;
        *guard = client;
        Ok(())
    }

    pub async fn update_certificates(&self, url: &str) -> anyhow::Result<()> {
        match self.get_with_hsts_check(url).await {
            Ok(resp) => {
                let pem = resp.bytes().await?;
                if let Some(parent) = Path::new(&self.cert_path).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&self.cert_path, &pem)?;
                self.reload_certificates().await?;
            }
            Err(e) => {
                log::error!("failed to fetch new certificate: {}", e);
            }
        }
        Ok(())
    }

    pub fn schedule_updates(self: Arc<Self>, url: String, interval: Duration) {
        tokio::spawn(async move {
            loop {
                if let Err(e) = self.update_certificates(&url).await {
                    log::error!("certificate update failed: {}", e);
                }
                tokio::time::sleep(interval).await;
            }
        });
    }
}
