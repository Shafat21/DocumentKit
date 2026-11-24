//! Optimize module: compress, repair, OCR.
//!
//! One file per tool. See CONTRIBUTING.md → "Implementing a tool".

use std::sync::Arc;

use dk_core::Tool;

mod compress_pdf;
mod ocr_pdf;
mod repair_pdf;

pub use compress_pdf::CompressPdf;
pub use ocr_pdf::OcrPdf;
pub use repair_pdf::RepairPdf;

/// Every tool in this module, in catalog order.
pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(CompressPdf), Arc::new(RepairPdf), Arc::new(OcrPdf)]
}

#[cfg(test)]
mod tests {
    use dk_core::{catalog, ModuleId};

    #[test]
    fn implements_every_catalog_tool_of_this_module() {
        let ids: Vec<_> = super::tools().iter().map(|t| t.spec().id).collect();
        let expected: Vec<_> = catalog::tools_in(ModuleId::Optimize)
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, expected);
    }
}
