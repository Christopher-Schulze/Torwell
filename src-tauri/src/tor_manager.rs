use crate::commands::RelayInfo;
use crate::error::{report_error, report_identity_error, ConnectionStep, Error, Result};
use arti_client::config::{
    BoolOrAuto, BridgeConfigBuilder, BridgesConfigBuilder, TorClientConfigBuilder,
};
use arti_client::{client::StreamPrefs, TorClient, TorClientConfig};
use async_trait::async_trait;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
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

#[cfg(test)]
pub(crate) static GEOIP_INIT_COUNT: AtomicUsize = AtomicUsize::new(0);

static GEOIP_DB: Lazy<GeoipDb> = Lazy::new(|| {
    #[cfg(test)]
    {
        GEOIP_INIT_COUNT.fetch_add(1, Ordering::SeqCst);
    }
    GeoipDb::new_embedded()
});

fn load_geoip_db(dir: &Path) -> Option<GeoipDb> {
    let v4 = std::fs::read_to_string(dir.join("geoip")).ok()?;
    let v6 = std::fs::read_to_string(dir.join("geoip6")).ok()?;
    GeoipDb::new_from_legacy_format(&v4, &v6).ok()
}

/// Helper to log an error for a given step and convert it into an
/// [`Error::ConnectionFailed`] variant.
fn log_and_convert_error(step: ConnectionStep, err: impl ToString) -> Error {
    let msg = err.to_string();
    log::error!("{}: {}", step, msg);
    Error::ConnectionFailed {
        step,
        source: msg,
        backtrace: Some(format!("{:?}", std::backtrace::Backtrace::capture())),
    }
}
use tor_linkspec::{HasAddrs, HasRelayIds};
use tor_rtcompat::PreferredRuntime;

const INITIAL_BACKOFF: std::time::Duration = std::time::Duration::from_secs(1);
const MAX_BACKOFF: std::time::Duration = std::time::Duration::from_secs(30);
const CONNECT_RATE_LIMIT: u32 = 5;
const CIRCUIT_RATE_LIMIT: u32 = 10;
const MAX_ISOLATION_TOKENS: usize = 100;
const PREWARM_CIRCUIT_COUNT: usize = 3;
const MAX_COUNTRY_MATCH_ATTEMPTS: usize = 5;
const COUNTRY_POLICY_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(250);

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
    PresetFile::from_str(include_str!("../src/lib/bridge_presets.json"))
}

pub fn load_bridge_presets_from_str(data: &str) -> Result<Vec<BridgePreset>> {
    PresetFile::from_str(data)
}

