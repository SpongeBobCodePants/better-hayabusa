use std::sync::Mutex;

pub mod commands;
pub mod db;
pub mod paths;
pub mod platform;
pub mod types;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

pub struct AppState {
    pub app_db: Mutex<rusqlite::Connection>,
    pub paths: paths::AppPaths,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            let app_paths = paths::AppPaths::from_current_exe()
                .expect("resolve app paths");
            let conn = db::app_db::open_or_create(&app_paths.app_db)
                .expect("open or create app.db");
            app.manage(AppState { app_db: Mutex::new(conn), paths: app_paths });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_version,
            commands::app::get_app_state,
            commands::app::set_app_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
