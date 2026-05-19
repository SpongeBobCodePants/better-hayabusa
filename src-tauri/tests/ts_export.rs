use bhc_lib::platform::Os;
use bhc_lib::types::AppVersion;
use ts_rs::TS;

#[test]
fn export_ts_types() {
    AppVersion::export_all().expect("export AppVersion");
    Os::export_all().expect("export Os");
}
