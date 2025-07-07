use crate::error::Result;
use crate::icmp;
use crate::secure_http::SecureHttpClient;
use crate::session::SessionManager;
use crate::tor_manager::{TorClientBehavior, TorManager};
use arti_client::TorClient;
use chrono::Utc;
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
use tokio::sync::Mutex;
use tor_rtcompat::PreferredRuntime;

/// Default location of the application configuration file
pub const DEFAULT_CONFIG_PATH: &str = "src-tauri/app_config.json";

/// Default number of log lines retained if no configuration is provided
pub const DEFAULT_MAX_LOG_LINES: usize = 1000;
/// Default session token lifetime in seconds
pub const DEFAULT_SESSION_TTL: u64 = 3600;

#[derive(Deserialize, Default)]
struct AppConfig {
    #[serde(default = "default_max_log_lines")]
    max_log_lines: usize,
    #[serde(default)]
    geoip_path: Option<String>,
}

fn default_max_log_lines() -> usize {
    DEFAULT_MAX_LOG_LINES
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
    /// Age of the oldest circuit in seconds
    pub oldest_circuit_age: Arc<Mutex<u64>>,
    /// Last recorded network latency in milliseconds
    pub latency_ms: Arc<Mutex<u64>>,
    /// Current CPU usage percentage
    pub cpu_usage: Arc<Mutex<f32>>,
    /// Network throughput in bytes per second
    pub network_throughput: Arc<Mutex<u64>>,
    /// Total traffic bytes at the last metrics update
    pub prev_traffic: Arc<Mutex<u64>>,
    /// Maximum memory usage before warning (in MB)
    pub max_memory_mb: u64,
    /// Maximum number of circuits before warning
    pub max_circuits: usize,
    /// Session manager for authentication tokens
    pub session: Arc<SessionManager>,
    /// Handle used to emit frontend events
    pub app_handle: Arc<Mutex<Option<AppHandle>>>,
    /// Current warning shown in the tray menu
    pub tray_warning: Arc<Mutex<Option<String>>>,
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
        let mut geoip_path = cfg.geoip_path.clone();
        if let Ok(val) = std::env::var("TORWELL_MAX_LOG_LINES") {
            if let Ok(n) = val.parse::<usize>() {
                max_log_lines = n;
            }
        }
        if let Ok(p) = std::env::var("TORWELL_GEOIP_PATH") {
            geoip_path = Some(p);
        }

        Self {
            tor_manager: Arc::new(TorManager::new_with_geoip(geoip_path.clone())),
            http_client: Arc::new(
                SecureHttpClient::new_default().expect("failed to create http client"),
            ),
            log_file,
            log_lock: Arc::new(Mutex::new(())),
            retry_counter: Arc::new(Mutex::new(0)),
            max_log_lines: Arc::new(Mutex::new(max_log_lines)),
            memory_usage: Arc::new(Mutex::new(0)),
            circuit_count: Arc::new(Mutex::new(0)),
            oldest_circuit_age: Arc::new(Mutex::new(0)),
            latency_ms: Arc::new(Mutex::new(0)),
            cpu_usage: Arc::new(Mutex::new(0.0)),
            network_throughput: Arc::new(Mutex::new(0)),
            prev_traffic: Arc::new(Mutex::new(0)),
            max_memory_mb: std::env::var("TORWELL_MAX_MEMORY_MB")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1024),
            max_circuits: std::env::var("TORWELL_MAX_CIRCUITS")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(20),
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
        let mut geoip_path = cfg.geoip_path.clone();
        if let Ok(val) = std::env::var("TORWELL_MAX_LOG_LINES") {
            if let Ok(n) = val.parse::<usize>() {
                max_log_lines = n;
            }
        }
        if let Ok(p) = std::env::var("TORWELL_GEOIP_PATH") {
            geoip_path = Some(p);
        }

        AppState {
            tor_manager: Arc::new(TorManager::new_with_geoip(geoip_path.clone())),
            http_client,
            log_file,
            log_lock: Arc::new(Mutex::new(())),
            retry_counter: Arc::new(Mutex::new(0)),
            max_log_lines: Arc::new(Mutex::new(max_log_lines)),
            memory_usage: Arc::new(Mutex::new(0)),
            circuit_count: Arc::new(Mutex::new(0)),
            oldest_circuit_age: Arc::new(Mutex::new(0)),
            latency_ms: Arc::new(Mutex::new(0)),
            cpu_usage: Arc::new(Mutex::new(0.0)),
            network_throughput: Arc::new(Mutex::new(0)),
            prev_traffic: Arc::new(Mutex::new(0)),
            max_memory_mb: std::env::var("TORWELL_MAX_MEMORY_MB")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1024),
            max_circuits: std::env::var("TORWELL_MAX_CIRCUITS")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(20),
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
                .schedule_updates(urls, std::time::Duration::from_secs(interval));
        }
    }

    /// Return the path to the log file as a string
    pub fn log_file_path(&self) -> String {
        self.log_file.to_string_lossy().into()
    }

    /// Update stored metrics
    pub async fn update_metrics(
        &self,
        memory: u64,
        circuits: usize,
        oldest_age: u64,
        cpu: f32,
        network: u64,
    ) {
        *self.memory_usage.lock().await = memory;
        *self.circuit_count.lock().await = circuits;
        *self.oldest_circuit_age.lock().await = oldest_age;
        *self.cpu_usage.lock().await = cpu;
        let mut net = network;
        if let Ok(stats) = self.tor_manager.traffic_stats().await {
            let total = stats.bytes_sent + stats.bytes_received;
            let mut prev = self.prev_traffic.lock().await;
            let diff = if total > *prev { total - *prev } else { 0 };
            *prev = total;
            if net == 0 {
                net = diff / 30;
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
            let _ = self.tor_manager.close_all_circuits().await;
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
            let _ = self.tor_manager.close_all_circuits().await;
            *self.tray_warning.lock().await = Some(msg.clone());
            self.update_tray_menu().await;
            self.emit_security_warning(msg).await;
        }

        if memory_mb <= self.max_memory_mb && circuits <= self.max_circuits {
            let mut warn = self.tray_warning.lock().await;
            if warn.is_some() {
                *warn = None;
                drop(warn);
                self.update_tray_menu().await;
            }
        }
        // Additional metrics like circuit build times could be stored here
    }

    /// Update network latency metric
    pub async fn update_latency(&self, latency: u64) {
        *self.latency_ms.lock().await = latency;
    }

    /// Retrieve current metrics
    pub async fn metrics(&self) -> (u64, usize, u64, f32, u64) {
        let mem = *self.memory_usage.lock().await;
        let circ = *self.circuit_count.lock().await;
        let age = *self.oldest_circuit_age.lock().await;
        let cpu = *self.cpu_usage.lock().await;
        let net = *self.network_throughput.lock().await;
        (mem, circ, age, cpu, net)
    }

    /// Retrieve current latency
    pub async fn latency(&self) -> u64 {
        *self.latency_ms.lock().await
    }

    /// Start periodic collection of performance metrics and emit events
    pub fn start_metrics_task(self: Arc<Self>, handle: AppHandle) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
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

            loop {
                interval.tick().await;

                let circ = match self.tor_manager.circuit_metrics().await {
                    Ok(c) => c,
                    Err(_) => crate::tor_manager::CircuitMetrics {
                        count: 0,
                        oldest_age: 0,
                        avg_create_ms: 0,
                        failed_attempts: 0,
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
                    (net_total - prev_net) / 30
                } else {
                    0
                };
                prev_net = net_total;

                let latency = match Self::measure_ping_latency().await {
                    Ok(v) => v,
                    Err(_) => 0,
                };

                self.update_metrics(mem, circ.count, circ.oldest_age, cpu, network)
                    .await;
                self.update_latency(latency).await;

                let failures = *self.http_client.update_failures.lock().await;
                if failures >= 3 {
                    let msg = format!("{failures} consecutive certificate update failures");
                    *self.tray_warning.lock().await = Some(msg.clone());
                    self.update_tray_menu().await;
                } else if failures == 0 {
                    let mut warn = self.tray_warning.lock().await;
                    if warn
                        .as_ref()
                        .map(|w| w.contains("certificate update"))
                        .unwrap_or(false)
                    {
                        *warn = None;
                        drop(warn);
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
                        "network_bytes": *self.network_throughput.lock().await
                    }),
                );
            }
        });
    }

    /// Store the application handle for emitting events
    pub async fn register_handle(&self, handle: AppHandle) {
        *self.app_handle.lock().await = Some(handle);
    }

    /// Update the system tray menu with current status and warning if set
    async fn update_tray_menu(&self) {
        if let Some(handle) = self.app_handle.lock().await.as_ref() {
            let connected = self.tor_manager.is_connected().await;
            let status = if connected {
                "Connected"
            } else {
                "Disconnected"
            };
            let mut menu = SystemTrayMenu::new()
                .add_item(CustomMenuItem::new("status", format!("Status: {}", status)).disabled())
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
                .add_item(CustomMenuItem::new("settings", "Settings"))
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

            let tray = handle.tray_handle();
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

    /// Measure latency to a well-known host using an ICMP ping
    async fn measure_ping_latency() -> Result<u64> {
        match icmp::ping_host("google.com", 1).await {
            Ok(v) => Ok(v),
            Err(_) => Ok(0),
        }
    }
}
