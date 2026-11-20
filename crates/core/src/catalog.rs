//! Static catalog of every tool DocumentKit offers.
//!
//! The UI lists tools from here even before their engine exists. Each tool's
//! implementation (in its module crate) points at its `ToolSpec` by constant, so
//! a typo is a compile error, not a runtime lookup failure.
//!
//! Adding a tool: add one line to the `catalog!` block, then add the module file
//! (see CONTRIBUTING.md).

use crate::tool::{ModuleId, ToolSpec};

macro_rules! catalog {
    ($($konst:ident = $id:literal, $name:literal, $module:ident, $icon:literal, $desc:literal;)*) => {
        $(
            #[doc = $name]
            pub static $konst: ToolSpec = ToolSpec {
                id: $id,
                name: $name,
                desc: $desc,
                module: ModuleId::$module,
                icon: $icon,
            };
        )*

        /// Every tool, grouped by module, in display order.
        pub static TOOLS: &[&ToolSpec] = &[$(&$konst),*];
    };
}

catalog! {
    // Organize
    MERGE_PDF       = "merge-pdf",       "Merge PDF",         Organize, "merge",          "Combine multiple PDFs into one.";
    SPLIT_PDF       = "split-pdf",       "Split PDF",         Organize, "split",          "Separate a PDF into the pages you need.";
    REMOVE_PAGES    = "remove-pages",    "Remove pages",      Organize, "file-minus",     "Delete the pages you don't want.";
    EXTRACT_PAGES   = "extract-pages",   "Extract pages",     Organize, "file-output",    "Pull selected pages into a new PDF.";
    ORGANIZE_PDF    = "organize-pdf",    "Organize PDF",      Organize, "layout-grid",    "Reorder, rotate and arrange pages.";
    SCAN_TO_PDF     = "scan-to-pdf",     "Scan to PDF",       Organize, "scan",           "Turn paper documents into PDFs.";
    // Optimize
    COMPRESS_PDF    = "compress-pdf",    "Compress PDF",      Optimize, "minimize-2",     "Smaller files. The same great quality.";
    REPAIR_PDF      = "repair-pdf",      "Repair PDF",        Optimize, "wrench",         "Recover damaged or corrupt PDFs.";
    OCR_PDF         = "ocr-pdf",         "OCR PDF",           Optimize, "scan-text",      "Make scanned text searchable.";
    // Convert to PDF
    JPG_TO_PDF      = "jpg-to-pdf",      "JPG to PDF",        ConvertTo, "image",         "Bring your images together in a PDF.";
    WORD_TO_PDF     = "word-to-pdf",     "Word to PDF",       ConvertTo, "file-text",     "Turn Word documents into PDFs.";
    PPT_TO_PDF      = "ppt-to-pdf",      "PowerPoint to PDF", ConvertTo, "presentation",  "Turn slide decks into PDFs.";
    EXCEL_TO_PDF    = "excel-to-pdf",    "Excel to PDF",      ConvertTo, "sheet",         "Turn spreadsheets into PDFs.";
    HTML_TO_PDF     = "html-to-pdf",     "HTML to PDF",       ConvertTo, "code",          "Save web pages as PDFs.";
    // Convert from PDF
    PDF_TO_JPG      = "pdf-to-jpg",      "PDF to JPG",        ConvertFrom, "file-image",  "Export pages as images.";
    PDF_TO_WORD     = "pdf-to-word",     "PDF to Word",       ConvertFrom, "file-type",   "Turn PDFs into editable documents.";
    PDF_TO_PPT      = "pdf-to-ppt",      "PDF to PowerPoint", ConvertFrom, "presentation","Turn PDFs into editable slides.";
    PDF_TO_EXCEL    = "pdf-to-excel",    "PDF to Excel",      ConvertFrom, "sheet",       "Pull tables into spreadsheets.";
    PDF_TO_PDFA     = "pdf-to-pdfa",     "PDF to PDF/A",      ConvertFrom, "archive",     "Archive-ready PDFs for the long term.";
    // Edit
    ROTATE_PDF      = "rotate-pdf",      "Rotate PDF",        Edit, "rotate-cw",          "Turn pages the right way up.";
    PAGE_NUMBERS    = "page-numbers",    "Add page numbers",  Edit, "list-ordered",       "Number your pages in any style.";
    WATERMARK       = "watermark",       "Add watermark",     Edit, "droplet",            "Stamp text or an image on pages.";
    CROP_PDF        = "crop-pdf",        "Crop PDF",          Edit, "crop",               "Trim margins and page areas.";
    EDIT_PDF        = "edit-pdf",        "Edit PDF",          Edit, "pencil",             "Add text, images and annotations.";
    PDF_FORMS       = "pdf-forms",       "PDF forms",         Edit, "clipboard-list",     "Fill in and create PDF forms.";
    // Security
    UNLOCK_PDF      = "unlock-pdf",      "Unlock PDF",        Security, "lock-open",      "Remove a password you know.";
    PROTECT_PDF     = "protect-pdf",     "Protect PDF",       Security, "lock",           "Add a password to your PDF.";
    SIGN_PDF        = "sign-pdf",        "Sign PDF",          Security, "pen-tool",       "Add your signature in a few clicks.";
    REDACT_PDF      = "redact-pdf",      "Redact PDF",        Security, "eraser",         "Permanently black out sensitive text.";
    COMPARE_PDF     = "compare-pdf",     "Compare PDF",       Security, "git-compare",    "Spot the differences between two PDFs.";
    // Intelligence
    AI_SUMMARIZER   = "ai-summarizer",   "AI Summarizer",     Intelligence, "sparkles",   "Find the key takeaways. Faster.";
    TRANSLATE_PDF   = "translate-pdf",   "Translate PDF",     Intelligence, "languages",  "Read your PDF in another language.";
    PDF_TO_MARKDOWN = "pdf-to-markdown", "PDF to Markdown",   Intelligence, "file-code",  "Clean Markdown from any PDF.";
    // Scan
    SCANNER         = "scanner",         "Document scanner",  Scan, "camera",             "Capture documents with a scanner or camera.";
    DESKEW          = "deskew",          "Auto crop & deskew", Scan, "crop",              "Straighten and trim scanned pages.";
}

