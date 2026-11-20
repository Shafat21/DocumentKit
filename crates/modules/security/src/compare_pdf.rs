//! Compare PDF: spot the differences between two PDFs.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Diffs text and visuals of two PDFs and writes a highlighted report.
#[derive(Debug, Default, Clone, Copy)]
pub struct ComparePdf;

impl Tool for ComparePdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::COMPARE_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
