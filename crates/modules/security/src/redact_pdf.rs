//! Redact PDF: permanently black out sensitive text.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Removes the underlying content (not just covers it) in selected areas.
#[derive(Debug, Default, Clone, Copy)]
pub struct RedactPdf;

impl Tool for RedactPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::REDACT_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
