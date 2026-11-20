//! Add watermark: stamp text or an image on pages.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Overlays a text or image watermark on selected pages.
#[derive(Debug, Default, Clone, Copy)]
pub struct Watermark;

impl Tool for Watermark {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::WATERMARK
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