/// Tool ids shown on the "Popular" tab, in order.
pub static POPULAR: &[&ToolSpec] = &[
    &MERGE_PDF,
    &SPLIT_PDF,
    &COMPRESS_PDF,
    &PDF_TO_WORD,
    &JPG_TO_PDF,
    &EDIT_PDF,
    &SIGN_PDF,
    &OCR_PDF,
    &AI_SUMMARIZER,
];

/// Tools belonging to one module, in display order.
pub fn tools_in(module: ModuleId) -> impl Iterator<Item = &'static ToolSpec> {
    TOOLS.iter().copied().filter(move |t| t.module == module)
}

/// Look up a tool by its id.
pub fn by_id(id: &str) -> Option<&'static ToolSpec> {
    TOOLS.iter().copied().find(|t| t.id == id)
}

/// Case-insensitive match on name or description. Empty query matches all.
pub fn matches(spec: &ToolSpec, query: &str) -> bool {
    let q = query.trim();
    if q.is_empty() {
        return true;
    }
    let q = q.to_lowercase();
    spec.name.to_lowercase().contains(&q) || spec.desc.to_lowercase().contains(&q)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn has_35_tools_in_8_modules() {
        assert_eq!(TOOLS.len(), 35);
        let modules: HashSet<_> = TOOLS.iter().map(|t| t.module).collect();
        assert_eq!(modules.len(), ModuleId::ALL.len());
    }

    #[test]
    fn ids_are_unique() {
        let ids: HashSet<_> = TOOLS.iter().map(|t| t.id).collect();
        assert_eq!(ids.len(), TOOLS.len());
    }

    #[test]
    fn popular_tools_are_in_catalog() {
        for p in POPULAR {
            assert_eq!(by_id(p.id), Some(*p));
        }
    }

    #[test]
    fn tools_are_grouped_by_module_order() {
        let order: Vec<_> = TOOLS.iter().map(|t| t.module).collect();
        let mut deduped = order.clone();
        deduped.dedup();
        assert_eq!(deduped, ModuleId::ALL.to_vec());
    }

    #[test]
    fn search_matches_name_and_desc() {
        assert!(matches(&COMPRESS_PDF, "compress"));
        assert!(matches(&COMPRESS_PDF, "SMALLER"));
        assert!(matches(&COMPRESS_PDF, "  "));
        assert!(!matches(&COMPRESS_PDF, "watermark"));
    }
}
