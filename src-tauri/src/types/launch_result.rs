use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::types::ProjectInfo;

/// Result of `check_last_open_project()` at app boot. Drives whether
/// the frontend lands on the dashboard, on Home, or on the sticky-fail
/// takeover screen.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "../../src/lib/generated/")]
pub enum LaunchResult {
    /// Sticky session succeeded; this project was loaded into AppState
    /// and the frontend should route to `/projects/current`.
    Loaded { info: ProjectInfo },

    /// Sticky session failed; frontend shows the sticky-fail takeover.
    /// `reason` is a human-readable description; `path` and `name` are
    /// from the dead recent_projects entry (which has been removed).
    Failed {
        path: String,
        name: String,
        reason: String,
    },

    /// Manual open found the project folder or .bh/project.db missing.
    /// The recents row was NOT cleaned — the frontend prompts the user
    /// before removing it.
    Missing {
        path: String,
        name: String,
        reason: String,
    },

    /// `app_schema_version` of the project is newer than the app's.
    /// Frontend shows the schema-mismatch takeover screen.
    SchemaTooNew {
        path: String,
        name: String,
        project_version: u32,
        app_version: u32,
    },

    /// No `last_open_project_path` is set in app.db. Frontend lands on Home.
    NoneSet,

    /// `launch_behavior` setting is `"home_page"`. Frontend lands on Home.
    Disabled,
}
