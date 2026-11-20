//! PDF forms: fill in and create PDF forms.

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};

/// Fills AcroForm fields or adds new ones.
#[derive(Debug, Default, Clone, Copy)]
pub struct PdfForms;

impl Tool for PdfForms {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::PDF_FORMS
    }

    fn run(&self, _job: Job, _progress: &dyn Progress) -> Result<JobOutput> {
        Err(Error::NotImplemented(self.spec().id))
    }
}
