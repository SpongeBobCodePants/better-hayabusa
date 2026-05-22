use tauri::State;

use crate::commands::error::CommandError;
use crate::db::app_db;
use crate::types::AppVersion;
use crate::AppState;

#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> Result<AppVersion, CommandError> {
    Ok(AppVersion {
        version: app.package_info().version.to_string(),
    })
}

#[tauri::command]
pub fn get_app_state(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<String>, CommandError> {
    let conn = state.app_db.lock()?;
    app_db::get_state(&conn, &key).map_err(|e| CommandError::Db { message: e.to_string() })
}

#[tauri::command]
pub fn set_app_state(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), CommandError> {
    let conn = state.app_db.lock()?;
    app_db::set_state(&conn, &key, &value).map_err(|e| CommandError::Db { message: e.to_string() })
}
