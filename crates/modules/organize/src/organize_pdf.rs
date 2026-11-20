//! Organize PDF: reorder, rotate and arrange pages.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Applies a new page order (and per-page rotation) to a PDF.
#[derive(Debug, Default, Clone, Copy)]
pub struct OrganizePdf;

impl Tool for OrganizePdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::ORGANIZE_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
