use crate::types::AppVersion;

#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> Result<AppVersion, String> {
    Ok(AppVersion {
        version: app.package_info().version.to_string(),
    })
}
