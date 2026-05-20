use std::path::PathBuf;

use bhc_lib::db::app_db;
use bhc_lib::db::migrations::CURRENT_PROJECT_SCHEMA_VERSION;
use bhc_lib::project::lifecycle::{check_last_open_project, create_project, open_project, OpenOutcome};
use bhc_lib::types::LaunchResult;
use tempfile::tempdir;

#[test]
fn create_project_writes_db_and_log_and_recents_row() {
    let app_tmp = tempdir().unwrap();
    let app_db_path = app_tmp.path().join("app.db");
    let app_conn = app_db::open_or_create(&app_db_path).unwrap();

    let project_tmp = tempdir().unwrap();
    let parent_folder = project_tmp.path();

    let info = create_project(
        &app_conn,
        parent_folder,
        "Test Project",
        Some("A description"),
    )
    .expect("create_project");

    // The returned folder_path is the timestamped subfolder, inside the parent.
    let project_folder = PathBuf::from(&info.folder_path);
    assert!(
        info.folder_path.starts_with(&parent_folder.display().to_string()),
        "project folder should be inside parent_folder"
    );
    assert_ne!(
        project_folder.as_path(),
        parent_folder,
        "project folder should be a subfolder, not the parent itself"
    );

    // project.db exists at the expected path inside the subfolder.
    let project_db = project_folder.join(".bh").join("project.db");
    assert!(project_db.exists(), "project.db should exist");

    // activity.log exists in the subfolder.
    let activity_log = project_folder.join(".bh").join("activity.log");
    assert!(activity_log.exists(), "activity.log should exist");
    let log_contents = std::fs::read_to_string(&activity_log).unwrap();
    assert!(log_contents.contains("project_opened"));
    assert!(log_contents.contains("Test Project"));

    // recent_projects row inserted with the subfolder path.
    let count: i64 = app_conn.query_row(
        "SELECT COUNT(*) FROM recent_projects WHERE path = ?1",
        [info.folder_path.as_str()],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 1);

    // ProjectInfo has the right shape.
    assert_eq!(info.project.name, "Test Project");
    assert_eq!(info.project.description.as_deref(), Some("A description"));
    assert_eq!(info.project.app_schema_version, 1);
}

#[test]
fn create_project_rejects_invalid_name() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();

    let result = create_project(&app_conn, project_tmp.path(), "bad/name", None);
    use bhc_lib::project::lifecycle::LifecycleError;
    assert!(matches!(result, Err(LifecycleError::InvalidName { .. })));
}

#[test]
fn create_project_rejects_description_over_250_chars() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();
    let long = "x".repeat(251);
    let result = create_project(&app_conn, project_tmp.path(), "Test", Some(&long));
    use bhc_lib::project::lifecycle::LifecycleError;
    assert!(matches!(result, Err(LifecycleError::InvalidDescription { .. })));
}

#[test]
fn create_project_inside_existing_project_errors() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();

    let project_tmp = tempdir().unwrap();
    let parent = project_tmp.path();

    // First create: makes <parent>/First_ts/.bh/project.db
    let info = create_project(&app_conn, parent, "First", None).unwrap();
    let first_project_folder = PathBuf::from(&info.folder_path);

    // Now try to create a second project using the FIRST project's
    // folder as the parent. That parent already has .bh/project.db, so
    // the conflict check should reject it.
    let result = create_project(&app_conn, &first_project_folder, "Second", None);

    use bhc_lib::project::lifecycle::LifecycleError;
    assert!(matches!(result, Err(LifecycleError::AlreadyExists { .. })));
}

#[test]
fn open_project_returns_loaded_for_compatible_schema() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();

    let info = create_project(&app_conn, project_tmp.path(), "Test", None).unwrap();
    let project_folder = PathBuf::from(&info.folder_path);

    let outcome = open_project(&app_conn, &project_folder).expect("open_project");
    match outcome {
        OpenOutcome::Loaded { info, .. } => {
            assert_eq!(info.project.name, "Test");
            assert_eq!(info.project.app_schema_version, CURRENT_PROJECT_SCHEMA_VERSION);
        }
        OpenOutcome::SchemaTooNew { .. } => panic!("expected Loaded"),
    }
}

