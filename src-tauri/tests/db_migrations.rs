use bhc_lib::db::migrations::{run_migrations, APP_MIGRATIONS};
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn run_app_migrations_creates_expected_tables() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("app.db");
    let conn = Connection::open(&db_path).unwrap();

    run_migrations(&conn, APP_MIGRATIONS).expect("run migrations");

    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();

    assert!(tables.contains(&"app_state".to_string()));
    assert!(tables.contains(&"recent_projects".to_string()));
    assert!(tables.contains(&"global_tools".to_string()));
    assert!(tables.contains(&"_migrations".to_string()));
}

#[test]
fn run_app_migrations_is_idempotent() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("app.db");
    let conn = Connection::open(&db_path).unwrap();

    run_migrations(&conn, APP_MIGRATIONS).expect("first run");
    run_migrations(&conn, APP_MIGRATIONS).expect("second run is a no-op");

    let applied: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name = '001_init'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(applied, 1, "001_init should be recorded exactly once");
}

#[test]
fn schema_version_is_set_by_001_init() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("app.db");
    let conn = Connection::open(&db_path).unwrap();

    run_migrations(&conn, APP_MIGRATIONS).expect("run migrations");

    let version: String = conn
        .query_row(
            "SELECT value FROM app_state WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, "1");
}

#[test]
fn open_or_create_works_on_a_fresh_path() {
    use bhc_lib::db::app_db::open_or_create;

    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("app.db");
    assert!(!db_path.exists());

    let conn = open_or_create(&db_path).expect("open or create");
    assert!(db_path.exists());

    // Foreign keys should be on.
    let fk: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0)).unwrap();
    assert_eq!(fk, 1);
}
