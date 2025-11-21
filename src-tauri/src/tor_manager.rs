use crate::commands::RelayInfo;
use crate::error::{ConnectionStep, Error, Result};
use arti_client::config::{
    TorClientConfigBuilder,
};
use arti_client::{StreamPrefs, TorClient, TorClientConfig};
use async_trait::async_trait;
use chrono::Utc;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU32;
use std::path::Path;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use toml;
use tor_circmgr::isolation::{IsolationToken, StreamIsolation};
use tor_dirmgr::Timeliness;
use tor_geoip::{CountryCode, GeoipDb};
use std::time::Instant;

#[cfg(test)]
pub(crate) static GEOIP_INIT_COUNT: AtomicUsize = AtomicUsize::new(0);

static GEOIP_DB: Lazy<Arc<GeoipDb>> = Lazy::new(|| {
    #[cfg(test)]
    {
        GEOIP_INIT_COUNT.fetch_add(1, Ordering::SeqCst);
    }
    GeoipDb::new_embedded()
});

fn load_geoip_db(dir: &Path) -> Option<Arc<GeoipDb>> {
    let v4 = std::fs::read_to_string(dir.join("geoip")).ok()?;
    let v6 = std::fs::read_to_string(dir.join("geoip6")).ok()?;
    GeoipDb::new_from_legacy_format(&v4, &v6).ok().map(Arc::new)
}

/// Helper to log an error for a given step and convert it into an
/// [`Error::ConnectionFailed`] variant.
fn log_and_convert_error(step: ConnectionStep, err: impl ToString) -> Error {
    let msg = err.to_string();
    log::error!("{}: {}", step, msg);
    Error::ConnectionFailed {
        step,
        source_message: msg,
        backtrace: format!("{:?}", std::backtrace::Backtrace::capture()),
    }
}
use tor_rtcompat::PreferredRuntime;

const INITIAL_BACKOFF: std::time::Duration = std::time::Duration::from_secs(1);
const MAX_BACKOFF: std::time::Duration = std::time::Duration::from_secs(30);
const CONNECT_RATE_LIMIT: u32 = 5;
const CIRCUIT_RATE_LIMIT: u32 = 10;
const MAX_ISOLATION_TOKENS: usize = 100;
const PREWARM_CIRCUIT_COUNT: usize = 3;
const MAX_COUNTRY_MATCH_ATTEMPTS: usize = 5;
const COUNTRY_POLICY_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(250);
const DEFAULT_ROUTE_CODES: &[&str] = &["DE", "NL", "SE"];
const DEFAULT_FAST_COUNTRY_CODES: &[&str] = &[
    "CA", "CH", "DE", "DK", "EE", "FI", "FR", "GB", "IS", "JP", "LT", "LU", "LV", "NL", "NO", "SE",
    "SG", "US",
];

/// Simple traffic statistics returned from [`TorManager::traffic_stats`].
#[derive(Debug, Clone)]
pub struct TrafficStats {
    /// Total bytes sent through the Tor client.
    pub bytes_sent: u64,
    /// Total bytes received through the Tor client.
    pub bytes_received: u64,
}

