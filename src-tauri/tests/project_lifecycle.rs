use bhc_lib::db::app_db;
use bhc_lib::db::migrations::CURRENT_PROJECT_SCHEMA_VERSION;
use bhc_lib::project::lifecycle::{create_project, open_project, OpenOutcome};
use tempfile::tempdir;

#[test]
fn create_project_writes_db_and_log_and_recents_row() {
    let app_tmp = tempdir().unwrap();
    let app_db_path = app_tmp.path().join("app.db");
    let app_conn = app_db::open_or_create(&app_db_path).unwrap();

    let project_tmp = tempdir().unwrap();
    let project_folder = project_tmp.path();

    let info = create_project(
        &app_conn,
        project_folder,
        "Test Project",
        Some("A description"),
    )
    .expect("create_project");

    // project.db exists at the expected path
    let project_db = project_folder.join(".bhc").join("project.db");
    assert!(project_db.exists(), "project.db should exist");

    // activity.log exists
    let activity_log = project_folder.join(".bhc").join("activity.log");
    assert!(activity_log.exists(), "activity.log should exist");
    let log_contents = std::fs::read_to_string(&activity_log).unwrap();
    assert!(log_contents.contains("project_opened"));
    assert!(log_contents.contains("Test Project"));

    // recent_projects row inserted
    let count: i64 = app_conn.query_row(
        "SELECT COUNT(*) FROM recent_projects WHERE path = ?1",
        [project_folder.to_str().unwrap()],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 1);

    // ProjectInfo has the right shape
    assert_eq!(info.project.name, "Test Project");
    assert_eq!(info.project.description.as_deref(), Some("A description"));
    assert_eq!(info.project.app_schema_version, 1);
    assert_eq!(info.folder_path, project_folder.display().to_string());
}

#[test]
fn create_project_in_folder_that_already_has_project_errors() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();

    let project_tmp = tempdir().unwrap();
    let folder = project_tmp.path();

    create_project(&app_conn, folder, "First", None).unwrap();
    let result = create_project(&app_conn, folder, "Second", None);

    // Should be a LifecycleError::AlreadyExists
    use bhc_lib::project::lifecycle::LifecycleError;
    assert!(matches!(result, Err(LifecycleError::AlreadyExists { .. })));
}

#[test]
fn open_project_returns_loaded_for_compatible_schema() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();
    let folder = project_tmp.path();

    create_project(&app_conn, folder, "Test", None).unwrap();

    let outcome = open_project(&app_conn, folder).expect("open_project");
    match outcome {
        OpenOutcome::Loaded { info, .. } => {
            assert_eq!(info.project.name, "Test");
            assert_eq!(info.project.app_schema_version, CURRENT_PROJECT_SCHEMA_VERSION);
        }
        OpenOutcome::SchemaTooNew { .. } => panic!("expected Loaded"),
    }
}

#[test]
fn open_project_on_missing_bhc_errors_not_a_project() {
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
    let folder = project_tmp.path();

    create_project(&app_conn, folder, "Test", None).unwrap();

    // Forcibly bump the schema version in project.db to simulate a future app.
    let db_path = folder.join(".bhc").join("project.db");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute("UPDATE projects SET app_schema_version = 99", []).unwrap();
    drop(conn);

    let outcome = open_project(&app_conn, folder).expect("open returns Ok");
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
    let folder = project_tmp.path();

    create_project(&app_conn, folder, "Test", None).unwrap();

    let before: String = app_conn.query_row(
        "SELECT last_opened_at FROM recent_projects WHERE path = ?1",
        [folder.to_str().unwrap()],
        |r| r.get(0),
    ).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(1100)); // RFC3339 is sec precision

    open_project(&app_conn, folder).unwrap();

    let after: String = app_conn.query_row(
        "SELECT last_opened_at FROM recent_projects WHERE path = ?1",
        [folder.to_str().unwrap()],
        |r| r.get(0),
    ).unwrap();

    assert_ne!(before, after, "open_project should bump last_opened_at");
}
