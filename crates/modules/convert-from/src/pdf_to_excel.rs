//! PDF to Excel: pull tables into spreadsheets.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Detects tables and writes them to an .xlsx workbook.
#[derive(Debug, Default, Clone, Copy)]
pub struct PdfToExcel;

impl Tool for PdfToExcel {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::PDF_TO_EXCEL
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