/// Basic circuit metrics.
#[derive(Debug, Clone)]
pub struct CircuitMetrics {
    /// Number of active circuits.
    pub count: usize,
    /// Age of the oldest circuit in seconds.
    pub oldest_age: u64,
    /// Average circuit creation time in milliseconds.
    pub avg_create_ms: u64,
    /// Number of failed circuit creation attempts.
    pub failed_attempts: u64,
    /// Whether all fields contain real values (`true`) or estimates (`false`).
    pub complete: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct BridgePreset {
    pub name: String,
    pub bridges: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CircuitPolicyReport {
    pub requested_entry: Option<String>,
    pub requested_middle: Option<String>,
    pub requested_exit: Option<String>,
    pub effective_entry: Option<String>,
    pub effective_middle: Option<String>,
    pub effective_exit: Option<String>,
    pub matches_policy: bool,
    pub relays: Vec<RelayInfo>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TorrcProfile {
    pub generated_at: String,
    pub config: String,
    pub entry: String,
    pub middle: String,
    pub exit: String,
    pub requested_entry: Option<String>,
    pub requested_middle: Option<String>,
    pub requested_exit: Option<String>,
    pub fast_fallback: Vec<String>,
    pub bridges: Vec<String>,
    pub fast_only: bool,
}

#[derive(serde::Deserialize)]
struct PresetFile {
    #[serde(default)]
    presets: Vec<BridgePreset>,
}

impl PresetFile {
    fn from_str(s: &str) -> Result<Vec<BridgePreset>> {
        let val: PresetFile = serde_json::from_str(s).map_err(|e| Error::Io(e.to_string()))?;
        Ok(val.presets)
    }
}

pub fn load_default_bridge_presets() -> Result<Vec<BridgePreset>> {
    PresetFile::from_str(include_str!("../../src/lib/bridge_presets.json"))
}

pub fn load_bridge_presets_from_str(data: &str) -> Result<Vec<BridgePreset>> {
    PresetFile::from_str(data)
}

#[derive(Debug)]
pub struct RetryInfo {
    pub attempt: u32,
    pub delay: std::time::Duration,
    pub error: Error,
}

#[derive(Debug, Clone, Default)]
struct CircuitCountryPrefs {
    entry: Option<String>,
    middle: Option<String>,
    exit: Option<String>,
}

impl CircuitCountryPrefs {
    fn is_restricted(&self) -> bool {
        self.entry.is_some() || self.middle.is_some() || self.exit.is_some()
    }

    fn matches(&self, relays: &[RelayInfo]) -> bool {
        if relays.is_empty() {
            return false;
        }

        if let Some(entry) = &self.entry {
            if !relays
                .first()
                .map(|info| info.country.eq_ignore_ascii_case(entry))
                .unwrap_or(false)
            {
                return false;
            }
        }

        if let Some(exit) = &self.exit {
            if !relays
                .last()
                .map(|info| info.country.eq_ignore_ascii_case(exit))
                .unwrap_or(false)
            {
                return false;
            }
        }

        if let Some(middle) = &self.middle {
            if relays.len() < 3 {
                return false;
            }
            let middle_slice = &relays[1..relays.len() - 1];
            if middle_slice.is_empty() {
                return false;
            }
            if !middle_slice
                .iter()
                .all(|info| info.country.eq_ignore_ascii_case(middle))
            {
                return false;
            }
        }

        true
    }
}

#[async_trait]
pub trait TorClientBehavior: Send + Sync + Sized + 'static {
    async fn create_bootstrapped(config: TorClientConfig) -> std::result::Result<Self, String>;
    async fn create_bootstrapped_with_progress<P>(
        config: TorClientConfig,
        progress: &mut P,
    ) -> std::result::Result<Self, String>
    where
        P: FnMut(u8, String) + Send;
    fn retire_all_circs(&self);
    fn build_new_circuit(&self) -> impl std::future::Future<Output = std::result::Result<(), String>> + Send;
    async fn launch_socks(&self, port: u16) -> std::result::Result<u16, String>;
}

#[async_trait]
impl TorClientBehavior for TorClient<PreferredRuntime> {
    async fn create_bootstrapped(config: TorClientConfig) -> std::result::Result<Self, String> {
        TorClient::create_bootstrapped(config)
            .await
            .map_err(|e| e.to_string())
    }

    async fn create_bootstrapped_with_progress<P>(
        config: TorClientConfig,
        progress: &mut P,
    ) -> std::result::Result<Self, String>
    where
        P: FnMut(u8, String) + Send,
    {
        use futures::StreamExt;

        let client = TorClient::builder()
            .config(config)
            .bootstrap_behavior(arti_client::BootstrapBehavior::Manual)
            .create_unbootstrapped_async()
            .await
            .map_err(|e| e.to_string())?;

        {
            let mut events = client.bootstrap_events();
            let mut bootstrap = client.bootstrap();
            tokio::pin!(bootstrap);

            loop {
                tokio::select! {
                    ev = events.next() => {
                        if let Some(ev) = ev {
                            let pct = (ev.as_frac() * 100.0).round() as u8;
                            progress(pct, ev.to_string());
                        } else {
                            break;
                        }
                    }
                    res = &mut bootstrap => {
                        res.map_err(|e| e.to_string())?;
                        break;
                    }
                }
            }
        } // bootstrap is dropped here, releasing the borrow on client

        progress(100, "done".into());

        Ok(client)
    }

    fn retire_all_circs(&self) {
        // TODO: Find a replacement for `retire_all_circs`
        // self.circmgr().retire_all_circs();
    }

    fn build_new_circuit(&self) -> impl std::future::Future<Output = std::result::Result<(), String>> + Send {
        async {
            let _stream = self.connect(("www.google.com", 80)).await.map_err(|e| e.to_string())?;
            Ok(())
        }
    }

    async fn launch_socks(&self, port: u16) -> std::result::Result<u16, String> {
        crate::socks::start_socks_proxy(self.clone(), port)
            .await
            .map_err(|e| e.to_string())
    }
}
pub struct TorManager<C = TorClient<PreferredRuntime>> {
    client: Arc<Mutex<Option<C>>>,
    socks_port: Arc<Mutex<Option<u16>>>,
    isolation_tokens: Arc<Mutex<HashMap<String, (IsolationToken, std::time::Instant)>>>,
    exit_country: Arc<Mutex<Option<CountryCode>>>,
    entry_country: Arc<Mutex<Option<CountryCode>>>,
    middle_country: Arc<Mutex<Option<CountryCode>>>,
    bridges: Arc<Mutex<Vec<String>>>,
    torrc_config: Arc<Mutex<String>>,
    country_cache: Arc<Mutex<HashMap<String, String>>>,
    geoip_db: Arc<GeoipDb>,
    connect_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    circuit_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl<C> Clone for TorManager<C> {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            socks_port: Arc::clone(&self.socks_port),
            isolation_tokens: Arc::clone(&self.isolation_tokens),
            exit_country: Arc::clone(&self.exit_country),
            entry_country: Arc::clone(&self.entry_country),
            middle_country: Arc::clone(&self.middle_country),
            bridges: Arc::clone(&self.bridges),
            torrc_config: Arc::clone(&self.torrc_config),
            country_cache: Arc::clone(&self.country_cache),
            geoip_db: self.geoip_db.clone(),
            connect_limiter: Arc::clone(&self.connect_limiter),
            circuit_limiter: Arc::clone(&self.circuit_limiter),
        }
    }
}

impl<C: TorClientBehavior> TorManager<C> {
    fn normalise_country_code_str(code: &str) -> Option<String> {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return None;
        }
        let upper = trimmed.to_ascii_uppercase();
        upper.parse::<CountryCode>().ok()?;
        Some(upper)
    }

