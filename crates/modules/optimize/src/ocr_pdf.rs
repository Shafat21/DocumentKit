//! OCR PDF: make scanned text searchable.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Adds an invisible text layer recognised from page images.
#[derive(Debug, Default, Clone, Copy)]
pub struct OcrPdf;

impl Tool for OcrPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::OCR_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
