//! Translate PDF: read your PDF in another language.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Translates a PDF's text into a target language, keeping layout where possible.
#[derive(Debug, Default, Clone, Copy)]
pub struct TranslatePdf;

impl Tool for TranslatePdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::TRANSLATE_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
