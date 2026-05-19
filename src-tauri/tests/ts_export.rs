//! Not a real test — a codegen trigger. Calling `export_all()` writes TS
//! definitions to `../../src/lib/generated/` (see `#[ts(export_to=...)]` on
//! each exported type). Must run before `pnpm build` so the frontend's type
//! imports resolve. Generated files are gitignored.

use bhc_lib::platform::Os;
use bhc_lib::types::AppVersion;
use ts_rs::TS;

#[test]
fn export_ts_types() {
    AppVersion::export_all().expect("export AppVersion");
    Os::export_all().expect("export Os");
}
