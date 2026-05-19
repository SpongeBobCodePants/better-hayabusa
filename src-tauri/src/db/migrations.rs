use rusqlite::{Connection, OptionalExtension};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("sqlite error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("migration {0} failed: {1}")]
    Migration(&'static str, rusqlite::Error),
}

/// A single migration: a stable name + its SQL.
pub struct Migration {
    pub name: &'static str,
    pub sql: &'static str,
}

pub const APP_MIGRATIONS: &[Migration] = &[Migration {
    name: "001_init",
    sql: include_str!("../../migrations/app/001_init.sql"),
}];

pub fn run_migrations(conn: &Connection, migrations: &[Migration]) -> Result<(), MigrationError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (name TEXT PRIMARY KEY, applied_at TEXT NOT NULL)",
        [],
    )?;

    for m in migrations {
        let already_applied: Option<String> = conn
            .query_row(
                "SELECT name FROM _migrations WHERE name = ?1",
                [m.name],
                |row| row.get(0),
            )
            .optional()?;

        if already_applied.is_some() {
            continue;
        }

        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(m.sql).map_err(|e| MigrationError::Migration(m.name, e))?;
        tx.execute(
            "INSERT INTO _migrations (name, applied_at) VALUES (?1, datetime('now'))",
            [m.name],
        )?;
        tx.commit()?;
    }

    Ok(())
}
