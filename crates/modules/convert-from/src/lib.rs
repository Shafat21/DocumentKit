//! Convert-from-PDF module: PDF to JPG, Word, PowerPoint, Excel, PDF/A.
//!
//! One file per tool. See CONTRIBUTING.md → "Implementing a tool".

use std::sync::Arc;

use dk_core::Tool;

mod pdf_to_excel;
mod pdf_to_jpg;
mod pdf_to_pdfa;
mod pdf_to_ppt;
mod pdf_to_word;

pub use pdf_to_excel::PdfToExcel;
pub use pdf_to_jpg::PdfToJpg;
pub use pdf_to_pdfa::PdfToPdfa;
pub use pdf_to_ppt::PdfToPpt;
pub use pdf_to_word::PdfToWord;

/// Every tool in this module, in catalog order.
pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(PdfToJpg),
        Arc::new(PdfToWord),
        Arc::new(PdfToPpt),
        Arc::new(PdfToExcel),
        Arc::new(PdfToPdfa),
    ]
}

#[cfg(test)]
mod tests {
    use dk_core::{catalog, ModuleId};

    #[test]
    fn implements_every_catalog_tool_of_this_module() {
        let ids: Vec<_> = super::tools().iter().map(|t| t.spec().id).collect();
        let expected: Vec<_> = catalog::tools_in(ModuleId::ConvertFrom).map(|t| t.id).collect();
        assert_eq!(ids, expected);
    }
}
