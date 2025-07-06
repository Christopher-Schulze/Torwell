use crate::error::{Error, Result};
use crate::icmp;
use crate::state::{AppState, LogEntry};
use crate::tor_manager::BridgePreset;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use keyring;
use log::Level;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::time::Duration;
use std::time::Instant;
use sysinfo::{PidExt, System, SystemExt};
use tauri::{Manager, State};
use tokio::sync::Mutex;

/// Total bytes sent and received through Tor.
#[derive(Serialize, Clone)]
pub struct TrafficStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Information about a single relay in the active circuit.
///
/// `country` is an ISO 3166-1 alpha-2 code derived from the relay's IP address.
#[derive(Serialize, Clone)]
pub struct RelayInfo {
    pub nickname: String,
    pub ip_address: String,
    pub country: String,
}

/// Memory and circuit metrics.
#[derive(Serialize, Clone)]
pub struct Metrics {
    pub memory_bytes: u64,
    pub circuit_count: usize,
    pub oldest_circuit_age: u64,
    pub cpu_percent: f32,
    pub network_bytes: u64,
}

const INVOCATION_WINDOW: Duration = Duration::from_secs(60);
static INVOCATIONS: Lazy<Mutex<HashMap<&'static str, Vec<Instant>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const LOG_LIMIT: u32 = 20;
static LOG_LIMITER: Lazy<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    Lazy::new(|| RateLimiter::direct(Quota::per_minute(NonZeroU32::new(LOG_LIMIT).unwrap())));

const API_LIMIT: u32 = 60;
static API_LIMITER: Lazy<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> =
    Lazy::new(|| RateLimiter::direct(Quota::per_minute(NonZeroU32::new(API_LIMIT).unwrap())));

const MAX_PING_COUNT: u8 = 10;
static HOST_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9.-]+$").unwrap());

fn check_api_rate() -> Result<()> {
    API_LIMITER
        .check()
        .map_err(|_| Error::RateLimitExceeded("api".into()))
}

async fn track_call(name: &'static str) -> usize {
    let mut map = INVOCATIONS.lock().await;
    let now = Instant::now();
    let entry = map.entry(name).or_insert_with(Vec::new);
    entry.retain(|t| now.duration_since(*t) <= INVOCATION_WINDOW);
    entry.push(now);
    entry.len()
}

#[tauri::command]
pub async fn request_token(state: State<'_, AppState>) -> Result<String> {
    check_api_rate()?;
    if let Some(tok) = state.session.take_startup_token().await {
        Ok(tok)
    } else {
        Ok(state.create_session().await)
    }
}

