//! Split PDF: separate a PDF into the pages you need.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Splits one PDF into several by page ranges.
#[derive(Debug, Default, Clone, Copy)]
pub struct SplitPdf;

impl Tool for SplitPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::SPLIT_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
