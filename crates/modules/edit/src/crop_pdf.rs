//! Crop PDF: trim margins and page areas.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Sets a new crop box on selected pages.
#[derive(Debug, Default, Clone, Copy)]
pub struct CropPdf;

impl Tool for CropPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::CROP_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
