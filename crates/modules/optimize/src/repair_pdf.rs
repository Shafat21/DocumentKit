//! Repair PDF: recover damaged or corrupt PDFs.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Rebuilds the cross-reference table and salvages readable objects.
#[derive(Debug, Default, Clone, Copy)]
pub struct RepairPdf;

impl Tool for RepairPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::REPAIR_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
