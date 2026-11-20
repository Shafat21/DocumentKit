//! JPG to PDF: bring your images together in a PDF.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Places each input image on its own PDF page.
#[derive(Debug, Default, Clone, Copy)]
pub struct JpgToPdf;

impl Tool for JpgToPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::JPG_TO_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
