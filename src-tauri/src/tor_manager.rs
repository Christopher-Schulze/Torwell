use crate::commands::RelayInfo;
use crate::error::{Error, Result};
use arti_client::{TorClient, TorClientConfig, ReconfigureMode};
use std::sync::Arc;
use tokio::sync::Mutex;
use tor_rtcompat::PreferredRuntime;

pub struct TorManager {
    client: Arc<Mutex<Option<TorClient<PreferredRuntime>>>>,
}

impl TorManager {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        if self.is_connected().await {
            return Err(Error::AlreadyConnected);
        }
        let config = TorClientConfig::default();
        let tor_client = TorClient::create_bootstrapped(config).await?;
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

    pub async fn is_connected(&self) -> bool {
        self.client.lock().await.is_some()
    }

    pub async fn get_active_circuit(&self) -> Result<Vec<RelayInfo>> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;
        
        let circuit = client.get_circ_mgr()?.peek_for_exit(None, None)?;

        let relays = circuit
            .path().unwrap().hops().iter()
            .map(|hop| {
                let nickname = hop.nickname().unwrap_or("$unknown").to_string();
                let ip_address = hop.addrs().get(0).map_or_else(
                    || "?.?.?.?".to_string(),
                    |addr| addr.to_string(),
                );
                // The 'arti-client' API does not directly expose country information for relays in a simple way.
                // This would require a GeoIP lookup, which is out of scope for this implementation.
                let country = "XX".to_string();
                RelayInfo {
                    nickname,
                    ip_address,
                    country,
                }
            })
            .collect();

        Ok(relays)
    }

    pub async fn new_identity(&self) -> Result<()> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or(Error::NotConnected)?;
        
        client.reconfigure(&Default::default(), arti_client::config::ReconfigureMode::ExpireAllCircuits)?;
        Ok(())
    }
}