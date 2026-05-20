use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[derive(Debug, Error)]
pub enum ActivityLogError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("timestamp: {0}")]
    Timestamp(#[from] time::error::Format),
}

/// Events that can be appended to a project's `.bhc/activity.log`.
/// M2 ships project_opened and settings_changed; M3/M4 extend with
/// job and run events.
#[derive(Debug, Clone)]
pub enum ActivityEvent {
    ProjectOpened    { name: String },
    SettingsChanged  { key: String, value: String },
}

impl ActivityEvent {
    fn event_type(&self) -> &'static str {
        match self {
            ActivityEvent::ProjectOpened { .. }    => "project_opened",
            ActivityEvent::SettingsChanged { .. }  => "settings_changed",
        }
    }

    fn details(&self) -> String {
        match self {
            ActivityEvent::ProjectOpened { name } => format!("name={}", quote(name)),
            ActivityEvent::SettingsChanged { key, value } => format!("key={key} value={value}"),
        }
    }
}

fn quote(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Appends an event line to the given log path. Creates the file if
/// missing. Format: `<UTC ISO 8601> | <event_type> | <details>`.
///
/// Best-effort: if the write fails, the error bubbles up. Callers may
/// choose to log-and-continue (for read-only events like project_opened)
/// or roll back the accompanying DB transaction (for state-changing
/// events).
pub fn append_event(log_path: &Path, event: ActivityEvent) -> Result<(), ActivityLogError> {
    let now = OffsetDateTime::now_utc().format(&Rfc3339)?;
    let line = format!("{} | {} | {}\n", now, event.event_type(), event.details());

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}