    fn ensure_unique_route(requested: &[Option<String>], fallback: &[String]) -> Vec<String> {
        let mut used: HashSet<String> = HashSet::new();
        let mut result = Vec::with_capacity(requested.len());

        for (idx, candidate) in requested.iter().enumerate() {
            let mut assigned: Option<String> = None;

            if let Some(code) = candidate {
                let upper = code.to_ascii_uppercase();
                if !used.contains(&upper) {
                    used.insert(upper.clone());
                    assigned = Some(upper);
                }
            }

            if assigned.is_none() {
                if let Some(fallback_code) = fallback
                    .iter()
                    .find(|code| !used.contains(&code.to_ascii_uppercase()))
                {
                    let upper = fallback_code.to_ascii_uppercase();
                    used.insert(upper.clone());
                    assigned = Some(upper);
                }
            }

            if assigned.is_none() {
                if let Some(code) = candidate {
                    let upper = code.to_ascii_uppercase();
                    assigned = Some(upper);
                }
            }

            if assigned.is_none() {
                let fallback_idx = fallback.get(idx).cloned().unwrap_or_else(|| {
                    DEFAULT_ROUTE_CODES[idx.min(DEFAULT_ROUTE_CODES.len() - 1)].to_string()
                });
                assigned = Some(fallback_idx.to_ascii_uppercase());
            }

            result.push(assigned.expect("route must resolve"));
        }

        result
    }

    pub fn new() -> Self {
        Self::new_with_geoip::<&Path>(None)
    }

    pub fn new_with_geoip<P: AsRef<Path>>(geoip_dir: Option<P>) -> Self {
        let db = geoip_dir
            .as_ref()
            .and_then(|p| load_geoip_db(p.as_ref()))
            .unwrap_or_else(|| GEOIP_DB.clone());

        let manager = Self {
            client: Arc::new(Mutex::new(None)),
            socks_port: Arc::new(Mutex::new(None)),
            isolation_tokens: Arc::new(Mutex::new(HashMap::new())),
            exit_country: Arc::new(Mutex::new(None)),
            entry_country: Arc::new(Mutex::new(None)),
            middle_country: Arc::new(Mutex::new(None)),
            bridges: Arc::new(Mutex::new(Vec::new())),
            torrc_config: Arc::new(Mutex::new(String::new())),
            country_cache: Arc::new(Mutex::new(HashMap::new())),
            geoip_db: db,
            connect_limiter: Arc::new(RateLimiter::direct(Quota::per_minute(
                NonZeroU32::new(CONNECT_RATE_LIMIT).unwrap(),
            ))),
            circuit_limiter: Arc::new(RateLimiter::direct(Quota::per_minute(
                NonZeroU32::new(CIRCUIT_RATE_LIMIT).unwrap(),
            ))),
        };

        let cleanup_clone = manager.clone();
        tokio::spawn(async move {
            let interval = std::time::Duration::from_secs(60 * 10);
            let max_age = std::time::Duration::from_secs(60 * 60);
            loop {
                tokio::time::sleep(interval).await;
                cleanup_clone.cleanup_isolation_tokens(max_age).await;
            }
        });

        manager
    }

    pub async fn cleanup_isolation_tokens(&self, max_age: std::time::Duration) {
        let now = std::time::Instant::now();
        let mut tokens = self.isolation_tokens.lock().await;
        tokens.retain(|_, (_, ts)| now.duration_since(*ts) <= max_age);
    }

    async fn set_country(
        target: &Arc<Mutex<Option<CountryCode>>>,
        step: &str,
        country: Option<String>,
    ) -> Result<()> {
        let mut guard = target.lock().await;
        if let Some(cc) = country {
            let code = cc.parse::<CountryCode>().map_err(|e| {
                log::error!("{step}: invalid code {cc} - {e}");
                Error::ConfigError {
                    step: step.into(),
                    source_message: e.to_string(),
                    backtrace: format!("{:?}", std::backtrace::Backtrace::capture()),
                }
            })?;
            *guard = Some(code);
        } else {
            *guard = None;
        }
        Ok(())
    }

