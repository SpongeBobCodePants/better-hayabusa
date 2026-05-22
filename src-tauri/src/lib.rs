use std::sync::Mutex;

pub mod commands;
pub mod db;
pub mod paths;
pub mod project;
pub mod types;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

/// Holds the currently-open project's metadata + DB connection.
/// Single project at a time per M2 design.
pub struct CurrentProject {
    pub info: types::ProjectInfo,
    pub db: Mutex<rusqlite::Connection>,
}

pub struct AppState {
    pub app_db: Mutex<rusqlite::Connection>,
    pub paths: paths::AppPaths,
    pub current_project: Mutex<Option<CurrentProject>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_paths = paths::AppPaths::from_current_exe()
                .expect("resolve app paths");
            let conn = db::app_db::open_or_create(&app_paths.app_db)
                .expect("open or create app.db");
            app.manage(AppState {
                app_db: Mutex::new(conn),
                paths: app_paths,
                current_project: Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_version,
            commands::app::get_app_state,
            commands::app::set_app_state,
            commands::projects::create_project,
            commands::projects::open_project,
            commands::projects::close_project,
            commands::projects::get_current_project,
            commands::projects::check_last_open_project_cmd,
            commands::projects::list_all_projects,
            commands::projects::remove_recent_project,
            commands::projects::delete_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
