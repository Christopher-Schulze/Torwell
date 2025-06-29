use crate::commands::RelayInfo;
use crate::error::{Error, Result};
use arti_client::{client::StreamPrefs, TorClient, TorClientConfig};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tor_circmgr::isolation::{IsolationToken, StreamIsolation};
use tor_dirmgr::Timeliness;
use tor_geoip::CountryCode;
use tor_linkspec::{HasAddrs, HasRelayIds};
use tor_rtcompat::PreferredRuntime;

const INITIAL_BACKOFF: std::time::Duration = std::time::Duration::from_secs(1);
const MAX_BACKOFF: std::time::Duration = std::time::Duration::from_secs(30);

#[async_trait]
pub trait TorClientBehavior: Send + Sync + Sized + 'static {
    async fn create_bootstrapped(config: TorClientConfig) -> std::result::Result<Self, String>;
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
            .build_circuit((&*netdir).into(), &[], StreamIsolation::no_isolation(), None)
            .await
            .map_err(|e| e.to_string())
    }
}
pub struct TorManager<C = TorClient<PreferredRuntime>> {
    client: Arc<Mutex<Option<C>>>,
    isolation_tokens: Arc<Mutex<HashMap<String, IsolationToken>>>,
    exit_country: Arc<Mutex<Option<CountryCode>>>,
}

impl<C: TorClientBehavior> TorManager<C> {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            isolation_tokens: Arc::new(Mutex::new(HashMap::new())),
            exit_country: Arc::new(Mutex::new(None)),
        }
    }

    async fn connect_once(&self) -> Result<()> {
        if self.is_connected().await {
            return Err(Error::AlreadyConnected);
        }
        let config = TorClientConfig::default();
        let tor_client = C::create_bootstrapped(config)
            .await
            .map_err(|e| Error::Bootstrap(e))?;
        *self.client.lock().await = Some(tor_client);
        Ok(())
    }

    pub async fn connect(&self) -> Result<()> {
        self.connect_once().await
    }

    pub async fn connect_with_backoff<F>(&self, max_retries: u32, mut on_retry: F) -> Result<()>
    where
        F: FnMut(u32, &Error) + Send,
    {
        let mut attempt = 0;
        let mut delay = INITIAL_BACKOFF;
        loop {
            match self.connect_once().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    on_retry(attempt + 1, &e);
                    attempt += 1;
                    if attempt > max_retries {
                        return Err(e);
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

    async fn lookup_country_code(client: &Client, ip: &str) -> Option<String> {
        if ip.contains('?') {
            return None;
        }
        let addr = ip
            .parse::<SocketAddr>()
            .map(|sa| sa.ip().to_string())
            .unwrap_or_else(|_| ip.split(':').next().unwrap_or(ip).to_string());
        let url = format!("https://ipapi.co/{}/country/", addr);
        if let Ok(resp) = client.get(url).send().await {
            if resp.status().is_success() {
                if let Ok(text) = resp.text().await {
                    let code = text.trim();
                    if code.len() == 2 {
                        return Some(code.to_uppercase());
                    }
                }
            }
        }
        None
    }

    pub async fn set_exit_country(&self, country: Option<String>) -> Result<()> {
        let mut guard = self.exit_country.lock().await;
        if let Some(cc) = country {
            let code = CountryCode::new(&cc).map_err(|e| Error::Tor(e.to_string()))?;
            *guard = Some(code);
        } else {
            *guard = None;
        }
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        self.client.lock().await.is_some()
    }


    pub async fn new_identity(&self) -> Result<()> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;

        // Force new configuration and circuits
        client
            .reconfigure(&TorClientConfig::default())
            .map_err(|e| Error::Identity(e))?;
        client.retire_all_circs();

        // Build fresh circuit
        client
            .build_new_circuit()
            .await
            .map_err(|e| Error::Circuit(e))?;

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
            .get_or_launch_exit((&*netdir).into(), &[], StreamIsolation::no_isolation(), None)
            .await
            .map_err(|e| Error::Circuit(e.to_string()))?;

        let hops: Vec<_> = circuit
            .path_ref()
            .map_err(|e| Error::Circuit(e.to_string()))?
            .hops()
            .iter()
            .cloned()
            .collect();

        let http = Client::new();
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
                let country = Self::lookup_country_code(&http, &ip_address)
                    .await
                    .unwrap_or_else(|| "??".to_string());
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
        let token = tokens.entry(domain).or_insert_with(IsolationToken::new);

        let netdir = client
            .dirmgr()
            .netdir(Timeliness::Timely)
            .map_err(|e| Error::NetDir(e.to_string()))?;

        let isolation = StreamIsolation::builder()
            .owner_token(*token)
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
            .map_err(|e| Error::Circuit(e.to_string()))?;

        let hops: Vec<_> = circuit
            .path_ref()
            .map_err(|e| Error::Circuit(e.to_string()))?
            .hops()
            .iter()
            .cloned()
            .collect();

        let http = Client::new();
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
                let country = Self::lookup_country_code(&http, &ip_address)
                    .await
                    .unwrap_or_else(|| "??".to_string());
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
}
