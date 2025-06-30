use crate::error::Result;
use crate::state::{AppState, LogEntry};
use serde::Serialize;
use tauri::{Manager, State};

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

#[tauri::command]
pub async fn connect(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    let tor_manager = state.tor_manager.clone();

    // Fire and forget
    tokio::spawn(async move {
        // Inform the frontend that we are connecting
        if let Err(e) = app_handle.emit_all(
            "tor-status-update",
            serde_json::json!({ "status": "CONNECTING", "bootstrapProgress": 0, "retryCount": 0, "retryDelay": 0 }),
        ) {
            log::error!("Failed to emit status update: {}", e);
        }

        // Perform the actual connection
        match tor_manager
            .connect_with_backoff(
                5,
                |attempt, delay, err| {
                    let _ = app_handle.emit_all(
                        "tor-status-update",
                        serde_json::json!({
                            "status": "RETRYING",
                            "retryCount": attempt,
                            "retryDelay": delay.as_secs(),
                            "errorMessage": err.to_string()
                        }),
                    );
                },
                |progress| {
                    let _ = app_handle.emit_all(
                        "tor-status-update",
                        serde_json::json!({
                            "status": "CONNECTING",
                            "bootstrapProgress": progress
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
    if let Err(e) = app_handle.emit_all(
        "tor-status-update",
        serde_json::json!({ "status": "DISCONNECTING", "retryCount": 0, "retryDelay": 0 }),
    ) {
        log::error!("Failed to emit status update: {}", e);
    }

    state.tor_manager.disconnect().await?;

    if let Err(e) = app_handle.emit_all(
        "tor-status-update",
        serde_json::json!({ "status": "DISCONNECTED", "bootstrapProgress": 0, "retryCount": 0, "retryDelay": 0 }),
    ) {
        log::error!("Failed to emit status update: {}", e);
    }

    Ok(())
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<String> {
    if state.tor_manager.is_connected().await {
        Ok("CONNECTED".to_string())
    } else {
        Ok("DISCONNECTED".to_string())
    }
}

#[tauri::command]
pub async fn get_active_circuit(state: State<'_, AppState>) -> Result<Vec<RelayInfo>> {
    state.tor_manager.get_active_circuit().await
}

#[tauri::command]
pub async fn get_isolated_circuit(
    state: State<'_, AppState>,
    domain: String,
) -> Result<Vec<RelayInfo>> {
    state.tor_manager.get_isolated_circuit(domain).await
}

#[tauri::command]
pub async fn set_exit_country(state: State<'_, AppState>, country: Option<String>) -> Result<()> {
    state.tor_manager.set_exit_country(country).await
}

#[tauri::command]
pub async fn set_bridges(state: State<'_, AppState>, bridges: Vec<String>) -> Result<()> {
    state.tor_manager.set_bridges(bridges).await
}

#[tauri::command]
pub async fn get_traffic_stats(state: State<'_, AppState>) -> Result<TrafficStats> {
    let stats = state.tor_manager.traffic_stats().await?;
    Ok(TrafficStats {
        bytes_sent: stats.bytes_sent,
        bytes_received: stats.bytes_received,
    })
}

#[tauri::command]
pub async fn new_identity(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
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
    state.read_logs().await
}

#[tauri::command]
pub async fn clear_logs(state: State<'_, AppState>) -> Result<()> {
    state.clear_log_file().await
}

#[tauri::command]
pub async fn get_log_file_path(state: State<'_, AppState>) -> Result<String> {
    Ok(state.log_file_path())
}
