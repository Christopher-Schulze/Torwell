use crate::commands::RelayInfo;
use crate::error::{Error, Result};
use arti_client::{TorClient, TorClientConfig};
use maxminddb::{geoip2::Country, Reader};
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tor_circmgr::isolation::StreamIsolation;
use tor_dirmgr::Timeliness;
use tor_linkspec::{HasAddrs, HasRelayIds};
use tor_rtcompat::PreferredRuntime;

pub struct TorManager {
    client: Arc<Mutex<Option<TorClient<PreferredRuntime>>>>,
    geoip_reader: Arc<Mutex<Option<Reader<Vec<u8>>>>>,
    geoip_path: PathBuf,
}

impl TorManager {
    pub fn new() -> Self {
        let geoip_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("GeoLite2-Country.mmdb");
        let geoip_reader = Reader::open_readfile(&geoip_path).ok();
        Self {
            client: Arc::new(Mutex::new(None)),
            geoip_reader: Arc::new(Mutex::new(geoip_reader)),
            geoip_path,
        }
    }

    pub async fn connect(&self) -> Result<()> {
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

    pub async fn disconnect(&self) -> Result<()> {
        let mut client_guard = self.client.lock().await;
        if client_guard.take().is_none() {
            return Err(Error::NotConnected);
        }
        // Client is dropped here, which handles shutdown.
        Ok(())
    }

    async fn lookup_country(&self, ip: IpAddr) -> Option<String> {
        let reader_guard = self.geoip_reader.lock().await;
        if let Some(reader) = reader_guard.as_ref() {
            if let Ok(country) = reader.lookup::<Country>(ip) {
                if let Some(info) = country.country {
                    if let Some(code) = info.iso_code {
                        return Some(code.to_string());
                    }
                }
            }
        }
        None
    }

    pub async fn update_geoip_database(&self, url: &str) -> Result<()> {
        let bytes = reqwest::get(url).await?.bytes().await?;
        tokio::fs::write(&self.geoip_path, &bytes).await?;
        let reader = Reader::open_readfile(&self.geoip_path).map_err(|e| Error::Io(e.to_string()))?;
        *self.geoip_reader.lock().await = Some(reader);
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

        let mut relays = Vec::new();
        let hops = circuit
            .path_ref()
            .map_err(|e| Error::Circuit(e.to_string()))?
            .hops();
        for hop in hops.iter() {
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
                let country = if let Ok(ip) = ip_address.split(':').next().unwrap_or("").parse::<IpAddr>() {
                    self.lookup_country(ip).await.unwrap_or_else(|| "??".to_string())
                } else {
                    "??".to_string()
                };
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
