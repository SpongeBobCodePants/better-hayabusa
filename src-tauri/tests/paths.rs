use bhc_lib::paths::AppPaths;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn app_paths_from_exe_dir_places_db_next_to_exe() {
    let tmp = tempdir().unwrap();
    let fake_exe = tmp.path().join("bhc.exe");

    let paths = AppPaths::from_exe(&fake_exe).unwrap();

    assert_eq!(paths.exe_dir, tmp.path());
    assert_eq!(paths.app_db, tmp.path().join("app.db"));
    assert_eq!(paths.tools_dir, tmp.path().join("tools"));
}

#[test]
fn app_paths_errors_on_exe_without_parent() {
    let result = AppPaths::from_exe(&PathBuf::new());
    assert!(result.is_err(), "exe path with no parent should error");
}
