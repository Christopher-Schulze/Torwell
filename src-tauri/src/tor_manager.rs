use crate::commands::RelayInfo;
use crate::error::{Error, Result};
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
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
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
use tor_linkspec::{HasAddrs, HasRelayIds};
use tor_rtcompat::PreferredRuntime;

const INITIAL_BACKOFF: std::time::Duration = std::time::Duration::from_secs(1);
const MAX_BACKOFF: std::time::Duration = std::time::Duration::from_secs(30);
const CONNECT_RATE_LIMIT: u32 = 5;
const CIRCUIT_RATE_LIMIT: u32 = 10;

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
    isolation_tokens: Arc<Mutex<HashMap<String, Vec<(IsolationToken, std::time::Instant)>>>>,
    exit_country: Arc<Mutex<Option<CountryCode>>>,
    bridges: Arc<Mutex<Vec<String>>>,
    country_cache: Arc<Mutex<HashMap<String, String>>>,
    connect_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    circuit_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl<C> Clone for TorManager<C> {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            isolation_tokens: Arc::clone(&self.isolation_tokens),
            exit_country: Arc::clone(&self.exit_country),
            bridges: Arc::clone(&self.bridges),
            country_cache: Arc::clone(&self.country_cache),
            connect_limiter: Arc::clone(&self.connect_limiter),
            circuit_limiter: Arc::clone(&self.circuit_limiter),
        }
    }
}

