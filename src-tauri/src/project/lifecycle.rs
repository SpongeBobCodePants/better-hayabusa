use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::db::{app_db, project_db};
use crate::db::migrations::CURRENT_PROJECT_SCHEMA_VERSION;
use crate::project::activity_log::{append_event, ActivityEvent};
use crate::project::conflict::{check_folder, ConflictCheckError, FolderState};
use crate::types::{Project, ProjectInfo};

#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error("folder already a project: {path}")]
    AlreadyExists { path: String },
    #[error("folder is not a project: {path}")]
    NotAProject { path: String },
    #[error("folder not found: {path}")]
    NotFound { path: String },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("project db: {0}")]
    ProjectDb(#[from] project_db::ProjectDbError),
    #[error("app db: {0}")]
    AppDb(#[from] app_db::AppDbError),
    #[error("activity log: {0}")]
    ActivityLog(#[from] crate::project::activity_log::ActivityLogError),
    #[error("conflict check: {0}")]
    Conflict(#[from] ConflictCheckError),
}

/// Path helpers — keep `.bhc/` layout in one place.
pub fn bhc_dir(folder: &Path) -> PathBuf { folder.join(".bhc") }
pub fn project_db_path(folder: &Path) -> PathBuf { bhc_dir(folder).join("project.db") }
pub fn activity_log_path(folder: &Path) -> PathBuf { bhc_dir(folder).join("activity.log") }

/// Creates a new project at `folder`. Bootstraps `.bhc/project.db`,
/// inserts the `projects` row, writes the first activity log entry, and
/// adds an entry to app.db's `recent_projects`.
///
/// Returns the loaded `ProjectInfo` ready for the caller to install as
/// the current project.
pub fn create_project(
    app_conn: &Connection,
    folder: &Path,
    name: &str,
    description: Option<&str>,
) -> Result<ProjectInfo, LifecycleError> {
    // 1. Conflict check.
    match check_folder(folder)? {
        FolderState::Eligible => {}
        FolderState::ExistingProject => {
            return Err(LifecycleError::AlreadyExists {
                path: folder.display().to_string(),
            });
        }
    }

    // 2. Create .bhc/ directory.
    fs::create_dir_all(bhc_dir(folder))?;

    // 3. Open project.db (runs migrations).
    let project_conn = project_db::open_or_create(&project_db_path(folder))?;

    // 4. Insert projects row.
    let now = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("RFC3339 format never fails for current_utc time");
    project_conn.execute(
        "INSERT INTO projects (name, description, created_at, app_schema_version) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, description, now, CURRENT_PROJECT_SCHEMA_VERSION],
    )?;

    // 5. Read the inserted row.
    let project: Project = project_conn.query_row(
        "SELECT id, name, description, created_at, app_schema_version FROM projects LIMIT 1",
        [],
        |row| Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            app_schema_version: row.get::<_, u32>(4)?,
        }),
    )?;

    // 6. Activity log.
    append_event(
        &activity_log_path(folder),
        ActivityEvent::ProjectOpened { name: name.to_string() },
    )?;

    // 7. Upsert recent_projects.
    let folder_str = folder.display().to_string();
    app_db::upsert_recent_project(app_conn, &folder_str, name)?;

    Ok(ProjectInfo { project, folder_path: folder_str })
}

/// Possible outcomes of `open_project`. The schema-version mismatch is
/// not an Err because it's a designed UX state (user-recoverable: upgrade
/// the app), not a system failure.
pub enum OpenOutcome {
    Loaded {
        info: ProjectInfo,
        connection: Connection, // caller installs into AppState
    },
    SchemaTooNew {
        path: String,
        name: String,
        project_version: u32,
        app_version: u32,
    },
}

/// Opens an existing project. Validates that `.bhc/project.db` exists,
/// runs forward migrations (no-op if up to date), checks the project's
/// stored schema version against the app's, logs the open event, bumps
/// `recent_projects.last_opened_at`.
pub fn open_project(
    app_conn: &Connection,
    folder: &Path,
) -> Result<OpenOutcome, LifecycleError> {
    let db_path = project_db_path(folder);
    if !db_path.exists() {
        return Err(LifecycleError::NotAProject {
            path: folder.display().to_string(),
        });
    }

    let project_conn = project_db::open_or_create(&db_path)?;

    // Read schema version.
    let project_version = project_db::read_schema_version(&project_conn)?
        .ok_or_else(|| LifecycleError::NotAProject {
            path: folder.display().to_string(),
        })?;

    if project_version > CURRENT_PROJECT_SCHEMA_VERSION {
        let name: String = project_conn
            .query_row("SELECT name FROM projects LIMIT 1", [], |r| r.get(0))
            .unwrap_or_else(|_| String::from("(unknown)"));

        return Ok(OpenOutcome::SchemaTooNew {
            path: folder.display().to_string(),
            name,
            project_version,
            app_version: CURRENT_PROJECT_SCHEMA_VERSION,
        });
    }

    // Read the project row.
    let project: Project = project_conn.query_row(
        "SELECT id, name, description, created_at, app_schema_version FROM projects LIMIT 1",
        [],
        |row| Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            app_schema_version: row.get::<_, u32>(4)?,
        }),
    )?;

    let folder_str = folder.display().to_string();

    // Log + upsert recents (best-effort log; recents upsert is required).
    let _ = append_event(
        &activity_log_path(folder),
        ActivityEvent::ProjectOpened { name: project.name.clone() },
    );
    app_db::upsert_recent_project(app_conn, &folder_str, &project.name)?;

    Ok(OpenOutcome::Loaded {
        info: ProjectInfo { project, folder_path: folder_str },
        connection: project_conn,
    })
}
