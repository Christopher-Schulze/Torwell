mod commands;
mod error;
mod state;
mod tor_manager;

use state::AppState;

pub fn run() {
    let app_state = AppState::default();
    let geoip_path = std::env::var("GEOIP_DB_PATH").unwrap_or_else(|_| "geoip.mmdb".into());
    let geoip_manager = app_state.tor_manager.clone();
    tauri::async_runtime::spawn(async move {
        let _ = geoip_manager.load_geoip_db(&geoip_path).await;
    });

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::disconnect,
            commands::get_status,
            commands::get_active_circuit,
            commands::new_identity,
            commands::update_geoip_database,
            commands::get_logs,
            commands::clear_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
