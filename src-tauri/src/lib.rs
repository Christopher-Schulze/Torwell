mod commands;
mod error;
mod secure_http;
mod session;
mod state;
mod tor_manager;

use secure_http::SecureHttpClient;
use state::AppState;
use std::time::Duration;
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu};

pub fn run() {
    let http_client = tauri::async_runtime::block_on(async {
        SecureHttpClient::init(
            secure_http::DEFAULT_CONFIG_PATH,
            None,
            None,
            None,
            Some(Duration::from_secs(60 * 60 * 24)),
        )
        .expect("failed to initialize http client")
    });
    let app_state = AppState::new(http_client.clone());

    let quit = CustomMenuItem::new("quit", "Quit");
    let show = CustomMenuItem::new("show", "Show");
    let connect = CustomMenuItem::new("connect", "Connect");
    let disconnect = CustomMenuItem::new("disconnect", "Disconnect");
    let logs = CustomMenuItem::new("show_logs", "Show Logs");
    let settings = CustomMenuItem::new("settings", "Settings");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show.clone())
        .add_item(connect.clone())
        .add_item(disconnect.clone())
        .add_item(logs.clone())
        .add_item(settings.clone())
        .add_item(quit.clone());
    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => std::process::exit(0),
                "show" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "connect" => {
                    let state = app.state::<AppState>();
                    let handle = app.handle();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = commands::connect(handle.clone(), state.clone()).await {
                            log::error!("tray connect failed: {}", e);
                        }
                        state.update_tray_menu().await;
                    });
                }
                "disconnect" => {
                    let state = app.state::<AppState>();
                    let handle = app.handle();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = commands::disconnect(handle.clone(), state.clone()).await {
                            log::error!("tray disconnect failed: {}", e);
                        }
                        state.update_tray_menu().await;
                    });
                }
                "show_logs" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.emit("open-logs", ());
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "settings" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.emit("open-settings", ());
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "warning" => {
                    let state = app.state::<AppState>();
                    tauri::async_runtime::spawn(async move {
                        state.clear_tray_warning().await;
                    });
                }
                _ => {}
            },
            _ => {}
        })
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
            let handle = app.handle();
            let http_client = state.http_client.clone();
            let state_clone = state.clone();
            tauri::async_runtime::block_on(async move {
                state_clone.register_handle(handle.clone()).await;
                state_clone.update_tray_menu().await;
                http_client
                    .set_warning_callback(move |msg| {
                        let st = state.clone();
                        tauri::async_runtime::spawn(async move {
                            st.emit_security_warning(msg).await;
                        });
                    })
                    .await;
            });
            state.start_metrics_task(handle);
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
            commands::set_worker_config,
            commands::list_bridge_presets,
            commands::get_traffic_stats,
            commands::get_metrics,
            commands::get_logs,
            commands::clear_logs,
            commands::get_log_file_path,
            commands::set_log_limit,
            commands::ping_host,
            commands::get_secure_key,
            commands::set_secure_key,
            commands::request_token
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
