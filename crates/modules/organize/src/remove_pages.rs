//! Remove pages: delete the pages you don't want.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Writes a copy of the PDF without the selected pages.
#[derive(Debug, Default, Clone, Copy)]
pub struct RemovePages;

impl Tool for RemovePages {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::REMOVE_PAGES
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