#[derive(Debug, Clone)]
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
    fn reconfigure(&self, config: &TorClientConfig) -> std::result::Result<(), String>;
    fn retire_all_circs(&self);
    async fn build_new_circuit(&self) -> std::result::Result<(), String>;
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

        progress(100, "done".into());

        Ok(client)
    }

    fn reconfigure(&self, config: &TorClientConfig) -> std::result::Result<(), String> {
        self.reconfigure(config).map_err(|e| e.to_string())
    }

    fn retire_all_circs(&self) {
        self.circmgr().retire_all_circs();
    }

    async fn build_new_circuit(&self) -> std::result::Result<(), String> {
        let netdir = self
            .dirmgr()
            .netdir(Timeliness::Timely)
            .map_err(|e| e.to_string())?;
        self.circmgr()
            .build_circuit(
                (&*netdir).into(),
                &[],
                StreamIsolation::no_isolation(),
                None,
            )
            .await
            .map_err(|e| e.to_string())
    }
}
pub struct TorManager<C = TorClient<PreferredRuntime>> {
    client: Arc<Mutex<Option<C>>>,
    isolation_tokens: Arc<Mutex<HashMap<String, (IsolationToken, std::time::Instant)>>>,
    exit_country: Arc<Mutex<Option<CountryCode>>>,
    entry_country: Arc<Mutex<Option<CountryCode>>>,
    middle_country: Arc<Mutex<Option<CountryCode>>>,
    bridges: Arc<Mutex<Vec<String>>>,
    torrc_config: Arc<Mutex<String>>,
    country_cache: Arc<Mutex<HashMap<String, String>>>,
    geoip_db: GeoipDb,
    connect_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    circuit_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl<C> Clone for TorManager<C> {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
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
    pub fn new() -> Self {
        Self::new_with_geoip(None)
    }

    pub fn new_with_geoip<P: AsRef<Path>>(geoip_dir: Option<P>) -> Self {
        let db = geoip_dir
            .as_ref()
            .and_then(|p| load_geoip_db(p.as_ref()))
            .unwrap_or_else(|| GEOIP_DB.clone());

        let manager = Self {
            client: Arc::new(Mutex::new(None)),
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
            let code = CountryCode::new(&cc).map_err(|e| {
                log::error!("{step}: invalid code {cc} - {e}");
                Error::ConfigError {
                    step: step.into(),
                    source: e.to_string(),
                    backtrace: Some(format!("{:?}", std::backtrace::Backtrace::capture())),
                }
            })?;
            *guard = Some(code);
        } else {
            *guard = None;
        }
        Ok(())
    }

    async fn build_config(&self) -> Result<TorClientConfig> {
        let bridges = self.bridges.lock().await.clone();
        let exit_country = self.exit_country.lock().await.clone();
        let torrc = self.torrc_config.lock().await.clone();

        use arti_client::config::{
            BoolOrAuto, BridgeConfigBuilder, BridgesConfigBuilder, TorClientConfigBuilder,
        };

        let mut builder = if torrc.trim().is_empty() {
            TorClientConfigBuilder::default()
        } else {
            let val: toml::Value = toml::from_str(&torrc).map_err(|e| Error::ConfigError {
                step: "torrc_parse".into(),
                source: e.to_string(),
                backtrace: Some(format!("{:?}", std::backtrace::Backtrace::capture())),
            })?;
            let cfg: TorClientConfigBuilder = val.try_into().map_err(|e| Error::ConfigError {
                step: "torrc_convert".into(),
                source: e.to_string(),
                backtrace: Some(format!("{:?}", std::backtrace::Backtrace::capture())),
            })?;
            cfg
        };

        if let Some(cc) = exit_country {
            builder.exit_country(cc);
        }
        if !bridges.is_empty() {
            let mut bridge_builder = BridgesConfigBuilder::default();
            bridge_builder.enabled(BoolOrAuto::Explicit(true));
            for line in bridges {
                let b: BridgeConfigBuilder = line
                    .parse()
                    .map_err(|e| Error::BridgeParse(e.to_string()))?;
                bridge_builder.bridges().push(b);
            }
            builder.bridges(bridge_builder);
        }
        builder.build().map_err(|e| Error::ConfigError {
            step: "config_build".into(),
            source: e.to_string(),
            backtrace: Some(format!("{:?}", std::backtrace::Backtrace::capture())),
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
            log::error!("connect_once: already connected");
            return Err(Error::AlreadyConnected);
        }
        progress(0, "starting".into());
        let config = self
            .build_config()
            .await
            .map_err(|e| log_and_convert_error(ConnectionStep::BuildConfig, e))?;
        let tor_client = C::create_bootstrapped_with_progress(config, progress)
            .await
            .map_err(|e| log_and_convert_error(ConnectionStep::Bootstrap, e))?;
        *self.client.lock().await = Some(tor_client);
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
                    if attempt == 0 && matches!(e, Error::AlreadyConnected) {
                        return Err(e);
                    }
                    if let Error::ConnectionFailed { step, source, .. } = &e {
                        last_step = *step;
                        last_error = source.clone();
                    } else {
                        last_error = e.to_string();
                    }
                    attempt += 1;
                    on_retry(RetryInfo {
                        attempt,
                        delay,
                        error: e.clone(),
                    });
                    if attempt > max_retries {
                        log::error!(
                            "connect_with_backoff: retries exceeded ({} attempts) - {}",
                            attempt,
                            e
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

    pub async fn set_torrc_config(&self, config: String) {
        let mut guard = self.torrc_config.lock().await;
        *guard = config;
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
        let config = self
            .build_config()
            .await
            .map_err(|e| report_identity_error("build_config", e))?;
        client
            .reconfigure(&config)
            .map_err(|e| report_identity_error("reconfigure", e))?;
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
            .map_err(|e| report_identity_error("build_circuit", e))?;

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
        let client_guard = self.client.lock().await;
        let _client = client_guard.as_ref().ok_or_else(|| {
            log::error!("list_circuit_ids: not connected");
            Error::NotConnected
        })?;

        #[cfg(feature = "experimental-api")]
        {
            use arti_client::client::CircuitInfoExt as _;
            let circs = _client
                .circmgr()
                .list_circuits()
                .map_err(|e| report_error("list_circuits", e))?;
            return Ok(circs.iter().map(|c| c.id().into()).collect());
        }

        #[cfg(not(feature = "experimental-api"))]
        {
            Ok(Vec::new())
        }
    }

    /// Close a specific circuit by its ID.
    pub async fn close_circuit(&self, id: u64) -> Result<()> {
        let client_guard = self.client.lock().await;
        let _client = client_guard.as_ref().ok_or_else(|| {
            log::error!("close_circuit: not connected");
            Error::NotConnected
        })?;

        #[cfg(feature = "experimental-api")]
        {
            use arti_client::client::CircuitInfoExt as _;
            let circs = _client
                .circmgr()
                .list_circuits()
                .map_err(|e| report_error("list_circuits", e))?;
            for c in circs {
                if c.id().into() == id {
                    _client
                        .circmgr()
                        .close_circuit(c.id())
                        .map_err(|e| report_error("close_circuit", e))?;
                    break;
                }
            }
        }
        Ok(())
    }
}

impl TorManager {
    pub async fn get_active_circuit(&self) -> Result<Vec<RelayInfo>> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            log::error!("traffic_stats: not connected");
            Error::NotConnected
        })?;

        let netdir = client
            .dirmgr()
            .netdir(Timeliness::Timely)
            .map_err(|e| Error::NetDir(e.to_string()))?;
        let prefs = self.current_country_prefs().await;
        for attempt in 0..MAX_COUNTRY_MATCH_ATTEMPTS {
            log::info!(
                "building active circuit attempt {} with policy entry={:?} middle={:?} exit={:?}",
                attempt + 1,
                prefs.entry,
                prefs.middle,
                prefs.exit
            );
            let circuit = client
                .circmgr()
                .get_or_launch_exit(
                    (&*netdir).into(),
                    &[],
                    StreamIsolation::no_isolation(),
                    None,
                )
                .await
                .map_err(|e| report_error("build_circuit", e))?;

            let hops: Vec<_> = circuit
                .path_ref()
                .map_err(|e| report_error("path", e))?
                .hops()
                .iter()
                .cloned()
                .collect();

            let mut relays = Vec::new();
            for hop in hops {
                if let Some(relay) = hop.as_chan_target() {
                    let nickname = relay
                        .rsa_identity()
                        .map(|id| {
                            format!("${}", id.to_string().chars().take(8).collect::<String>())
                        })
                        .unwrap_or_else(|| "$unknown".to_string());
                    let ip_address = relay
                        .addrs()
                        .get(0)
                        .map_or_else(|| "?.?.?.?".to_string(), |addr| addr.to_string());
                    let country = self
                        .lookup_country_code(&ip_address)
                        .await
                        .unwrap_or_else(|_| "??".to_string());
                    relays.push(RelayInfo {
                        nickname,
                        ip_address,
                        country,
                    });
                } else {
                    relays.push(RelayInfo {
                        nickname: "<virtual>".to_string(),
                        ip_address: "?.?.?.?".to_string(),
                        country: "??".to_string(),
                    });
                }
            }

            if Self::matches_policy(&relays, &prefs) {
                return Ok(relays);
            }

            Self::log_policy_miss(&prefs, &relays, attempt);
            drop(relays);
            drop(circuit);
            client.retire_all_circs();
            tokio::time::sleep(COUNTRY_POLICY_RETRY_DELAY).await;
        }

        Err(Error::Circuit(
            "unable to satisfy circuit country preferences".into(),
        ))
    }

    pub async fn get_isolated_circuit(&self, domain: String) -> Result<Vec<RelayInfo>> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            log::error!("circuit_metrics: not connected");
            Error::NotConnected
        })?;

