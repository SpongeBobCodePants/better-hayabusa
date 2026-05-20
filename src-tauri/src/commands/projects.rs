use std::path::PathBuf;
use std::sync::Mutex;

use tauri::State;

use crate::commands::error::CommandError;
use crate::db::app_db;
use crate::project::lifecycle::{
    self, check_last_open_project as check_last,
    clear_sticky_session, create_project as create_p, open_project as open_p,
    set_sticky_session, LifecycleError, OpenOutcome,
};
use crate::types::{LaunchResult, ProjectInfo, RecentProject, RecentProjectListEntry};
use crate::{AppState, CurrentProject};

impl From<LifecycleError> for CommandError {
    fn from(e: LifecycleError) -> Self {
        match e {
            LifecycleError::AlreadyExists { path } => CommandError::AlreadyExists { path },
            LifecycleError::NotAProject { path } => CommandError::NotAProject { path },
            LifecycleError::NotFound { path } => CommandError::NotFound { path },
            LifecycleError::InvalidName { reason } => CommandError::InvalidName { reason },
            LifecycleError::InvalidDescription { reason } => CommandError::InvalidDescription { reason },
            LifecycleError::Io(e) => CommandError::Io { message: e.to_string() },
            LifecycleError::Sql(e) => CommandError::Db { message: e.to_string() },
            LifecycleError::ProjectDb(e) => CommandError::Db { message: e.to_string() },
            LifecycleError::AppDb(e) => CommandError::Db { message: e.to_string() },
            LifecycleError::ActivityLog(e) => CommandError::Io { message: e.to_string() },
            LifecycleError::Conflict(e) => CommandError::Internal { message: e.to_string() },
        }
    }
}

#[tauri::command]
pub fn create_project(
    state: State<'_, AppState>,
    // `folder_path` is the user-picked PARENT directory; the backend
    // creates a timestamped `<name>_YYYY.MM.DD_HHMMSS/` subfolder inside.
    folder_path: String,
    name: String,
    description: Option<String>,
) -> Result<ProjectInfo, CommandError> {
    let parent = PathBuf::from(&folder_path);
    let app_conn = state.app_db.lock()?;

    let info = create_p(&app_conn, &parent, &name, description.as_deref())?;

    // The returned folder_path is the timestamped subfolder — re-open
    // its project.db to grab a Connection for AppState.
    let project_folder = PathBuf::from(&info.folder_path);
    let connection = crate::db::project_db::open_or_create(&lifecycle::project_db_path(&project_folder))
        .map_err(|e| CommandError::Db { message: e.to_string() })?;

    *state.current_project.lock()? = Some(CurrentProject {
        info: info.clone(),
        db: Mutex::new(connection),
    });

    set_sticky_session(&app_conn, &project_folder)?;
    Ok(info)
}

