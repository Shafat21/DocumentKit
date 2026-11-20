//! PDF to Word: turn PDFs into editable documents.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Reconstructs text and layout into a .docx document.
#[derive(Debug, Default, Clone, Copy)]
pub struct PdfToWord;

impl Tool for PdfToWord {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::PDF_TO_WORD
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
