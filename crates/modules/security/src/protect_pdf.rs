//! Protect PDF: add a password to your PDF.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Encrypts a PDF (AES-256) with a user password.
#[derive(Debug, Default, Clone, Copy)]
pub struct ProtectPdf;

impl Tool for ProtectPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::PROTECT_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
