use bhc_lib::db::app_db::{get_state, open_or_create, set_state};
use tempfile::tempdir;

#[test]
fn get_state_returns_none_for_missing_key() {
    let tmp = tempdir().unwrap();
    let conn = open_or_create(&tmp.path().join("app.db")).unwrap();
    let v = get_state(&conn, "theme").unwrap();
    assert!(v.is_none());
}

#[test]
fn set_state_then_get_state_returns_value() {
    let tmp = tempdir().unwrap();
    let conn = open_or_create(&tmp.path().join("app.db")).unwrap();
    set_state(&conn, "theme", "dark").unwrap();
    let v = get_state(&conn, "theme").unwrap();
    assert_eq!(v, Some("dark".to_string()));
}

#[test]
fn set_state_overwrites_existing_value() {
    let tmp = tempdir().unwrap();
    let conn = open_or_create(&tmp.path().join("app.db")).unwrap();
    set_state(&conn, "theme", "dark").unwrap();
    set_state(&conn, "theme", "light").unwrap();
    let v = get_state(&conn, "theme").unwrap();
    assert_eq!(v, Some("light".to_string()));
}
