//! DocumentKit core.
//!
//! Everything shared by the app and the feature modules, with no UI dependency:
//! - [`tool`]: the [`Tool`] contract every feature implements.
//! - [`catalog`]: static list of modules and tools shown in the UI.
//! - [`store`]: local SQLite persistence (settings, recent documents).
//! - [`theme`]: light / dark / system theme preference.
//! - [`platform`]: per-OS paths and labels.
//! - [`roadmap`]: development roadmap parsed from `roadmap.json`.
#![warn(missing_docs)]

pub mod catalog;
pub mod error;
pub mod platform;
pub mod roadmap;
#[cfg(feature = "sqlite")]
pub mod store;
pub mod theme;
pub mod tool;

pub use error::{Error, Result};
pub use theme::Theme;
pub use tool::{Job, JobOutput, ModuleId, NoProgress, Progress, Tool, ToolSpec};
