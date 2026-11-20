//! PDF to Markdown: clean Markdown from any PDF.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Extracts headings, paragraphs, lists and tables into Markdown.
#[derive(Debug, Default, Clone, Copy)]
pub struct PdfToMarkdown;

impl Tool for PdfToMarkdown {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::PDF_TO_MARKDOWN
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
