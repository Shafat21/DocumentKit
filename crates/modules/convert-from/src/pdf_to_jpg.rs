//! PDF to JPG: export pages as images.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Rasterises each page to a JPG file.
#[derive(Debug, Default, Clone, Copy)]
pub struct PdfToJpg;

impl Tool for PdfToJpg {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::PDF_TO_JPG
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
