use crate::error::Result;
use crate::tor_manager::TorManager;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

pub struct AppState {
    pub tor_manager: Arc<TorManager>,
    /// Path to the persistent log file
    pub log_file: PathBuf,
    /// Mutex used to serialize file writes
    pub log_lock: Arc<Mutex<()>>,
    /// Maximum number of lines to retain in the log file
    pub max_log_lines: usize,
}

impl Default for AppState {
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

    pub async fn add_log(&self, message: String) -> Result<()> {
        let _guard = self.log_lock.lock().await;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .await?;
        file.write_all(message.as_bytes()).await?;
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

    pub async fn read_logs(&self) -> Result<Vec<String>> {
        let _guard = self.log_lock.lock().await;
        let contents = fs::read_to_string(&self.log_file).await.unwrap_or_default();
        Ok(contents.lines().map(|l| l.to_string()).collect())
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
