use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderState {
    /// Folder is empty, or has unrelated files but no `.bhc/project.db`.
    /// Safe target for `create_project`.
    Eligible,
    /// Folder already has `.bhc/project.db`. `create_project` would error;
    /// caller should offer "Open it instead?" UX.
    ExistingProject,
}

#[derive(Debug, thiserror::Error)]
pub enum ConflictCheckError {
    #[error("folder does not exist: {0}")]
    NotFound(String),
    #[error("path is not a directory: {0}")]
    NotADirectory(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Inspects a folder to decide whether `create_project` would conflict
/// with an existing project. Does NOT decide eligibility for any other
/// reason (write permissions, disk space, etc.) — those surface as I/O
/// errors at create time.
pub fn check_folder(folder: &Path) -> Result<FolderState, ConflictCheckError> {
    if !folder.exists() {
        return Err(ConflictCheckError::NotFound(folder.display().to_string()));
    }
    if !folder.is_dir() {
        return Err(ConflictCheckError::NotADirectory(folder.display().to_string()));
    }

    let project_db = folder.join(".bhc").join("project.db");
    if project_db.exists() {
        Ok(FolderState::ExistingProject)
    } else {
        Ok(FolderState::Eligible)
    }
}
