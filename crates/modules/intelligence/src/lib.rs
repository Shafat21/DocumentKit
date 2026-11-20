//! Intelligence module: AI summarizer, translate, PDF to Markdown.
//!
//! Privacy model (local model vs opt-in cloud) is undecided; write an ADR in
//! docs/adr/ before adding any network or model code here.
//!
//! One file per tool. See CONTRIBUTING.md → "Implementing a tool".

use std::sync::Arc;

use dk_core::Tool;

mod ai_summarizer;
mod pdf_to_markdown;
mod translate_pdf;

pub use ai_summarizer::AiSummarizer;
pub use pdf_to_markdown::PdfToMarkdown;
pub use translate_pdf::TranslatePdf;

/// Every tool in this module, in catalog order.
pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(AiSummarizer),
        Arc::new(TranslatePdf),
        Arc::new(PdfToMarkdown),
    ]
}

#[cfg(test)]
mod tests {
    use dk_core::{catalog, ModuleId};

    #[test]
    fn implements_every_catalog_tool_of_this_module() {
        let ids: Vec<_> = super::tools().iter().map(|t| t.spec().id).collect();
        let expected: Vec<_> = catalog::tools_in(ModuleId::Intelligence).map(|t| t.id).collect();
        assert_eq!(ids, expected);
    }
}
