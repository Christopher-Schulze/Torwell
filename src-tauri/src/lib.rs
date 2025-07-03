mod commands;
mod error;
mod secure_http;
mod session;
mod state;
mod tor_manager;

use secure_http::SecureHttpClient;
use state::AppState;
use std::time::Duration;

pub fn run() {
    let http_client = tauri::async_runtime::block_on(async {
        SecureHttpClient::init(
            secure_http::DEFAULT_CONFIG_PATH,
            None,
            None,
            Some(Duration::from_secs(60 * 60 * 24)),
        )
        .expect("failed to initialize http client")
    });
    let app_state = AppState::new(http_client);

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
            let state = app.state::<AppState>();
            state.clone().start_metrics_task(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::disconnect,
            commands::get_status,
            commands::get_active_circuit,
            commands::get_isolated_circuit,
            commands::set_exit_country,
            commands::set_bridges,
            commands::get_traffic_stats,
            commands::get_metrics,
            commands::get_logs,
            commands::clear_logs,
            commands::get_log_file_path,
            commands::set_log_limit,
            commands::ping_host
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
