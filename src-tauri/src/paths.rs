use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PathError {
    #[error("executable path has no parent directory: {0}")]
    NoParent(PathBuf),
    #[error("could not resolve current executable: {0}")]
    CurrentExe(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub exe_dir: PathBuf,
    pub app_db: PathBuf,
    pub tools_dir: PathBuf,
}

impl AppPaths {
    pub fn from_exe(exe: &Path) -> Result<Self, PathError> {
        let exe_dir = exe
            .parent()
            .ok_or_else(|| PathError::NoParent(exe.to_path_buf()))?
            .to_path_buf();

        Ok(Self {
            app_db: exe_dir.join("app.db"),
            tools_dir: exe_dir.join("tools"),
            exe_dir,
        })
    }

    pub fn from_current_exe() -> Result<Self, PathError> {
        let exe = std::env::current_exe()?;
        Self::from_exe(&exe)
    }
}
