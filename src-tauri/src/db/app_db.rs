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

pub fn get_state(conn: &Connection, key: &str) -> Result<Option<String>, AppDbError> {
    use rusqlite::OptionalExtension;
    let v = conn
        .query_row(
            "SELECT value FROM app_state WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    Ok(v)
}

pub fn set_state(conn: &Connection, key: &str, value: &str) -> Result<(), AppDbError> {
    conn.execute(
        "INSERT INTO app_state (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [key, value],
    )?;
    Ok(())
}