    async fn build_config(&self) -> Result<TorClientConfig> {
        // Warning check for obfs4proxy
        if let Ok(path_var) = std::env::var("PATH") {
            let found = std::env::split_paths(&path_var)
                .any(|p| p.join("obfs4proxy").exists() || p.join("obfs4proxy.exe").exists());
            if !found {
                log::warn!("obfs4proxy binary not found in PATH. Obfuscation may fail.");
            } else {
                log::info!("obfs4proxy binary found in PATH.");
            }
        }

        let mut torrc = self.torrc_config.lock().await.clone();
        let bridges = self.get_bridges().await;

        // Inject bridges into torrc string to avoid builder API complexity
        if !bridges.is_empty() {
            torrc.push_str("\n[bridges]\n");
            torrc.push_str("enabled = true\n");
            torrc.push_str("bridges = [\n");
            for bridge in &bridges {
                // Escape quotes if necessary, but assuming bridge lines are somewhat sane
                torrc.push_str(&format!("  \"{}\",\n", bridge.replace('"', "\\\"")));
            }
            torrc.push_str("]\n");

            // Add default obfs4 transport if needed
            // We check if any bridge line contains "obfs4"
            if bridges.iter().any(|b| b.contains("obfs4")) {
                torrc.push_str("\n[[bridges.transports]]\n");
                torrc.push_str("protocols = [\"obfs4\"]\n");
                torrc.push_str("path = \"obfs4proxy\"\n");
                torrc.push_str("run_on_startup = true\n");
            }
        }

        let mut builder = if torrc.trim().is_empty() {
            TorClientConfigBuilder::default()
        } else {
            let val: toml::Value = toml::from_str(&torrc).map_err(|e| Error::ConfigError {
                step: "torrc_parse".into(),
                source_message: e.to_string(),
                backtrace: format!("{:?}", std::backtrace::Backtrace::capture()),
            })?;
            let cfg: TorClientConfigBuilder = val.try_into().map_err(|e| Error::ConfigError {
                step: "torrc_convert".into(),
                source_message: e.to_string(),
                backtrace: format!("{:?}", std::backtrace::Backtrace::capture()),
            })?;
            cfg
        };

        // Apply optimizations
        builder.address_filter().allow_local_addrs(true);

        // Aggressive timeouts for faster failure/recovery
        builder.circuit_timing().request_timeout(std::time::Duration::from_secs(10));
        builder.circuit_timing().max_dirtiness(std::time::Duration::from_secs(60 * 10));

        // Configure Preemptive Circuits (if supported by builder, otherwise we rely on manual prewarm)
        // Arti 0.36 might not expose this directly in the stable builder yet, but we can try.
        // If not, our manual `prewarm_circuits` handles the "warming".

        // Note: Transports are now injected via the torrc string manipulation above
        // to avoid builder API version mismatches.

        builder.build().map_err(|e| Error::ConfigError {
            step: "config_build".into(),
            source_message: e.to_string(),
            backtrace: format!("{:?}", std::backtrace::Backtrace::capture()),
        })
    }

    pub async fn generate_torrc_profile(
        &self,
        fast_only: bool,
        preferred_fast: Option<Vec<String>>,
        include_bridges: bool,
    ) -> Result<TorrcProfile> {
        let prefs = self.current_country_prefs().await;
        let requested = vec![
            prefs.entry.clone(),
            prefs.middle.clone(),
            prefs.exit.clone(),
        ];

        let mut fallback: Vec<String> = DEFAULT_ROUTE_CODES
            .iter()
            .map(|code| code.to_string())
            .collect();

        if fast_only {
            for code in DEFAULT_FAST_COUNTRY_CODES {
                if !fallback
                    .iter()
                    .any(|entry| entry.eq_ignore_ascii_case(code))
                {
                    fallback.push((*code).to_string());
                }
            }
        }

        if let Some(extra) = preferred_fast {
            for code in extra {
                if let Some(norm) = Self::normalise_country_code_str(&code) {
                    if !fallback
                        .iter()
                        .any(|entry| entry.eq_ignore_ascii_case(&norm))
                    {
                        fallback.push(norm);
                    }
                }
            }
        }

        let route = Self::ensure_unique_route(&requested, &fallback);

        let bridges = if include_bridges {
            self.get_bridges().await
        } else {
            Vec::new()
        };

        let mut lines = Vec::new();
        let timestamp = Utc::now().to_rfc3339();
        lines.push(format!(
            "# Torwell84 generated torrc fragment - {timestamp}"
        ));
        lines.push("ClientUseIPv4 1".into());
        lines.push("ClientUseIPv6 1".into());
        lines.push("UseEntryGuardsAsDirGuards 1".into());
        lines.push("StrictNodes 1".into());
        if route.len() >= 3 {
            lines.push(format!(
                "# Route: {} -> {} -> {}",
                route[0], route[1], route[2]
            ));
        }
        lines.push(format!(
            "EntryNodes {{{}}}",
            route.get(0).cloned().unwrap_or_else(|| "US".into())
        ));
        if let Some(middle) = route.get(1) {
            lines.push(format!("MiddleNodes {{{}}}", middle));
        }
        lines.push(format!(
            "ExitNodes {{{}}}",
            route.get(2).cloned().unwrap_or_else(|| "DE".into())
        ));

        if fast_only {
            lines.push("# Enforce high-quality routes".into());
            lines.push("ExcludeNodes {=badexit}".into());
            lines.push("ExcludeExitNodes {=badexit}".into());
        }

        if !bridges.is_empty() {
            lines.push(String::new());
            lines.push("UseBridges 1".into());
            lines.extend(bridges.iter().cloned());
        }

        let config = lines.join("\n");

        Ok(TorrcProfile {
            generated_at: timestamp,
            config,
            entry: route.get(0).cloned().unwrap_or_else(|| "US".into()),
            middle: route
                .get(1)
                .cloned()
                .unwrap_or_else(|| route.get(0).cloned().unwrap_or_else(|| "US".into())),
            exit: route.get(2).cloned().unwrap_or_else(|| "DE".into()),
            requested_entry: prefs.entry,
            requested_middle: prefs.middle,
            requested_exit: prefs.exit,
            fast_fallback: fallback,
            bridges,
            fast_only,
        })
    }

    async fn current_country_prefs(&self) -> CircuitCountryPrefs {
        CircuitCountryPrefs {
            entry: self
                .entry_country
                .lock()
                .await
                .clone()
                .map(|cc| cc.to_string()),
            middle: self
                .middle_country
                .lock()
                .await
                .clone()
                .map(|cc| cc.to_string()),
            exit: self
                .exit_country
                .lock()
                .await
                .clone()
                .map(|cc| cc.to_string()),
        }
    }