        let mut tokens = self.isolation_tokens.lock().await;
        let entry = tokens
            .entry(domain.clone())
            .or_insert((IsolationToken::new(), std::time::Instant::now()));
        entry.1 = std::time::Instant::now();

        if tokens.len() > MAX_ISOLATION_TOKENS {
            if let Some(oldest_key) = tokens
                .iter()
                .min_by_key(|(_, (_, ts))| *ts)
                .map(|(k, _)| k.clone())
            {
                tokens.remove(&oldest_key);
            }
        }

        let token = entry.0;

        let netdir = client
            .dirmgr()
            .netdir(Timeliness::Timely)
            .map_err(|e| Error::NetDir(e.to_string()))?;

        let isolation = StreamIsolation::builder()
            .owner_token(token)
            .build()
            .map_err(|e| Error::Circuit(e.to_string()))?;

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
            let circuit = client
                .circmgr()
                .get_or_launch_exit((&*netdir).into(), &[], isolation, prefs)
                .await
                .map_err(|e| report_error("build_circuit", e))?;

            let hops: Vec<_> = circuit
                .path_ref()
                .map_err(|e| report_error("path", e))?
                .hops()
                .iter()
                .cloned()
                .collect();

            let mut relays = Vec::new();
            for hop in hops {
                if let Some(relay) = hop.as_chan_target() {
                    let nickname = relay
                        .rsa_identity()
                        .map(|id| {
                            format!("${}", id.to_string().chars().take(8).collect::<String>())
                        })
                        .unwrap_or_else(|| "$unknown".to_string());
                    let ip_address = relay
                        .addrs()
                        .get(0)
                        .map_or_else(|| "?.?.?.?".to_string(), |addr| addr.to_string());
                    let country = self
                        .lookup_country_code(&ip_address)
                        .await
                        .unwrap_or_else(|_| "??".to_string());
                    relays.push(RelayInfo {
                        nickname,
                        ip_address,
                        country,
                    });
                } else {
                    relays.push(RelayInfo {
                        nickname: "<virtual>".to_string(),
                        ip_address: "?.?.?.?".to_string(),
                        country: "??".to_string(),
                    });
                }
            }

