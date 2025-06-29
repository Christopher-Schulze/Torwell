use reqwest::{Client, ClientBuilder};
use rustls::pki_types::CertificateDer;
use rustls::version::{TLS12, TLS13};
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile as pemfile;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Location of the pinned server certificate
pub const DEFAULT_CERT_PATH: &str = "certs/server.pem";

pub struct SecureHttpClient {
    client: Arc<Mutex<Client>>,
    cert_path: String,
}

impl Clone for SecureHttpClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            cert_path: self.cert_path.clone(),
        }
    }
}

impl SecureHttpClient {
    fn build_client<P: AsRef<Path>>(path: P) -> anyhow::Result<Client> {
        let mut store = RootCertStore::empty();
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let certs: Vec<CertificateDer<'static>> =
            pemfile::certs(&mut reader).collect::<Result<_, _>>()?;
        store.add_parsable_certificates(certs);

        let config = ClientConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&TLS13, &TLS12])?
            .with_root_certificates(store)
            .with_no_client_auth();

        let client = ClientBuilder::new()
            .use_preconfigured_tls(config)
            .https_only(true)
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .build()?;
        Ok(client)
    }

    pub fn new<P: AsRef<Path>>(cert_path: P) -> anyhow::Result<Self> {
        let path = cert_path.as_ref().to_owned();
        let client = Self::build_client(&path)?;
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            cert_path: path.to_string_lossy().to_string(),
        })
    }

    pub fn new_default() -> anyhow::Result<Self> {
        Self::new(DEFAULT_CERT_PATH)
    }

    /// Initialize a client using the default certificate and optionally start
    /// periodic updates if a URL is provided.
    pub async fn init(
        cert_url: Option<String>,
        interval: Option<Duration>,
    ) -> anyhow::Result<Arc<Self>> {
        let client = Arc::new(Self::new_default()?);
        if let Some(url) = cert_url.clone() {
            // Perform an initial check on startup
            if let Err(e) = client.update_certificates(&url).await {
                log::error!("initial certificate update failed: {}", e);
            }
        }
        if let (Some(url), Some(int)) = (cert_url, interval) {
            client.clone().schedule_updates(url, int);
        }
        Ok(client)
    }

    pub async fn get_text(&self, url: &str) -> reqwest::Result<String> {
        let client = self.client.lock().await;
        let resp = client.get(url).send().await?;
        if resp.headers().get("strict-transport-security").is_none() {
            log::warn!("HSTS header missing for {}", url);
        }
        resp.text().await
    }

    pub async fn reload_certificates(&self) -> anyhow::Result<()> {
        let client = Self::build_client(&self.cert_path)?;
        let mut guard = self.client.lock().await;
        *guard = client;
        Ok(())
    }

    pub async fn update_certificates(&self, url: &str) -> anyhow::Result<()> {
        let resp = {
            let client = self.client.lock().await;
            client.get(url).send().await
        };
        match resp {
            Ok(resp) => {
                let pem = resp.bytes().await?;
                if let Some(parent) = Path::new(&self.cert_path).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&self.cert_path, &pem)?;
                self.reload_certificates().await?;
            }
            Err(e) => {
                log::error!("failed to fetch new certificate: {}", e);
            }
        }
        Ok(())
    }

    pub fn schedule_updates(self: Arc<Self>, url: String, interval: Duration) {
        tokio::spawn(async move {
            loop {
                if let Err(e) = self.update_certificates(&url).await {
                    log::error!("certificate update failed: {}", e);
                }
                tokio::time::sleep(interval).await;
            }
        });
    }
}
