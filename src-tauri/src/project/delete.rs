use std::fs;
use std::path::Path;

use rusqlite::Connection;
use thiserror::Error;

use crate::db::app_db;

#[derive(Debug, Error)]
pub enum DeleteError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("app db: {0}")]
    AppDb(#[from] app_db::AppDbError),
}

/// Filesystem half of the delete operation: recursively removes `folder`
/// iff it's actually a Better Hayabusa project (`.bh/project.db` is
/// present). If the folder is missing or exists but isn't a project,
/// this is a no-op — protecting against accidental recursive deletion
/// of unrelated user data via stale recents entries.
///
/// Split out from [`delete_project`] so the Tauri command can offload
/// the potentially slow `fs::remove_dir_all` to `spawn_blocking` without
/// holding a DB Connection across the await.
///
/// **Safety:** the caller MUST first close any open Connection to this
/// project's project.db, or the file lock will prevent deletion on
/// Windows.
pub fn remove_project_folder_if_present(folder: &Path) -> std::io::Result<()> {
    let project_db = folder.join(".bh").join("project.db");
    if folder.exists() && project_db.exists() {
        fs::remove_dir_all(folder)?;
    }
    Ok(())
}

/// DB-half of the delete operation: removes the `recent_projects` row
/// for `folder_str` and clears `last_open_project_path` if it targets
/// the same folder. Idempotent — safe to call when the row isn't there.
pub fn clean_recents_and_sticky(
    app_conn: &Connection,
    folder_str: &str,
) -> Result<(), DeleteError> {
    app_db::remove_recent_project(app_conn, folder_str)?;
    let sticky = app_db::get_state(app_conn, "last_open_project_path")?;
    if sticky.as_deref() == Some(folder_str) {
        app_conn.execute(
            "DELETE FROM app_state WHERE key = 'last_open_project_path'",
            [],
        )?;
    }
    Ok(())
}

/// Recursively deletes the project folder at `folder` and removes the
/// corresponding `recent_projects` entry from app.db. Also clears the
/// sticky-session pointer if it targets the same folder, so the next
/// launch doesn't show a Failed takeover for a path that's been removed.
///
/// Convenience wrapper composing [`remove_project_folder_if_present`]
/// and [`clean_recents_and_sticky`]. The Tauri command splits these two
/// halves apart so the FS work can run off the main thread.
///
/// **Safety:** the caller MUST first close any open Connection to this
/// project's project.db, or the file lock will prevent deletion on
/// Windows.
pub fn delete_project(app_conn: &Connection, folder: &Path) -> Result<(), DeleteError> {
    remove_project_folder_if_present(folder)?;
    clean_recents_and_sticky(app_conn, &folder.display().to_string())
}
