use bhc_lib::project::conflict::{check_folder, FolderState};
use std::fs;
use tempfile::tempdir;

#[test]
fn empty_folder_is_eligible() {
    let tmp = tempdir().unwrap();
    let state = check_folder(tmp.path()).unwrap();
    assert_eq!(state, FolderState::Eligible);
}

#[test]
fn folder_with_bh_project_db_is_existing_project() {
    let tmp = tempdir().unwrap();
    fs::create_dir(tmp.path().join(".bh")).unwrap();
    fs::write(tmp.path().join(".bh").join("project.db"), b"").unwrap();

    let state = check_folder(tmp.path()).unwrap();
    assert_eq!(state, FolderState::ExistingProject);
}

#[test]
fn folder_with_other_files_but_no_bh_is_eligible() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("readme.txt"), b"hi").unwrap();
    let state = check_folder(tmp.path()).unwrap();
    assert_eq!(state, FolderState::Eligible);
}

#[test]
fn nonexistent_folder_returns_not_found() {
    let result = check_folder(std::path::Path::new("C:\\definitely-does-not-exist-xyz"));
    assert!(matches!(result, Err(_)));
}
