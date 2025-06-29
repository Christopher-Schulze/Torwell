use crate::commands::RelayInfo;
use crate::error::{Error, Result};
use arti_client::{TorClient, TorClientConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tor_circmgr::isolation::StreamIsolation;
use tor_circmgr::{IsolationToken, TargetPort};
use tor_dirmgr::Timeliness;
use tor_linkspec::{HasAddrs, HasRelayIds};
use tor_rtcompat::PreferredRuntime;
use std::time::Duration;

pub struct TorManager {
    client: Arc<Mutex<Option<TorClient<PreferredRuntime>>>>,
    isolation_tokens: Arc<Mutex<HashMap<String, IsolationToken>>>,
    exit_ports: Arc<Mutex<Vec<u16>>>,
}

impl TorManager {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            isolation_tokens: Arc::new(Mutex::new(HashMap::new())),
            exit_ports: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        if self.is_connected().await {
            return Err(Error::AlreadyConnected);
        }
        const MAX_RETRIES: u32 = 5;
        let mut attempts = 0;
        let mut delay = Duration::from_secs(1);
        loop {
            let config = TorClientConfig::default();
            match TorClient::create_bootstrapped(config.clone()).await {
                Ok(tor_client) => {
                    *self.client.lock().await = Some(tor_client);
                    return Ok(());
                }
                Err(e) => {
                    attempts += 1;
                    if attempts > MAX_RETRIES {
                        return Err(e.into());
                    }
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, Duration::from_secs(32));
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

    pub async fn is_connected(&self) -> bool {
        self.client.lock().await.is_some()
    }

    pub async fn get_active_circuit(&self) -> Result<Vec<RelayInfo>> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;

        // We need a netdir to get an exit circuit.
        let netdir = client.dirmgr().netdir(Timeliness::Timely)?;
        // We get a circuit by requesting one for a generic exit.
        let circuit = client
            .circmgr()
            .get_or_launch_exit((&*netdir).into(), &[], StreamIsolation::no_isolation(), None)
            .await?;

        let relays = circuit
            .path_ref()?
            .hops()
            .iter()
            .map(|hop| {
                if let Some(relay) = hop.as_chan_target() {
                    // Use relay ID as identifier since nickname is no longer available
                    let nickname = relay.rsa_identity()
                        .map(|id| format!("${}", id.to_string().chars().take(8).collect::<String>()))
                        .unwrap_or_else(|| "$unknown".to_string());
                    let ip_address = relay.addrs().get(0).map_or_else(
                        || "?.?.?.?".to_string(),
                        |addr| addr.to_string(),
                    );
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

    pub async fn set_exit_policy(&self, ports: Vec<u16>) {
        let mut policy = self.exit_ports.lock().await;
        *policy = ports;
    }

    async fn isolation_for_domain(&self, domain: Option<String>) -> StreamIsolation {
        match domain {
            Some(d) => {
                let token = {
                    let mut map = self.isolation_tokens.lock().await;
                    map.entry(d).or_insert_with(IsolationToken::new).clone()
                };
                StreamIsolation::builder()
                    .owner_token(token)
                    .build()
                    .unwrap_or_else(|_| StreamIsolation::no_isolation())
            }
            None => StreamIsolation::no_isolation(),
        }
    }

    pub async fn get_circuit(&self, domain: Option<String>) -> Result<Vec<RelayInfo>> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;

        let netdir = client.dirmgr().netdir(Timeliness::Timely)?;
        let isolation = self.isolation_for_domain(domain).await;
        let ports = {
            let p = self.exit_ports.lock().await;
            p.iter().map(|port| TargetPort::ipv4(*port)).collect::<Vec<_>>()
        };
        let circuit = client
            .circmgr()
            .get_or_launch_exit((&*netdir).into(), &ports, isolation, None)
            .await?;

        let relays = circuit
            .path_ref()?
            .hops()
            .iter()
            .map(|hop| {
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
                    RelayInfo {
                        nickname,
                        ip_address,
                        country: "XX".to_string(),
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

    pub async fn new_identity(&self) -> Result<()> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;

        // Force new configuration and circuits
        client.reconfigure(&TorClientConfig::default())?;
        client.circmgr().retire_all_circs();
        
        // Build fresh circuit
        let netdir = client.dirmgr().netdir(Timeliness::Timely)?;
        client.circmgr().build_circuit((&*netdir).into(), &[], StreamIsolation::no_isolation(), None).await?;
        
        Ok(())
    }
}