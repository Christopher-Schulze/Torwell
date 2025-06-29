use crate::commands::RelayInfo;
use crate::error::{Error, Result};
use arti_client::{client::StreamPrefs, TorClient, TorClientConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tor_circmgr::isolation::{IsolationToken, StreamIsolation};
use tor_geoip::CountryCode;
use tor_dirmgr::Timeliness;
use tor_linkspec::{HasAddrs, HasRelayIds};
use tor_rtcompat::PreferredRuntime;

pub struct TorManager {
    client: Arc<Mutex<Option<TorClient<PreferredRuntime>>>>,
    isolation_tokens: Arc<Mutex<HashMap<String, IsolationToken>>>,
    exit_country: Arc<Mutex<Option<CountryCode>>>,
}

impl TorManager {
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
        let tor_client = TorClient::create_bootstrapped(config)
            .await
            .map_err(|e| Error::Bootstrap(e.to_string()))?;
        *self.client.lock().await = Some(tor_client);
        Ok(())
    }

    pub async fn connect(&self) -> Result<()> {
        self.connect_once().await
    }

    pub async fn connect_with_backoff(&self, max_retries: u32) -> Result<()> {
        let mut attempt = 0;
        let mut delay = std::time::Duration::from_secs(1);
        loop {
            match self.connect_once().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    attempt += 1;
                    if attempt > max_retries {
                        return Err(e);
                    }
                    tokio::time::sleep(delay).await;
                    delay *= 2;
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

    pub async fn get_active_circuit(&self) -> Result<Vec<RelayInfo>> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;

        // We need a netdir to get an exit circuit.
        let netdir = client
            .dirmgr()
            .netdir(Timeliness::Timely)
            .map_err(|e| Error::NetDir(e.to_string()))?;
        // We get a circuit by requesting one for a generic exit.
        let circuit = client
            .circmgr()
            .get_or_launch_exit(
                (&*netdir).into(),
                &[],
                StreamIsolation::no_isolation(),
                None,
            )
            .await
            .map_err(|e| Error::Circuit(e.to_string()))?;

        let relays = circuit
            .path_ref()
            .map_err(|e| Error::Circuit(e.to_string()))?
            .hops()
            .iter()
            .map(|hop| {
                if let Some(relay) = hop.as_chan_target() {
                    // Use relay ID as identifier since nickname is no longer available
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
                    RelayInfo {
                        nickname,
                        ip_address,
                        country: "XX".to_string(), // Placeholder
                    }
                } else {
                    RelayInfo {
                        nickname: "<virtual>".to_string(),
                        ip_address: "?.?.?.?".to_string(),
                        country: "XX".to_string(),
                    }
                }
            })
            .collect();

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

        let relays = circuit
            .path_ref()
            .map_err(|e| Error::Circuit(e.to_string()))?
            .hops()
            .iter()
            .map(|hop| {
                if let Some(relay) = hop.as_chan_target() {
                    let nickname = relay
                        .rsa_identity()
                        .map(|id| format!("${}", id.to_string().chars().take(8).collect::<String>()))
                        .unwrap_or_else(|| "$unknown".to_string());
                    let ip_address = relay
                        .addrs()
                        .get(0)
                        .map_or_else(|| "?.?.?.?".to_string(), |addr| addr.to_string());
                    RelayInfo { nickname, ip_address, country: "XX".to_string() }
                } else {
                    RelayInfo { nickname: "<virtual>".to_string(), ip_address: "?.?.?.?".to_string(), country: "XX".to_string() }
                }
            })
            .collect();

        Ok(relays)
    }

    pub async fn new_identity(&self) -> Result<()> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;

        // Force new configuration and circuits
        client
            .reconfigure(&TorClientConfig::default())
            .map_err(|e| Error::Identity(e.to_string()))?;
        client.circmgr().retire_all_circs();

        // Build fresh circuit
        let netdir = client
            .dirmgr()
            .netdir(Timeliness::Timely)
            .map_err(|e| Error::NetDir(e.to_string()))?;
        client
            .circmgr()
            .build_circuit(
                (&*netdir).into(),
                &[],
                StreamIsolation::no_isolation(),
                None,
            )
            .await
            .map_err(|e| Error::Circuit(e.to_string()))?;

        Ok(())
    }
}
