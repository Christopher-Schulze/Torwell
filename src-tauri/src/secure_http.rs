use crate::core::executor::{TaskError, TaskScheduler};
use crate::error::Error;
use anyhow::anyhow;
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use reqwest::{Client, ClientBuilder, Url};
use rustls::crypto::ring::{self, cipher_suite};
use rustls::crypto::{self, CryptoProvider};
use rustls::pki_types::CertificateDer;
use rustls::suites::CipherSuite;
use rustls::version::{TLS12, TLS13};
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile as pemfile;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::BufReader;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use urlencoding::encode;

#[cfg(feature = "hsm")]
use pkcs11::Ctx;

/// Location of the pinned server certificate. The path is relative to the
/// repository root so updates persist across runs.
pub const DEFAULT_CERT_PATH: &str = "src-tauri/certs/server.pem";

/// Default URL for retrieving updated certificates
pub const DEFAULT_CERT_URL: &str = "https://certs.torwell.com/server.pem";

/// Default location of the certificate configuration file
pub const DEFAULT_CONFIG_PATH: &str = "src-tauri/certs/cert_config.json";

#[derive(Deserialize)]
struct CertConfig {
    #[serde(default = "default_cert_path")]
    cert_path: String,
    #[serde(default)]
    cert_path_windows: Option<String>,
    #[serde(default)]
    cert_path_macos: Option<String>,
    #[serde(default = "default_cert_url")]
    cert_url: String,
    #[serde(default)]
    fallback_cert_url: Option<String>,
    #[serde(default = "default_min_tls_version")]
    min_tls_version: Option<String>,
    #[serde(default = "default_update_interval")]
    update_interval: u64,
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

fn default_update_interval() -> u64 {
    60 * 60 * 24
}

impl CertConfig {
    fn load<P: AsRef<Path>>(path: P) -> Self {
        let data = std::fs::read_to_string(path).ok();
        let mut cfg = if let Some(data) = data {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        };

        cfg.apply_platform_path();
        cfg
    }