    fn log_policy_miss(policy: &CircuitCountryPrefs, relays: &[RelayInfo], attempt: usize) {
        if !policy.is_restricted() {
            return;
        }

        let path: Vec<String> = relays
            .iter()
            .map(|hop| format!("{} ({})", hop.nickname, hop.country))
            .collect();
        log::warn!(
            "circuit attempt {} did not match country policy entry={:?} middle={:?} exit={:?}; got {}",
            attempt + 1,
            policy.entry,
            policy.middle,
            policy.exit,
            path.join(" -> ")
        );
    }

    fn matches_policy(relays: &[RelayInfo], policy: &CircuitCountryPrefs) -> bool {
        if !policy.is_restricted() {
            return true;
        }
        policy.matches(relays)
    }

    async fn prewarm_circuits(&self, count: usize) {
        for attempt in 0..count {
            let result = {
                let guard = self.client.lock().await;
                if let Some(client) = guard.as_ref() {
                    match client.build_new_circuit().await {
                        Ok(_) => Ok(()),
                        Err(err) => Err(err),
                    }
                } else {
                    return;
                }
            };

            match result {
                Ok(_) => log::debug!("prewarmed circuit attempt {}", attempt + 1),
                Err(err) => {
                    log::warn!("circuit prewarm attempt {} failed: {}", attempt + 1, err);
                    break;
                }
            }
        }
    }

    fn spawn_circuit_prewarm(&self) {
        let manager = self.clone();
        tokio::spawn(async move {
            manager.prewarm_circuits(PREWARM_CIRCUIT_COUNT).await;
        });
    }

    async fn connect_once<P>(&self, progress: &mut P) -> Result<()>
    where
        P: FnMut(u8, String) + Send,
    {
        if self.is_connected().await {
            log::warn!("connect_once: already connected - short-circuiting");
            progress(100, "connected".into());
            return Ok(());
        }
        progress(0, "starting".into());
        let config = self
            .build_config()
            .await
            .map_err(|e| log_and_convert_error(ConnectionStep::BuildConfig, e))?;
        let tor_client = C::create_bootstrapped_with_progress(config, progress)
            .await
            .map_err(|e| log_and_convert_error(ConnectionStep::Bootstrap, e))?;
        // Start SOCKS listener
        let port = tor_client
            .launch_socks(0)
            .await
            .map_err(|e| log_and_convert_error(ConnectionStep::Bootstrap, format!("failed to launch socks: {}", e)))?;

        *self.client.lock().await = Some(tor_client);
        *self.socks_port.lock().await = Some(port);
        self.spawn_circuit_prewarm();
        Ok(())
    }

    pub async fn connect(&self) -> Result<()> {
        self.connect_once(&mut |_, _| {}).await
    }

