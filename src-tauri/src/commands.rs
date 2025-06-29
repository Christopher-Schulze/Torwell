use crate::error::Result;
use crate::state::AppState;
use serde::Serialize;
use tauri::{Manager, State};

#[derive(Serialize, Clone)]
pub struct RelayInfo {
    pub nickname: String,
    pub ip_address: String,
    pub country: String,
}

#[derive(Serialize, Clone)]
pub struct TrafficMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

#[tauri::command]
pub async fn connect(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    let tor_manager = state.tor_manager.clone();

    // Fire and forget
    tokio::spawn(async move {
        // Inform the frontend that we are connecting
        if let Err(e) = app_handle.emit_all("tor-status-update", serde_json::json!({ "status": "CONNECTING", "bootstrapProgress": 0 })) {
            log::error!("Failed to emit status update: {}", e);
        }

        // Perform the actual connection
        match tor_manager.connect().await {
            Ok(_) => {
                if let Err(e) = app_handle.emit_all("tor-status-update", serde_json::json!({ "status": "CONNECTED", "bootstrapProgress": 100 })) {
                    log::error!("Failed to emit status update: {}", e);
                }
            }
            Err(e) => {
                if let Err(e_emit) = app_handle.emit_all("tor-status-update", serde_json::json!({ "status": "ERROR", "errorMessage": e.to_string() })) {
                    log::error!("Failed to emit error status update: {}", e_emit);
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn disconnect(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    if let Err(e) = app_handle.emit_all("tor-status-update", serde_json::json!({ "status": "DISCONNECTING" })) {
        log::error!("Failed to emit status update: {}", e);
    }
    
    state.tor_manager.disconnect().await?;

    if let Err(e) = app_handle.emit_all("tor-status-update", serde_json::json!({ "status": "DISCONNECTED", "bootstrapProgress": 0 })) {
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
pub async fn new_identity(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    state.tor_manager.new_identity().await?;
    // Emit event to update frontend
    app_handle.emit_all("tor-status-update", serde_json::json!({ "status": "NEW_IDENTITY" }))?;
    Ok(())
}

#[tauri::command]
pub async fn get_traffic_metrics(state: State<'_, AppState>) -> Result<TrafficMetrics> {
    let (sent, received) = state.tor_manager.traffic_metrics();
    Ok(TrafficMetrics { bytes_sent: sent, bytes_received: received })
}
#[tauri::command]
pub async fn get_logs(state: State<'_, AppState>) -> Result<Vec<String>> {
    state.read_logs().await
}

#[tauri::command]
pub async fn clear_logs(state: State<'_, AppState>) -> Result<()> {
    state.clear_log_file().await
}

