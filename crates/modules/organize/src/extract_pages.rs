//! Extract pages: pull selected pages into a new PDF.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Writes a new PDF containing only the selected pages.
#[derive(Debug, Default, Clone, Copy)]
pub struct ExtractPages;

impl Tool for ExtractPages {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::EXTRACT_PAGES
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