    fn apply_platform_path(&mut self) {
        #[cfg(target_os = "windows")]
        if let Some(p) = &self.cert_path_windows {
            self.cert_path = p.clone();
        }
        #[cfg(target_os = "macos")]
        if let Some(p) = &self.cert_path_macos {
            self.cert_path = p.clone();
        }
    }
}

impl Default for CertConfig {
    fn default() -> Self {
        Self {
            cert_path: DEFAULT_CERT_PATH.to_string(),
            cert_path_windows: Some("%APPDATA%\\Torwell84\\server.pem".into()),
            cert_path_macos: Some("/Library/Application Support/Torwell84/server.pem".into()),
            cert_url: DEFAULT_CERT_URL.to_string(),
            fallback_cert_url: None,
            min_tls_version: default_min_tls_version(),
            update_interval: default_update_interval(),
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

#[cfg(fuzzing)]
pub fn fuzz_parse_max_age(header: &str) -> Option<u64> {
    parse_max_age(header)
}

#[cfg(fuzzing)]
pub fn fuzz_tls_version(version: Option<&str>) {
    let _ = parse_tls_version(version);
}

#[cfg(feature = "hsm")]
pub struct HsmKeyPair {
    pub key: Vec<u8>,
    pub cert: Vec<u8>,
}

#[cfg(feature = "hsm")]
pub(crate) fn init_hsm() -> anyhow::Result<(Ctx, Option<HsmKeyPair>)> {
    use base64::{engine::general_purpose, Engine as _};
    use pkcs11::types::*;

    let module = std::env::var("TORWELL_HSM_LIB")
        .unwrap_or_else(|_| "/usr/lib/softhsm/libsofthsm2.so".into());
    let slot_var = std::env::var("TORWELL_HSM_SLOT").unwrap_or_else(|_| "0".into());
    let slot: CK_SLOT_ID = slot_var
        .parse()
        .map_err(|_| anyhow!("invalid TORWELL_HSM_SLOT value: {}", slot_var))?;
    let pin = std::env::var("TORWELL_HSM_PIN").unwrap_or_else(|_| "1234".into());
    if pin.is_empty() {
        return Err(anyhow!("invalid HSM PIN"));
    }
    let key_label = std::env::var("TORWELL_HSM_KEY_LABEL").unwrap_or_else(|_| "tls-key".into());
    let cert_label = std::env::var("TORWELL_HSM_CERT_LABEL").unwrap_or_else(|_| "tls-cert".into());

    // Allow tests to bypass actual HSM access
    if let (Ok(kb64), Ok(cb64)) = (
        std::env::var("TORWELL_HSM_MOCK_KEY"),
        std::env::var("TORWELL_HSM_MOCK_CERT"),
    ) {
        let mut ctx = Ctx::new(module)?;
        ctx.initialize(None)?;
        return Ok((
            ctx,
            Some(HsmKeyPair {
                key: general_purpose::STANDARD.decode(kb64)?,
                cert: general_purpose::STANDARD.decode(cb64)?,
            }),
        ));
    }

    let mut ctx = Ctx::new(module.clone())?;
    ctx.initialize(None)?;

    use pkcs11::errors::Error as Pkcs11Error;
    let session = match ctx.open_session(slot, CKF_SERIAL_SESSION | CKF_RW_SESSION, None, None) {
        Ok(s) => s,
        Err(Pkcs11Error::Pkcs11(rv))
            if rv == CKR_SLOT_ID_INVALID || rv == CKR_TOKEN_NOT_PRESENT =>
        {
            return Err(anyhow!("invalid HSM slot: {}", slot));
        }
        Err(e) => return Err(anyhow!(e)),
    };
    if let Err(e) = ctx.login(session, CKU_USER, Some(&pin)) {
        let _ = ctx.close_session(session);
        return match e {
            Pkcs11Error::Pkcs11(rv)
                if rv == CKR_PIN_INCORRECT || rv == CKR_PIN_INVALID || rv == CKR_PIN_LEN_RANGE =>
            {
                Err(anyhow!("invalid HSM PIN"))
            }
            other => Err(anyhow!(other)),
        };
    }

    let mut tmpl = vec![
        CK_ATTRIBUTE::new(CKA_CLASS).with_ck_ulong(&CKO_PRIVATE_KEY),
        CK_ATTRIBUTE::new(CKA_LABEL).with_string(&key_label),
    ];
    ctx.find_objects_init(session, &tmpl)?;
    let objs = ctx.find_objects(session, 1)?;
    ctx.find_objects_final(session)?;
    let key = if let Some(obj) = objs.get(0) {
        let mut attr = vec![CK_ATTRIBUTE::new(CKA_VALUE)];
        let (_, attrs) = ctx.get_attribute_value(session, *obj, &mut attr)?;
        attrs[0].get_bytes().unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut tmpl = vec![
        CK_ATTRIBUTE::new(CKA_CLASS).with_ck_ulong(&CKO_CERTIFICATE),
        CK_ATTRIBUTE::new(CKA_LABEL).with_string(&cert_label),
    ];
    ctx.find_objects_init(session, &tmpl)?;
    let objs = ctx.find_objects(session, 1)?;
    ctx.find_objects_final(session)?;
    let cert = if let Some(obj) = objs.get(0) {
        let mut attr = vec![CK_ATTRIBUTE::new(CKA_VALUE)];
        let (_, attrs) = ctx.get_attribute_value(session, *obj, &mut attr)?;
        attrs[0].get_bytes().unwrap_or_default()
    } else {
        Vec::new()
    };

    let _ = ctx.logout(session);
    let _ = ctx.close_session(session);

    Ok((ctx, Some(HsmKeyPair { key, cert })))
}

#[cfg(feature = "hsm")]
pub(crate) fn finalize_hsm(mut ctx: Ctx) {
    let _ = ctx.finalize();
}

fn strong_provider(min_tls: reqwest::tls::Version) -> CryptoProvider {
    let mut provider = ring::default_provider();
    provider.cipher_suites.retain(|suite| {
        let strong = matches!(
            suite.common().suite,
            CipherSuite::TLS13_AES_256_GCM_SHA384
                | CipherSuite::TLS13_CHACHA20_POLY1305_SHA256
                | CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
                | CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
                | CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
                | CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
        );
        if !strong {
            return false;
        }

        let has_tls13 = suite.common().versions.contains(&TLS13);
        let has_tls12 = suite.common().versions.contains(&TLS12);
        match min_tls {
            reqwest::tls::Version::TLS_1_3 => has_tls13,
            _ => has_tls12 || has_tls13,
        }
    });
    provider
}

pub struct SecureHttpClient {
    client: Arc<Mutex<Client>>,
    cert_path: String,
    hsts: Arc<Mutex<HashMap<String, Instant>>>,
    insecure_hosts: Arc<RwLock<HashSet<String>>>,
    min_tls: reqwest::tls::Version,
    warning_cb: Arc<Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>,
    pending_warnings: Arc<Mutex<Vec<String>>>,
    update_failures: Arc<Mutex<u32>>,
    update_backoff: Arc<Mutex<Option<Duration>>>,
    worker_urls: Arc<Mutex<VecDeque<String>>>,
    worker_token: Arc<Mutex<Option<String>>>,
    update_task: Mutex<Option<JoinHandle<()>>>,
    security_warning_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    scheduler: TaskScheduler,
}

impl Clone for SecureHttpClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            cert_path: self.cert_path.clone(),
            hsts: self.hsts.clone(),
            insecure_hosts: self.insecure_hosts.clone(),
            min_tls: self.min_tls,
            warning_cb: self.warning_cb.clone(),
            pending_warnings: self.pending_warnings.clone(),
            update_failures: self.update_failures.clone(),
            update_backoff: self.update_backoff.clone(),
            worker_urls: self.worker_urls.clone(),
            worker_token: self.worker_token.clone(),
            update_task: Mutex::new(None),
            security_warning_limiter: self.security_warning_limiter.clone(),
            scheduler: self.scheduler.clone(),
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
        #[cfg(feature = "hsm")]
        let hsm = match init_hsm() {
            Ok(ctx) => Some(ctx),
            Err(e) => return Err(anyhow!("failed to initialize HSM: {e}")),
        };

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

        let mut config = if let Some((ctx, pair)) = hsm {
            let mut cfg = if let Some(keys) = pair {
                use rustls::pki_types::{CertificateDer, PrivateKeyDer};
                if !keys.key.is_empty() && !keys.cert.is_empty() {
                    let certs = vec![CertificateDer::from(keys.cert).into_owned()];
                    let key = PrivateKeyDer::try_from(keys.key)?;
                    builder
                        .with_root_certificates(store.clone())
                        .with_client_auth_cert(certs, key)?
                } else {
                    builder
                        .with_root_certificates(store.clone())
                        .with_no_client_auth()
                }
            } else {
                builder
                    .with_root_certificates(store.clone())
                    .with_no_client_auth()
            };
            finalize_hsm(ctx);
            cfg
        } else {
            builder
                .with_root_certificates(store.clone())
                .with_no_client_auth()
        };

        config.enable_ocsp_stapling = true;
        Ok(config)
    }

    fn build_client<P: AsRef<Path>>(
        path: P,
        min_tls: reqwest::tls::Version,
    ) -> anyhow::Result<Client> {
        // Enforce TLS 1.2 as the minimum supported protocol regardless of
        // external configuration. Lower values are bumped to TLS 1.2.
        let enforced = match min_tls {
            reqwest::tls::Version::TLS_1_0 | reqwest::tls::Version::TLS_1_1 => {
                reqwest::tls::Version::TLS_1_2
            }
            _ => min_tls,
        };

        let config = Self::build_tls_config_with_min_tls(&path, enforced)?;

        let client = ClientBuilder::new()
            .use_preconfigured_tls(config)
            .https_only(true)
            .min_tls_version(enforced)
            .build()?;
        Ok(client)
    }

    pub fn new<P: AsRef<Path>>(cert_path: P) -> anyhow::Result<Self> {
        Self::new_with_min_tls(cert_path, reqwest::tls::Version::TLS_1_2)
    }

    fn map_scheduler_error(context: &str, err: TaskError) -> anyhow::Error {
        match err {
            TaskError::Canceled => anyhow!(format!("{context} cancelled by scheduler")),
            TaskError::Panicked { message, .. } => {
                anyhow!(format!("{context} panicked: {message}"))
            }
        }
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
            insecure_hosts: Arc::new(RwLock::new(HashSet::new())),
            min_tls,
            warning_cb: Arc::new(Mutex::new(None)),
            pending_warnings: Arc::new(Mutex::new(Vec::new())),
            update_failures: Arc::new(Mutex::new(0)),
            update_backoff: Arc::new(Mutex::new(None)),
            worker_urls: Arc::new(Mutex::new(VecDeque::new())),
            worker_token: Arc::new(Mutex::new(None)),
            update_task: Mutex::new(None),
            security_warning_limiter: Arc::new(RateLimiter::direct(Quota::per_minute(
                NonZeroU32::new(6).unwrap(),
            ))),
            scheduler: TaskScheduler::global(),
        })
    }

