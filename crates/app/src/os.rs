//! Small OS integrations.

use std::path::Path;

/// Show a file in the system file manager (selected where the OS supports it).
pub fn reveal(path: &Path) {
    if let Err(e) = reveal_impl(path) {
        tracing::warn!("could not open the file manager: {e}");
    }
}

#[cfg(target_os = "windows")]
fn reveal_impl(path: &Path) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    // Explorer wants `/select,"<path>"` verbatim. Windows paths can't contain `"`.
    std::process::Command::new("explorer")
        .raw_arg(format!("/select,\"{}\"", path.display()))
        .spawn()
        .map(drop)
}

#[cfg(target_os = "macos")]
fn reveal_impl(path: &Path) -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn()
        .map(drop)
}

#[cfg(target_os = "linux")]
fn reveal_impl(path: &Path) -> std::io::Result<()> {
    std::process::Command::new("xdg-open")
        .arg(path.parent().unwrap_or(path))
        .spawn()
        .map(drop)
}

// shortcut: no file manager hand-off on mobile yet.
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn reveal_impl(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Open a file with the system's default app.
pub fn open(path: &Path) {
    if let Err(e) = open_impl(path) {
        tracing::warn!("could not open file: {e}");
    }
}

#[cfg(target_os = "windows")]
fn open_impl(path: &Path) -> std::io::Result<()> {
    // explorer hands the file to its registered app without opening a console window.
    std::process::Command::new("explorer")
        .arg(path)
        .spawn()
        .map(drop)
}

#[cfg(target_os = "macos")]
fn open_impl(path: &Path) -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg(path)
        .spawn()
        .map(drop)
}

#[cfg(target_os = "linux")]
fn open_impl(path: &Path) -> std::io::Result<()> {
    std::process::Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(drop)
}

// shortcut: no default-app hand-off on mobile yet.
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn open_impl(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Folders to look for the pdfium library in: next to the app, then (dev builds)
/// `crates/app/assets/pdfium/`.
pub fn pdfium_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs: Vec<_> = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .into_iter()
        .collect();
    if cfg!(debug_assertions) {
        dirs.push(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/pdfium").into());
    }
    dirs
}
