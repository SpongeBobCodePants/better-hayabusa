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

/// Recursively deletes the project folder at `folder` and removes the
/// corresponding `recent_projects` entry from app.db. Also clears the
/// sticky-session pointer if it targets the same folder, so the next
/// launch doesn't show a Failed takeover for a path that's been removed.
///
/// Only removes the folder if it actually is a Better Hayabusa project
/// (i.e. `.bh/project.db` is present). If the folder is missing, or
/// exists but is not a project, the recents row is still cleaned but
/// the folder itself is left untouched — protecting against accidental
/// recursive deletion of unrelated user data via stale recents entries.
///
/// **Safety:** the caller MUST first close any open Connection to this
/// project's project.db, or the file lock will prevent deletion on
/// Windows.
pub fn delete_project(app_conn: &Connection, folder: &Path) -> Result<(), DeleteError> {
    let project_db = folder.join(".bh").join("project.db");
    if folder.exists() && project_db.exists() {
        fs::remove_dir_all(folder)?;
    }
    let folder_str = folder.display().to_string();
    app_db::remove_recent_project(app_conn, &folder_str)?;

    let sticky = app_db::get_state(app_conn, "last_open_project_path")?;
    if sticky.as_deref() == Some(folder_str.as_str()) {
        app_conn.execute(
            "DELETE FROM app_state WHERE key = 'last_open_project_path'",
            [],
        )?;
    }
    Ok(())
}