    pub fn new_default() -> anyhow::Result<Self> {
        Self::new(DEFAULT_CERT_PATH)
    }

    /// Provide a callback to emit security warnings
    pub async fn set_warning_callback<F>(&self, cb: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        let mut guard = self.warning_cb.lock().await;
        *guard = Some(Box::new(cb));
        if let Some(callback) = guard.as_ref() {
            let mut pending = self.pending_warnings.lock().await;
            for msg in pending.drain(..) {
                callback(msg);
            }
        }
    }

    /// Configure proxy workers and authentication token
    pub async fn set_worker_config(&self, workers: Vec<String>, token: Option<String>) {
        *self.worker_urls.lock().await = VecDeque::from(workers);
        *self.worker_token.lock().await = token;
    }

    pub fn set_insecure_hosts(&self, hosts: Vec<String>) {
        let prepared = Self::prepare_insecure_hosts(hosts);
        let mut guard = self
            .insecure_hosts
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        *guard = prepared;
    }

    pub fn insecure_hosts(&self) -> Vec<String> {
        let guard = self
            .insecure_hosts
            .read()
            .unwrap_or_else(|poison| poison.into_inner());
        guard.iter().cloned().collect()
    }

    fn prepare_insecure_hosts(hosts: Vec<String>) -> HashSet<String> {
        let mut prepared = HashSet::new();
        for entry in hosts {
            for host in Self::normalize_host(&entry) {
                if !host.is_empty() {
                    prepared.insert(host);
                }
            }
        }
        prepared
    }