#[tauri::command]
pub fn open_project(
    state: State<'_, AppState>,
    folder_path: String,
) -> Result<LaunchResult, CommandError> {
    let folder = PathBuf::from(&folder_path);
    let app_conn = state.app_db.lock()?;

    // Pre-flight: if the folder or project.db is gone, return Missing
    // instead of bubbling NotAProject. We deliberately do NOT auto-clean
    // the recents row here — the frontend prompts the user before
    // removing it. The sticky-restore path
    // (lifecycle::check_last_open_project) keeps its existing auto-clean
    // behavior; that one fires at app boot when the user didn't pick
    // anything, so the takeover UX is appropriate there.
    let project_db = lifecycle::project_db_path(&folder);
    if !folder.exists() || !project_db.exists() {
        let name = app_conn
            .query_row(
                "SELECT name FROM recent_projects WHERE path = ?1",
                [&folder_path],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .unwrap_or_else(|| String::from("(unknown)"));
        let reason = if !folder.exists() {
            "Folder no longer exists.".to_string()
        } else {
            "Project metadata (.bh/project.db) is missing.".to_string()
        };
        // Don't auto-clean: let the frontend prompt the user.
        return Ok(LaunchResult::Missing {
            path: folder_path,
            name,
            reason,
        });
    }

    match open_p(&app_conn, &folder)? {
        OpenOutcome::Loaded { info, connection } => {
            *state.current_project.lock()? = Some(CurrentProject {
                info: info.clone(),
                db: Mutex::new(connection),
            });
            set_sticky_session(&app_conn, &folder)?;
            Ok(LaunchResult::Loaded { info })
        }
        OpenOutcome::SchemaTooNew { path, name, project_version, app_version } => {
            // Don't install; let frontend show the upgrade screen.
            Ok(LaunchResult::SchemaTooNew { path, name, project_version, app_version })
        }
    }
}

#[tauri::command]
pub fn close_project(state: State<'_, AppState>) -> Result<(), CommandError> {
    *state.current_project.lock()? = None;
    let app_conn = state.app_db.lock()?;
    clear_sticky_session(&app_conn)?;
    Ok(())
}

#[tauri::command]
pub fn get_current_project(state: State<'_, AppState>) -> Result<Option<ProjectInfo>, CommandError> {
    let current = state.current_project.lock()?;
    Ok(current.as_ref().map(|cp| cp.info.clone()))
}

#[tauri::command]
pub fn check_last_open_project_cmd(state: State<'_, AppState>) -> Result<LaunchResult, CommandError> {
    let app_conn = state.app_db.lock()?;
    let result = check_last(&app_conn)?;

    // If Loaded, install into AppState (re-open to get the connection).
    if let LaunchResult::Loaded { ref info } = result {
        drop(app_conn); // release lock before re-acquiring through open_p
        let app_conn = state.app_db.lock()?;
        let folder = PathBuf::from(&info.folder_path);
        match open_p(&app_conn, &folder)? {
            OpenOutcome::Loaded { info: i, connection } => {
                *state.current_project.lock()? = Some(CurrentProject {
                    info: i,
                    db: Mutex::new(connection),
                });
            }
            OpenOutcome::SchemaTooNew { .. } => {
                // Shouldn't happen — check_last would have returned SchemaTooNew already.
                return Err(CommandError::Internal {
                    message: "open_project returned SchemaTooNew after check_last said Loaded".into(),
                });
            }
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn list_recent_projects(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<RecentProject>, CommandError> {
    let app_conn = state.app_db.lock()?;

    // If no explicit limit, read recent_projects_count from settings (default 5).
    let lim = match limit {
        Some(n) => n,
        None => app_db::get_state(&app_conn, "recent_projects_count")
            .ok()
            .flatten()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(5),
    };

    let rows = app_db::list_recent_projects(&app_conn)
        .map_err(|e| CommandError::Db { message: e.to_string() })?;
    let out: Vec<RecentProject> = rows
        .into_iter()
        .take(lim as usize)
        .map(|(path, name, last_opened_at)| RecentProject { path, name, last_opened_at })
        .collect();
    Ok(out)
}

#[tauri::command]
pub fn list_all_projects(state: State<'_, AppState>) -> Result<Vec<RecentProjectListEntry>, CommandError> {
    let app_conn = state.app_db.lock()?;
    let rows = app_db::list_recent_projects(&app_conn)
        .map_err(|e| CommandError::Db { message: e.to_string() })?;

    let mut out = Vec::with_capacity(rows.len());
    for (path, name, last_opened_at) in rows {
        let folder = std::path::Path::new(&path);
        let log = folder.join(".bh").join("activity.log");
        let last_modified = std::fs::metadata(&log)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| {
                use time::OffsetDateTime;
                use time::format_description::well_known::Rfc3339;
                OffsetDateTime::from(t).format(&Rfc3339).ok()
            });
        let project_db = lifecycle::project_db_path(folder);
        let folder_exists = project_db.exists();
        let description = if folder_exists {
            rusqlite::Connection::open(&project_db)
                .ok()
                .and_then(|conn| {
                    conn.query_row(
                        "SELECT description FROM projects LIMIT 1",
                        [],
                        |r| r.get::<_, Option<String>>(0),
                    )
                    .ok()
                })
                .flatten()
        } else {
            None
        };
        out.push(RecentProjectListEntry {
            path,
            name,
            description,
            last_opened_at,
            last_modified,
            folder_exists,
        });
    }
    Ok(out)
}

#[tauri::command]
pub fn remove_recent_project(
    state: State<'_, AppState>,
    folder_path: String,
) -> Result<(), CommandError> {
    let app_conn = state.app_db.lock()?;
    app_db::remove_recent_project(&app_conn, &folder_path)
        .map(|_| ())
        .map_err(|e| CommandError::Db { message: e.to_string() })
}

#[tauri::command]
pub fn delete_project(
    state: State<'_, AppState>,
    folder_path: String,
) -> Result<(), CommandError> {
    let folder = PathBuf::from(&folder_path);

    // If this is the currently-open project, close it first (drops DB Connection).
    {
        let mut current = state.current_project.lock()?;
        if let Some(cp) = current.as_ref() {
            if cp.info.folder_path == folder_path {
                *current = None;
            }
        }
    }

    let app_conn = state.app_db.lock()?;
    crate::project::delete::delete_project(&app_conn, &folder)
        .map_err(|e| match e {
            crate::project::delete::DeleteError::Io(e) => CommandError::Io { message: e.to_string() },
            crate::project::delete::DeleteError::Sql(e) => CommandError::Db { message: e.to_string() },
            crate::project::delete::DeleteError::AppDb(e) => CommandError::Db { message: e.to_string() },
        })?;
    Ok(())
}
