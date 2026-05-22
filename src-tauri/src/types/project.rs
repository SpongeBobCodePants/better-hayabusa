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
/// `.bh/activity.log` in the project folder, ISO 8601 UTC), a
/// `folder_exists` flag (true iff `<path>/.bh/project.db` exists at
/// list time — lets the chooser branch on stale rows without an
/// extra IPC round-trip), and a `description` read from the project's
/// `project.db` at list time (None when the folder/db is gone or the
/// project has no description set).
/// Used by the chooser table only.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
pub struct RecentProjectListEntry {
    pub path: String,
    pub name: String,
    pub description: Option<String>,    // None when folder/db gone or unset
    pub last_opened_at: String,
    pub last_modified: Option<String>,  // None if activity.log missing
    pub folder_exists: bool,            // true iff `<path>/.bh/project.db` exists
}
