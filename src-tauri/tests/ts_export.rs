//! Not a real test — a codegen trigger. Calling `export_all()` writes TS
//! definitions to `../../src/lib/generated/` (see `#[ts(export_to=...)]` on
//! each exported type). Must run before `pnpm build` so the frontend's type
//! imports resolve. Generated files are gitignored.

use bhc_lib::commands::error::CommandError;
use bhc_lib::platform::Os;
use bhc_lib::types::{
    AppVersion, LaunchResult, Project, ProjectInfo, RecentProject, RecentProjectListEntry,
};
use ts_rs::TS;

#[test]
fn export_ts_types() {
    AppVersion::export_all().expect("export AppVersion");
    Os::export_all().expect("export Os");
    Project::export_all().expect("export Project");
    ProjectInfo::export_all().expect("export ProjectInfo");
    RecentProject::export_all().expect("export RecentProject");
    RecentProjectListEntry::export_all().expect("export RecentProjectListEntry");
    LaunchResult::export_all().expect("export LaunchResult");
    CommandError::export_all().expect("export CommandError");
}
