use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension};
use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

use crate::db::{app_db, project_db};
use crate::db::migrations::CURRENT_PROJECT_SCHEMA_VERSION;
use crate::project::activity_log::{append_event, ActivityEvent};
use crate::project::conflict::{check_folder, ConflictCheckError, FolderState};
use crate::project::name::{validate_project_description, validate_project_name};
use crate::types::{LaunchResult, Project, ProjectInfo};

#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error("folder already a project: {path}")]
    AlreadyExists { path: String },
    #[error("folder is not a project: {path}")]
    NotAProject { path: String },
    #[error("folder not found: {path}")]
    NotFound { path: String },
    #[error("invalid name: {reason}")]
    InvalidName { reason: String },
    #[error("invalid description: {reason}")]
    InvalidDescription { reason: String },
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

/// Path helpers — keep `.bh/` layout in one place.
pub fn bh_dir(folder: &Path) -> PathBuf { folder.join(".bh") }
pub fn project_db_path(folder: &Path) -> PathBuf { bh_dir(folder).join("project.db") }
pub fn activity_log_path(folder: &Path) -> PathBuf { bh_dir(folder).join("activity.log") }

/// Creates a new project. `parent_folder` is the user-picked parent
/// directory; this function creates a timestamped subfolder
/// `<parent_folder>/<name>_YYYY.MM.DD_HHMMSS/` (UTC) and places `.bh/`
/// inside it. Bootstraps `.bh/project.db`, inserts the `projects` row,
/// writes the first activity log entry, and adds an entry to app.db's
/// `recent_projects`.
///
/// Returns the loaded `ProjectInfo` with `folder_path` set to the
/// timestamped subfolder (not the parent) — ready for the caller to
/// install as the current project.
pub fn create_project(
    app_conn: &Connection,
    parent_folder: &Path,
    name: &str,
    description: Option<&str>,
) -> Result<ProjectInfo, LifecycleError> {
    // 1. Validate the name against Windows filename rules (re-validates
    //    what the frontend already checked; never trust the frontend).
    validate_project_name(name)
        .map_err(|reason| LifecycleError::InvalidName { reason })?;
    validate_project_description(description)
        .map_err(|reason| LifecycleError::InvalidDescription { reason })?;

    // 2. Conflict check on the parent. If the parent itself is already a
    //    project, the user is trying to create a project inside another
    //    project — reject the same way as before.
    match check_folder(parent_folder)? {
        FolderState::Eligible => {}
        FolderState::ExistingProject => {
            return Err(LifecycleError::AlreadyExists {
                path: parent_folder.display().to_string(),
            });
        }
    }

    // 3. Compute timestamped subfolder name.
    let ts_format = format_description!("[year].[month].[day]_[hour][minute][second]");
    let ts = OffsetDateTime::now_utc()
        .format(&ts_format)
        .expect("compile-time format description never fails for current_utc time");
    let project_folder_name = format!("{name}_{ts}");
    let project_folder = parent_folder.join(&project_folder_name);

    // 4. Create the timestamped project folder. Reject if the path
    //    already exists — second-precision timestamps can collide on
    //    rapid same-name creates, and `create_dir_all` would silently
    //    reuse the folder, leaving project.db's `SELECT ... LIMIT 1`
    //    non-deterministic across multiple `projects` rows.
    if project_folder.exists() {
        return Err(LifecycleError::AlreadyExists {
            path: project_folder.display().to_string(),
        });
    }
    fs::create_dir_all(&project_folder)?;

    // 5. Create .bh/ directory inside the project folder.
    fs::create_dir_all(bh_dir(&project_folder))?;

    // 6. Open project.db (runs migrations).
    let project_conn = project_db::open_or_create(&project_db_path(&project_folder))?;

    // 7. Insert projects row.
    let now = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("RFC3339 format never fails for current_utc time");
    project_conn.execute(
        "INSERT INTO projects (name, description, created_at, app_schema_version) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, description, now, CURRENT_PROJECT_SCHEMA_VERSION],
    )?;

    // 8. Read the inserted row.
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

    // 9. Activity log.
    append_event(
        &activity_log_path(&project_folder),
        ActivityEvent::ProjectOpened { name: name.to_string() },
    )?;

    // 10. Upsert recent_projects with the timestamped subfolder path.
    let folder_str = project_folder.display().to_string();
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

/// Opens an existing project. Validates that `.bh/project.db` exists,
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

/// Drops the AppState current_project handle. Called from the command
/// layer (this Rust API doesn't own AppState).
///
/// Side effect: clears `last_open_project_path` in app.db, so a subsequent
/// launch with sticky session enabled lands on Home.
pub fn clear_sticky_session(app_conn: &Connection) -> Result<(), LifecycleError> {
    app_conn.execute(
        "DELETE FROM app_state WHERE key = 'last_open_project_path'",
        [],
    )?;
    Ok(())
}

/// Sets `last_open_project_path` so the next launch can sticky-restore.
pub fn set_sticky_session(app_conn: &Connection, folder: &Path) -> Result<(), LifecycleError> {
    app_db::set_state(app_conn, "last_open_project_path", &folder.display().to_string())?;
    Ok(())
}

/// Wrapper around `LaunchResult` that also carries the live project DB
/// `Connection` when the outcome is `Loaded`. The Tauri command layer
/// installs the connection into `AppState`; the TS-exported shape is
/// just `LaunchResult` (via `.result`).
pub struct LaunchOutcome {
    pub result: LaunchResult,
    pub connection: Option<Connection>,
}

/// Run at app launch. Decides whether to sticky-restore, land on Home,
/// or show a failure screen.
///
/// When this returns `LaunchResult::Loaded`, the project DB has already
/// been opened (with its side effects: activity log + recents bump) and
/// the live connection is returned in `LaunchOutcome.connection`. The
/// command layer must use that connection instead of reopening, or the
/// open side effects will fire twice per launch.
pub fn check_last_open_project(app_conn: &Connection) -> Result<LaunchOutcome, LifecycleError> {
    // Honor the launch_behavior setting.
    let behavior = app_db::get_state(app_conn, "launch_behavior")?
        .unwrap_or_else(|| "last_project".to_string());
    if behavior == "home_page" {
        return Ok(LaunchOutcome { result: LaunchResult::Disabled, connection: None });
    }

    let last_path = match app_db::get_state(app_conn, "last_open_project_path")? {
        Some(p) => p,
        None => return Ok(LaunchOutcome { result: LaunchResult::NoneSet, connection: None }),
    };

    let folder = PathBuf::from(&last_path);

    // Look up the friendly name from recents (if any) for error reporting.
    let name = app_conn
        .query_row(
            "SELECT name FROM recent_projects WHERE path = ?1",
            [&last_path],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .unwrap_or_else(|| String::from("(unknown)"));

    // Folder still exists?
    if !folder.exists() {
        app_db::remove_recent_project(app_conn, &last_path)?;
        clear_sticky_session(app_conn)?;
        return Ok(LaunchOutcome {
            result: LaunchResult::Failed {
                path: last_path,
                name,
                reason: "Folder no longer exists.".to_string(),
            },
            connection: None,
        });
    }

    // project.db still there?
    if !project_db_path(&folder).exists() {
        app_db::remove_recent_project(app_conn, &last_path)?;
        clear_sticky_session(app_conn)?;
        return Ok(LaunchOutcome {
            result: LaunchResult::Failed {
                path: last_path,
                name,
                reason: "Project metadata (.bh/project.db) is missing.".to_string(),
            },
            connection: None,
        });
    }

    // Try opening. Hold onto the live connection so the command layer
    // can install it without a second `open_project` call (which would
    // double-log + double-bump-recents).
    match open_project(app_conn, &folder)? {
        OpenOutcome::Loaded { info, connection } => Ok(LaunchOutcome {
            result: LaunchResult::Loaded { info },
            connection: Some(connection),
        }),
        OpenOutcome::SchemaTooNew { path, name, project_version, app_version } => {
            Ok(LaunchOutcome {
                result: LaunchResult::SchemaTooNew { path, name, project_version, app_version },
                connection: None,
            })
        }
    }
}
