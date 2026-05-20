use bhc_lib::db::app_db;
use bhc_lib::project::lifecycle::create_project;
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
