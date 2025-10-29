use crate::core::executor::{SchedulerSnapshot, TaskScheduler};
use crate::error::{Error, Result};
use crate::icmp;
use crate::secure_http::SecureHttpClient;
use crate::session::SessionManager;
use crate::tor_manager::{TorClientBehavior, TorManager};
use arti_client::TorClient;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use directories::ProjectDirs;
use log::Level;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{PidExt, System, SystemExt};
#[cfg(target_os = "macos")]
use tauri::NativeImage;
use tauri::{AppHandle, CustomMenuItem, SystemTrayMenu};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, RwLock};
use tor_rtcompat::PreferredRuntime;

/// Default location of the application configuration file
pub const DEFAULT_CONFIG_PATH: &str = "src-tauri/app_config.json";

/// Default number of log lines retained if no configuration is provided
pub const DEFAULT_MAX_LOG_LINES: usize = 1000;
/// Default session token lifetime in seconds
pub const DEFAULT_SESSION_TTL: u64 = 3600;
/// Default maximum number of metric lines
pub const DEFAULT_MAX_METRIC_LINES: usize = 10_000;
/// Default maximum metrics file size in megabytes
pub const DEFAULT_MAX_METRIC_MB: usize = 5;
/// Default interval for metric collection in seconds
pub const DEFAULT_METRIC_INTERVAL_SECS: u64 = 30;
/// Default number of metric entries returned when no limit is specified
pub const DEFAULT_METRIC_FETCH_LIMIT: usize = 100;
/// Default number of connection timeline events to retain
pub const DEFAULT_MAX_CONNECTION_EVENTS: usize = 720;

#[derive(Deserialize, Default)]
struct AppConfig {
    #[serde(default = "default_max_log_lines")]
    max_log_lines: usize,
    #[serde(default)]
    geoip_path: Option<String>,
    #[serde(default)]
    metrics_file: Option<String>,
    #[serde(default = "default_max_metric_lines")]
    max_metric_lines: usize,
    #[serde(default = "default_max_metric_mb")]
    max_metric_mb: usize,
    #[serde(default = "default_insecure_hosts")]
    insecure_allowed_hosts: Vec<String>,
}

fn default_max_log_lines() -> usize {
    DEFAULT_MAX_LOG_LINES
}

fn default_max_metric_lines() -> usize {
    DEFAULT_MAX_METRIC_LINES
}

fn default_max_metric_mb() -> usize {
    DEFAULT_MAX_METRIC_MB
}

fn default_insecure_hosts() -> Vec<String> {
    vec!["127.0.0.1".into(), "localhost".into()]
}

