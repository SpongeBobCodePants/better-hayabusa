use std::path::PathBuf;

use bhc_lib::db::app_db;
use bhc_lib::project::delete::delete_project;
use bhc_lib::project::lifecycle::create_project;
use tempfile::tempdir;

#[test]
fn delete_project_removes_folder_and_recents_row() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();
    let parent = project_tmp.path().to_path_buf();

    let info = create_project(&app_conn, &parent, "Test", None).unwrap();
    let project_folder = PathBuf::from(&info.folder_path);

    assert!(project_folder.exists());
    assert!(project_folder.join(".bh").join("project.db").exists());

    delete_project(&app_conn, &project_folder).expect("delete");

    assert!(!project_folder.exists(), "project folder should be gone");
    // The parent should still exist (we only delete the project subfolder).
    assert!(parent.exists(), "parent folder should remain");

    let count: i64 = app_conn
        .query_row(
            "SELECT COUNT(*) FROM recent_projects WHERE path = ?1",
            [info.folder_path.as_str()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn delete_project_with_evidence_files_deletes_them_too() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();
    let parent = project_tmp.path().to_path_buf();

    let info = create_project(&app_conn, &parent, "Test", None).unwrap();
    let project_folder = PathBuf::from(&info.folder_path);

    // Drop some "evidence" files inside the timestamped project folder.
    std::fs::write(project_folder.join("evidence-DC01.evtx"), b"fake").unwrap();
    std::fs::create_dir(project_folder.join("subfolder")).unwrap();
    std::fs::write(project_folder.join("subfolder").join("more.evtx"), b"fake").unwrap();

    delete_project(&app_conn, &project_folder).expect("delete");
    assert!(!project_folder.exists());
}

#[test]
fn delete_project_on_missing_folder_still_cleans_recents() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();

    // Insert a stale recents entry.
    app_db::upsert_recent_project(&app_conn, "C:\\does-not-exist", "Ghost").unwrap();

    delete_project(&app_conn, std::path::Path::new("C:\\does-not-exist"))
        .expect("delete should succeed (recents-only cleanup)");

    let count: i64 = app_conn
        .query_row(
            "SELECT COUNT(*) FROM recent_projects WHERE path = ?1",
            ["C:\\does-not-exist"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}
