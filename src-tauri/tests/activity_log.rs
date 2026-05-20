use bhc_lib::project::activity_log::{append_event, ActivityEvent};
use std::fs;
use tempfile::tempdir;

#[test]
fn append_event_creates_log_file_with_header_line() {
    let tmp = tempdir().unwrap();
    let log_path = tmp.path().join("activity.log");

    append_event(
        &log_path,
        ActivityEvent::ProjectOpened { name: "Test Project".to_string() },
    )
    .expect("append");

    let contents = fs::read_to_string(&log_path).unwrap();
    assert!(contents.contains("project_opened"));
    assert!(contents.contains("name=\"Test Project\""));
    // ISO 8601 UTC pattern
    assert!(contents.contains("Z |"));
}

#[test]
fn append_event_appends_to_existing_log() {
    let tmp = tempdir().unwrap();
    let log_path = tmp.path().join("activity.log");

    append_event(&log_path, ActivityEvent::ProjectOpened { name: "P1".into() }).unwrap();
    append_event(&log_path, ActivityEvent::SettingsChanged { key: "x".into(), value: "y".into() }).unwrap();

    let contents = fs::read_to_string(&log_path).unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("project_opened"));
    assert!(lines[1].contains("settings_changed"));
    assert!(lines[1].contains("key=x"));
    assert!(lines[1].contains("value=y"));
}

#[test]
fn append_event_quoting_handles_names_with_quotes() {
    let tmp = tempdir().unwrap();
    let log_path = tmp.path().join("activity.log");

    append_event(
        &log_path,
        ActivityEvent::ProjectOpened { name: "She said \"hi\"".to_string() },
    )
    .expect("append");

    let contents = fs::read_to_string(&log_path).unwrap();
    // Inner quotes escaped to \"
    assert!(contents.contains("name=\"She said \\\"hi\\\"\""));
}