impl AppConfig {
    fn load<P: AsRef<Path>>(path: P) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    pub level: String,
    pub timestamp: String,
    pub message: String,
    #[serde(default)]
    pub stack: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MetricPoint {
    #[serde(rename = "time")]
    pub time: i64,
    #[serde(rename = "memoryMB")]
    pub memory_mb: u64,
    #[serde(rename = "circuitCount")]
    pub circuit_count: usize,
    #[serde(rename = "latencyMs")]
    pub latency_ms: u64,
    #[serde(rename = "oldestAge")]
    pub oldest_age: u64,
    #[serde(rename = "avgCreateMs")]
    pub avg_create_ms: u64,
    #[serde(rename = "failedAttempts")]
    pub failed_attempts: u64,
    #[serde(rename = "cpuPercent")]
    pub cpu_percent: f32,
    #[serde(rename = "networkBytes")]
    pub network_bytes: u64,
    #[serde(rename = "networkTotal")]
    #[serde(default)]
    pub network_total: u64,
    #[serde(rename = "complete")]
    #[serde(default)]
    pub complete: bool,
    #[serde(rename = "schedulerP50Us")]
    #[serde(default)]
    pub scheduler_p50_us: u64,
    #[serde(rename = "schedulerP95Us")]
    #[serde(default)]
    pub scheduler_p95_us: u64,
    #[serde(rename = "schedulerP99Us")]
    #[serde(default)]
    pub scheduler_p99_us: u64,
    #[serde(rename = "schedulerQueueDepth")]
    #[serde(default)]
    pub scheduler_queue_depth: u64,
}

#[derive(Clone, Debug)]
pub struct ConnectionEvent {
    pub timestamp: DateTime<Utc>,
    pub status: String,
    pub message: Option<String>,
    pub detail: Option<String>,
    pub retry_count: Option<u32>,
    pub latency_ms: Option<u64>,
    pub memory_bytes: Option<u64>,
    pub circuit_count: Option<usize>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionEventSnapshot {
    pub timestamp: String,
    pub status: String,
    pub message: Option<String>,
    pub detail: Option<String>,
    pub retry_count: Option<u32>,
    pub latency_ms: Option<u64>,
    pub memory_bytes: Option<u64>,
    pub circuit_count: Option<usize>,
}

impl From<&ConnectionEvent> for ConnectionEventSnapshot {
    fn from(event: &ConnectionEvent) -> Self {
        Self {
            timestamp: event.timestamp.to_rfc3339(),
            status: event.status.clone(),
            message: event.message.clone(),
            detail: event.detail.clone(),
            retry_count: event.retry_count,
            latency_ms: event.latency_ms,
            memory_bytes: event.memory_bytes,
            circuit_count: event.circuit_count,
        }
    }
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionHealthSummary {
    pub total_events: usize,
    pub connected_events: usize,
    pub error_events: usize,
    pub disconnect_events: usize,
    pub last_event: Option<ConnectionEventSnapshot>,
    pub last_connected_at: Option<String>,
    pub last_error_at: Option<String>,
    pub current_uptime_seconds: Option<u64>,
    pub longest_uptime_seconds: Option<u64>,
    pub availability_percent: f32,
    pub retry_attempts_last_hour: u32,
}

#[derive(Clone)]
pub struct AppState<C: TorClientBehavior = TorClient<PreferredRuntime>> {
    pub tor_manager: Arc<RwLock<Arc<TorManager<C>>>>,
    pub http_client: Arc<SecureHttpClient>,
    pub scheduler: TaskScheduler,
    /// Path to the persistent log file
    pub log_file: PathBuf,
    /// Mutex used to serialize file writes
    pub log_lock: Arc<Mutex<()>>,
    /// Optional path to store metric points
    pub metrics_file: Option<PathBuf>,
    /// Mutex used to serialize metric writes
    pub metrics_lock: Arc<Mutex<()>>,
    /// Counter for connection retries
    pub retry_counter: Arc<Mutex<u32>>,
    /// Maximum number of lines to retain in the log file
    pub max_log_lines: Arc<Mutex<usize>>,
    /// Maximum number of metric lines retained
    pub max_metric_lines: usize,
    /// Maximum metrics file size in megabytes
    pub max_metric_mb: usize,
    /// Current memory usage in bytes
    pub memory_usage: Arc<Mutex<u64>>,
    /// Current number of circuits
    pub circuit_count: Arc<Mutex<usize>>,
    /// Age of the oldest circuit in seconds
    pub oldest_circuit_age: Arc<Mutex<u64>>,
    /// Last recorded network latency in milliseconds
    pub latency_ms: Arc<Mutex<u64>>,
    /// Current CPU usage percentage
    pub cpu_usage: Arc<Mutex<f32>>,
    /// Network throughput in bytes per second
    pub network_throughput: Arc<Mutex<u64>>,
    /// Total network traffic in bytes since start
    pub network_total: Arc<Mutex<u64>>,
    /// Total traffic bytes at the last metrics update
    pub prev_traffic: Arc<Mutex<u64>>,
    /// Maximum memory usage before warning (in MB)
    pub max_memory_mb: u64,
    /// Maximum number of circuits before warning
    pub max_circuits: usize,
    /// Interval for metrics collection in seconds
    pub metric_interval_secs: u64,
    /// Rolling buffer of connection state changes
    pub connection_events: Arc<Mutex<Vec<ConnectionEvent>>>,
    /// Maximum number of connection events to retain
    pub max_connection_events: usize,
    /// Session manager for authentication tokens
    pub session: Arc<SessionManager>,
    /// Handle used to emit frontend events
    pub app_handle: Arc<Mutex<Option<AppHandle>>>,
    /// Current warning shown in the tray menu
    pub tray_warning: Arc<Mutex<Option<String>>>,
    /// Flag to avoid concurrent auto reconnect attempts
    pub reconnect_in_progress: Arc<Mutex<bool>>,
    /// Timestamp when the client last entered the connected state
    pub connected_since: Arc<Mutex<Option<DateTime<Utc>>>>,
}

impl<C: TorClientBehavior> Default for AppState<C> {
    fn default() -> Self {
        let log_file = if let Some(proj) = ProjectDirs::from("", "", "torwell84") {
            let path = proj.data_dir().join("torwell.log");
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            path
        } else {
            let path = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("torwell.log");
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            path
        };

        let cfg = AppConfig::load(DEFAULT_CONFIG_PATH);
        let mut max_log_lines = cfg.max_log_lines;
        let mut max_metric_lines = cfg.max_metric_lines;
        let mut max_metric_mb = cfg.max_metric_mb;
        let mut geoip_path = cfg.geoip_path.clone();
        let mut max_connection_events = DEFAULT_MAX_CONNECTION_EVENTS;
        let metric_interval_secs = std::env::var("TORWELL_METRIC_INTERVAL")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_METRIC_INTERVAL_SECS);
        if let Ok(val) = std::env::var("TORWELL_MAX_LOG_LINES") {
            if let Ok(n) = val.parse::<usize>() {
                max_log_lines = n;
            }
        }
        if let Ok(val) = std::env::var("TORWELL_MAX_METRIC_LINES") {
            if let Ok(n) = val.parse::<usize>() {
                max_metric_lines = n;
            }
        }
        if let Ok(val) = std::env::var("TORWELL_MAX_METRIC_MB") {
            if let Ok(n) = val.parse::<usize>() {
                max_metric_mb = n;
            }
        }
        if let Ok(val) = std::env::var("TORWELL_MAX_CONNECTION_EVENTS") {
            if let Ok(n) = val.parse::<usize>() {
                max_connection_events = n.max(10);
            }
        }
        if let Ok(p) = std::env::var("TORWELL_GEOIP_PATH") {
            geoip_path = Some(p);
        }

        let http_client =
            Arc::new(SecureHttpClient::new_default().expect("failed to create http client"));
        http_client.set_insecure_hosts(cfg.insecure_allowed_hosts.clone());

