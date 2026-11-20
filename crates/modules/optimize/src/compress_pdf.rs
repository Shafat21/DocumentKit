//! Compress PDF: smaller files, the same great quality.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Re-encodes images and streams to reduce file size.
#[derive(Debug, Default, Clone, Copy)]
pub struct CompressPdf;

impl Tool for CompressPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::COMPRESS_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
