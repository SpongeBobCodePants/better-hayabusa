use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One row from `projects` in project.db. Single row per DB,
/// constrained at the application layer.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,             // UTC ISO 8601
    pub app_schema_version: u32,
}

/// What the frontend gets back when a project is opened — `Project` plus
/// the resolved folder path so the UI can display it without re-querying.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
pub struct ProjectInfo {
    pub project: Project,
    pub folder_path: String,            // absolute path the user picked
}

/// One row from `recent_projects` in app.db.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub last_opened_at: String,         // UTC ISO 8601
}

/// Same as RecentProject plus a computed `last_modified` (mtime of
/// `.bhc/activity.log` in the project folder, ISO 8601 UTC).
/// Used by the chooser table only.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
pub struct RecentProjectListEntry {
    pub path: String,
    pub name: String,
    pub last_opened_at: String,
    pub last_modified: Option<String>,  // None if activity.log missing
}
