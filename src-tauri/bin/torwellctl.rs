use anyhow::Result;
use clap::{Parser, Subcommand};
use torwell84::secure_http::{SecureHttpClient, DEFAULT_CONFIG_PATH};
use torwell84::state::AppState;

#[derive(Parser)]
#[command(
    name = "torwellctl",
    author,
    version,
    about = "Torwell84 command line utility"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import a worker list from a text file
    ImportWorkers {
        /// Path to file with one URL per line
        file: String,
        /// Optional worker authentication token
        #[arg(short, long)]
        token: Option<String>,
    },
    /// Export currently configured workers
    ExportWorkers,
    /// Manually update pinned certificate
    UpdateCert,
    /// Display stored metrics
    ShowMetrics,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let client = SecureHttpClient::init(DEFAULT_CONFIG_PATH, None, None, None, None).await?;
    match args.command {
        Commands::ImportWorkers { file, token } => {
            let content = std::fs::read_to_string(file)?;
            let workers: Vec<String> = content
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            client.set_worker_config(workers.clone(), token).await;
            println!("Imported {} workers", workers.len());
        }
        Commands::ExportWorkers => {
            let workers = client.worker_urls().await;
            for w in workers {
                println!("{}", w);
            }
        }
        Commands::UpdateCert => {
            let cfg: serde_json::Value = std::fs::read_to_string(DEFAULT_CONFIG_PATH)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let mut urls = vec![cfg
                .get("cert_url")
                .and_then(|v| v.as_str())
                .unwrap_or(torwell84::secure_http::DEFAULT_CERT_URL)
                .to_string()];
            if let Some(fb) = cfg.get("fallback_cert_url").and_then(|v| v.as_str()) {
                urls.push(fb.to_string());
            }
            if let Ok(env_url) = std::env::var("TORWELL_CERT_URL") {
                urls[0] = env_url;
            }
            if let Ok(env_fb) = std::env::var("TORWELL_FALLBACK_CERT_URL") {
                if urls.len() == 1 {
                    urls.push(env_fb);
                } else {
                    urls[1] = env_fb;
                }
            }
            client.update_certificates_from(&urls).await?;
            println!("Certificate updated");
        }
        Commands::ShowMetrics => {
            let state = AppState::new(client.clone());
            let metrics = state.load_metrics().await?;
            for m in metrics {
                println!("{:#?}", m);
            }
        }
    }
    Ok(())
}