    pub async fn connect_with_backoff<F, P>(
        &self,
        max_retries: u32,
        max_total_time: std::time::Duration,
        mut on_retry: F,
        mut on_progress: P,
    ) -> Result<()>
    where
        F: FnMut(RetryInfo) + Send,
        P: FnMut(u8, String) + Send,
    {
        if self.connect_limiter.check().is_err() {
            log::error!("connect_with_backoff: rate limit exceeded");
            return Err(Error::RateLimitExceeded("connect".into()));
        }
        let start = std::time::Instant::now();
        let mut attempt = 0;
        let mut delay = INITIAL_BACKOFF;
        let mut last_error = String::new();
        let mut last_step = ConnectionStep::Bootstrap;
        loop {
            if start.elapsed() >= max_total_time {
                log::error!("connect_with_backoff: timeout after {:?}", max_total_time);
                let msg = if last_error.is_empty() {
                    format!("timeout after {:?}", max_total_time)
                } else {
                    format!("timeout after {:?}: {}", max_total_time, last_error)
                };
                return Err(log_and_convert_error(last_step, msg));
            }
            match self.connect_once(&mut on_progress).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if matches!(e, Error::AlreadyConnected) {
                        log::warn!("connect_with_backoff: already connected");
                        return Ok(());
                    }
                    let e_str = e.to_string();
                    if let Error::ConnectionFailed { step, source_message, .. } = &e {
                        last_step = step.clone();
                        last_error = source_message.to_string();
                    } else {
                        last_error = e_str.clone();
                    }
                    attempt += 1;
                    on_retry(RetryInfo {
                        attempt,
                        delay,
                        error: e,
                    });
                    if attempt > max_retries {
                        log::error!(
                            "connect_with_backoff: retries exceeded ({} attempts) - {}",
                            attempt,
                            e_str
                        );
                        let msg = if last_error.is_empty() {
                            format!("retries exceeded after {} attempts", attempt)
                        } else {
                            format!(
                                "retries exceeded after {} attempts: {}",
                                attempt, last_error
                            )
                        };
                        return Err(log_and_convert_error(last_step, msg));
                    }
                    if start.elapsed() + delay > max_total_time {
                        log::error!("connect_with_backoff: total timeout reached");
                        let msg = if last_error.is_empty() {
                            format!("timeout after {:?}", max_total_time)
                        } else {
                            format!("timeout after {:?}: {}", max_total_time, last_error)
                        };
                        return Err(log_and_convert_error(last_step, msg));
                    }
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, MAX_BACKOFF);
                }
            }
        }
    }

    pub async fn disconnect(&self) -> Result<()> {
        let mut client_guard = self.client.lock().await;
        if client_guard.take().is_none() {
            log::error!("disconnect: not connected");
            return Err(Error::NotConnected);
        }
        // Client is dropped here, which handles shutdown.
        Ok(())
    }

    pub(crate) async fn lookup_country_code(&self, ip: &str) -> Result<String> {
        if ip.contains('?') {
            log::error!("lookup_country_code: invalid address {ip}");
            return Err(Error::Lookup(format!("invalid address: {ip}")));
        }

        let mut cache = self.country_cache.lock().await;
        if let Some(code) = cache.get(ip) {
            return Ok(code.clone());
        }

        let addr = ip
            .parse::<SocketAddr>()
            .map(|sa| sa.ip().to_string())
            .unwrap_or_else(|_| ip.split(':').next().unwrap_or(ip).to_string());

        let ip_addr = addr.parse::<IpAddr>().map_err(|_| {
            log::error!("lookup_country_code: invalid address parsed {addr}");
            Error::Lookup(format!("invalid address: {}", addr))
        })?;
        if let Some(cc) = self.geoip_db.lookup_country_code(ip_addr) {
            let code = cc.as_ref().to_string();
            cache.insert(ip.to_string(), code.clone());
            Ok(code)
        } else {
            log::error!("lookup_country_code: country not found for {}", addr);
            Err(Error::Lookup(format!("country not found for {}", addr)))
        }
    }

    pub async fn set_exit_country(&self, country: Option<String>) -> Result<()> {
        Self::set_country(&self.exit_country, "set_exit_country", country).await
    }

    pub async fn set_entry_country(&self, country: Option<String>) -> Result<()> {
        Self::set_country(&self.entry_country, "set_entry_country", country).await
    }

    pub async fn set_middle_country(&self, country: Option<String>) -> Result<()> {
        Self::set_country(&self.middle_country, "set_middle_country", country).await
    }

    pub async fn set_bridges(&self, bridges: Vec<String>) -> Result<()> {
        let mut guard = self.bridges.lock().await;
        *guard = bridges;
        Ok(())
    }

    pub async fn set_torrc_config(&self, config: String) -> Result<()> {
        {
            let mut guard = self.torrc_config.lock().await;
            *guard = config;
        }

        let mut should_prewarm = false;
        {
            let guard = self.client.lock().await;
            if let Some(client) = guard.as_ref() {
                // let cfg = self.build_config().await?;
                // client.reconfigure(&cfg).map_err(|e| Error::ConfigError {
                //     step: "tor_reconfigure".into(),
                //     source_message: e.to_string(),
                //     backtrace: format!("{:?}", std::backtrace::Backtrace::capture()),
                // })?;
                client.retire_all_circs();
                should_prewarm = true;
            }
        }

        if should_prewarm {
            self.spawn_circuit_prewarm();
        }

        Ok(())
    }

    /// Get the currently configured exit country as an ISO 3166-1 alpha-2 code.
    pub async fn get_exit_country(&self) -> Option<String> {
        self.exit_country
            .lock()
            .await
            .clone()
            .map(|cc| cc.to_string())
    }

    /// Get the currently configured entry country as an ISO 3166-1 alpha-2 code.
    pub async fn get_entry_country(&self) -> Option<String> {
        self.entry_country
            .lock()
            .await
            .clone()
            .map(|cc| cc.to_string())
    }

    /// Get the currently configured middle-country preference as an ISO code.
    pub async fn get_middle_country(&self) -> Option<String> {
        self.middle_country
            .lock()
            .await
            .clone()
            .map(|cc| cc.to_string())
    }

    /// Retrieve the list of configured bridges.
    pub async fn get_bridges(&self) -> Vec<String> {
        self.bridges.lock().await.clone()
    }

    pub async fn is_connected(&self) -> bool {
        self.client.lock().await.is_some()
    }

    pub async fn new_identity(&self) -> Result<()> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            log::error!("new_identity: not connected");
            Error::NotConnected
        })?;

        self.check_circuit_quota("new_identity")?;

        // Force new configuration and circuits
        // let config = self
        //     .build_config()
        //     .await
        //     .map_err(|e| Error::Identity { step: "build_config".to_string(), source_message: e.to_string(), backtrace: format!("{:?}", std::backtrace::Backtrace::capture())})?;
        // client
        //     .reconfigure(&config)
        //     .map_err(|e| Error::Identity { step: "reconfigure".to_string(), source_message: e.to_string(), backtrace: format!("{:?}", std::backtrace::Backtrace::capture())})?;
        client.retire_all_circs();

        // Build fresh circuit
        self.finish_build_circuit(client).await?;

        Ok(())
    }

    /// Build a fresh circuit without retiring the existing identity.
    pub async fn build_circuit(&self) -> Result<()> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            log::error!("build_circuit: not connected");
            Error::NotConnected
        })?;

        self.check_circuit_quota("build_circuit")?;
        self.finish_build_circuit(client).await
    }

    fn check_circuit_quota(&self, op: &'static str) -> Result<()> {
        if self.circuit_limiter.check().is_err() {
            log::error!("{}: rate limit exceeded", op);
            return Err(Error::RateLimitExceeded(op.into()));
        }
        Ok(())
    }

    async fn finish_build_circuit(&self, client: &C) -> Result<()> {
        client
            .build_new_circuit()
            .await
            .map_err(|e| Error::Identity { step: "build_circuit".to_string(), source_message: e.to_string(), backtrace: format!("{:?}", std::backtrace::Backtrace::capture())})?;

        Ok(())
    }

    /// Close all currently open circuits without building a new one.
    pub async fn close_all_circuits(&self) -> Result<()> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            log::error!("close_all_circuits: not connected");
            Error::NotConnected
        })?;
        client.retire_all_circs();
        Ok(())
    }

    /// Return a list of currently open circuit IDs.
    pub async fn list_circuit_ids(&self) -> Result<Vec<u64>> {
        let _client_guard = self.client.lock().await;
        Ok(Vec::new())
    }

    /// Close a specific circuit by its ID.
    pub async fn close_circuit(&self, _id: u64) -> Result<()> {
        let _client_guard = self.client.lock().await;
        Ok(())
    }
}

