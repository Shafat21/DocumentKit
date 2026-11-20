//! HTML to PDF: save web pages as PDFs.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Renders local HTML files to PDF.
#[derive(Debug, Default, Clone, Copy)]
pub struct HtmlToPdf;

impl Tool for HtmlToPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::HTML_TO_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