    fn normalize_host(entry: &str) -> Vec<String> {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }
        let candidate = if trimmed.contains("://") {
            trimmed.to_string()
        } else {
            format!("http://{}", trimmed)
        };
        if let Ok(url) = Url::parse(&candidate) {
            if let Some(host) = url.host_str() {
                let host_lower = host.to_lowercase();
                let mut result = vec![host_lower.clone()];
                if let Some(port) = url.port() {
                    result.push(format!("{}:{}", host_lower, port));
                }
                result
            } else {
                Vec::new()
            }
        } else {
            vec![trimmed.trim_matches('/').to_lowercase()]
        }
    }

    fn is_http_allowed(&self, host: &str, port: Option<u16>) -> bool {
        let guard = self
            .insecure_hosts
            .read()
            .unwrap_or_else(|poison| poison.into_inner());
        let host_key = host.to_lowercase();
        if guard.contains(&host_key) {
            return true;
        }
        if let Some(port) = port {
            let key = format!("{}:{}", host_key, port);
            if guard.contains(&key) {
                return true;
            }
        }
        false
    }

    async fn notify_http_attempt(&self, host: &str, url: &Url, allowed: bool) {
        let message = if allowed {
            format!(
                "Allowing insecure HTTP request to {} ({}) via allowlist",
                host, url
            )
        } else {
            format!("Blocked insecure HTTP request to {} ({})", host, url)
        };

        log::warn!("{}", message);

        if self.security_warning_limiter.check().is_ok() {
            self.emit_warning(message).await;
        }
    }

