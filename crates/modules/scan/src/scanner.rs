//! Document scanner: capture documents with a scanner or camera.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Captures pages from a connected scanner (desktop) or the camera (mobile).
#[derive(Debug, Default, Clone, Copy)]
pub struct Scanner;

impl Tool for Scanner {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::SCANNER
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
