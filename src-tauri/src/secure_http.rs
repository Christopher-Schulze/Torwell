use reqwest::{Client, ClientBuilder};
use rustls::pki_types::CertificateDer;
use rustls::version::{TLS12, TLS13};
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile as pemfile;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct SecureHttpClient {
    client: Client,
    cert_path: String,
}

impl SecureHttpClient {
    pub fn new<P: AsRef<Path>>(cert_path: P) -> anyhow::Result<Self> {
        let path = cert_path.as_ref().to_owned();

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

        Ok(Self {
            client,
            cert_path: path.to_string_lossy().to_string(),
        })
    }

    pub async fn get_text(&self, url: &str) -> reqwest::Result<String> {
        let resp = self.client.get(url).send().await?;
        if resp.headers().get("strict-transport-security").is_none() {
            log::warn!("HSTS header missing for {}", url);
        }
        resp.text().await
    }

    pub async fn update_certificates(&self, url: &str) -> anyhow::Result<()> {
        match self.client.get(url).send().await {
            Ok(resp) => {
                let pem = resp.bytes().await?;
                std::fs::write(&self.cert_path, &pem)?;
            }
            Err(e) => {
                log::error!("failed to fetch new certificate: {}", e);
            }
        }
        Ok(())
    }
}
