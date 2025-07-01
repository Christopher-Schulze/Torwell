use crate::error::{Error, Result};
use crate::state::{AppState, LogEntry};
use log::Level;
use serde::Serialize;
use std::time::{Duration, Instant};
use sysinfo::{PidExt, System, SystemExt};
use tauri::{Manager, State};
use governor::{Quota, RateLimiter};
use governor::state::{InMemoryState, NotKeyed};
use governor::clock::DefaultClock;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Mutex;

const STATS_WINDOW: Duration = Duration::from_secs(60);

static INVOCATIONS: Lazy<Mutex<HashMap<&'static str, (Instant, u32)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static CONNECT_LIMITER: Lazy<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> = Lazy::new(|| {
    RateLimiter::direct(Quota::per_minute(NonZeroU32::new(3).unwrap()))
});

static LOG_LIMITER: Lazy<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> = Lazy::new(|| {
    RateLimiter::direct(Quota::per_minute(NonZeroU32::new(10).unwrap()))
});

fn record_invocation(name: &'static str) {
    let mut map = INVOCATIONS.lock().unwrap();
    let entry = map.entry(name).or_insert((Instant::now(), 0));
    if entry.0.elapsed() > STATS_WINDOW {
        entry.0 = Instant::now();
        entry.1 = 0;
    }
    entry.1 += 1;
}

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
}

#[tauri::command]
pub async fn connect(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    record_invocation("connect");
    if CONNECT_LIMITER.check().is_err() {
        let err = Error::RateLimited("connect".into());
        let _ = app_handle.emit_all(
            "tor-status-update",
            serde_json::json!({ "status": "RATE_LIMIT", "errorMessage": "connect command rate limited" }),
        );
        return Err(err);
    }
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
                Duration::from_secs(60),
                |attempt, delay, err| {
                    let err_str = err.to_string();
                    let sc = state_clone.clone();
                    tokio::spawn(async move {
                        sc.increment_retry_counter().await;
                        let _ = sc
                            .add_log(
                                Level::Warn,
                                format!("connection attempt {} failed: {}", attempt, err_str),
                            )
                            .await;
                    });
                    let _ = app_handle.emit_all(
                        "tor-status-update",
                        serde_json::json!({
                            "status": "RETRYING",
                            "retryCount": attempt,
                            "retryDelay": delay.as_secs(),
                            "errorMessage": err_str
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
            }
            Err(e) => {
                if let Err(e_emit) = app_handle.emit_all(
                    "tor-status-update",
                    serde_json::json!({
                        "status": "ERROR",
                        "errorMessage": e.to_string(),
                        "bootstrapMessage": "",
                        "retryCount": 0, "retryDelay": 0
                    }),
                ) {
                    log::error!("Failed to emit error status update: {}", e_emit);
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn disconnect(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    record_invocation("disconnect");
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

    Ok(())
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<String> {
    record_invocation("get_status");
    if state.tor_manager.is_connected().await {
        Ok("CONNECTED".to_string())
    } else {
        Ok("DISCONNECTED".to_string())
    }
}

#[tauri::command]
pub async fn get_active_circuit(state: State<'_, AppState>) -> Result<Vec<RelayInfo>> {
    record_invocation("get_active_circuit");
    state.tor_manager.get_active_circuit().await
}

#[tauri::command]
pub async fn get_isolated_circuit(
    state: State<'_, AppState>,
    domain: String,
) -> Result<Vec<RelayInfo>> {
    record_invocation("get_isolated_circuit");
    state.tor_manager.get_isolated_circuit(domain).await
}

#[tauri::command]
pub async fn set_exit_country(state: State<'_, AppState>, country: Option<String>) -> Result<()> {
    record_invocation("set_exit_country");
    state.tor_manager.set_exit_country(country).await
}

#[tauri::command]
pub async fn set_bridges(state: State<'_, AppState>, bridges: Vec<String>) -> Result<()> {
    record_invocation("set_bridges");
    state.tor_manager.set_bridges(bridges).await
}

#[tauri::command]
pub async fn get_traffic_stats(state: State<'_, AppState>) -> Result<TrafficStats> {
    record_invocation("get_traffic_stats");
    let stats = state.tor_manager.traffic_stats().await?;
    Ok(TrafficStats {
        bytes_sent: stats.bytes_sent,
        bytes_received: stats.bytes_received,
    })
}

#[tauri::command]
pub async fn get_metrics(state: State<'_, AppState>) -> Result<Metrics> {
    record_invocation("get_metrics");
    let circ = state.tor_manager.circuit_metrics().await?;
    let mut sys = sysinfo::System::new();
    let pid = sysinfo::get_current_pid().map_err(|e| Error::Io(e.to_string()))?;
    sys.refresh_process(pid);
    let mem = sys.process(pid).map(|p| p.memory()).unwrap_or(0);
    state.update_metrics(mem, circ.count).await;

    if mem / 1024 / 1024 > state.max_memory_mb {
        let _ = state
            .add_log(
                Level::Warn,
                format!(
                    "memory usage {} MB exceeds limit {}",
                    mem / 1024 / 1024,
                    state.max_memory_mb
                ),
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
            )
            .await;
    }

    Ok(Metrics {
        memory_bytes: mem,
        circuit_count: circ.count,
        oldest_circuit_age: circ.oldest_age,
    })
}

#[tauri::command]
pub async fn new_identity(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    record_invocation("new_identity");
    state.tor_manager.new_identity().await?;
    // Emit event to update frontend
    app_handle.emit_all(
        "tor-status-update",
        serde_json::json!({ "status": "NEW_IDENTITY" }),
    )?;
    Ok(())
}
#[tauri::command]
pub async fn get_logs(state: State<'_, AppState>) -> Result<Vec<LogEntry>> {
    record_invocation("get_logs");
    LOG_LIMITER
        .check()
        .map_err(|_| Error::RateLimited("get_logs".into()))?;
    state.read_logs().await
}

#[tauri::command]
pub async fn clear_logs(state: State<'_, AppState>) -> Result<()> {
    record_invocation("clear_logs");
    state.clear_log_file().await
}

#[tauri::command]
pub async fn get_log_file_path(state: State<'_, AppState>) -> Result<String> {
    record_invocation("get_log_file_path");
    Ok(state.log_file_path())
}
