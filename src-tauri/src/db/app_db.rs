use std::path::Path;

use rusqlite::Connection;

use crate::db::migrations::{run_migrations, MigrationError, APP_MIGRATIONS};

#[derive(Debug, thiserror::Error)]
pub enum AppDbError {
    #[error("sqlite: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("migration: {0}")]
    Migration(#[from] MigrationError),
}

/// Opens (creating if missing) the app.db at the given path and runs migrations.
pub fn open_or_create(db_path: &Path) -> Result<Connection, AppDbError> {
    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    run_migrations(&conn, APP_MIGRATIONS)?;
    Ok(conn)
}
