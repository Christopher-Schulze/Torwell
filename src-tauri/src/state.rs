use crate::error::Result;
use crate::tor_manager::{TorClientBehavior, TorManager};
use arti_client::TorClient;
use chrono::Utc;
use log::Level;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    pub level: String,
    pub timestamp: String,
    pub message: String,
}

pub struct AppState<C: TorClientBehavior = TorClient<PreferredRuntime>> {
    pub tor_manager: Arc<TorManager<C>>,
    /// Path to the persistent log file
    pub log_file: PathBuf,
    /// Mutex used to serialize file writes
    pub log_lock: Arc<Mutex<()>>,
    /// Maximum number of lines to retain in the log file
    pub max_log_lines: usize,
}

impl<C: TorClientBehavior> Default for AppState<C> {
    fn default() -> Self {
        let log_file = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("torwell.log");
        if let Some(parent) = log_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let max_log_lines = std::env::var("TORWELL_MAX_LOG_LINES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(Self::DEFAULT_MAX_LINES);

        Self {
            tor_manager: Arc::new(TorManager::new()),
            log_file,
            log_lock: Arc::new(Mutex::new(())),
            max_log_lines,
        }
    }
}

impl AppState {
    const DEFAULT_MAX_LINES: usize = 1000;

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

    async fn trim_logs(&self) -> Result<()> {
        let contents = fs::read_to_string(&self.log_file).await.unwrap_or_default();
        let mut lines: Vec<&str> = contents.lines().collect();
        if lines.len() > self.max_log_lines {
            lines = lines[lines.len() - self.max_log_lines..].to_vec();
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
}
