//! Edit PDF: add text, images and annotations.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Applies a list of edits (text, images, annotations) to a PDF.
#[derive(Debug, Default, Clone, Copy)]
pub struct EditPdf;

impl Tool for EditPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::EDIT_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
