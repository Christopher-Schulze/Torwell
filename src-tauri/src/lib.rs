mod commands;
mod error;
mod secure_http;
mod state;
mod tor_manager;

use state::AppState;

pub fn run() {
    let app_state = AppState::default();

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
            commands::get_isolated_circuit,
            commands::set_exit_country,
            commands::get_traffic_stats,
            commands::get_logs,
            commands::clear_logs,
            commands::get_log_file_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