#[tauri::command]
pub async fn connect(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    track_call("connect").await;
    check_api_rate()?;
    let tor_manager = state.tor_manager.clone();
    let state_clone = state.inner().clone();

    // Fire and forget
    tokio::spawn(async move {
        // Inform the frontend that we are connecting
        if let Err(e) = app_handle.emit_all(
            "tor-status-update",
            serde_json::json!({ "status": "CONNECTING", "bootstrapProgress": 0, "bootstrapMessage": "", "retryCount": 0, "retryDelay": 0 }),
        ) {
            log::error!("Failed to emit status update: {}", e);
        }

        // Perform the actual connection
        let _ = state_clone.reset_retry_counter().await;
        match tor_manager
            .connect_with_backoff(
                5,
                Duration::from_secs(60), // place to capture circuit build duration metrics
                |attempt, delay, err| {
                    let err_str = err.to_string();
                    let sc = state_clone.clone();
                    tokio::spawn(async move {
                        sc.increment_retry_counter().await;
                        let _ = sc
                            .add_log(
                                Level::Warn,
                                format!("connection attempt {} failed: {}", attempt, err_str),
                                None,
                            )
                            .await;
                    });
                    let (step, source) = match err {
                        Error::ConnectionFailed { step, source }
                        | Error::Identity { step, source } => (step, source),
                        _ => ("", ""),
                    };
                    let _ = app_handle.emit_all(
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
                    let _ = app_handle.emit_all(
                        "tor-status-update",
                        serde_json::json!({
                            "status": "CONNECTING",
                            "bootstrapProgress": progress,
                            "bootstrapMessage": msg
                        }),
                    );
                },
            )
            .await
        {
            Ok(_) => {
                if let Err(e) = app_handle.emit_all(
                    "tor-status-update",
                    serde_json::json!({
                        "status": "CONNECTED",
                        "bootstrapProgress": 100,
                        "bootstrapMessage": "done",
                        "retryCount": 0, "retryDelay": 0
                    }),
                ) {
                    log::error!("Failed to emit status update: {}", e);
                }
                state_clone.update_tray_menu().await;
            }
            Err(e) => {
                let (step, source) = match &e {
                    Error::ConnectionFailed { step, source } | Error::Identity { step, source } => {
                        (step.as_str(), source.as_str())
                    }
                    _ => ("", ""),
                };
                if let Err(e_emit) = app_handle.emit_all(
                    "tor-status-update",
                    serde_json::json!({
                        "status": "ERROR",
                        "errorMessage": e.to_string(),
                        "errorStep": step,
                        "errorSource": source,
                        "bootstrapMessage": "",
                        "retryCount": 0, "retryDelay": 0
                    }),
                ) {
                    log::error!("Failed to emit error status update: {}", e_emit);
                }
                state_clone.update_tray_menu().await;
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn disconnect(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    track_call("disconnect").await;
    check_api_rate()?;
    if let Err(e) = app_handle.emit_all(
        "tor-status-update",
        serde_json::json!({ "status": "DISCONNECTING", "bootstrapMessage": "", "retryCount": 0, "retryDelay": 0 }),
    ) {
        log::error!("Failed to emit status update: {}", e);
    }

    state.tor_manager.disconnect().await?;

    if let Err(e) = app_handle.emit_all(
        "tor-status-update",
        serde_json::json!({ "status": "DISCONNECTED", "bootstrapProgress": 0, "bootstrapMessage": "", "retryCount": 0, "retryDelay": 0 }),
    ) {
        log::error!("Failed to emit status update: {}", e);
    }

    state.update_tray_menu().await;

    Ok(())
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<String> {
    track_call("get_status").await;
    check_api_rate()?;
    if state.tor_manager.is_connected().await {
        Ok("CONNECTED".to_string())
    } else {
        Ok("DISCONNECTED".to_string())
    }
}

#[tauri::command]
pub async fn get_active_circuit(state: State<'_, AppState>) -> Result<Vec<RelayInfo>> {
    track_call("get_active_circuit").await;
    check_api_rate()?;
    state.tor_manager.get_active_circuit().await
}

#[tauri::command]
pub async fn get_isolated_circuit(
    state: State<'_, AppState>,
    domain: String,
) -> Result<Vec<RelayInfo>> {
    track_call("get_isolated_circuit").await;
    check_api_rate()?;
    state.tor_manager.get_isolated_circuit(domain).await
}

#[tauri::command]
pub async fn set_exit_country(state: State<'_, AppState>, country: Option<String>) -> Result<()> {
    track_call("set_exit_country").await;
    check_api_rate()?;
    state.tor_manager.set_exit_country(country).await
}

#[tauri::command]
pub async fn set_bridges(state: State<'_, AppState>, bridges: Vec<String>) -> Result<()> {
    track_call("set_bridges").await;
    check_api_rate()?;
    state.tor_manager.set_bridges(bridges).await
}

#[tauri::command]
pub async fn set_worker_config(
    state: State<'_, AppState>,
    workers: Vec<String>,
    token: Option<String>,
) -> Result<()> {
    check_api_rate()?;
    state.http_client.set_worker_config(workers, token).await;
    Ok(())
}

#[tauri::command]
pub async fn set_hsm_config(
    state: State<'_, AppState>,
    lib: Option<String>,
    slot: Option<u64>,
) -> Result<()> {
    check_api_rate()?;
    state
        .http_client
        .set_hsm_config(lib, slot)
        .await
        .map_err(|e| Error::Io(e.to_string()))?
        ;
    Ok(())
}

#[tauri::command]
pub async fn list_bridge_presets() -> Result<Vec<BridgePreset>> {
    crate::tor_manager::load_default_bridge_presets()
}

#[tauri::command]
pub async fn get_traffic_stats(state: State<'_, AppState>) -> Result<TrafficStats> {
    track_call("get_traffic_stats").await;
    check_api_rate()?;
    let stats = state.tor_manager.traffic_stats().await?;
    Ok(TrafficStats {
        bytes_sent: stats.bytes_sent,
        bytes_received: stats.bytes_received,
    })
}

#[tauri::command]
pub async fn get_metrics(state: State<'_, AppState>) -> Result<Metrics> {
    track_call("get_metrics").await;
    check_api_rate()?;
    let circ = state.tor_manager.circuit_metrics().await?; // capture more metrics like build time when available
    let mut sys = sysinfo::System::new();
    let pid = sysinfo::get_current_pid().map_err(|e| Error::Io(e.to_string()))?;
    sys.refresh_process(pid);
    sys.refresh_networks();
    let mem = sys.process(pid).map(|p| p.memory()).unwrap_or(0);
    let cpu = sys.process(pid).map(|p| p.cpu_usage()).unwrap_or(0.0);
    state
        .update_metrics(mem, circ.count, circ.oldest_age, cpu, 0)
        .await;

    if mem / 1024 / 1024 > state.max_memory_mb {
        let _ = state
            .add_log(
                Level::Warn,
                format!(
                    "memory usage {} MB exceeds limit {}",
                    mem / 1024 / 1024,
                    state.max_memory_mb
                ),
                None,
            )
            .await;
    }

    if circ.count > state.max_circuits {
        let _ = state
            .add_log(
                Level::Warn,
                format!(
                    "circuit count {} exceeds limit {}",
                    circ.count, state.max_circuits
                ),
                None,
            )
            .await;
    }

    Ok(Metrics {
        memory_bytes: mem,
        circuit_count: circ.count,
        oldest_circuit_age: circ.oldest_age,
        cpu_percent: cpu,
        network_bytes: 0,
    })
}

#[tauri::command]
pub async fn new_identity(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    track_call("new_identity").await;
    check_api_rate()?;
    state.tor_manager.new_identity().await?; // potential metric: measure time to build new circuit
                                             // Emit event to update frontend
    app_handle.emit_all(
        "tor-status-update",
        serde_json::json!({ "status": "NEW_IDENTITY" }),
    )?;
    Ok(())
}

#[tauri::command]
pub async fn list_circuits(state: State<'_, AppState>) -> Result<Vec<u64>> {
    track_call("list_circuits").await;
    check_api_rate()?;
    state.tor_manager.list_circuit_ids().await
}

#[tauri::command]
pub async fn close_circuit(state: State<'_, AppState>, id: u64) -> Result<()> {
    track_call("close_circuit").await;
    check_api_rate()?;
    state.tor_manager.close_circuit(id).await
}
#[tauri::command]
pub async fn get_logs(state: State<'_, AppState>, token: String) -> Result<Vec<LogEntry>> {
    track_call("get_logs").await;
    check_api_rate()?;
    if !state.validate_session(&token).await {
        log::error!("get_logs: invalid token");
        return Err(Error::InvalidToken);
    }
    if LOG_LIMITER.check().is_err() {
        log::error!("get_logs: rate limit exceeded");
        return Err(Error::RateLimitExceeded("get_logs".into()));
    }
    state.read_logs().await
}

#[tauri::command]
pub async fn clear_logs(state: State<'_, AppState>, token: String) -> Result<()> {
    track_call("clear_logs").await;
    check_api_rate()?;
    if !state.validate_session(&token).await {
        log::error!("clear_logs: invalid token");
        return Err(Error::InvalidToken);
    }
    state.clear_log_file().await
}

#[tauri::command]
pub async fn get_log_file_path(state: State<'_, AppState>, token: String) -> Result<String> {
    track_call("get_log_file_path").await;
    check_api_rate()?;
    if !state.validate_session(&token).await {
        log::error!("get_log_file_path: invalid token");
        return Err(Error::InvalidToken);
    }
    Ok(state.log_file_path())
}

#[tauri::command]
pub async fn set_log_limit(state: State<'_, AppState>, limit: usize) -> Result<()> {
    check_api_rate()?;
    state.set_max_log_lines(limit).await
}

#[tauri::command]
pub async fn ping_host(
    state: State<'_, AppState>,
    token: String,
    host: Option<String>,
    count: Option<u8>,
) -> Result<u64> {
    track_call("ping_host").await;
    check_api_rate()?;
    if !state.validate_session(&token).await {
        log::error!("ping_host: invalid token");
        return Err(Error::InvalidToken);
    }
    let host = host.unwrap_or_else(|| "google.com".to_string());
    if !HOST_RE.is_match(&host) {
        log::error!("ping_host: invalid host '{}'", host);
        return Err(Error::Io("invalid host".into()));
    }
    let count = count.unwrap_or(5).min(MAX_PING_COUNT);
    icmp::ping_host(&host, count)
        .await
        .map_err(|e| Error::Io(e.to_string()))
}

#[tauri::command]
pub async fn dns_lookup(
    state: State<'_, AppState>,
    token: String,
    host: String,
) -> Result<Vec<String>> {
    track_call("dns_lookup").await;
    check_api_rate()?;
    if !state.validate_session(&token).await {
        log::error!("dns_lookup: invalid token");
        return Err(Error::InvalidToken);
    }
    if !HOST_RE.is_match(&host) {
        log::error!("dns_lookup: invalid host '{}'", host);
        return Err(Error::Io("invalid host".into()));
    }
    let entries = tokio::net::lookup_host((&host[..], 0))
        .await
        .map_err(|e| Error::Io(e.to_string()))?;
    Ok(entries.map(|a| a.ip().to_string()).collect())
}

#[tauri::command]
pub async fn traceroute_host(
    state: State<'_, AppState>,
    token: String,
    host: String,
    max_hops: Option<u8>,
) -> Result<Vec<String>> {
    use traceroute::TraceResult;
    track_call("traceroute_host").await;
    check_api_rate()?;
    if !state.validate_session(&token).await {
        log::error!("traceroute_host: invalid token");
        return Err(Error::InvalidToken);
    }
    if !HOST_RE.is_match(&host) {
        log::error!("traceroute_host: invalid host '{}'", host);
        return Err(Error::Io("invalid host".into()));
    }
    let host_clone = host.clone();
    let limit = max_hops.unwrap_or(30) as usize;
    let hops = tokio::task::spawn_blocking(move || {
        let addr = format!("{}:0", host_clone);
        let trace: TraceResult = traceroute::execute(addr.as_str()).map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for hop in trace.take(limit) {
            let hop = hop.map_err(|e| e.to_string())?;
            out.push(hop.host.ip().to_string());
        }
        Ok::<_, String>(out)
    })
    .await
    .map_err(|e| Error::Io(e.to_string()))??;
    Ok(hops)
}

#[tauri::command]
pub async fn get_secure_key(state: State<'_, AppState>, token: String) -> Result<Option<String>> {
    track_call("get_secure_key").await;
    check_api_rate()?;
    if !state.validate_session(&token).await {
        log::error!("get_secure_key: invalid token");
        return Err(Error::InvalidToken);
    }
    let entry =
        keyring::Entry::new("torwell84", "aes-key").map_err(|e| Error::Io(e.to_string()))?;
    match entry.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(Error::Io(e.to_string())),
    }
}

#[tauri::command]
pub async fn set_secure_key(
    state: State<'_, AppState>,
    token: String,
    value: String,
) -> Result<()> {
    track_call("set_secure_key").await;
    check_api_rate()?;
    if !state.validate_session(&token).await {
        log::error!("set_secure_key: invalid token");
        return Err(Error::InvalidToken);
    }
    let entry =
        keyring::Entry::new("torwell84", "aes-key").map_err(|e| Error::Io(e.to_string()))?;
    entry
        .set_password(&value)
        .map_err(|e| Error::Io(e.to_string()))
}

#[tauri::command]
pub async fn reconnect(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    track_call("reconnect").await;
    check_api_rate()?;

    // Attempt graceful disconnect; ignore errors if already disconnected
    let _ = state.tor_manager.disconnect().await;

    // Reuse existing connect logic
    connect(app_handle, state).await
}

#[tauri::command]
pub async fn show_dashboard(app_handle: tauri::AppHandle) -> Result<()> {
    if let Some(window) = app_handle.get_window("main") {
        let _ = window.emit("open-dashboard", ());
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}
