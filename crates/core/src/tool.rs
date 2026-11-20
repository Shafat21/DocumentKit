//! The contract every DocumentKit tool implements.
//!
//! Tools are plain, blocking Rust. The app runs them on a worker thread, so a
//! tool never touches async code or the UI. Rules for implementors:
//! - stream files from `job.inputs`; never read a whole large file into memory,
//! - call `progress.report` per page (the app throttles UI updates),
//! - check `job.is_cancelled()` per page and return `Error::Cancelled`,
//! - write only into `job.output_dir`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::Result;

/// Feature module a tool belongs to. One crate per module under `crates/modules/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleId {
    /// Merge, split, reorder pages.
    Organize,
    /// Compress, repair, OCR.
    Optimize,
    /// Other formats into PDF.
    ConvertTo,
    /// PDF into other formats.
    ConvertFrom,
    /// Rotate, watermark, crop, edit, forms.
    Edit,
    /// Passwords, signatures, redaction, compare.
    Security,
    /// AI summary, translation, Markdown export.
    Intelligence,
    /// Camera / scanner capture.
    Scan,
}

impl ModuleId {
    /// All modules in display order.
    pub const ALL: [ModuleId; 8] = [
        ModuleId::Organize,
        ModuleId::Optimize,
        ModuleId::ConvertTo,
        ModuleId::ConvertFrom,
        ModuleId::Edit,
        ModuleId::Security,
        ModuleId::Intelligence,
        ModuleId::Scan,
    ];

    /// Stable lowercase key, used in routes and storage.
    pub const fn key(self) -> &'static str {
        match self {
            ModuleId::Organize => "organize",
            ModuleId::Optimize => "optimize",
            ModuleId::ConvertTo => "convert-to",
            ModuleId::ConvertFrom => "convert-from",
            ModuleId::Edit => "edit",
            ModuleId::Security => "security",
            ModuleId::Intelligence => "intelligence",
            ModuleId::Scan => "scan",
        }
    }

    /// Human-readable name.
    pub const fn name(self) -> &'static str {
        match self {
            ModuleId::Organize => "Organize",
            ModuleId::Optimize => "Optimize",
            ModuleId::ConvertTo => "Convert to PDF",
            ModuleId::ConvertFrom => "Convert from PDF",
            ModuleId::Edit => "Edit",
            ModuleId::Security => "Security",
            ModuleId::Intelligence => "Intelligence",
            ModuleId::Scan => "Scan",
        }
    }

    /// Lucide icon name (kebab-case) for this module.
    pub const fn icon(self) -> &'static str {
        match self {
            ModuleId::Organize => "layers",
            ModuleId::Optimize => "gauge",
            ModuleId::ConvertTo => "file-input",
            ModuleId::ConvertFrom => "file-output",
            ModuleId::Edit => "square-pen",
            ModuleId::Security => "shield",
            ModuleId::Intelligence => "sparkles",
            ModuleId::Scan => "scan-line",
        }
    }
}

/// Static description of a tool. Lives in [`crate::catalog`]; one per tool.
#[derive(Debug, PartialEq, Eq)]
pub struct ToolSpec {
    /// Stable kebab-case id, e.g. `merge-pdf`.
    pub id: &'static str,
    /// Display name.
    pub name: &'static str,
    /// One-line description shown on the tool card.
    pub desc: &'static str,
    /// Owning module.
    pub module: ModuleId,
    /// Lucide icon name (kebab-case).
    pub icon: &'static str,
}

/// One unit of work handed to a tool.
#[derive(Debug, Clone)]
pub struct Job {
    /// Input files, in user-chosen order.
    pub inputs: Vec<PathBuf>,
    /// Directory the tool writes its results into.
    pub output_dir: PathBuf,
    /// Tool-specific options (page ranges, quality, ...). Each tool documents its keys.
    pub options: serde_json::Value,
    cancel: Arc<AtomicBool>,
}

impl Job {
    /// New job with no options.
    pub fn new(inputs: Vec<PathBuf>, output_dir: PathBuf) -> Self {
        Self {
            inputs,
            output_dir,
            options: serde_json::Value::Null,
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Handle the UI keeps to cancel this job later.
    pub fn cancel_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel)
    }

    /// True once the user pressed Cancel. Tools check this per page.
    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
}

/// Files a tool produced.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JobOutput {
    /// Paths of written files, inside `Job::output_dir`.
    pub files: Vec<PathBuf>,
}

/// Progress sink a tool reports into. Implementations must be cheap and non-blocking.
pub trait Progress: Send + Sync {
    /// `pct` is 0..=100. `msg` is a short status like "Page 3 of 40".
    fn report(&self, pct: u8, msg: &str);
}

/// Progress sink that ignores everything (tests, CLI).
#[derive(Debug, Clone, Copy, Default)]
pub struct NoProgress;

impl Progress for NoProgress {
    fn report(&self, _pct: u8, _msg: &str) {}
}

/// A DocumentKit tool. One struct per tool, one file per struct.
pub trait Tool: Send + Sync {
    /// Static catalog entry for this tool.
    fn spec(&self) -> &'static ToolSpec;

    /// Run the job to completion on the current (worker) thread.
    fn run(&self, job: Job, progress: &dyn Progress) -> Result<JobOutput>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_keys_are_unique() {
        let mut keys: Vec<_> = ModuleId::ALL.iter().map(|m| m.key()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), ModuleId::ALL.len());
    }

    #[test]
    fn cancel_handle_flips_job() {
        let job = Job::new(vec![], PathBuf::from("out"));
        assert!(!job.is_cancelled());
        job.cancel_handle().store(true, Ordering::Relaxed);
        assert!(job.is_cancelled());
    }
}
