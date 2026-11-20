//! PDF to PowerPoint: turn PDFs into editable slides.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Turns each page into a .pptx slide.
#[derive(Debug, Default, Clone, Copy)]
pub struct PdfToPpt;

impl Tool for PdfToPpt {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::PDF_TO_PPT
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
