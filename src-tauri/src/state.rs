use crate::error::Result;
use crate::secure_http::SecureHttpClient;
use crate::tor_manager::{TorClientBehavior, TorManager};
use arti_client::TorClient;
use chrono::Utc;
use log::Level;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tor_rtcompat::PreferredRuntime;

/// Default location of the application configuration file
pub const DEFAULT_CONFIG_PATH: &str = "src-tauri/app_config.json";

/// Default number of log lines to keep when no setting is provided
pub const DEFAULT_MAX_LOG_LINES: usize = 1000;

#[derive(Deserialize, Default)]
struct AppConfig {
    #[serde(default = "default_max_log_lines")]
    max_log_lines: usize,
}

fn default_max_log_lines() -> usize {
    DEFAULT_MAX_LOG_LINES
}

impl AppConfig {
    fn load<P: AsRef<Path>>(path: P) -> Self {
        let data = std::fs::read_to_string(path).ok();
        if let Some(data) = data {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    pub level: String,
    pub timestamp: String,
    pub message: String,
}

#[derive(Clone)]
pub struct AppState<C: TorClientBehavior = TorClient<PreferredRuntime>> {
    pub tor_manager: Arc<TorManager<C>>,
    pub http_client: Arc<SecureHttpClient>,
    /// Path to the persistent log file
    pub log_file: PathBuf,
    /// Mutex used to serialize file writes
    pub log_lock: Arc<Mutex<()>>,
    /// Counter for connection retries
    pub retry_counter: Arc<Mutex<u32>>,
    /// Maximum number of lines to retain in the log file
    pub max_log_lines: Arc<Mutex<usize>>,
    /// Current memory usage in bytes
    pub memory_usage: Arc<Mutex<u64>>,
    /// Current number of circuits
    pub circuit_count: Arc<Mutex<usize>>,
    /// Maximum memory usage before warning (in MB)
    pub max_memory_mb: u64,
    /// Maximum number of circuits before warning
    pub max_circuits: usize,
}

impl<C: TorClientBehavior> Default for AppState<C> {
    fn default() -> Self {
        let log_file = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("torwell.log");
        if let Some(parent) = log_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let cfg = AppConfig::load(DEFAULT_CONFIG_PATH);
        let max_log_lines = std::env::var("TORWELL_MAX_LOG_LINES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(cfg.max_log_lines);

        Self {
            tor_manager: Arc::new(TorManager::new()),
            http_client: Arc::new(
                SecureHttpClient::new_default().expect("failed to create http client"),
            ),
            log_file,
            log_lock: Arc::new(Mutex::new(())),
            retry_counter: Arc::new(Mutex::new(0)),
            max_log_lines: Arc::new(Mutex::new(max_log_lines)),
            memory_usage: Arc::new(Mutex::new(0)),
            circuit_count: Arc::new(Mutex::new(0)),
            max_memory_mb: std::env::var("TORWELL_MAX_MEMORY_MB")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1024),
            max_circuits: std::env::var("TORWELL_MAX_CIRCUITS")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(20),
        }
    }
}

impl<C: TorClientBehavior> AppState<C> {
    pub fn new(http_client: Arc<SecureHttpClient>) -> Self {
        let log_file = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("torwell.log");
        if let Some(parent) = log_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let cfg = AppConfig::load(DEFAULT_CONFIG_PATH);
        let max_log_lines = std::env::var("TORWELL_MAX_LOG_LINES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(cfg.max_log_lines);

        AppState {
            tor_manager: Arc::new(TorManager::new()),
            http_client,
            log_file,
            log_lock: Arc::new(Mutex::new(())),
            retry_counter: Arc::new(Mutex::new(0)),
            max_log_lines: Arc::new(Mutex::new(max_log_lines)),
            memory_usage: Arc::new(Mutex::new(0)),
            circuit_count: Arc::new(Mutex::new(0)),
            max_memory_mb: std::env::var("TORWELL_MAX_MEMORY_MB")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1024),
            max_circuits: std::env::var("TORWELL_MAX_CIRCUITS")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(20),
        }
    }

    pub async fn increment_retry_counter(&self) {
        let mut guard = self.retry_counter.lock().await;
        *guard += 1;
    }

    pub async fn reset_retry_counter(&self) {
        let mut guard = self.retry_counter.lock().await;
        *guard = 0;
    }

    pub async fn add_log(&self, level: Level, message: String) -> Result<()> {
        let _guard = self.log_lock.lock().await;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .await?;
        let entry = LogEntry {
            level: level.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            message,
        };
        let json = serde_json::to_string(&entry)?;
        file.write_all(json.as_bytes()).await?;
        file.write_all(b"\n").await?;
        drop(file);
        self.trim_logs().await?;
        Ok(())
    }

    pub async fn set_max_log_lines(&self, limit: usize) -> Result<()> {
        *self.max_log_lines.lock().await = limit;
        self.trim_logs().await
    }

    async fn trim_logs(&self) -> Result<()> {
        let contents = fs::read_to_string(&self.log_file).await.unwrap_or_default();
        let max_lines = *self.max_log_lines.lock().await;
        let mut lines: Vec<&str> = contents.lines().collect();
        if lines.len() > max_lines {
            lines = lines[lines.len() - max_lines..].to_vec();
            fs::write(&self.log_file, lines.join("\n")).await?;
        }
        Ok(())
    }

    pub async fn read_logs(&self) -> Result<Vec<LogEntry>> {
        let _guard = self.log_lock.lock().await;
        let contents = fs::read_to_string(&self.log_file).await.unwrap_or_default();
        let mut entries = Vec::new();
        for line in contents.lines() {
            if let Ok(entry) = serde_json::from_str::<LogEntry>(line) {
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    pub async fn clear_log_file(&self) -> Result<()> {
        let _guard = self.log_lock.lock().await;
        fs::write(&self.log_file, b"").await?;
        Ok(())
    }

    /// Return the path to the log file as a string
    pub fn log_file_path(&self) -> String {
        self.log_file.to_string_lossy().into()
    }

    /// Update stored metrics
    pub async fn update_metrics(&self, memory: u64, circuits: usize) {
        *self.memory_usage.lock().await = memory;
        *self.circuit_count.lock().await = circuits;
    }

    /// Retrieve current metrics
    pub async fn metrics(&self) -> (u64, usize) {
        let mem = *self.memory_usage.lock().await;
        let circ = *self.circuit_count.lock().await;
        (mem, circ)
    }
}
