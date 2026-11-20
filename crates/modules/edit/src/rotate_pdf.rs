//! Rotate PDF: turn pages the right way up.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Rotates selected pages by 90° steps.
#[derive(Debug, Default, Clone, Copy)]
pub struct RotatePdf;

impl Tool for RotatePdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::ROTATE_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
