//! PowerPoint to PDF: turn slide decks into PDFs.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Renders .pptx slides to PDF pages.
#[derive(Debug, Default, Clone, Copy)]
pub struct PptToPdf;

impl Tool for PptToPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::PPT_TO_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
