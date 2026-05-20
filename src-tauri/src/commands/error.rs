use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Typed error returned from every Tauri command. Frontend discriminates
/// on `kind` to decide UI behavior (schema-mismatch screen vs. inline
/// alert vs. toast).
///
/// Add new variants here when a command needs to surface a structured
/// failure case to the UI — never funnel real errors through `Internal`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "../../src/lib/generated/")]
pub enum CommandError {
    NotFound        { path: String },
    AlreadyExists   { path: String },
    NotAProject     { path: String },
    SchemaTooNew    { project_version: u32, app_version: u32 },
    Io              { message: String },
    Db              { message: String },
    Internal        { message: String },
}

impl From<rusqlite::Error> for CommandError {
    fn from(e: rusqlite::Error) -> Self {
        CommandError::Db { message: e.to_string() }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(e: std::io::Error) -> Self {
        CommandError::Io { message: e.to_string() }
    }
}

impl<T> From<std::sync::PoisonError<T>> for CommandError {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        CommandError::Internal { message: format!("mutex poisoned: {e}") }
    }
}
