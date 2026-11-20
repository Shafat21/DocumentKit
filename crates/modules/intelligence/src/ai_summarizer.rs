//! AI Summarizer: find the key takeaways, faster.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Produces a short summary of a PDF's text.
#[derive(Debug, Default, Clone, Copy)]
pub struct AiSummarizer;

impl Tool for AiSummarizer {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::AI_SUMMARIZER
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
