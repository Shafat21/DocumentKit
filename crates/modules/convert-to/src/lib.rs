//! Convert-to-PDF module: JPG, Word, PowerPoint, Excel, HTML to PDF.
//!
//! One file per tool. See CONTRIBUTING.md → "Implementing a tool".

use std::sync::Arc;

use dk_core::Tool;

mod excel_to_pdf;
mod html_to_pdf;
mod jpg_to_pdf;
mod ppt_to_pdf;
mod word_to_pdf;

pub use excel_to_pdf::ExcelToPdf;
pub use html_to_pdf::HtmlToPdf;
pub use jpg_to_pdf::JpgToPdf;
pub use ppt_to_pdf::PptToPdf;
pub use word_to_pdf::WordToPdf;

/// Every tool in this module, in catalog order.
pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(JpgToPdf),
        Arc::new(WordToPdf),
        Arc::new(PptToPdf),
        Arc::new(ExcelToPdf),
        Arc::new(HtmlToPdf),
    ]
}

#[cfg(test)]
mod tests {
    use dk_core::{catalog, ModuleId};

    #[test]
    fn implements_every_catalog_tool_of_this_module() {
        let ids: Vec<_> = super::tools().iter().map(|t| t.spec().id).collect();
        let expected: Vec<_> = catalog::tools_in(ModuleId::ConvertTo)
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, expected);
    }
}
