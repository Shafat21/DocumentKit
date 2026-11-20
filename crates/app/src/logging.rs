//! File logging: `<data_dir>/logs/documentkit.YYYY-MM-DD.log`, 7 days kept.
//!
//! Writes happen on a background thread (non-blocking), so logging never stalls the UI.
//! Never log file contents, passwords or other secrets.

use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;

/// Install the global logger. Returns `None` (logging disabled) if the log dir is unusable.
pub fn init(data_dir: &Path) -> Option<WorkerGuard> {
    // The appender prunes old files at startup and errors if the folder doesn't exist yet.
    let log_dir = data_dir.join("logs");
    std::fs::create_dir_all(&log_dir).ok()?;
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("documentkit")
        .filename_suffix("log")
        .max_log_files(7)
        .build(log_dir)
        .ok()?;
    let (writer, guard) = tracing_appender::non_blocking(appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false)
        .try_init()
        .ok()?;
    Some(guard)
}