        Self {
            tor_manager: Arc::new(RwLock::new(Arc::new(TorManager::new_with_geoip(
                geoip_path.clone(),
            )))),
            http_client,
            scheduler: TaskScheduler::global(),
            log_file,
            log_lock: Arc::new(Mutex::new(())),
            metrics_file: cfg
                .metrics_file
                .clone()
                .or_else(|| std::env::var("TORWELL_METRICS_FILE").ok())
                .map(PathBuf::from)
                .or_else(|| {
                    if let Some(proj) = ProjectDirs::from("", "", "torwell84") {
                        let path = proj.data_dir().join("metrics.json");
                        if let Some(p) = path.parent() {
                            let _ = std::fs::create_dir_all(p);
                        }
                        Some(path)
                    } else {
                        let path = std::env::current_dir()
                            .unwrap_or_else(|_| PathBuf::from("."))
                            .join("metrics.json");
                        if let Some(p) = path.parent() {
                            let _ = std::fs::create_dir_all(p);
                        }
                        Some(path)
                    }
                }),
            metrics_lock: Arc::new(Mutex::new(())),
            retry_counter: Arc::new(Mutex::new(0)),
            max_log_lines: Arc::new(Mutex::new(max_log_lines)),
            max_metric_lines,
            max_metric_mb,
            memory_usage: Arc::new(Mutex::new(0)),
            circuit_count: Arc::new(Mutex::new(0)),
            oldest_circuit_age: Arc::new(Mutex::new(0)),
            latency_ms: Arc::new(Mutex::new(0)),
            cpu_usage: Arc::new(Mutex::new(0.0)),
            network_throughput: Arc::new(Mutex::new(0)),
            network_total: Arc::new(Mutex::new(0)),
            prev_traffic: Arc::new(Mutex::new(0)),
            max_memory_mb: std::env::var("TORWELL_MAX_MEMORY_MB")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1024),
            max_circuits: std::env::var("TORWELL_MAX_CIRCUITS")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(20),
            metric_interval_secs,
            connection_events: Arc::new(Mutex::new(Vec::new())),
            max_connection_events,
            session: SessionManager::new(Duration::from_secs(
                std::env::var("TORWELL_SESSION_TTL")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_SESSION_TTL),
            )),
            app_handle: Arc::new(Mutex::new(None)),
            tray_warning: Arc::new(Mutex::new(None)),
            reconnect_in_progress: Arc::new(Mutex::new(false)),
            connected_since: Arc::new(Mutex::new(None)),
        }
    }
}

impl<C: TorClientBehavior> AppState<C> {
    pub fn new(http_client: Arc<SecureHttpClient>) -> Self {
        let log_file = if let Some(proj) = ProjectDirs::from("", "", "torwell84") {
            let path = proj.data_dir().join("torwell.log");
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            path
        } else {
            let path = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("torwell.log");
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            path
        };

        let cfg = AppConfig::load(DEFAULT_CONFIG_PATH);
        let mut max_log_lines = cfg.max_log_lines;
        let mut max_metric_lines = cfg.max_metric_lines;
        let mut max_metric_mb = cfg.max_metric_mb;
        let mut geoip_path = cfg.geoip_path.clone();
        let mut max_connection_events = DEFAULT_MAX_CONNECTION_EVENTS;
        let metric_interval_secs = std::env::var("TORWELL_METRIC_INTERVAL")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_METRIC_INTERVAL_SECS);
        if let Ok(val) = std::env::var("TORWELL_MAX_LOG_LINES") {
            if let Ok(n) = val.parse::<usize>() {
                max_log_lines = n;
            }
        }
        if let Ok(val) = std::env::var("TORWELL_MAX_METRIC_LINES") {
            if let Ok(n) = val.parse::<usize>() {
                max_metric_lines = n;
            }
        }
        if let Ok(val) = std::env::var("TORWELL_MAX_METRIC_MB") {
            if let Ok(n) = val.parse::<usize>() {
                max_metric_mb = n;
            }
        }
        if let Ok(val) = std::env::var("TORWELL_MAX_CONNECTION_EVENTS") {
            if let Ok(n) = val.parse::<usize>() {
                max_connection_events = n.max(10);
            }
        }
        if let Ok(p) = std::env::var("TORWELL_GEOIP_PATH") {
            geoip_path = Some(p);
        }

        http_client.set_insecure_hosts(cfg.insecure_allowed_hosts.clone());