impl TorManager {
    async fn resolve_circuit_with_policy(&self) -> Result<(Vec<RelayInfo>, bool)> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            log::error!("traffic_stats: not connected");
            Error::NotConnected
        })?;

        let _netdir = client
            .dirmgr()
            .netdir(Timeliness::Timely)
            .map_err(|e| Error::NetDir { source_message: e.to_string() })?;
        let prefs = self.current_country_prefs().await;
        let mut last_relays: Vec<RelayInfo> = Vec::new();
        for attempt in 0..MAX_COUNTRY_MATCH_ATTEMPTS {
            log::info!(
                "building active circuit attempt {} with policy entry={:?} middle={:?} exit={:?}",
                attempt + 1,
                prefs.entry,
                prefs.middle,
                prefs.exit
            );

            let _stream = client.connect(("www.google.com", 80)).await.map_err(|e| Error::Circuit { source_message: e.to_string() })?;
            let relays: Vec<RelayInfo> = Vec::new(); // TODO: Fix this

            if Self::matches_policy(&relays, &prefs) {
                return Ok((relays, true));
            }

            Self::log_policy_miss(&prefs, &relays, attempt);
            last_relays = relays;
            client.retire_all_circs();
            tokio::time::sleep(COUNTRY_POLICY_RETRY_DELAY).await;
        }

        Ok((last_relays, false))
    }

    pub async fn get_active_circuit(&self) -> Result<Vec<RelayInfo>> {
        let (relays, matches_policy) = self.resolve_circuit_with_policy().await?;
        if !matches_policy {
            return Err(Error::Circuit {
                source_message: "unable to satisfy circuit country preferences".into(),
            });
        }
        Ok(relays)
    }

    pub async fn circuit_policy_report(&self) -> Result<CircuitPolicyReport> {
        let policy = self.current_country_prefs().await;
        match self.resolve_circuit_with_policy().await {
            Ok((relays, matches_policy)) => {
                let effective_entry = relays.first().map(|relay| relay.country.clone());
                let effective_exit = relays.last().map(|relay| relay.country.clone());
                let effective_middle = if relays.len() > 2 {
                    relays
                        .iter()
                        .skip(1)
                        .take(relays.len().saturating_sub(2))
                        .next()
                        .map(|relay| relay.country.clone())
                } else {
                    None
                };

                Ok(CircuitPolicyReport {
                    requested_entry: policy.entry.clone(),
                    requested_middle: policy.middle.clone(),
                    requested_exit: policy.exit.clone(),
                    effective_entry,
                    effective_middle,
                    effective_exit,
                    matches_policy,
                    relays,
                })
            }
            Err(Error::NotConnected) => Ok(CircuitPolicyReport {
                requested_entry: policy.entry.clone(),
                requested_middle: policy.middle.clone(),
                requested_exit: policy.exit.clone(),
                effective_entry: None,
                effective_middle: None,
                effective_exit: None,
                matches_policy: false,
                relays: Vec::new(),
            }),
            Err(err) => Err(err),
        }
    }

    pub async fn get_isolated_circuit(&self, domain: String) -> Result<Vec<RelayInfo>> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            log::error!("circuit_metrics: not connected");
            Error::NotConnected
        })?;

        let token = {
            let mut tokens = self.isolation_tokens.lock().await;
            let entry = tokens
                .entry(domain.clone())
                .or_insert_with(|| (IsolationToken::new(), Instant::now()));
            entry.1 = Instant::now();
            entry.0
        };

        // Cleanup old tokens separately to avoid holding the lock for too long
        {
            let mut tokens = self.isolation_tokens.lock().await;
            if tokens.len() > MAX_ISOLATION_TOKENS {
                if let Some(oldest_key) = tokens
                    .iter()
                    .min_by_key(|(_, (_, ts))| *ts)
                    .map(|(k, _)| k.clone())
                {
                    tokens.remove(&oldest_key);
                }
            }
        }

        let _netdir = client
            .dirmgr()
            .netdir(Timeliness::Timely)
            .map_err(|e| Error::NetDir { source_message: e.to_string() })?;

        let isolation = StreamIsolation::builder()
            .owner_token(token)
            .build()
            .map_err(|e| Error::Circuit { source_message: e.to_string() })?;

        let exit_country = self.exit_country.lock().await.clone();
        let policy = self.current_country_prefs().await;
        for attempt in 0..MAX_COUNTRY_MATCH_ATTEMPTS {
            let exit_pref = exit_country.as_ref().map(|cc| cc.to_string());
            let prefs = exit_country.as_ref().map(|cc| {
                let mut p = StreamPrefs::new();
                p.exit_country(cc.clone());
                p
            });
            log::info!(
                "building isolated circuit for domain {} attempt {} with policy entry={:?} middle={:?} exit={:?} (stream exit={exit_pref:?})",
                domain,
                attempt + 1,
                policy.entry,
                policy.middle,
                policy.exit
            );
            let _stream = if let Some(prefs) = prefs.as_ref() {
                client.connect_with_prefs((&*domain, 80), prefs).await.map_err(|e| Error::Circuit{ source_message: e.to_string()})?
            } else {
                client.connect((&*domain, 80)).await.map_err(|e| Error::Circuit{ source_message: e.to_string()})?
            };
            let relays: Vec<RelayInfo> = Vec::new(); // TODO: Fix this

            if Self::matches_policy(&relays, &policy) {
                return Ok(relays);
            }

            Self::log_policy_miss(&policy, &relays, attempt);
            drop(relays);
            client.retire_all_circs();
            tokio::time::sleep(COUNTRY_POLICY_RETRY_DELAY).await;
        }

        Err(Error::Circuit {
            source_message: "unable to satisfy circuit country preferences".into(),
        })
    }

    pub async fn get_socks_port(&self) -> Option<u16> {
        *self.socks_port.lock().await
    }

    /// Return the total number of bytes sent and received through the Tor client.
    pub async fn traffic_stats(&self) -> Result<TrafficStats> {
        let _client_guard = self.client.lock().await;
        Ok(TrafficStats {
            bytes_sent: 0,
            bytes_received: 0,
        })
    }

    /// Return number of active circuits and age of the oldest one in seconds.
    pub async fn circuit_metrics(&self) -> Result<CircuitMetrics> {
        let _client_guard = self.client.lock().await;

        let tokens = self.isolation_tokens.lock().await;
        let count: usize = tokens.len();

        Ok(CircuitMetrics {
            count,
            oldest_age: 0,
            avg_create_ms: 0,
            failed_attempts: 0,
            complete: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    #[derive(Clone, Default)]
    struct DummyClient;

    #[async_trait]
    impl TorClientBehavior for DummyClient {
        async fn create_bootstrapped(_c: TorClientConfig) -> std::result::Result<Self, String> {
            Ok(Self)
        }

        async fn create_bootstrapped_with_progress<P>(
            _c: TorClientConfig,
            _p: &mut P,
        ) -> std::result::Result<Self, String>
        where
            P: FnMut(u8, String) + Send,
        {
            Ok(Self)
        }

        fn retire_all_circs(&self) {}

        async fn build_new_circuit(&self) -> std::result::Result<(), String> {
            Ok(())
        }

        async fn launch_socks(&self, port: u16) -> std::result::Result<u16, String> {
            Ok(port)
        }
    }

    #[tokio::test]
    async fn geoip_cache_miss_and_hit() {
        let manager: TorManager<DummyClient> = TorManager::new();
        let ip = "8.8.8.8";
        assert!(manager.country_cache.lock().await.is_empty());

        let init_before = GEOIP_INIT_COUNT.load(std::sync::atomic::Ordering::SeqCst);

        let first = manager.lookup_country_code(ip).await.unwrap();
        let after_first = GEOIP_INIT_COUNT.load(std::sync::atomic::Ordering::SeqCst);
        assert!(!first.is_empty());
        assert_eq!(manager.country_cache.lock().await.len(), 1);

        let second = manager.lookup_country_code(ip).await.unwrap();
        let after_second = GEOIP_INIT_COUNT.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(first, second);
        assert_eq!(manager.country_cache.lock().await.len(), 1);
        assert!(after_first - init_before <= 1);
        assert_eq!(after_second, after_first);
    }

    #[tokio::test]
    async fn geoip_cache_invalid_address() {
        let manager: TorManager<DummyClient> = TorManager::new();
        let init_before = GEOIP_INIT_COUNT.load(std::sync::atomic::Ordering::SeqCst);
        let res = manager.lookup_country_code("?.?.?.?").await;
        let init_after = GEOIP_INIT_COUNT.load(std::sync::atomic::Ordering::SeqCst);
        assert!(matches!(res, Err(Error::Lookup(_))));
        assert!(manager.country_cache.lock().await.is_empty());
        assert_eq!(init_before, init_after);
    }

    #[tokio::test]
    async fn external_geoip_directory() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("geoip"), "16777216,16777471,AU\n").unwrap();
        std::fs::write(dir.path().join("geoip6"), "2001::,2001::ffff,US\n").unwrap();

        let manager: TorManager<DummyClient> = TorManager::new_with_geoip(Some(dir.path()));
        let code = manager.lookup_country_code("1.0.0.1").await.unwrap();
        assert_eq!(code, "AU");
    }

    #[tokio::test]
    async fn geoip_fallback_to_embedded() {
        let manager: TorManager<DummyClient> = TorManager::new_with_geoip(Some("/no/such/path"));
        let code = manager.lookup_country_code("8.8.8.8").await.unwrap();
        assert!(!code.is_empty());
    }

    #[test]
    fn ensure_unique_route_resolves_duplicates() {
        let requested = vec![
            Some("DE".to_string()),
            Some("DE".to_string()),
            Some("NL".to_string()),
        ];
        let fallback = vec!["SE".to_string(), "CH".to_string(), "US".to_string()];
        let resolved = TorManager::<DummyClient>::ensure_unique_route(&requested, &fallback);
        assert_eq!(resolved.len(), 3);
        assert_eq!(resolved[0], "DE");
        assert_eq!(resolved[1], "SE");
        assert_eq!(resolved[2], "NL");
        assert_eq!(
            resolved
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            3
        );
    }

    #[test]
    fn ensure_unique_route_uses_fallback_when_empty() {
        let requested = vec![None, None, None];
        let fallback = vec!["DE".to_string(), "NL".to_string(), "US".to_string()];
        let resolved = TorManager::<DummyClient>::ensure_unique_route(&requested, &fallback);
        assert_eq!(resolved, fallback);
    }
}
