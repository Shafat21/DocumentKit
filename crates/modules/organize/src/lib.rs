//! Organize module: merge, split, remove pages, extract pages, organize, scan to PDF.
//!
//! One file per tool. See CONTRIBUTING.md → "Implementing a tool".

use std::sync::Arc;

use dk_core::Tool;

mod extract_pages;
mod merge_pdf;
mod organize_pdf;
pub mod pdf;
pub mod preview;
mod remove_pages;
mod scan_to_pdf;
mod split_pdf;
mod toc;

pub use extract_pages::ExtractPages;
pub use merge_pdf::MergePdf;
pub use organize_pdf::OrganizePdf;
pub use remove_pages::RemovePages;
pub use scan_to_pdf::ScanToPdf;
pub use split_pdf::SplitPdf;

/// Every tool in this module, in catalog order.
pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(MergePdf),
        Arc::new(SplitPdf),
        Arc::new(RemovePages),
        Arc::new(ExtractPages),
        Arc::new(OrganizePdf),
        Arc::new(ScanToPdf),
    ]
}

#[cfg(test)]
mod tests {
    use dk_core::{catalog, ModuleId};

    #[test]
    fn implements_every_catalog_tool_of_this_module() {
        let ids: Vec<_> = super::tools().iter().map(|t| t.spec().id).collect();
        let expected: Vec<_> = catalog::tools_in(ModuleId::Organize)
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, expected);
    }
}
