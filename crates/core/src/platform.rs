//! Per-platform paths and labels. The only place in core with `cfg(target_os)`.

use std::path::PathBuf;

/// Name of the OS the app was built for, shown as "Designed for …".
pub const OS_LABEL: &str = if cfg!(target_os = "windows") {
    "Windows"
} else if cfg!(target_os = "macos") {
    "macOS"
} else if cfg!(target_os = "ios") {
    "iOS"
} else if cfg!(target_os = "android") {
    "Android"
} else if cfg!(target_os = "linux") {
    "Linux"
} else {
    "your device"
};

/// App data directory, created if missing.
///
/// Windows `%APPDATA%\DocumentKit`, macOS `~/Library/Application Support/DocumentKit`,
/// Linux `~/.local/share/DocumentKit`.
// shortcut: Android/iOS have no `dirs::data_dir()`; falls back to the working dir
// (the app sandbox). Upgrade to the platform files-dir API when mobile ships.
pub fn data_dir() -> std::io::Result<PathBuf> {
    let dir = dirs::data_dir()
        .map(|d| d.join("DocumentKit"))
        .unwrap_or_else(|| PathBuf::from("documentkit-data"));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
