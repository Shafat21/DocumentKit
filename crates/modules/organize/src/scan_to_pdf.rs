//! Scan to PDF: turn paper documents into PDFs.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Builds a PDF from scanned page images.
#[derive(Debug, Default, Clone, Copy)]
pub struct ScanToPdf;

impl Tool for ScanToPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::SCAN_TO_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
