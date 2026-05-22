use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use crate::db::migrations::{run_migrations, MigrationError, PROJECT_MIGRATIONS};

#[derive(Debug, thiserror::Error)]
pub enum ProjectDbError {
    #[error("sqlite: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("migration: {0}")]
    Migration(#[from] MigrationError),
}

/// Opens (creating if missing) the project.db at the given path and runs
/// migrations. Does NOT check schema version against the app — that's the
/// caller's responsibility (see project::lifecycle::open_project).
pub fn open_or_create(db_path: &Path) -> Result<Connection, ProjectDbError> {
    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    run_migrations(&conn, PROJECT_MIGRATIONS)?;
    Ok(conn)
}

/// Opens an existing project.db without creating it if missing and without
/// running migrations. Errors if the file is gone. Use this where the
/// caller expects the DB to already exist and creating a fresh one would
/// mask a real failure (e.g. delete-rollback paths).
pub fn open_existing(db_path: &Path) -> Result<Connection, ProjectDbError> {
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_URI,
    )?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    Ok(conn)
}

/// Reads `projects.app_schema_version` from the given project.db connection.
/// Returns None if the projects row doesn't exist yet (a freshly migrated,
/// not-yet-bootstrapped DB).
pub fn read_schema_version(conn: &Connection) -> Result<Option<u32>, rusqlite::Error> {
    use rusqlite::OptionalExtension;
    conn.query_row(
        "SELECT app_schema_version FROM projects LIMIT 1",
        [],
        |row| row.get::<_, u32>(0),
    )
    .optional()
}
