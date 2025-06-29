use crate::commands::RelayInfo;
use crate::error::{Error, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use sysinfo::{NetworksExt, NetworkExt, RefreshKind, System};
use arti_client::{TorClient, TorClientConfig};
use std::sync::Arc;
use tokio::sync::Mutex;
use tor_circmgr::isolation::StreamIsolation;
use tor_dirmgr::Timeliness;
use tor_linkspec::{HasAddrs, HasRelayIds};
use tor_rtcompat::PreferredRuntime;

pub struct TorManager {
    client: Arc<Mutex<Option<TorClient<PreferredRuntime>>>>,
    start_sent: AtomicU64,
    start_received: AtomicU64,
}

impl TorManager {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            start_sent: AtomicU64::new(0),
            start_received: AtomicU64::new(0),
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
        self.capture_baseline();
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

    fn capture_baseline(&self) {
        let mut sys = System::new_with_specifics(RefreshKind::new().with_networks());
        sys.refresh_networks();
        let (sent, received) = sys
            .networks()
            .iter()
            .fold((0u64, 0u64), |acc, (_, data)| {
                (acc.0 + data.total_transmitted(), acc.1 + data.total_received())
            });
        self.start_sent.store(sent, Ordering::SeqCst);
        self.start_received.store(received, Ordering::SeqCst);
    }

    pub fn traffic_metrics(&self) -> (u64, u64) {
        let mut sys = System::new_with_specifics(RefreshKind::new().with_networks());
        sys.refresh_networks();
        let (sent, received) = sys
            .networks()
            .iter()
            .fold((0u64, 0u64), |acc, (_, data)| {
                (acc.0 + data.total_transmitted(), acc.1 + data.total_received())
            });
        (
            sent.saturating_sub(self.start_sent.load(Ordering::SeqCst)),
            received.saturating_sub(self.start_received.load(Ordering::SeqCst)),
        )
    }
}
