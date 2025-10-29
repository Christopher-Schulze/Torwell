mod commands;
mod error;
#[cfg(feature = "mobile")]
mod http_bridge;
#[path = "lib/mod.rs"]
pub mod lib;
mod secure_http;
mod session;
mod state;
mod tor_manager;

use open;
use secure_http::SecureHttpClient;
use state::AppState;
use std::time::Duration;
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu};

pub fn run() {
    let http_client = tauri::async_runtime::block_on(async {
        SecureHttpClient::init(secure_http::DEFAULT_CONFIG_PATH, None, None, None, None)
            .expect("failed to initialize http client")
    });
    let app_state = AppState::new(http_client.clone());

    #[cfg(feature = "mobile")]
    http_bridge::start(app_state.clone());

    let quit = CustomMenuItem::new("quit", "Quit");
    let show = CustomMenuItem::new("show", "Show");
    let connect = CustomMenuItem::new("connect", "Connect");
    let disconnect = CustomMenuItem::new("disconnect", "Disconnect");
    let logs = CustomMenuItem::new("show_logs", "Show Logs");
    let dashboard = CustomMenuItem::new("show_dashboard", "Show Dashboard");
    let reconnect = CustomMenuItem::new("reconnect", "Reconnect");
    let settings = CustomMenuItem::new("settings", "Settings");
    let open_logs_file = CustomMenuItem::new("open_logs_file", "Open Log File");
    let open_settings_file = CustomMenuItem::new("open_settings_file", "Open Settings File");
    let initial_connected = tauri::async_runtime::block_on(async {
        let mgr = app_state.tor_manager.read().await.clone();
        mgr.is_connected().await
    });
    let status = if initial_connected {
        "Connected"
    } else {
        "Disconnected"
    };
    let mut tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("status", format!("Status: {}", status)).disabled())
        .add_item(show.clone());
    if initial_connected {
        tray_menu = tray_menu.add_item(disconnect.clone());
    } else {
        tray_menu = tray_menu.add_item(connect.clone());
    }
    tray_menu = tray_menu
        .add_item(reconnect.clone())
        .add_item(logs.clone())
        .add_item(open_logs_file.clone())
        .add_item(dashboard.clone())
        .add_item(settings.clone())
        .add_item(open_settings_file.clone())
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
                "reconnect" => {
                    let state = app.state::<AppState>();
                    let handle = app.handle();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = commands::reconnect(handle.clone(), state.clone()).await {
                            log::error!("tray reconnect failed: {}", e);
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
                "show_dashboard" => {
                    let handle = app.handle();
                    tauri::async_runtime::spawn(async move {
                        let _ = commands::show_dashboard(handle).await;
                    });
                }
                "open_logs_file" => {
                    let state = app.state::<AppState>();
                    let path = state.log_file_path();
                    if let Err(e) = open::that(path) {
                        log::error!("failed to open log file: {e}");
                    }
                }
                "settings" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.emit("open-settings", ());
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "open_settings_file" => {
                    if let Err(e) = open::that(crate::state::DEFAULT_CONFIG_PATH) {
                        log::error!("failed to open settings file: {e}");
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
            commands::get_status_summary,
            commands::get_connection_timeline,
            commands::get_connection_health_summary,
            commands::get_active_circuit,
            commands::get_circuit_policy_report,
            commands::get_isolated_circuit,
            commands::set_exit_country,
            commands::set_entry_country,
            commands::set_middle_country,
            commands::set_bridges,
            commands::set_torrc_config,
            commands::generate_torrc_profile,
            commands::set_worker_config,
            commands::validate_worker_token,
            commands::set_hsm_config,
            commands::list_bridge_presets,
            commands::get_traffic_stats,
            commands::get_metrics,
            commands::list_circuits,
            commands::close_circuit,
            commands::get_logs,
            commands::clear_logs,
            commands::get_log_file_path,
            commands::set_log_limit,
            commands::load_metrics,
            commands::set_update_interval,
            commands::set_geoip_path,
            commands::set_insecure_hosts,
            commands::build_circuit,
            commands::ping_host,
            commands::ping_host_series,
            commands::dns_lookup,
            commands::traceroute_host,
            commands::lookup_country,
            commands::get_secure_key,
            commands::set_secure_key,
            commands::reconnect,
            commands::show_dashboard,
            commands::request_token
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
