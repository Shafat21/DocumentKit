//! PDF to PDF/A: archive-ready PDFs for the long term.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Converts a PDF to a PDF/A-compliant file (fonts embedded, no external refs).
#[derive(Debug, Default, Clone, Copy)]
pub struct PdfToPdfa;

impl Tool for PdfToPdfa {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::PDF_TO_PDFA
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
