//! Sign PDF: add your signature in a few clicks.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Places a visual signature and, optionally, a digital signature.
#[derive(Debug, Default, Clone, Copy)]
pub struct SignPdf;

impl Tool for SignPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::SIGN_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
