//! Add page numbers: number your pages in any style.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Stamps page numbers at a chosen position and format.
#[derive(Debug, Default, Clone, Copy)]
pub struct PageNumbers;

impl Tool for PageNumbers {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::PAGE_NUMBERS
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
