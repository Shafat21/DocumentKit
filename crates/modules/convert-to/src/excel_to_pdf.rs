//! Excel to PDF: turn spreadsheets into PDFs.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Renders .xlsx sheets to PDF pages.
#[derive(Debug, Default, Clone, Copy)]
pub struct ExcelToPdf;

impl Tool for ExcelToPdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::EXCEL_TO_PDF
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