    /// Update HSM library path and slot then reload TLS configuration
    pub async fn set_hsm_config(
        &self,
        lib: Option<String>,
        slot: Option<u64>,
    ) -> anyhow::Result<()> {
        if let Some(l) = lib {
            std::env::set_var("TORWELL_HSM_LIB", &l);
        } else {
            std::env::remove_var("TORWELL_HSM_LIB");
        }
        if let Some(s) = slot {
            std::env::set_var("TORWELL_HSM_SLOT", s.to_string());
        } else {
            std::env::remove_var("TORWELL_HSM_SLOT");
        }
        self.reload_certificates().await
    }

    async fn emit_warning(&self, msg: String) {
        if let Some(cb) = self.warning_cb.lock().await.as_ref() {
            cb(msg);
        } else {
            self.pending_warnings.lock().await.push(msg);
        }
    }

    async fn handle_hsts_header(&self, resp: &reqwest::Response) {
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
            self.emit_warning(format!("HSTS header missing for {}", resp.url()))
                .await;
        }
    }

    async fn normalize_request_url(&self, url: &str) -> Result<Url, Error> {
        let mut parsed = Url::parse(url).map_err(|e| Error::ConfigError {
            step: "secure_http::get_with_hsts_check".into(),
            source: format!("invalid url {url}: {e}"),
            backtrace: None,
        })?;

        self.enforce_transport_policy(url, &mut parsed).await?;

        Ok(parsed)
    }

    async fn enforce_transport_policy(
        &self,
        original: &str,
        parsed: &mut Url,
    ) -> Result<(), Error> {
        if parsed.scheme() != "http" {
            return Ok(());
        }

        let host = parsed.host_str().ok_or_else(|| Error::ConfigError {
            step: "secure_http::get_with_hsts_check".into(),
            source: format!("missing host for url {original}"),
            backtrace: None,
        })?;

        let port = parsed.port();
        if !self.is_http_allowed(host, port) {
            self.notify_http_attempt(host, parsed, false).await;
            return Err(Error::InsecureScheme {
                host: host.to_string(),
                url: parsed.to_string(),
            });
        }

        self.notify_http_attempt(host, parsed, true).await;

        let upgrade = {
            let map = self.hsts.lock().await;
            map.get(host)
                .map(|exp| *exp > Instant::now())
                .unwrap_or(false)
        };
        if upgrade {
            parsed.set_scheme("https").ok();
        }

        Ok(())
    }

    /// Initialize a client using settings from a configuration file and
    /// optionally start periodic updates.
    pub async fn init<P: AsRef<Path>>(
        config_path: P,
        cert_path: Option<String>,
        cert_url: Option<String>,
        fallback_cert_url: Option<String>,
        interval: Option<Duration>,
    ) -> anyhow::Result<Arc<Self>> {
        let mut cfg = CertConfig::load(config_path);

        if let Ok(env_url) = std::env::var("TORWELL_CERT_URL") {
            cfg.cert_url = env_url;
        }
        if let Ok(env_path) = std::env::var("TORWELL_CERT_PATH") {
            cfg.cert_path = env_path;
        }
        if let Ok(env_fallback) = std::env::var("TORWELL_FALLBACK_CERT_URL") {
            cfg.fallback_cert_url = Some(env_fallback);
        }
        if let Ok(env_int) = std::env::var("TORWELL_UPDATE_INTERVAL") {
            if let Ok(sec) = env_int.parse::<u64>() {
                cfg.update_interval = sec;
            }
        }

        if let Some(path) = cert_path {
            cfg.cert_path = path;
        }
        if let Some(url) = cert_url {
            cfg.cert_url = url;
        }
        if let Some(fallback) = fallback_cert_url {
            cfg.fallback_cert_url = Some(fallback);
        }
        let update_interval = interval.unwrap_or_else(|| Duration::from_secs(cfg.update_interval));

        if cfg.cert_url.contains("example.com") {
            log::warn!(
                "certificate update URL still points to example.com; update cert_url in cert_config.json"
            );
        }
        let min_tls = parse_tls_version(cfg.min_tls_version.as_deref());
        let client = Arc::new(Self::new_with_min_tls(&cfg.cert_path, min_tls)?);

        if cfg.cert_url.contains("example.com") {
            client
                .emit_warning(
                    "Certificate URL still points to example.com; update cert_url in cert_config.json".to_string(),
                )
                .await;
        }

        // Always try to refresh certificates on startup using the currently
        // pinned certificate for validation.
        let mut urls = vec![cfg.cert_url.clone()];
        if let Some(ref fb) = cfg.fallback_cert_url {
            urls.push(fb.clone());
        }

        if let Err(e) = client.update_certificates_from(&urls).await {
            log::error!("initial certificate update failed: {}", e);
        }

        if update_interval.as_secs() > 0 {
            client.clone().schedule_updates(urls, update_interval).await;
        }
        Ok(client)
    }

    async fn get_with_hsts_check(&self, url: &str) -> Result<reqwest::Response, Error> {
        let parsed = self.normalize_request_url(url).await?;

        // clone the client to avoid holding the lock while awaiting network I/O
        let client = {
            let guard = self.client.lock().await;
            guard.clone()
        };

        let token = { self.worker_token.lock().await.clone() };
        let worker_count = { self.worker_urls.lock().await.len() };
        for _ in 0..worker_count {
            let worker = {
                let mut guard = self.worker_urls.lock().await;
                guard.pop_front().map(|w| {
                    guard.push_back(w.clone());
                    w
                })
            };
            if let Some(w) = worker {
                let mut target = w.clone();
                let encoded = encode(parsed.as_str());
                if target.contains('?') {
                    target.push('&');
                } else {
                    target.push('?');
                }
                target.push_str("url=");
                target.push_str(&encoded);
                let mut req = client.get(target);
                if let Some(tok) = token.as_ref() {
                    req = req.header("X-Proxy-Token", tok);
                }
                match req.send().await {
                    Ok(resp) => {
                        self.handle_hsts_header(&resp).await;
                        return Ok(resp);
                    }
                    Err(e) => {
                        log::warn!("worker {} unreachable: {}", w, e);
                    }
                }
            }
        }
        let resp = client
            .get(parsed.clone())
            .send()
            .await
            .map_err(Error::from)?;
        self.handle_hsts_header(&resp).await;
        Ok(resp)
    }

    pub async fn get_text(&self, url: &str) -> Result<String, Error> {
        let resp = self.get_with_hsts_check(url).await?;
        resp.text().await.map_err(Error::from)
    }

    /// Send JSON data to an HTTP endpoint using the pinned TLS configuration.
    pub async fn post_json(&self, url: &str, body: &Value) -> Result<(), Error> {
        let normalized = self.normalize_request_url(url).await?;
        let normalized_str = normalized.as_str().to_string();
        let client = { self.client.lock().await.clone() };
        let token = { self.worker_token.lock().await.clone() };
        let worker_count = { self.worker_urls.lock().await.len() };
        for _ in 0..worker_count {
            let worker = {
                let mut guard = self.worker_urls.lock().await;
                guard.pop_front().map(|w| {
                    guard.push_back(w.clone());
                    w
                })
            };
            if let Some(w) = worker {
                let mut target = w.clone();
                let encoded = encode(&normalized_str);
                if target.contains('?') {
                    target.push('&');
                } else {
                    target.push('?');
                }
                target.push_str("url=");
                target.push_str(&encoded);
                let mut req = client.post(target).json(body);
                if let Some(tok) = token.as_ref() {
                    req = req.header("X-Proxy-Token", tok);
                }
                match req.send().await {
                    Ok(resp) => {
                        self.handle_hsts_header(&resp).await;
                        return Ok(());
                    }
                    Err(e) => {
                        log::warn!("worker {} unreachable: {}", w, e);
                    }
                }
            }
        }
        let resp = client
            .post(normalized)
            .json(body)
            .send()
            .await
            .map_err(Error::from)?;
        self.handle_hsts_header(&resp).await;
        Ok(())
    }

    pub async fn reload_certificates(&self) -> anyhow::Result<()> {
        let path = self.cert_path.clone();
        let min_tls = self.min_tls;
        let scheduler = self.scheduler.clone();
        let client = scheduler
            .spawn("reload_certificates", move || {
                Self::build_client(&path, min_tls)
            })
            .await
            .map_err(|err| Self::map_scheduler_error("reload_certificates", err))??;
        let mut guard = self.client.lock().await;
        *guard = client;
        Ok(())
    }

    pub async fn update_certificates(&self, url: &str) -> anyhow::Result<()> {
        let resp = self.get_with_hsts_check(url).await?;
        let pem = resp.bytes().await?.to_vec();
        let path = self.cert_path.clone();
        let scheduler = self.scheduler.clone();
        scheduler
            .spawn("write_certificates", move || -> anyhow::Result<()> {
                if let Some(parent) = Path::new(&path).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, &pem)?;
                Ok(())
            })
            .await
            .map_err(|err| Self::map_scheduler_error("write_certificates", err))??;
        self.reload_certificates().await?;
        Ok(())
    }

    pub async fn update_certificates_from(&self, urls: &[String]) -> anyhow::Result<()> {
        for url in urls {
            match self.update_certificates(url).await {
                Ok(_) => {
                    let mut cnt = self.update_failures.lock().await;
                    *cnt = 0;
                    *self.update_backoff.lock().await = None;
                    return Ok(());
                }
                Err(e) => {
                    log::error!("failed to fetch new certificate from {}: {}", url, e);
                }
            }
        }
        {
            let mut cnt = self.update_failures.lock().await;
            *cnt += 1;
            if *cnt >= 3 {
                self.emit_warning(format!("{} consecutive certificate update failures", *cnt))
                    .await;
                *self.update_backoff.lock().await = Some(Duration::from_secs(60 * 60));
            }
        }
        Err(anyhow::anyhow!("all certificate update attempts failed"))
    }

    pub async fn schedule_updates(self: Arc<Self>, urls: Vec<String>, interval: Duration) {
        let mut guard = self.update_task.lock().await;
        if let Some(handle) = guard.take() {
            handle.abort();
        }
        let client = self.clone();
        let handle = tokio::spawn(async move {
            loop {
                if let Err(e) = client.update_certificates_from(&urls).await {
                    log::error!("certificate update failed: {}", e);
                }
                let extra = {
                    let mut guard = client.update_backoff.lock().await;
                    guard.take().unwrap_or_else(|| Duration::from_secs(0))
                };
                tokio::time::sleep(interval + extra).await;
            }
        });
        *guard = Some(handle);
    }
}
