use bhc_lib::types::AppVersion;

#[test]
fn app_version_shape_round_trips_json() {
    let v = AppVersion { version: "0.1.0".to_string() };
    let json = serde_json::to_string(&v).unwrap();
    let parsed: AppVersion = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.version, "0.1.0");
}
