use crate::core::executor::TaskError;
use crate::error::{Error, Result};
use crate::icmp;
use crate::renderer::FrameMetricsSnapshot;
use crate::state::{
    AppState, ConnectionEventSnapshot, ConnectionHealthSummary, LogEntry, MetricPoint,
};
use crate::system_proxy;
use crate::tor_manager::{BridgePreset, CircuitPolicyReport, RetryInfo, TorrcProfile};
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
use sysinfo::System;
use tauri::{Manager, State};
use tokio::sync::Mutex;

/// Total bytes sent and received through Tor.
#[derive(Serialize, Clone)]
pub struct TrafficStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Information about a single relay in the active circuit.
#[derive(Serialize, Clone, Debug)]
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
    pub total_network_bytes: u64,
    pub complete: bool,
}

/// Aggregated status information used by the frontend dashboard.
#[derive(Serialize, Clone)]
pub struct StatusSummary {
    pub status: String,
    pub connected_since: Option<String>,
    pub uptime_seconds: Option<u64>,
    pub total_traffic_bytes: u64,
    pub network_bytes_per_sec: u64,
    pub total_network_bytes: u64,
    pub latency_ms: u64,
    pub memory_bytes: u64,
    pub circuit_count: usize,
    pub oldest_circuit_age: u64,
    pub cpu_percent: f32,
    pub tray_warning: Option<String>,
    pub retry_count: u32,
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

// Logic extracted for internal use (e.g. tray menu)
pub async fn perform_connect(app_handle: tauri::AppHandle, state: AppState) -> Result<()> {
    let tor_manager = state.tor_manager.clone();
    let state_clone = state.clone();

    // Fire and forget
    tokio::spawn(async move {
        if let Err(e) = app_handle.emit_all(
            "tor-status-update",
            serde_json::json!({
                "status": "CONNECTING",
                "bootstrapProgress": 0,
                "bootstrapMessage": "",
                "retryCount": 0,
                "retryDelay": 0,
                "errorStep": null,
                "errorSource": null,
                "errorMessage": null
            }),
        ) {
            log::error!("Failed to emit status update: {}", e);
        }

        state_clone
            .record_connection_event(
                "CONNECTING",
                Some("Attempting Tor connection".into()),
                None,
                None,
            )
            .await;

        let _ = state_clone.reset_retry_counter().await;
        let mgr = tor_manager.read().await.clone();
        match mgr
            .connect_with_backoff(
                5,
                Duration::from_secs(60),
                |info: RetryInfo| {
                    let attempt = info.attempt;
                    let delay = info.delay;
                    let err = &info.error;
                    let err_str = err.to_string();
                    let sc = state_clone.clone();
                    let err_clone = err_str.clone();
                    tokio::spawn(async move {
                        sc.increment_retry_counter().await;
                        let _ = sc
                            .add_log(
                                Level::Warn,
                                format!("connection attempt {} failed: {}", attempt, err_clone),
                                None,
                            )
                            .await;
                    });
                    let (step, source_message) = match err {
                        Error::ConnectionFailed { step, source_message, .. } => {
                            (step.to_string(), source_message.to_string())
                        }
                        Error::Identity { step, source_message, .. } => (step.clone(), source_message.to_string()),
                        Error::NetworkFailure { step, source_message, .. } => (step.clone(), source_message.to_string()),
                        Error::ConfigError { step, source_message, .. } => (step.clone(), source_message.to_string()),
                        _ => (String::new(), String::new()),
                    };
                    let _ = app_handle.emit_all(
                        "tor-status-update",
                        serde_json::json!({
                            "status": "RETRYING",
                            "retryCount": attempt,
                            "retryDelay": delay.as_secs(),
                            "errorMessage": err_str,
                            "errorStep": step,
                            "errorSource": source_message
                        }),
                    );
                    let mut detail_text = if !step.is_empty() && !source_message.is_empty() {
                        format!("{step} ({source_message}) – {err_str}")
                    } else if !step.is_empty() {
                        format!("{step} – {err_str}")
                    } else if !source_message.is_empty() {
                        format!("{source_message} – {err_str}")
                    } else {
                        err_str.clone()
                    };
                    let delay_secs = delay.as_secs();
                    if delay_secs > 0 {
                        detail_text.push_str(&format!(" – retrying in {delay_secs}s"));
                    }
                    let message = format!("Attempt {attempt} failed");
                    let sc_event = state_clone.clone();
                    tokio::spawn(async move {
                        sc_event
                            .record_connection_event(
                                "RETRYING",
                                Some(message),
                                Some(detail_text),
                                Some(attempt),
                            )
                            .await;
                    });
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
                        "retryCount": 0,
                        "retryDelay": 0,
                        "errorStep": null,
                        "errorSource": null,
                        "errorMessage": null
                    }),
                ) {
                    log::error!("Failed to emit status update: {}", e);
                }
                state_clone.mark_connected_now().await;
                state_clone
                    .record_connection_event(
                        "CONNECTED",
                        Some("Tor connection established".into()),
                        None,
                        None,
                    )
                    .await;

                if let Some(port) = mgr.get_socks_port().await {
                    let proxy_url = format!("socks5h://127.0.0.1:{}", port);
                    if let Err(e) = state_clone.http_client.set_proxy(Some(proxy_url)).await {
                        log::error!("Failed to set proxy: {}", e);
                    } else {
                        log::info!("Configured http client with proxy at 127.0.0.1:{}", port);
                    }
                }

                state_clone.update_tray_menu().await;

                // Enable System Proxy if enabled
                if state_clone.is_system_proxy_enabled().await {
                    let res = tokio::task::spawn_blocking(|| {
                        system_proxy::enable_global_proxy(9150)
                    }).await;

                    match res {
                        Ok(Ok(_)) => {
                            let _ = state_clone
                                .add_log(
                                    Level::Info,
                                    "System proxy enabled on port 9150".to_string(),
                                    None,
                                )
                                .await;
                            let _ = app_handle.emit_all("system-proxy-update", serde_json::json!({ "enabled": true }));
                        }
                        Ok(Err(e)) => {
                            let _ = state_clone
                                .add_log(
                                    Level::Warn,
                                    format!("Failed to set system proxy: {}", e),
                                    None,
                                )
                                .await;
                        }
                        Err(join_err) => {
                            log::error!("Failed to spawn proxy task: {}", join_err);
                        }
                    }
                }
            }
            Err(e) => {
                let (step, source_message) = match &e {
                    Error::ConnectionFailed { step, source_message, .. } => {
                        (step.to_string(), source_message.to_string())
                    }
                    Error::Identity { step, source_message, .. } => (step.clone(), source_message.to_string()),
                    Error::NetworkFailure { step, source_message, .. } => (step.clone(), source_message.to_string()),
                    Error::ConfigError { step, source_message, .. } => (step.clone(), source_message.to_string()),
                    _ => (String::new(), String::new()),
                };
                if let Err(e_emit) = app_handle.emit_all(
                    "tor-status-update",
                    serde_json::json!({
                        "status": "ERROR",
                        "errorMessage": e.to_string(),
                        "errorStep": step,
                        "errorSource": source_message,
                        "bootstrapMessage": "",
                        "retryCount": 0, "retryDelay": 0
                    }),
                ) {
                    log::error!("Failed to emit error status update: {}", e_emit);
                }
                state_clone.mark_disconnected().await;
                state_clone.update_tray_menu().await;
                let detail_text = if step.is_empty() && source_message.is_empty() {
                    e.to_string()
                } else if source_message.is_empty() {
                    format!("{} – {}", step, e)
                } else {
                    format!("{} ({}) – {}", step, source_message, e)
                };
                state_clone
                    .record_connection_event(
                        "ERROR",
                        Some("Tor connection failed".into()),
                        Some(detail_text),
                        None,
                    )
                    .await;
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn connect(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    track_call("connect").await;
    check_api_rate()?;
    perform_connect(app_handle, (*state).clone()).await
}

pub async fn perform_disconnect(app_handle: tauri::AppHandle, state: AppState) -> Result<()> {
    if let Err(e) = app_handle.emit_all(
        "tor-status-update",
        serde_json::json!({
            "status": "DISCONNECTING",
            "bootstrapMessage": "",
            "retryCount": 0,
            "retryDelay": 0,
            "errorStep": null,
            "errorSource": null,
            "errorMessage": null
        }),
    ) {
        log::error!("Failed to emit status update: {}", e);
    }

    state
        .record_connection_event(
            "DISCONNECTING",
            Some("Disconnect requested".into()),
            None,
            None,
        )
        .await;

    // Disable System Proxy
    let _ = tokio::task::spawn_blocking(|| {
        if let Err(e) = system_proxy::disable_global_proxy() {
            log::error!("Failed to disable system proxy: {}", e);
        }
    }).await;
    let _ = app_handle.emit_all("system-proxy-update", serde_json::json!({ "enabled": false }));

    {
        let mgr = state.tor_manager.read().await.clone();
        mgr.disconnect().await?;
    }

    if let Err(e) = state.http_client.set_proxy(None).await {
        log::error!("Failed to clear proxy: {}", e);
    }

    state.mark_disconnected().await;

    if let Err(e) = app_handle.emit_all(
        "tor-status-update",
        serde_json::json!({
            "status": "DISCONNECTED",
            "bootstrapProgress": 0,
            "bootstrapMessage": "",
            "retryCount": 0,
            "retryDelay": 0,
            "errorStep": null,
            "errorSource": null,
            "errorMessage": null
        }),
    ) {
        log::error!("Failed to emit status update: {}", e);
    }

    state
        .record_connection_event(
            "DISCONNECTED",
            Some("Tor connection closed".into()),
            None,
            None,
        )
        .await;

    state.update_tray_menu().await;

    Ok(())
}

#[tauri::command]
pub async fn disconnect(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    track_call("disconnect").await;
    check_api_rate()?;
    perform_disconnect(app_handle, (*state).clone()).await
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<String> {
    track_call("get_status").await;
    check_api_rate()?;
    if {
        let mgr = state.tor_manager.read().await.clone();
        mgr.is_connected().await
    } {
        Ok("CONNECTED".to_string())
    } else {
        Ok("DISCONNECTED".to_string())
    }
}

#[tauri::command]
pub async fn get_status_summary(state: State<'_, AppState>) -> Result<StatusSummary> {
    track_call("get_status_summary").await;
    check_api_rate()?;

    let mgr = state.tor_manager.read().await.clone();
    let is_connected = mgr.is_connected().await;
    let total_traffic_bytes = if is_connected {
        mgr.traffic_stats()
            .await
            .map(|stats| stats.bytes_sent + stats.bytes_received)
            .unwrap_or(0)
    } else {
        0
    };

    let (
        memory_bytes,
        circuit_count,
        oldest_circuit_age,
        cpu_percent,
        network_bytes_per_sec,
        total_network_bytes,
    ) = state.metrics().await;
    let latency_ms = state.latency().await;
    let connected_since = state.connected_since().await.map(|ts| ts.to_rfc3339());
    let uptime_seconds = state.connection_uptime().await;
    let tray_warning = state.tray_warning_message().await;
    let retry_count = state.retry_counter_value().await;

    Ok(StatusSummary {
        status: if is_connected {
            "CONNECTED".into()
        } else {
            "DISCONNECTED".into()
        },
        connected_since,
        uptime_seconds,
        total_traffic_bytes,
        network_bytes_per_sec,
        total_network_bytes,
        latency_ms,
        memory_bytes,
        circuit_count,
        oldest_circuit_age,
        cpu_percent,
        tray_warning,
        retry_count,
    })
}

#[tauri::command]
pub async fn get_connection_timeline(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<ConnectionEventSnapshot>> {
    track_call("get_connection_timeline").await;
    check_api_rate()?;
    Ok(state.connection_events_snapshot(limit).await)
}

#[tauri::command]
pub async fn get_connection_health_summary(
    state: State<'_, AppState>,
) -> Result<ConnectionHealthSummary> {
    track_call("get_connection_health_summary").await;
    check_api_rate()?;
    Ok(state.connection_health_summary().await)
}

#[tauri::command]
pub async fn get_active_circuit(state: State<'_, AppState>) -> Result<Vec<RelayInfo>> {
    track_call("get_active_circuit").await;
    check_api_rate()?;
    {
        let mgr = state.tor_manager.read().await.clone();
        mgr.get_active_circuit().await
    }
}

#[tauri::command]
pub async fn get_circuit_policy_report(state: State<'_, AppState>) -> Result<CircuitPolicyReport> {
    track_call("get_circuit_policy_report").await;
    check_api_rate()?;
    {
        let mgr = state.tor_manager.read().await.clone();
        mgr.circuit_policy_report().await
    }
}

#[tauri::command]
pub async fn get_isolated_circuit(
    state: State<'_, AppState>,
    domain: String,
) -> Result<Vec<RelayInfo>> {
    track_call("get_isolated_circuit").await;
    check_api_rate()?;
    {
        let mgr = state.tor_manager.read().await.clone();
        mgr.get_isolated_circuit(domain).await
    }
}

#[tauri::command]
pub async fn set_exit_country(state: State<'_, AppState>, country: Option<String>) -> Result<()> {
    track_call("set_exit_country").await;
    check_api_rate()?;
    {
        let mgr = state.tor_manager.read().await.clone();
        mgr.set_exit_country(country).await
    }
}

#[tauri::command]
pub async fn set_entry_country(state: State<'_, AppState>, country: Option<String>) -> Result<()> {
    track_call("set_entry_country").await;
    check_api_rate()?;
    {
        let mgr = state.tor_manager.read().await.clone();
        mgr.set_entry_country(country).await
    }
}

#[tauri::command]
pub async fn set_middle_country(state: State<'_, AppState>, country: Option<String>) -> Result<()> {
    track_call("set_middle_country").await;
    check_api_rate()?;
    {
        let mgr = state.tor_manager.read().await.clone();
        mgr.set_middle_country(country).await
    }
}

#[tauri::command]
pub async fn set_bridges(state: State<'_, AppState>, bridges: Vec<String>) -> Result<()> {
    track_call("set_bridges").await;
    check_api_rate()?;
    {
        let mgr = state.tor_manager.read().await.clone();
        mgr.set_bridges(bridges).await
    }
}

#[tauri::command]
pub async fn set_torrc_config(state: State<'_, AppState>, config: String) -> Result<()> {
    check_api_rate()?;
    {
        let mgr = state.tor_manager.read().await.clone();
        mgr.set_torrc_config(config).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_system_proxy(app_handle: tauri::AppHandle, state: State<'_, AppState>, enabled: bool) -> Result<()> {
    check_api_rate()?;
    state.set_system_proxy_enabled(enabled).await;
    let state_clone = state.inner().clone();

    // If connected, apply immediately
    let mgr = state.tor_manager.read().await.clone();
    if mgr.is_connected().await {
        let _ = tokio::task::spawn_blocking(move || {
            if enabled {
                if let Err(e) = system_proxy::enable_global_proxy(9150) {
                     log::error!("Failed to enable system proxy: {}", e);
                } else {
                    let _ = app_handle.emit_all("system-proxy-update", serde_json::json!({ "enabled": true }));
                    let _ = state_clone.add_log(Level::Info, "System proxy enabled via toggle".into(), None);
                }
            } else {
                if let Err(e) = system_proxy::disable_global_proxy() {
                     log::error!("Failed to disable system proxy: {}", e);
                } else {
                    let _ = app_handle.emit_all("system-proxy-update", serde_json::json!({ "enabled": false }));
                    let _ = state_clone.add_log(Level::Info, "System proxy disabled via toggle".into(), None);
                }
            }
        }); // Note: spawn_blocking is async, we don't await here to return quickly, but we should probably await if we want to report error?
            // For toggle, "fire and forget" but log error is acceptable UX usually, but let's stick to pattern.
    } else {
         // If not connected, just emit the preference update so UI stays in sync
         let _ = app_handle.emit_all("system-proxy-update", serde_json::json!({ "enabled": enabled }));
    }
    Ok(())
}

#[tauri::command]
pub async fn get_system_proxy_status(state: State<'_, AppState>) -> Result<bool> {
    check_api_rate()?;
    Ok(state.is_system_proxy_enabled().await)
}

#[tauri::command]
pub async fn generate_torrc_profile(
    state: State<'_, AppState>,
    fast_only: bool,
    preferred_fast_countries: Option<Vec<String>>,
    include_bridges: bool,
) -> Result<TorrcProfile> {
    track_call("generate_torrc_profile").await;
    check_api_rate()?;
    let mgr = state.tor_manager.read().await.clone();
    mgr.generate_torrc_profile(fast_only, preferred_fast_countries, include_bridges)
        .await
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
pub async fn validate_worker_token(state: State<'_, AppState>) -> Result<bool> {
    check_api_rate()?;
    match state.http_client.get_text("https://example.com").await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
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
        .map_err(|e| Error::Io(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn set_update_interval(state: State<'_, AppState>, interval: u64) -> Result<()> {
    check_api_rate()?;
    state.set_update_interval(interval).await;
    Ok(())
}

#[tauri::command]
pub async fn set_geoip_path(state: State<'_, AppState>, path: Option<String>) -> Result<()> {
    check_api_rate()?;
    state.set_geoip_path(path).await;
    Ok(())
}

#[tauri::command]
pub async fn set_insecure_hosts(state: State<'_, AppState>, hosts: Vec<String>) -> Result<()> {
    check_api_rate()?;
    state.set_insecure_hosts(hosts).await
}

#[tauri::command]
pub async fn list_bridge_presets() -> Result<Vec<BridgePreset>> {
    crate::tor_manager::load_default_bridge_presets()
}

#[tauri::command]
pub async fn get_traffic_stats(state: State<'_, AppState>) -> Result<TrafficStats> {
    track_call("get_traffic_stats").await;
    check_api_rate()?;
    let stats = {
        let mgr = state.tor_manager.read().await.clone();
        mgr.traffic_stats().await?
    };
    Ok(TrafficStats {
        bytes_sent: stats.bytes_sent,
        bytes_received: stats.bytes_received,
    })
}

#[tauri::command]
pub async fn get_metrics(state: State<'_, AppState>) -> Result<Metrics> {
    track_call("get_metrics").await;
    check_api_rate()?;
    let circ = {
        let mgr = state.tor_manager.read().await.clone();
        mgr.circuit_metrics().await?
    }; // capture more metrics like build time when available
    let mut sys = sysinfo::System::new();
    let pid = sysinfo::get_current_pid().map_err(|e| Error::Io(e.to_string()))?;
    sys.refresh_process(pid);
    let mut networks = sysinfo::Networks::new_with_refreshed_list();
    let mem = sys.process(pid).map(|p| p.memory()).unwrap_or(0);
    let cpu = sys.process(pid).map(|p| p.cpu_usage()).unwrap_or(0.0);
    state
        .update_metrics(mem, circ.count, circ.oldest_age, cpu, 0, 30)
        .await;
    let net = *state.network_throughput.lock().await;

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
        network_bytes: net,
        total_network_bytes: *state.network_total.lock().await,
        complete: circ.complete,
    })
}

#[tauri::command]
pub async fn new_identity(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    track_call("new_identity").await;
    check_api_rate()?;
    let result = {
        let mgr = state.tor_manager.read().await.clone();
        mgr.new_identity().await
    };

    match result {
        Ok(_) => {
            app_handle.emit_all(
                "tor-status-update",
                serde_json::json!({
                    "status": "NEW_IDENTITY",
                    "errorStep": null,
                    "errorSource": null,
                    "errorMessage": null
                }),
            )?;
            state
                .record_connection_event(
                    "NEW_IDENTITY",
                    Some("New Tor identity requested".into()),
                    None,
                    None,
                )
                .await;
            Ok(())
        }
        Err(e) => {
            let (step, source_message) = match &e {
                Error::Identity { step, source_message, .. } => (step.clone(), source_message.to_string()),
                Error::ConnectionFailed { step, source_message, .. } => (step.to_string(), source_message.to_string()),
                Error::NetworkFailure { step, source_message, .. } => (step.clone(), source_message.to_string()),
                Error::ConfigError { step, source_message, .. } => (step.clone(), source_message.to_string()),
                _ => (String::new(), String::new()),
            };
            if let Err(em) = app_handle.emit_all(
                "tor-status-update",
                serde_json::json!({
                    "status": "ERROR",
                    "errorMessage": e.to_string(),
                    "errorStep": step,
                    "errorSource": source_message
                }),
            ) {
                log::error!("Failed to emit error status update: {}", em);
            }
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn build_circuit(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    track_call("build_circuit").await;
    check_api_rate()?;
    let result = {
        let mgr = state.tor_manager.read().await.clone();
        mgr.build_circuit().await
    };

    match result {
        Ok(_) => {
            app_handle.emit_all(
                "tor-status-update",
                serde_json::json!({
                    "status": "NEW_CIRCUIT",
                    "errorStep": null,
                    "errorSource": null,
                    "errorMessage": null
                }),
            )?;
            state
                .record_connection_event(
                    "NEW_CIRCUIT",
                    Some("Manual circuit build triggered".into()),
                    None,
                    None,
                )
                .await;
            Ok(())
        }
        Err(e) => {
            let (step, source_message) = match &e {
                Error::Identity { step, source_message, .. } => (step.clone(), source_message.to_string()),
                Error::ConnectionFailed { step, source_message, .. } => (step.to_string(), source_message.to_string()),
                Error::NetworkFailure { step, source_message, .. } => (step.clone(), source_message.to_string()),
                Error::ConfigError { step, source_message, .. } => (step.clone(), source_message.to_string()),
                _ => (String::new(), String::new()),
            };
            if let Err(em) = app_handle.emit_all(
                "tor-status-update",
                serde_json::json!({
                    "status": "ERROR",
                    "errorMessage": e.to_string(),
                    "errorStep": step,
                    "errorSource": source_message
                }),
            ) {
                log::error!("Failed to emit error status update: {}", em);
            }
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn list_circuits(state: State<'_, AppState>) -> Result<Vec<u64>> {
    track_call("list_circuits").await;
    check_api_rate()?;
    {
        let mgr = state.tor_manager.read().await.clone();
        mgr.list_circuit_ids().await
    }
}

#[tauri::command]
pub async fn close_circuit(state: State<'_, AppState>, id: u64) -> Result<()> {
    track_call("close_circuit").await;
    check_api_rate()?;
    {
        let mgr = state.tor_manager.read().await.clone();
        mgr.close_circuit(id).await
    }
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
pub async fn load_metrics(
    state: State<'_, AppState>,
    token: String,
    limit: Option<usize>,
) -> Result<Vec<MetricPoint>> {
    track_call("load_metrics").await;
    check_api_rate()?;
    if !state.validate_session(&token).await {
        log::error!("load_metrics: invalid token");
        return Err(Error::InvalidToken);
    }
    state.load_metrics(limit).await
}

#[tauri::command]
pub async fn get_frame_metrics(state: State<'_, AppState>) -> Result<FrameMetricsSnapshot> {
    check_api_rate()?;
    Ok(state.renderer_service().metrics_snapshot())
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
pub async fn ping_host_series(
    state: State<'_, AppState>,
    token: String,
    host: Option<String>,
    count: Option<u8>,
) -> Result<Vec<u64>> {
    track_call("ping_host_series").await;
    check_api_rate()?;
    if !state.validate_session(&token).await {
        log::error!("ping_host_series: invalid token");
        return Err(Error::InvalidToken);
    }
    let host = host.unwrap_or_else(|| "google.com".to_string());
    if !HOST_RE.is_match(&host) {
        log::error!("ping_host_series: invalid host '{}'", host);
        return Err(Error::Io("invalid host".into()));
    }
    let count = count.unwrap_or(5).min(MAX_PING_COUNT);
    icmp::ping_host_series(&host, count)
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
    let scheduler = state.scheduler();
    let hops = scheduler
        .spawn(
            "traceroute_host",
            move || -> std::result::Result<Vec<String>, String> {
                let addr = format!("{}:0", host_clone);
                let trace: TraceResult =
                    traceroute::execute(addr.as_str()).map_err(|e| e.to_string())?;
                let mut out = Vec::new();
                for hop in trace.take(limit) {
                    let hop = hop.map_err(|e| e.to_string())?;
                    out.push(hop.host.ip().to_string());
                }
                Ok(out)
            },
        )
        .await
        .map_err(|err| match err {
            TaskError::Canceled => Error::Io("traceroute task cancelled".into()),
            TaskError::Panicked { message, .. } => Error::Io(message),
        })?
        .map_err(Error::Io)?;
    Ok(hops)
}

#[tauri::command]
pub async fn lookup_country(state: State<'_, AppState>, ip: String) -> Result<String> {
    track_call("lookup_country").await;
    check_api_rate()?;
    {
        let mgr = state.tor_manager.read().await.clone();
        mgr.lookup_country_code(&ip).await
    }
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

pub async fn perform_reconnect(app_handle: tauri::AppHandle, state: AppState) -> Result<()> {
    // Attempt graceful disconnect; ignore errors if already disconnected
    {
        let mgr = state.tor_manager.read().await.clone();
        let _ = mgr.disconnect().await;
    }

    // Reuse existing connect logic
    perform_connect(app_handle, state).await
}

#[tauri::command]
pub async fn reconnect(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    track_call("reconnect").await;
    check_api_rate()?;

    perform_reconnect(app_handle, (*state).clone()).await
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
