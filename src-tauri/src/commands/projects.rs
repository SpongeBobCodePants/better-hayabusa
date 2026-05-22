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

    // Write sticky session BEFORE installing current_project, so an I/O
    // failure here doesn't leave backend state with the new project
    // installed while the command reports failure to the frontend.
    set_sticky_session(&app_conn, &project_folder)?;
    *state.current_project.lock()? = Some(CurrentProject {
        info: info.clone(),
        db: Mutex::new(connection),
    });

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
            // Write sticky session BEFORE installing current_project so
            // a failure here doesn't leave backend state diverged from
            // what the frontend believes (command reports error → store
            // never switches, but current_project would have already
            // been swapped).
            set_sticky_session(&app_conn, &folder)?;
            *state.current_project.lock()? = Some(CurrentProject {
                info: info.clone(),
                db: Mutex::new(connection),
            });
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
    // Lock order: app_db then current_project. Matches create_project /
    // open_project / check_last_open_project_cmd / delete_project — a
    // single canonical order prevents the classic A→B vs B→A deadlock
    // if two commands ever race.
    let app_conn = state.app_db.lock()?;
    clear_sticky_session(&app_conn)?;
    *state.current_project.lock()? = None;
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
    let outcome = check_last(&app_conn)?;

    // If Loaded, the preflight has already opened the project and
    // handed us the live connection — install it directly. Reopening
    // here would double-log the project_opened event and double-bump
    // recent_projects.last_opened_at.
    if let LaunchResult::Loaded { ref info } = outcome.result {
        let connection = outcome.connection.ok_or_else(|| CommandError::Internal {
            message: "check_last_open_project returned Loaded without a connection".into(),
        })?;
        *state.current_project.lock()? = Some(CurrentProject {
            info: info.clone(),
            db: Mutex::new(connection),
        });
    }
    Ok(outcome.result)
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
    // Also clear sticky-session pointer if it targets this same path —
    // otherwise dismissing a stale recent leaves the next launch
    // trying to restore the dead path and showing a Failed takeover.
    let app_conn = state.app_db.lock()?;
    crate::project::delete::clean_recents_and_sticky(&app_conn, &folder_path).map_err(|e| match e {
        crate::project::delete::DeleteError::Io(e) => CommandError::Io { message: e.to_string() },
        crate::project::delete::DeleteError::Sql(e) => CommandError::Db { message: e.to_string() },
        crate::project::delete::DeleteError::AppDb(e) => CommandError::Db { message: e.to_string() },
    })
}

#[tauri::command]
pub async fn delete_project(
    state: State<'_, AppState>,
    folder_path: String,
) -> Result<(), CommandError> {
    let folder = PathBuf::from(&folder_path);

    // Step 1 (sync, fast): if this is the currently-open project, drop
    // the CurrentProject handle so its DB Connection releases — Windows
    // refuses to delete project.db while we hold a Connection to it.
    // Cache the ProjectInfo so we can best-effort reinstall the session
    // if delete then fails mid-way.
    let cached_info: Option<ProjectInfo> = {
        let mut current = state.current_project.lock()?;
        if matches!(current.as_ref(), Some(cp) if cp.info.folder_path == folder_path) {
            let info = current.as_ref().map(|cp| cp.info.clone());
            *current = None;
            info
        } else {
            None
        }
    };

    // Step 2 (off main thread): the actual recursive delete. Project
    // folders can hold many GB of evidence files; doing this on the
    // Tauri command thread would freeze the UI and stall sibling IPC.
    let folder_for_fs = folder.clone();
    let fs_result: std::io::Result<()> = tokio::task::spawn_blocking(move || {
        crate::project::delete::remove_project_folder_if_present(&folder_for_fs)
    })
    .await
    .map_err(|e| CommandError::Internal {
        message: format!("delete blocking task join error: {e}"),
    })?;

    if let Err(io_err) = fs_result {
        // Step 3a: FS delete failed. Best-effort: reinstall the session
        // so the user doesn't lose their open project just because the
        // disk delete failed. Use open_existing (no CREATE flag, no
        // migrations) so that if the original project.db was already
        // removed during a partial remove_dir_all, we surface the loss
        // instead of silently creating a fresh empty DB and pretending
        // the session is fine. If reopen fails, leave current as None
        // and let the user pick the project again from the chooser.
        if let Some(info) = cached_info {
            let project_folder = PathBuf::from(&info.folder_path);
            if let Ok(connection) = crate::db::project_db::open_existing(
                &lifecycle::project_db_path(&project_folder),
            ) {
                *state.current_project.lock()? = Some(CurrentProject {
                    info,
                    db: Mutex::new(connection),
                });
            }
        }
        return Err(CommandError::Io { message: io_err.to_string() });
    }

    // Step 3b (sync, fast): FS deletion succeeded (or wasn't needed —
    // not a project). Clean up the recents row + sticky pointer.
    let app_conn = state.app_db.lock()?;
    crate::project::delete::clean_recents_and_sticky(&app_conn, &folder_path).map_err(|e| {
        match e {
            crate::project::delete::DeleteError::Io(e) => {
                CommandError::Io { message: e.to_string() }
            }
            crate::project::delete::DeleteError::Sql(e) => {
                CommandError::Db { message: e.to_string() }
            }
            crate::project::delete::DeleteError::AppDb(e) => {
                CommandError::Db { message: e.to_string() }
            }
        }
    })?;
    Ok(())
}
