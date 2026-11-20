//! UI for each feature module: one folder per module, one file per tool page.
//!
//! Engine code lives in `crates/modules/<module>`; this side only renders
//! options, starts the job via `jobs::runner::start`, and shows progress.
//! Convention: `features/<module>/<tool>.rs` exports a `#[component]`, and
//! `router.rs` gets a route for it. See CONTRIBUTING.md.

pub mod convert_from;
pub mod convert_to;
pub mod edit;
pub mod intelligence;
pub mod optimize;
pub mod organize;
pub mod scan;
pub mod security;
