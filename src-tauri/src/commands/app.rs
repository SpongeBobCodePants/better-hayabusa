use tauri::State;

use crate::db::app_db;
use crate::types::AppVersion;
use crate::AppState;

#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> Result<AppVersion, String> {
    Ok(AppVersion {
        version: app.package_info().version.to_string(),
    })
}

#[tauri::command]
pub fn get_app_state(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let conn = state.app_db.lock().map_err(|e| e.to_string())?;
    app_db::get_state(&conn, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_app_state(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.app_db.lock().map_err(|e| e.to_string())?;
    app_db::set_state(&conn, &key, &value).map_err(|e| e.to_string())
}
