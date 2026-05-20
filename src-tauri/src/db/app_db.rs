use std::path::Path;

use rusqlite::Connection;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

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

/// Insert or update a recent_projects row, bumping last_opened_at to now.
pub fn upsert_recent_project(
    conn: &Connection,
    path: &str,
    name: &str,
) -> Result<(), AppDbError> {
    let now = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| AppDbError::Sql(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
    conn.execute(
        "INSERT INTO recent_projects (path, name, last_opened_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(path) DO UPDATE SET name = excluded.name, last_opened_at = excluded.last_opened_at",
        rusqlite::params![path, name, now],
    )?;
    Ok(())
}

/// Delete a recent_projects row by path. Returns true if a row was removed.
pub fn remove_recent_project(conn: &Connection, path: &str) -> Result<bool, AppDbError> {
    let n = conn.execute("DELETE FROM recent_projects WHERE path = ?1", [path])?;
    Ok(n > 0)
}

/// Read all recent_projects rows ordered by last_opened_at DESC.
pub fn list_recent_projects(conn: &Connection) -> Result<Vec<(String, String, String)>, AppDbError> {
    let mut stmt = conn.prepare(
        "SELECT path, name, last_opened_at FROM recent_projects ORDER BY last_opened_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}