impl<C: TorClientBehavior> TorManager<C> {
    pub fn new() -> Self {
        let manager = Self {
            client: Arc::new(Mutex::new(None)),
            isolation_tokens: Arc::new(Mutex::new(HashMap::new())),
            exit_country: Arc::new(Mutex::new(None)),
            bridges: Arc::new(Mutex::new(Vec::new())),
            country_cache: Arc::new(Mutex::new(HashMap::new())),
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
        tokens.retain(|_, list| {
            list.retain(|(_, ts)| now.duration_since(*ts) <= max_age);
            !list.is_empty()
        });
    }

    async fn build_config(&self) -> Result<TorClientConfig> {
        let bridges = self.bridges.lock().await.clone();
        let exit_country = self.exit_country.lock().await.clone();
        if bridges.is_empty() && exit_country.is_none() {
            Ok(TorClientConfig::default())
        } else {
            use arti_client::config::{
                BoolOrAuto, BridgeConfigBuilder, BridgesConfigBuilder, TorClientConfigBuilder,
            };

            let mut builder = TorClientConfigBuilder::default();
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
            builder.build().map_err(|e| Error::ConnectError(e.to_string()))
        }
    }

    async fn connect_once<P>(&self, progress: &mut P) -> Result<()>
    where
        P: FnMut(u8, String) + Send,
    {
        if self.is_connected().await {
            return Err(Error::AlreadyConnected);
        }
        progress(0, "starting".into());
        let config = self.build_config().await?;
        let tor_client = C::create_bootstrapped_with_progress(config, progress)
            .await
            .map_err(|e| Error::ConnectError(e))?;
        *self.client.lock().await = Some(tor_client);
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
        F: FnMut(u32, std::time::Duration, &Error) + Send,
        P: FnMut(u8, String) + Send,
    {
        if self.connect_limiter.check().is_err() {
            return Err(Error::RateLimited("connect".into()));
        }
        let start = std::time::Instant::now();
        let mut attempt = 0;
        let mut delay = INITIAL_BACKOFF;
        loop {
            if start.elapsed() >= max_total_time {
                return Err(Error::Timeout);
            }
            match self.connect_once(&mut on_progress).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    attempt += 1;
                    on_retry(attempt, delay, &e);
                    if attempt > max_retries {
                        return Err(Error::RetriesExceeded {
                            attempts: attempt,
                            error: e.to_string(),
                        });
                    }
                    if start.elapsed() + delay > max_total_time {
                        return Err(Error::Timeout);
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
            return Err(Error::NotConnected);
        }
        // Client is dropped here, which handles shutdown.
        Ok(())
    }

    pub(crate) async fn lookup_country_code(&self, ip: &str) -> Result<String> {
        if ip.contains('?') {
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

        let ip_addr = addr
            .parse::<IpAddr>()
            .map_err(|_| Error::Lookup(format!("invalid address: {}", addr)))?;
        if let Some(cc) = GEOIP_DB.lookup_country_code(ip_addr) {
            let code = cc.as_ref().to_string();
            cache.insert(ip.to_string(), code.clone());
            Ok(code)
        } else {
            Err(Error::Lookup(format!("country not found for {}", addr)))
        }
    }

    pub async fn set_exit_country(&self, country: Option<String>) -> Result<()> {
        let mut guard = self.exit_country.lock().await;
        if let Some(cc) = country {
            let code = CountryCode::new(&cc).map_err(|e| Error::ConnectError(e.to_string()))?;
            *guard = Some(code);
        } else {
            *guard = None;
        }
        Ok(())
    }

    pub async fn set_bridges(&self, bridges: Vec<String>) -> Result<()> {
        let mut guard = self.bridges.lock().await;
        *guard = bridges;
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

    /// Retrieve the list of configured bridges.
    pub async fn get_bridges(&self) -> Vec<String> {
        self.bridges.lock().await.clone()
    }

    pub async fn is_connected(&self) -> bool {
        self.client.lock().await.is_some()
    }

    pub async fn new_identity(&self) -> Result<()> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;

        if self.circuit_limiter.check().is_err() {
            return Err(Error::RateLimited("new_identity".into()));
        }

        // Force new configuration and circuits
        let config = self.build_config().await?;
        client
            .reconfigure(&config)
            .map_err(|e| Error::Identity(format!("reconfigure failed: {e}")))?;
        client.retire_all_circs();

        // Build fresh circuit
        client
            .build_new_circuit()
            .await
            .map_err(|e| {
                if e.to_lowercase().contains("timeout") {
                    Error::CircuitTimeout
                } else {
                    Error::Circuit(format!("failed to build new circuit: {e}"))
                }
            })?;

        Ok(())
    }
}

impl TorManager {
    pub async fn get_active_circuit(&self) -> Result<Vec<RelayInfo>> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;

        let netdir = client
            .dirmgr()
            .netdir(Timeliness::Timely)
            .map_err(|e| Error::NetDir(e.to_string()))?;
        let circuit = client
            .circmgr()
            .get_or_launch_exit(
                (&*netdir).into(),
                &[],
                StreamIsolation::no_isolation(),
                None,
            )
            .await
            .map_err(|e| {
                if e.to_lowercase().contains("timeout") {
                    Error::CircuitTimeout
                } else {
                    Error::Circuit(format!("failed to launch exit circuit: {e}"))
                }
            })?;

        let hops: Vec<_> = circuit
            .path_ref()
            .map_err(|e| {
                if e.to_lowercase().contains("timeout") {
                    Error::CircuitTimeout
                } else {
                    Error::Circuit(format!("failed to read circuit path: {e}"))
                }
            })?
            .hops()
            .iter()
            .cloned()
            .collect();

        let mut relays = Vec::new();
        for hop in hops {
            if let Some(relay) = hop.as_chan_target() {
                let nickname = relay
                    .rsa_identity()
                    .map(|id| format!("${}", id.to_string().chars().take(8).collect::<String>()))
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

        Ok(relays)
    }

    pub async fn get_isolated_circuit(&self, domain: String) -> Result<Vec<RelayInfo>> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;

        let mut tokens = self.isolation_tokens.lock().await;
        let domain_key = domain.clone();
        let entry = tokens.entry(domain).or_default();
        let token = IsolationToken::new();
        entry.push((token, std::time::Instant::now()));

        let netdir = client
            .dirmgr()
            .netdir(Timeliness::Timely)
            .map_err(|e| Error::NetDir(e.to_string()))?;

        let isolation = StreamIsolation::builder()
            .owner_token(token)
            .build()
            .expect("StreamIsolation builder failed");

        let prefs = {
            let guard = self.exit_country.lock().await;
            guard.map(|cc| {
                let mut p = StreamPrefs::new();
                p.exit_country(cc);
                p
            })
        };

        let circuit = client
            .circmgr()
            .get_or_launch_exit((&*netdir).into(), &[], isolation, prefs)
            .await
            .map_err(|e| {
                if e.to_lowercase().contains("timeout") {
                    Error::CircuitTimeout
                } else {
                    Error::Circuit(format!("failed to launch isolated circuit for {}: {}", domain_key, e))
                }
            })?;

        let hops: Vec<_> = circuit
            .path_ref()
            .map_err(|e| {
                if e.to_lowercase().contains("timeout") {
                    Error::CircuitTimeout
                } else {
                    Error::Circuit(format!("failed to read circuit path for {}: {}", domain_key, e))
                }
            })?
            .hops()
            .iter()
            .cloned()
            .collect();

        let mut relays = Vec::new();
        for hop in hops {
            if let Some(relay) = hop.as_chan_target() {
                let nickname = relay
                    .rsa_identity()
                    .map(|id| format!("${}", id.to_string().chars().take(8).collect::<String>()))
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

        Ok(relays)
    }

    /// Return the total number of bytes sent and received through the Tor client.
    pub async fn traffic_stats(&self) -> Result<TrafficStats> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;

        let stats = client.traffic_stats();
        Ok(TrafficStats {
            bytes_sent: stats.bytes_written(),
            bytes_received: stats.bytes_read(),
        })
    }

    /// Return number of active circuits and age of the oldest one in seconds.
    pub async fn circuit_metrics(&self) -> Result<CircuitMetrics> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;

        #[cfg(feature = "experimental-api")]
        {
            // Use new arti-client APIs when compiled with the experimental feature.
            // These APIs allow retrieving information about currently open circuits
            // including their creation time.
            use arti_client::client::CircuitInfoExt as _;

            if let Ok(circs) = client.circmgr().list_circuits() {
                let count = circs.len();
                let oldest_age = circs
                    .iter()
                    .filter_map(|c| c.created().elapsed().ok())
                    .map(|d| d.as_secs())
                    .max()
                    .unwrap_or(0);

                return Ok(CircuitMetrics { count, oldest_age });
            }
        }

        // Fallback when the arti-client APIs are unavailable.
        Ok(CircuitMetrics {
            count: 0,
            oldest_age: 0,
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
}
