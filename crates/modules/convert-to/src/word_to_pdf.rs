//! Word to PDF: turn Word documents into PDFs.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Renders .docx documents to PDF.
#[derive(Debug, Default, Clone, Copy)]
pub struct WordToPdf;

impl Tool for WordToPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::WORD_TO_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