#[test]
fn open_project_on_missing_bh_errors_not_a_project() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let empty_tmp = tempdir().unwrap();

    let result = open_project(&app_conn, empty_tmp.path());
    assert!(matches!(result, Err(bhc_lib::project::lifecycle::LifecycleError::NotAProject { .. })));
}

#[test]
fn open_project_with_too_new_schema_returns_schema_too_new() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();

    let info = create_project(&app_conn, project_tmp.path(), "Test", None).unwrap();
    let project_folder = PathBuf::from(&info.folder_path);

    // Forcibly bump the schema version in project.db to simulate a future app.
    let db_path = project_folder.join(".bh").join("project.db");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute("UPDATE projects SET app_schema_version = 99", []).unwrap();
    drop(conn);

    let outcome = open_project(&app_conn, &project_folder).expect("open returns Ok");
    match outcome {
        OpenOutcome::SchemaTooNew { project_version, app_version, .. } => {
            assert_eq!(project_version, 99);
            assert_eq!(app_version, CURRENT_PROJECT_SCHEMA_VERSION);
        }
        OpenOutcome::Loaded { .. } => panic!("expected SchemaTooNew"),
    }
}

#[test]
fn open_project_updates_last_opened_at_in_recents() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();

    let info = create_project(&app_conn, project_tmp.path(), "Test", None).unwrap();
    let project_folder = PathBuf::from(&info.folder_path);

    let before: String = app_conn.query_row(
        "SELECT last_opened_at FROM recent_projects WHERE path = ?1",
        [info.folder_path.as_str()],
        |r| r.get(0),
    ).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(1100)); // RFC3339 is sec precision

    open_project(&app_conn, &project_folder).unwrap();

    let after: String = app_conn.query_row(
        "SELECT last_opened_at FROM recent_projects WHERE path = ?1",
        [info.folder_path.as_str()],
        |r| r.get(0),
    ).unwrap();

    assert_ne!(before, after, "open_project should bump last_opened_at");
}

#[test]
fn check_last_open_when_none_set_returns_none_set() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();

    let result = check_last_open_project(&app_conn).unwrap();
    assert!(matches!(result, LaunchResult::NoneSet));
}

#[test]
fn check_last_open_when_disabled_returns_disabled() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    app_db::set_state(&app_conn, "launch_behavior", "home_page").unwrap();
    app_db::set_state(&app_conn, "last_open_project_path", "C:\\whatever").unwrap();

    let result = check_last_open_project(&app_conn).unwrap();
    assert!(matches!(result, LaunchResult::Disabled));
}

#[test]
fn check_last_open_with_loadable_project_returns_loaded() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();

    let project_tmp = tempdir().unwrap();
    let info = create_project(&app_conn, project_tmp.path(), "Test", None).unwrap();
    app_db::set_state(
        &app_conn,
        "last_open_project_path",
        info.folder_path.as_str(),
    ).unwrap();

    let result = check_last_open_project(&app_conn).unwrap();
    assert!(matches!(result, LaunchResult::Loaded { .. }));
}

#[test]
fn check_last_open_with_missing_folder_returns_failed_and_cleans_recents() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();

    let missing = "C:\\definitely-does-not-exist-xyz";
    app_db::upsert_recent_project(&app_conn, missing, "Ghost").unwrap();
    app_db::set_state(&app_conn, "last_open_project_path", missing).unwrap();

    let result = check_last_open_project(&app_conn).unwrap();
    match result {
        LaunchResult::Failed { path, name, .. } => {
            assert_eq!(path, missing);
            assert_eq!(name, "Ghost");
        }
        _ => panic!("expected Failed"),
    }

    // Recents entry should be gone.
    let count: i64 = app_conn.query_row(
        "SELECT COUNT(*) FROM recent_projects WHERE path = ?1",
        [missing],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 0);

    // last_open_project_path setting should be cleared.
    let v = app_db::get_state(&app_conn, "last_open_project_path").unwrap();
    assert!(v.is_none());
}
