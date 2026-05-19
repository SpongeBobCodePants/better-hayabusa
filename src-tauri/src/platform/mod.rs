use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export, export_to = "../../src/lib/generated/")]
pub enum Os {
    Windows,
    Macos,
    Linux,
    Other,
}

pub fn current_os() -> Os {
    if cfg!(target_os = "windows") {
        Os::Windows
    } else if cfg!(target_os = "macos") {
        Os::Macos
    } else if cfg!(target_os = "linux") {
        Os::Linux
    } else {
        Os::Other
    }
}
