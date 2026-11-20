//! Edit module: rotate, page numbers, watermark, crop, edit, forms.
//!
//! One file per tool. See CONTRIBUTING.md → "Implementing a tool".

use std::sync::Arc;

use dk_core::Tool;

mod crop_pdf;
mod edit_pdf;
mod page_numbers;
mod pdf_forms;
mod rotate_pdf;
mod watermark;

pub use crop_pdf::CropPdf;
pub use edit_pdf::EditPdf;
pub use page_numbers::PageNumbers;
pub use pdf_forms::PdfForms;
pub use rotate_pdf::RotatePdf;
pub use watermark::Watermark;

/// Every tool in this module, in catalog order.
pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(RotatePdf),
        Arc::new(PageNumbers),
        Arc::new(Watermark),
        Arc::new(CropPdf),
        Arc::new(EditPdf),
        Arc::new(PdfForms),
    ]
}

#[cfg(test)]
mod tests {
    use dk_core::{catalog, ModuleId};

    #[test]
    fn implements_every_catalog_tool_of_this_module() {
        let ids: Vec<_> = super::tools().iter().map(|t| t.spec().id).collect();
        let expected: Vec<_> = catalog::tools_in(ModuleId::Edit).map(|t| t.id).collect();
        assert_eq!(ids, expected);
    }
}
