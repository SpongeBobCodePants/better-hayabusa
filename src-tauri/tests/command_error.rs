use bhc_lib::commands::error::CommandError;

#[test]
fn not_found_round_trips_through_json() {
    let e = CommandError::NotFound { path: "C:\\nope".to_string() };
    let json = serde_json::to_string(&e).unwrap();
    assert!(json.contains("\"kind\":\"NotFound\""));
    assert!(json.contains("C:\\\\nope"));
    let parsed: CommandError = serde_json::from_str(&json).unwrap();
    matches!(parsed, CommandError::NotFound { .. });
}

#[test]
fn schema_too_new_includes_versions() {
    let e = CommandError::SchemaTooNew { project_version: 4, app_version: 3 };
    let json = serde_json::to_string(&e).unwrap();
    assert!(json.contains("\"project_version\":4"));
    assert!(json.contains("\"app_version\":3"));
}

#[test]
fn all_variants_serialize_with_kind_tag() {
    let cases = vec![
        CommandError::NotFound { path: "p".into() },
        CommandError::AlreadyExists { path: "p".into() },
        CommandError::NotAProject { path: "p".into() },
        CommandError::SchemaTooNew { project_version: 2, app_version: 1 },
        CommandError::Io { message: "m".into() },
        CommandError::Db { message: "m".into() },
        CommandError::Internal { message: "m".into() },
    ];
    for e in cases {
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"kind\":"), "missing kind tag in {json}");
    }
}
