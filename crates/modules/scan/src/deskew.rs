//! Auto crop & deskew: straighten and trim scanned pages.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Detects page edges and skew angle, then crops and rotates to fix them.
#[derive(Debug, Default, Clone, Copy)]
pub struct Deskew;

impl Tool for Deskew {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::DESKEW
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
