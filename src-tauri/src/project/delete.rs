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
/// corresponding `recent_projects` entry from app.db.
///
/// If the folder doesn't exist, just cleans up recents (no error).
///
/// **Safety:** the caller MUST first close any open Connection to this
/// project's project.db, or the file lock will prevent deletion on
/// Windows.
pub fn delete_project(app_conn: &Connection, folder: &Path) -> Result<(), DeleteError> {
    if folder.exists() {
        fs::remove_dir_all(folder)?;
    }
    app_db::remove_recent_project(app_conn, &folder.display().to_string())?;
    Ok(())
}
