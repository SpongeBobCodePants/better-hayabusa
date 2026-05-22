//! Project-name validation. Centralized so the frontend mirror in
//! `src/lib/helpers/validateProjectName.ts` can stay in sync with this
//! logic.

/// Returns `Ok(())` if `name` is a valid project name on Windows,
/// or `Err(reason)` with a human-readable explanation.
///
/// Validates the **trimmed** name. Callers pass the user's raw input;
/// this function trims internally.
pub fn validate_project_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Name cannot be empty.".to_string());
    }
    if trimmed.chars().count() > 100 {
        return Err("Name is too long (max 100 characters).".to_string());
    }
    const FORBIDDEN: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    for c in trimmed.chars() {
        if FORBIDDEN.contains(&c) {
            return Err(format!("Name cannot contain '{c}'."));
        }
        if (c as u32) < 0x20 {
            return Err("Name cannot contain control characters.".to_string());
        }
    }
    if trimmed.ends_with('.') || trimmed.ends_with(' ') {
        return Err("Name cannot end with a dot or space.".to_string());
    }
    const RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let base = trimmed.split('.').next().unwrap_or(trimmed).to_uppercase();
    if RESERVED.contains(&base.as_str()) {
        return Err(format!("'{trimmed}' is a Windows-reserved name."));
    }
    Ok(())
}

/// Returns `Ok(())` if `desc` is None or its length is at most 250
/// characters; `Err(reason)` otherwise.
pub fn validate_project_description(desc: Option<&str>) -> Result<(), String> {
    if let Some(d) = desc {
        if d.chars().count() > 250 {
            return Err("Description is too long (max 250 characters).".to_string());
        }
    }
    Ok(())
}
