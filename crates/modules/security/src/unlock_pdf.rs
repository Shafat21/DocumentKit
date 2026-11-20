//! Unlock PDF: remove a password you know.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Decrypts a PDF with the user-supplied password and saves an unlocked copy.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnlockPdf;

impl Tool for UnlockPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::UNLOCK_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
