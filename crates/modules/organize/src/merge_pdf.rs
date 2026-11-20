//! Merge PDF: combine multiple PDFs into one.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Combines `job.inputs`, in order, into one PDF.
#[derive(Debug, Default, Clone, Copy)]
pub struct MergePdf;

impl Tool for MergePdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::MERGE_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