        AppState {
            tor_manager: Arc::new(RwLock::new(Arc::new(TorManager::new_with_geoip(
                geoip_path.clone(),
            )))),
            http_client,
            scheduler: TaskScheduler::global(),
            log_file,
            log_lock: Arc::new(Mutex::new(())),
            metrics_file: cfg
                .metrics_file
                .clone()
                .or_else(|| std::env::var("TORWELL_METRICS_FILE").ok())
                .map(PathBuf::from)
                .or_else(|| {
                    if let Some(proj) = ProjectDirs::from("", "", "torwell84") {
                        let path = proj.data_dir().join("metrics.json");
                        if let Some(p) = path.parent() {
                            let _ = std::fs::create_dir_all(p);
                        }
                        Some(path)
                    } else {
                        let path = std::env::current_dir()
                            .unwrap_or_else(|_| PathBuf::from("."))
                            .join("metrics.json");
                        if let Some(p) = path.parent() {
                            let _ = std::fs::create_dir_all(p);
                        }
                        Some(path)
                    }
                }),
            metrics_lock: Arc::new(Mutex::new(())),
            retry_counter: Arc::new(Mutex::new(0)),
            max_log_lines: Arc::new(Mutex::new(max_log_lines)),
            max_metric_lines,
            max_metric_mb,
            memory_usage: Arc::new(Mutex::new(0)),
            circuit_count: Arc::new(Mutex::new(0)),
            oldest_circuit_age: Arc::new(Mutex::new(0)),
            latency_ms: Arc::new(Mutex::new(0)),
            cpu_usage: Arc::new(Mutex::new(0.0)),
            network_throughput: Arc::new(Mutex::new(0)),
            network_total: Arc::new(Mutex::new(0)),
            prev_traffic: Arc::new(Mutex::new(0)),
            max_memory_mb: std::env::var("TORWELL_MAX_MEMORY_MB")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1024),
            max_circuits: std::env::var("TORWELL_MAX_CIRCUITS")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(20),
            metric_interval_secs,
            connection_events: Arc::new(Mutex::new(Vec::new())),
            max_connection_events,
            session: SessionManager::new(Duration::from_secs(
                std::env::var("TORWELL_SESSION_TTL")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_SESSION_TTL),
            )),
            app_handle: Arc::new(Mutex::new(None)),
            tray_warning: Arc::new(Mutex::new(None)),
        }
    }

    const DEFAULT_MAX_LINES: usize = DEFAULT_MAX_LOG_LINES;

    pub async fn increment_retry_counter(&self) {
        let mut guard = self.retry_counter.lock().await;
        *guard += 1;
    }

    pub async fn reset_retry_counter(&self) {
        let mut guard = self.retry_counter.lock().await;
        *guard = 0;
    }

    /// Retrieve the current retry counter value
    pub async fn retry_counter_value(&self) -> u32 {
        *self.retry_counter.lock().await
    }

    /// Mark the Tor client as connected at the provided timestamp
    pub async fn mark_connected_at(&self, timestamp: DateTime<Utc>) {
        *self.connected_since.lock().await = Some(timestamp);
    }

    /// Mark the Tor client as connected using the current time
    pub async fn mark_connected_now(&self) {
        self.mark_connected_at(Utc::now()).await;
    }

    /// Mark the Tor client as disconnected
    pub async fn mark_disconnected(&self) {
        *self.connected_since.lock().await = None;
    }

    /// Return the timestamp when the client last connected, if available
    pub async fn connected_since(&self) -> Option<DateTime<Utc>> {
        self.connected_since.lock().await.clone()
    }

    /// Return the connection uptime in seconds if currently connected
    pub async fn connection_uptime(&self) -> Option<u64> {
        let guard = self.connected_since.lock().await;
        guard.map(|ts| {
            let diff = Utc::now() - ts;
            diff.num_seconds().max(0) as u64
        })
    }

    /// Retrieve the current tray warning message, if set
    pub async fn tray_warning_message(&self) -> Option<String> {
        self.tray_warning.lock().await.clone()
    }

    /// Create and return a new session token
    pub async fn create_session(&self) -> String {
        self.session.create_session().await
    }

    /// Validate an existing session token
    pub async fn validate_session(&self, token: &str) -> bool {
        self.session.validate(token).await
    }

    pub async fn add_log(
        &self,
        level: Level,
        message: String,
        stack: Option<String>,
    ) -> Result<()> {
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
            stack,
        };
        let json = serde_json::to_string(&entry)?;
        file.write_all(json.as_bytes()).await?;
        file.write_all(b"\n").await?;
        drop(file);

        // Optional hook: forward logs to a central server if configured.
        if let Ok(endpoint) = std::env::var("TORWELL_LOG_ENDPOINT") {
            let client = self.http_client.clone();
            let payload = serde_json::to_value(&entry)?;
            tokio::spawn(async move {
                if let Err(e) = client.post_json(&endpoint, &payload).await {
                    log::error!("failed to send log entry: {}", e);
                }
            });
        }

        self.trim_logs().await?;
        Ok(())
    }

    async fn trim_logs(&self) -> Result<()> {
        let contents = fs::read_to_string(&self.log_file).await.unwrap_or_default();
        let mut lines: Vec<&str> = contents.lines().collect();
        let limit = *self.max_log_lines.lock().await;
        if lines.len() > limit {
            let archive_dir = self
                .log_file
                .parent()
                .map(|p| p.join("archive"))
                .unwrap_or_else(|| PathBuf::from("archive"));
            fs::create_dir_all(&archive_dir).await?;
            let ts = Utc::now().format("%Y%m%d%H%M%S");
            let archive_path = archive_dir.join(format!("torwell-{}.log", ts));
            fs::rename(&self.log_file, &archive_path).await?;
            lines = lines[lines.len() - limit..].to_vec();
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

    async fn trim_metrics(&self) -> Result<()> {
        if let Some(path) = &self.metrics_file {
            let contents = fs::read_to_string(path).await.unwrap_or_default();
            let mut lines: Vec<&str> = contents.lines().collect();
            let line_limit = self.max_metric_lines;
            let size_limit = self.max_metric_mb * 1024 * 1024;
            let mut total_size: usize = lines.iter().map(|l| l.len() + 1).sum();

            let lines_exceeded = lines.len() > line_limit;
            let size_exceeded = total_size > size_limit;

            if lines_exceeded || size_exceeded {
                let archive_dir = path
                    .parent()
                    .map(|p| p.join("archive"))
                    .unwrap_or_else(|| PathBuf::from("archive"));
                fs::create_dir_all(&archive_dir).await?;
                let ts = Utc::now().format("%Y%m%d%H%M%S");
                let archive_path = archive_dir.join(format!("metrics-{}.json", ts));
                fs::rename(path, &archive_path).await?;

                if lines.len() > line_limit {
                    lines = lines[lines.len() - line_limit..].to_vec();
                    total_size = lines.iter().map(|l| l.len() + 1).sum();
                }
                while total_size > size_limit && !lines.is_empty() {
                    total_size -= lines[0].len() + 1;
                    lines.remove(0);
                }
                let mut data = lines.join("\n");
                if !data.is_empty() {
                    data.push('\n');
                }
                fs::write(path, data).await?;

                let msg = if lines_exceeded {
                    format!("metric log exceeded line limit {}", line_limit)
                } else {
                    format!("metric log exceeded size limit {} MB", self.max_metric_mb)
                };
                *self.tray_warning.lock().await = Some(msg.clone());
                self.update_tray_menu().await;
            }
        }
        Ok(())
    }

    /// Append a metric point to the metrics file if configured
    pub async fn append_metric(&self, point: &MetricPoint) -> Result<()> {
        if let Some(path) = &self.metrics_file {
            let _guard = self.metrics_lock.lock().await;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await?;
            let json = serde_json::to_string(point)?;
            file.write_all(json.as_bytes()).await?;
            file.write_all(b"\n").await?;
            drop(file);
            self.trim_metrics().await?;
        }
        Ok(())
    }

    /// Load stored metric points from the metrics file
    pub async fn load_metrics(&self, limit: Option<usize>) -> Result<Vec<MetricPoint>> {
        let limit = limit.unwrap_or(DEFAULT_METRIC_FETCH_LIMIT);
        if let Some(path) = &self.metrics_file {
            let _guard = self.metrics_lock.lock().await;
            let contents = fs::read_to_string(path).await.unwrap_or_default();
            let mut entries = Vec::new();
            for line in contents.lines().rev().take(limit) {
                if let Ok(entry) = serde_json::from_str::<MetricPoint>(line) {
                    entries.push(entry);
                }
            }
            entries.reverse();
            Ok(entries)
        } else {
            Ok(Vec::new())
        }
    }

    /// Update the maximum number of log lines and trim existing logs
    pub async fn set_max_log_lines(&self, limit: usize) -> Result<()> {
        *self.max_log_lines.lock().await = limit;
        self.trim_logs().await
    }

    /// Update the certificate update interval and restart the background task
    pub async fn set_update_interval(&self, interval: u64) {
        if interval > 0 {
            std::env::set_var("TORWELL_UPDATE_INTERVAL", interval.to_string());
        } else {
            std::env::remove_var("TORWELL_UPDATE_INTERVAL");
        }

        let cfg: serde_json::Value = std::fs::read_to_string(secure_http::DEFAULT_CONFIG_PATH)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let mut urls = vec![cfg
            .get("cert_url")
            .and_then(|v| v.as_str())
            .unwrap_or(secure_http::DEFAULT_CERT_URL)
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

        if interval > 0 {
            self.http_client
                .clone()
                .schedule_updates(urls, std::time::Duration::from_secs(interval))
                .await;
        }
    }

    /// Update the GeoIP directory and replace the Tor manager
    pub async fn set_geoip_path(&self, path: Option<String>) {
        if let Some(ref p) = path {
            std::env::set_var("TORWELL_GEOIP_PATH", p);
        } else {
            std::env::remove_var("TORWELL_GEOIP_PATH");
        }
        let new_mgr = Arc::new(TorManager::new_with_geoip(path));
        *self.tor_manager.write().await = new_mgr;
    }

    pub async fn set_insecure_hosts(&self, hosts: Vec<String>) -> Result<()> {
        self.http_client.set_insecure_hosts(hosts.clone());
        Self::persist_insecure_hosts(&hosts)?;
        Ok(())
    }

    fn persist_insecure_hosts(hosts: &[String]) -> Result<()> {
        let path = Path::new(DEFAULT_CONFIG_PATH);
        let contents = std::fs::read_to_string(path).unwrap_or_default();
        let mut config: serde_json::Value = if contents.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&contents).unwrap_or_else(|_| serde_json::json!({}))
        };

        config["insecure_allowed_hosts"] = serde_json::Value::Array(
            hosts
                .iter()
                .map(|h| serde_json::Value::String(h.clone()))
                .collect(),
        );

        let serialized = serde_json::to_string_pretty(&config).map_err(|e| Error::ConfigError {
            step: "state::persist_insecure_hosts".into(),
            source: e.to_string(),
            backtrace: None,
        })?;
        std::fs::write(path, serialized)?;
        Ok(())
    }

    /// Return the path to the log file as a string
    pub fn log_file_path(&self) -> String {
        self.log_file.to_string_lossy().into()
    }

    /// Access the shared CPU task scheduler.
    pub fn scheduler(&self) -> TaskScheduler {
        self.scheduler.clone()
    }

    /// Obtain a snapshot of scheduler latency and backlog statistics.
    pub fn scheduler_snapshot(&self) -> SchedulerSnapshot {
        self.scheduler.snapshot()
    }

    /// Update stored metrics
    pub async fn update_metrics(
        &self,
        memory: u64,
        circuits: usize,
        oldest_age: u64,
        cpu: f32,
        network: u64,
        interval: u64,
    ) {
        *self.memory_usage.lock().await = memory;
        *self.circuit_count.lock().await = circuits;
        *self.oldest_circuit_age.lock().await = oldest_age;
        *self.cpu_usage.lock().await = cpu;
        let mut net = network;
        if let Ok(stats) = {
            let mgr = self.tor_manager.read().await.clone();
            mgr.traffic_stats().await
        } {
            let total = stats.bytes_sent + stats.bytes_received;
            let mut prev = self.prev_traffic.lock().await;
            let diff = if total > *prev { total - *prev } else { 0 };
            *prev = total;
            *self.network_total.lock().await += diff;
            if net == 0 && interval > 0 {
                net = diff / interval;
            }
        }
        *self.network_throughput.lock().await = net;

        let memory_mb = memory / 1024 / 1024;
        if memory_mb > self.max_memory_mb {
            let msg = format!(
                "memory usage {} MB exceeds limit {}",
                memory_mb, self.max_memory_mb
            );
            let _ = self.add_log(Level::Warn, msg.clone(), None).await;
            let mgr = self.tor_manager.read().await.clone();
            let _ = mgr.close_all_circuits().await;
            *self.tray_warning.lock().await = Some(msg.clone());
            self.update_tray_menu().await;
            self.emit_security_warning(msg).await;
        }

        if circuits > self.max_circuits {
            let msg = format!(
                "circuit count {} exceeds limit {}",
                circuits, self.max_circuits
            );
            let _ = self.add_log(Level::Warn, msg.clone(), None).await;
            let mgr = self.tor_manager.read().await.clone();
            let _ = mgr.close_all_circuits().await;
            *self.tray_warning.lock().await = Some(msg.clone());
            self.update_tray_menu().await;
            self.emit_security_warning(msg).await;
        }

        // Warning should persist until the user acknowledges it via the tray
        // Additional metrics like circuit build times could be stored here
    }

    /// Update network latency metric
    pub async fn update_latency(&self, latency: u64) {
        *self.latency_ms.lock().await = latency;
    }

    /// Retrieve current metrics
    pub async fn metrics(&self) -> (u64, usize, u64, f32, u64, u64) {
        let mem = *self.memory_usage.lock().await;
        let circ = *self.circuit_count.lock().await;
        let age = *self.oldest_circuit_age.lock().await;
        let cpu = *self.cpu_usage.lock().await;
        let net = *self.network_throughput.lock().await;
        let total = *self.network_total.lock().await;
        (mem, circ, age, cpu, net, total)
    }

    /// Append a connection state change to the diagnostics buffer
    pub async fn record_connection_event<S: Into<String>>(
        &self,
        status: S,
        message: Option<String>,
        detail: Option<String>,
        retry_count: Option<u32>,
    ) {
        let status = status.into();
        let timestamp = Utc::now();
        let (memory_bytes, circuit_count, _, _, _, _) = self.metrics().await;
        let latency = self.latency().await;

        let event = ConnectionEvent {
            timestamp,
            status,
            message,
            detail,
            retry_count,
            latency_ms: Some(latency),
            memory_bytes: Some(memory_bytes),
            circuit_count: Some(circuit_count),
        };

        let mut events = self.connection_events.lock().await;
        events.push(event);
        if events.len() > self.max_connection_events {
            let excess = events.len() - self.max_connection_events;
            events.drain(0..excess);
        }
    }

    /// Return the most recent connection events as snapshots suitable for the UI
    pub async fn connection_events_snapshot(
        &self,
        limit: Option<usize>,
    ) -> Vec<ConnectionEventSnapshot> {
        let events = self.connection_events.lock().await;
        if events.is_empty() {
            return Vec::new();
        }
        let take = limit.unwrap_or(events.len()).min(events.len());
        let start = events.len().saturating_sub(take);
        events[start..]
            .iter()
            .map(ConnectionEventSnapshot::from)
            .collect()
    }

    /// Compute aggregated diagnostics about recent connection stability
    pub async fn connection_health_summary(&self) -> ConnectionHealthSummary {
        let events = {
            let guard = self.connection_events.lock().await;
            guard.clone()
        };
        let mut summary = ConnectionHealthSummary::default();

        if events.is_empty() {
            summary.current_uptime_seconds = self.connection_uptime().await;
            return summary;
        }

        let now = Utc::now();
        summary.total_events = events.len();

        let mut connected_since: Option<DateTime<Utc>> = None;
        let mut last_timestamp: Option<DateTime<Utc>> = None;
        let mut total_connected = ChronoDuration::seconds(0);
        let mut longest_uptime = ChronoDuration::seconds(0);
        let mut last_connected_at: Option<DateTime<Utc>> = None;
        let mut last_error_at: Option<DateTime<Utc>> = None;

        for event in &events {
            if let Some(prev) = last_timestamp {
                if connected_since.is_some() {
                    total_connected += event.timestamp - prev;
                }
            }

            match event.status.as_str() {
                "CONNECTED" => {
                    summary.connected_events += 1;
                    connected_since = Some(event.timestamp);
                    last_connected_at = Some(event.timestamp);
                }
                "DISCONNECTED" => {
                    summary.disconnect_events += 1;
                    if let Some(start) = connected_since.take() {
                        let span = event.timestamp - start;
                        if span > longest_uptime {
                            longest_uptime = span;
                        }
                    }
                }
                "DISCONNECTING" => {
                    summary.disconnect_events += 1;
                }
                "ERROR" => {
                    summary.error_events += 1;
                    last_error_at = Some(event.timestamp);
                    if let Some(start) = connected_since.take() {
                        let span = event.timestamp - start;
                        if span > longest_uptime {
                            longest_uptime = span;
                        }
                    }
                }
                "RETRYING" => {
                    summary.error_events += 1;
                    last_error_at = Some(event.timestamp);
                    connected_since = None;
                }
                _ => {}
            }

            last_timestamp = Some(event.timestamp);
        }

        if let Some(prev) = last_timestamp {
            if let Some(start) = connected_since {
                let span_to_now = now - start;
                total_connected += now - prev;
                if span_to_now > longest_uptime {
                    longest_uptime = span_to_now;
                }
            }
        }

        let first_timestamp = events.first().map(|event| event.timestamp);
        let observation_window = first_timestamp
            .map(|ts| now - ts)
            .unwrap_or_else(|| ChronoDuration::seconds(0));
        if observation_window.num_seconds() > 0 {
            let ratio = total_connected.num_seconds().max(0) as f64
                / observation_window.num_seconds().max(1) as f64;
            summary.availability_percent = (ratio * 100.0).min(100.0) as f32;
        }

        summary.last_event = events.last().map(ConnectionEventSnapshot::from);
        summary.last_connected_at = last_connected_at.map(|ts| ts.to_rfc3339());
        summary.last_error_at = last_error_at.map(|ts| ts.to_rfc3339());
        summary.current_uptime_seconds = self.connection_uptime().await;
        summary.longest_uptime_seconds = if longest_uptime.num_seconds() > 0 {
            Some(longest_uptime.num_seconds() as u64)
        } else {
            None
        };

        let hour_ago = now - ChronoDuration::hours(1);
        summary.retry_attempts_last_hour = events
            .iter()
            .filter(|event| event.status == "RETRYING" && event.timestamp >= hour_ago)
            .count() as u32;

        summary
    }

    /// Retrieve current latency
    pub async fn latency(&self) -> u64 {
        *self.latency_ms.lock().await
    }

    /// Start periodic collection of performance metrics and emit events
    pub fn start_metrics_task(self: Arc<Self>, handle: AppHandle) {
        tokio::spawn(async move {
            let interval_secs = self.metric_interval_secs;
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            let mut sys = System::new();
            let pid = match sysinfo::get_current_pid() {
                Ok(p) => p,
                Err(_) => return,
            };
            sys.refresh_process(pid);
            sys.refresh_networks();
            let mut prev_net: u64 = sys
                .networks()
                .iter()
                .map(|(_, data)| data.total_received() + data.total_transmitted())
                .sum();
            let mut last_connected = {
                let mgr = self.tor_manager.read().await.clone();
                mgr.is_connected().await
            };

            loop {
                interval.tick().await;

                // Automatically try to reconnect if disconnected
                if {
                    let mgr = self.tor_manager.read().await.clone();
                    !mgr.is_connected().await
                } {
                    self.clone().start_auto_reconnect(handle.clone());
                }

                let circ = match {
                    let mgr = self.tor_manager.read().await.clone();
                    mgr.circuit_metrics().await
                } {
                    Ok(c) => c,
                    Err(_) => crate::tor_manager::CircuitMetrics {
                        count: 0,
                        oldest_age: 0,
                        avg_create_ms: 0,
                        failed_attempts: 0,
                        complete: false,
                    },
                };

                sys.refresh_process(pid);
                sys.refresh_networks();
                let mem = sys.process(pid).map(|p| p.memory()).unwrap_or(0);
                let cpu = sys.process(pid).map(|p| p.cpu_usage()).unwrap_or(0.0);
                let net_total: u64 = sys
                    .networks()
                    .iter()
                    .map(|(_, data)| data.total_received() + data.total_transmitted())
                    .sum();
                let network = if net_total > prev_net {
                    (net_total - prev_net) / interval_secs
                } else {
                    0
                };
                prev_net = net_total;

                let latency = match Self::measure_ping_latency().await {
                    Ok(v) => v,
                    Err(_) => 0,
                };

                let current_connected = {
                    let mgr = self.tor_manager.read().await.clone();
                    mgr.is_connected().await
                };
                if current_connected != last_connected {
                    last_connected = current_connected;
                    if current_connected {
                        self.mark_connected_now().await;
                    } else {
                        self.mark_disconnected().await;
                    }
                    self.update_tray_menu().await;
                }

                self.update_metrics(
                    mem,
                    circ.count,
                    circ.oldest_age,
                    cpu,
                    network,
                    interval_secs,
                )
                .await;
                self.update_latency(latency).await;
                self.update_tray_menu().await;

                let scheduler_snapshot = self.scheduler_snapshot();

                let point = MetricPoint {
                    time: Utc::now().timestamp_millis(),
                    memory_mb: mem / 1_000_000,
                    circuit_count: circ.count,
                    latency_ms: latency,
                    oldest_age: circ.oldest_age,
                    avg_create_ms: circ.avg_create_ms,
                    failed_attempts: circ.failed_attempts,
                    cpu_percent: cpu,
                    network_bytes: *self.network_throughput.lock().await,
                    network_total: *self.network_total.lock().await,
                    complete: circ.complete,
                    scheduler_p50_us: scheduler_snapshot.p50_us,
                    scheduler_p95_us: scheduler_snapshot.p95_us,
                    scheduler_p99_us: scheduler_snapshot.p99_us,
                    scheduler_queue_depth: scheduler_snapshot.queue_depth,
                };
                let _ = self.append_metric(&point).await;

                let failures = *self.http_client.update_failures.lock().await;
                if failures >= 3 {
                    let mut guard = self.tray_warning.lock().await;
                    if guard.is_none() {
                        let msg = format!("{failures} consecutive certificate update failures");
                        *guard = Some(msg.clone());
                        self.update_tray_menu().await;
                    }
                }

                let _ = handle.emit_all(
                    "metrics-update",
                    serde_json::json!({
                        "memory_bytes": mem,
                        "circuit_count": circ.count,
                        "latency_ms": latency,
                        "oldest_age": circ.oldest_age,
                        "avg_create_ms": circ.avg_create_ms,
                        "failed_attempts": circ.failed_attempts,
                        "cpu_percent": cpu,
                        "network_bytes": *self.network_throughput.lock().await,
                        "total_network_bytes": *self.network_total.lock().await,
                        "complete": circ.complete
                    }),
                );
            }
        });
    }

    /// Store the application handle for emitting events
    pub async fn register_handle(&self, handle: AppHandle) {
        *self.app_handle.lock().await = Some(handle);
    }

    /// Build a new system tray menu based on the current state
    async fn build_tray_menu(&self) -> SystemTrayMenu {
        let connected = {
            let mgr = self.tor_manager.read().await.clone();
            mgr.is_connected().await
        };
        let status = if connected {
            "Connected"
        } else {
            "Disconnected"
        };
        let memory_mb = *self.memory_usage.lock().await / 1024 / 1024;
        let circuits = *self.circuit_count.lock().await;
        let mem_label = if memory_mb > self.max_memory_mb {
            format!("Memory: {} MB \u{26A0}\u{fe0f}", memory_mb)
        } else {
            format!("Memory: {} MB", memory_mb)
        };
        let circ_label = if circuits > self.max_circuits {
            format!("Circuits: {} \u{26A0}\u{fe0f}", circuits)
        } else {
            format!("Circuits: {}", circuits)
        };

        let mut menu = SystemTrayMenu::new()
            .add_item(CustomMenuItem::new("status", format!("Status: {}", status)).disabled())
            .add_item(CustomMenuItem::new("memory", mem_label).disabled())
            .add_item(CustomMenuItem::new("circuits", circ_label).disabled())
            .add_item(CustomMenuItem::new("show", "Show"));

        if connected {
            menu = menu.add_item(CustomMenuItem::new("disconnect", "Disconnect"));
        } else {
            menu = menu.add_item(CustomMenuItem::new("connect", "Connect"));
        }

        menu = menu
            .add_item(CustomMenuItem::new("reconnect", "Reconnect"))
            .add_item(CustomMenuItem::new("show_dashboard", "Show Dashboard"))
            .add_item(CustomMenuItem::new("show_logs", "Show Logs"))
            .add_item(CustomMenuItem::new("open_logs_file", "Open Log File"))
            .add_item(CustomMenuItem::new("settings", "Settings"))
            .add_item(CustomMenuItem::new(
                "open_settings_file",
                "Open Settings File",
            ))
            .add_item(CustomMenuItem::new("quit", "Quit"));

        if let Some(w) = self.tray_warning.lock().await.clone() {
            let mut item =
                CustomMenuItem::new("warning", format!("\u{26A0}\u{FE0F} {}", w)).disabled();
            #[cfg(target_os = "macos")]
            {
                item = item.native_image(NativeImage::Caution);
            }
            menu = menu.add_item(item);
        }

        menu
    }

    /// Update the system tray menu with current status and warning if set
    async fn update_tray_menu(&self) {
        if let Some(handle) = self.app_handle.lock().await.as_ref() {
            let menu = self.build_tray_menu().await;
            let tray = handle.tray_handle();
            // Fully recreate the tray menu to avoid stale entries
            let _ = tray.set_menu(SystemTrayMenu::new());
            let _ = tray.set_menu(menu);
        }
    }

    /// Clear any tray warning and refresh the tray menu
    pub async fn clear_tray_warning(&self) {
        *self.tray_warning.lock().await = None;
        self.update_tray_menu().await;
    }

    /// Emit a security warning event to the frontend
    pub async fn emit_security_warning(&self, message: String) {
        if let Some(handle) = self.app_handle.lock().await.as_ref() {
            let _ = handle.emit_all("security-warning", message.clone());

            #[cfg(target_os = "windows")]
            {
                use winrt_notification::{Duration, Toast};
                let _ = Toast::new(Toast::POWERSHELL_APP_ID)
                    .title("Torwell84 Warning")
                    .text1(&message)
                    .duration(Duration::Short)
                    .show();
            }

            #[cfg(target_os = "linux")]
            {
                use std::process::Command;
                let _ = Command::new("notify-send")
                    .arg("Torwell84 Warning")
                    .arg(&message)
                    .output();
            }

            #[cfg(target_os = "macos")]
            {
                let _ = tauri::api::notification::Notification::new(
                    &handle.config().tauri.bundle.identifier,
                )
                .title("Torwell84 Warning")
                .body(&message)
                .show();
            }

            #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
            {
                let _ = tauri::api::notification::Notification::new(
                    &handle.config().tauri.bundle.identifier,
                )
                .title("Torwell84 Warning")
                .body(&message)
                .show();
            }
        }
    }

    /// Attempt to reconnect if the Tor client is not connected
    pub fn start_auto_reconnect(self: Arc<Self>, handle: AppHandle) {
        tokio::spawn(async move {
            {
                let mut flag = self.reconnect_in_progress.lock().await;
                if *flag {
                    return;
                }
                *flag = true;
            }

            if let Err(e) = handle.emit_all(
                "tor-status-update",
                serde_json::json!({
                    "status": "CONNECTING",
                    "bootstrapProgress": 0,
                    "bootstrapMessage": "",
                    "retryCount": 0,
                    "retryDelay": 0
                }),
            ) {
                log::error!("Failed to emit status update: {}", e);
            }

            let mgr = self.tor_manager.read().await.clone();
            let state_clone = self.clone();
            let res = mgr
                .connect_with_backoff(
                    5,
                    Duration::from_secs(60),
                    |info| {
                        let attempt = info.attempt;
                        let delay = info.delay;
                        let err_str = info.error.to_string();
                        let st = state_clone.clone();
                        tokio::spawn(async move {
                            st.increment_retry_counter().await;
                            let _ = st
                                .add_log(
                                    Level::Warn,
                                    format!("connection attempt {} failed: {}", attempt, err_str),
                                    None,
                                )
                                .await;
                        });
                        let (step, source) = match info.error {
                            Error::ConnectionFailed { step, source, .. } => {
                                (step.to_string(), source)
                            }
                            Error::Identity { step, source, .. }
                            | Error::NetworkFailure { step, source, .. }
                            | Error::ConfigError { step, source, .. } => (step, source),
                            _ => (String::new(), String::new()),
                        };
                        let _ = handle.emit_all(
                            "tor-status-update",
                            serde_json::json!({
                                "status": "RETRYING",
                                "retryCount": attempt,
                                "retryDelay": delay.as_secs(),
                                "errorMessage": err_str,
                                "errorStep": step,
                                "errorSource": source
                            }),
                        );
                    },
                    |progress, msg| {
                        let _ = handle.emit_all(
                            "tor-status-update",
                            serde_json::json!({
                                "status": "CONNECTING",
                                "bootstrapProgress": progress,
                                "bootstrapMessage": msg
                            }),
                        );
                    },
                )
                .await;

            match res {
                Ok(_) => {
                    if let Err(e) = handle.emit_all(
                        "tor-status-update",
                        serde_json::json!({
                            "status": "CONNECTED",
                            "bootstrapProgress": 100,
                            "bootstrapMessage": "done",
                            "retryCount": 0,
                            "retryDelay": 0
                        }),
                    ) {
                        log::error!("Failed to emit status update: {}", e);
                    }
                    state_clone.mark_connected_now().await;
                    state_clone.update_tray_menu().await;
                }
                Err(e) => {
                    let (step, source) = match &e {
                        Error::ConnectionFailed { step, source, .. } => {
                            (step.to_string(), source.clone())
                        }
                        Error::Identity { step, source, .. }
                        | Error::NetworkFailure { step, source, .. }
                        | Error::ConfigError { step, source, .. } => (step.clone(), source.clone()),
                        _ => (String::new(), String::new()),
                    };
                    if let Err(em) = handle.emit_all(
                        "tor-status-update",
                        serde_json::json!({
                            "status": "ERROR",
                            "errorMessage": e.to_string(),
                            "errorStep": step,
                            "errorSource": source,
                            "bootstrapMessage": "",
                            "retryCount": 0,
                            "retryDelay": 0
                        }),
                    ) {
                        log::error!("Failed to emit error status update: {}", em);
                    }
                    state_clone.mark_disconnected().await;
                }
            }

            state_clone.reset_retry_counter().await;
            *state_clone.reconnect_in_progress.lock().await = false;
        });
    }

    /// Measure latency to a well-known host using an ICMP ping
    async fn measure_ping_latency() -> Result<u64> {
        match icmp::ping_host("google.com", 1).await {
            Ok(v) => Ok(v),
            Err(_) => Ok(0),
        }
    }
}
