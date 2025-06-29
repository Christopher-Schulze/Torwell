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
pub async fn get_isolated_circuit(state: State<'_, AppState>, domain: String) -> Result<Vec<RelayInfo>> {
    state.tor_manager.get_circuit(Some(domain)).await
}

#[tauri::command]
pub async fn set_exit_policy(state: State<'_, AppState>, ports: Vec<u16>) -> Result<()> {
    state.tor_manager.set_exit_policy(ports).await;
    Ok(())
}

#[tauri::command]
pub async fn new_identity(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    state.tor_manager.new_identity().await?;
    // Emit event to update frontend
    app_handle.emit_all("tor-status-update", serde_json::json!({ "status": "NEW_IDENTITY" }))?;
    Ok(())
}
#[tauri::command]
pub async fn get_logs() -> Result<Vec<String>> {
    // For now, this returns an empty list.
    // TODO: Implement actual log retrieval.
    Ok(vec![])
}

#[tauri::command]
pub async fn clear_logs() -> Result<()> {
    // For now, this does nothing.
    // TODO: Implement actual log clearing.
    Ok(())
}