            if Self::matches_policy(&relays, &policy) {
                return Ok(relays);
            }

            Self::log_policy_miss(&policy, &relays, attempt);
            drop(relays);
            drop(circuit);
            client.retire_all_circs();
            tokio::time::sleep(COUNTRY_POLICY_RETRY_DELAY).await;
        }

        Err(Error::Circuit(
            "unable to satisfy circuit country preferences".into(),
        ))
    }

    /// Return the total number of bytes sent and received through the Tor client.
    pub async fn traffic_stats(&self) -> Result<TrafficStats> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            log::error!("get_active_circuit: not connected");
            Error::NotConnected
        })?;

        let stats = client.traffic_stats();
        Ok(TrafficStats {
            bytes_sent: stats.bytes_written(),
            bytes_received: stats.bytes_read(),
        })
    }

    /// Return number of active circuits and age of the oldest one in seconds.
    pub async fn circuit_metrics(&self) -> Result<CircuitMetrics> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            log::error!("get_isolated_circuit: not connected");
            Error::NotConnected
        })?;

        #[cfg(feature = "experimental-api")]
        {
            // Use arti-client APIs to retrieve information about currently open
            // circuits and calculate their age.
            use arti_client::client::CircuitInfoExt as _;

            let circs = client
                .circmgr()
                .list_circuits()
                .map_err(|e| report_error("list_circuits", e))?;
            let count = circs.len();
            let oldest_age = circs
                .iter()
                .filter_map(|c| c.created().elapsed().ok())
                .map(|d| d.as_secs())
                .max()
                .unwrap_or(0);

            let mut total_build = 0u64;
            let mut build_count = 0u64;
            let mut failed_attempts = 0u64;
            for c in &circs {
                if let Some(d) = c.build_duration() {
                    total_build += d.as_millis() as u64;
                    build_count += 1;
                }
                failed_attempts += c.failed_attempts().unwrap_or(0) as u64;
            }
            let avg_create_ms = if build_count > 0 {
                total_build / build_count
            } else {
                0
            };

            return Ok(CircuitMetrics {
                count,
                oldest_age,
                avg_create_ms,
                failed_attempts,
                complete: true,
            });
        }

        #[cfg(not(feature = "experimental-api"))]
        {
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

        fn reconfigure(&self, _c: &TorClientConfig) -> std::result::Result<(), String> {
            Ok(())
        }

        fn retire_all_circs(&self) {}

        async fn build_new_circuit(&self) -> std::result::Result<(), String> {
            Ok(())
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
}
