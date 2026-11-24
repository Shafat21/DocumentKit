//! First-page previews rendered with pdfium.
//!
//! pdfium is a native library loaded at runtime. If it can't be found, previews are
//! unavailable and callers show an icon instead; nothing else is affected.
//! Move to a shared crate when PDF to JPG needs rendering too.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use dk_core::{Error, Result};
use pdfium_render::prelude::*;

use crate::pdf::display_name;

/// Load pdfium from the first folder that has it, else from the system library path.
pub fn bind(dirs: &[PathBuf]) -> Option<Pdfium> {
    let bindings = dirs
        .iter()
        .find_map(|dir| {
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(dir)).ok()
        })
        .or_else(|| Pdfium::bind_to_system_library().ok())?;
    Some(Pdfium::new(bindings))
}

/// First page as a JPEG, `width` pixels wide (height follows the page's aspect ratio).
pub fn first_page_jpeg(pdfium: &Pdfium, path: &Path, width: u16) -> Result<Vec<u8>> {
    render_pages(pdfium, path, 0..1, width)?
        .into_iter()
        .next()
        .unwrap_or_else(|| {
            Err(Error::Invalid(format!(
                "{}: no pages to preview",
                display_name(path)
            )))
        })
}

/// Render pages `range` (0-based, clamped to the page count), opening the file once.
/// One result per rendered page, so one bad page doesn't hide the rest.
pub fn render_pages(
    pdfium: &Pdfium,
    path: &Path,
    range: std::ops::Range<u16>,
    width: u16,
) -> Result<Vec<Result<Vec<u8>>>> {
    let fail = |e: PdfiumError| Error::Invalid(format!("{}: no preview ({e})", display_name(path)));
    let doc = pdfium.load_pdf_from_file(path, None).map_err(fail)?;
    let pages = doc.pages();
    let end = range.end.min(pages.len());
    let config = PdfRenderConfig::new().set_target_width(i32::from(width));

    Ok((range.start..end)
        .map(|index| {
            let page = pages.get(index).map_err(fail)?;
            let image = page
                .render_with_config(&config)
                .map_err(fail)?
                .as_image()
                .into_rgb8();
            let mut jpeg = Vec::new();
            image
                .write_to(&mut Cursor::new(&mut jpeg), ::image::ImageFormat::Jpeg)
                .map_err(|e| Error::Internal(format!("encode preview: {e}")))?;
            Ok(jpeg)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::testing::{make_pdf, TestDir};

    /// Runs only where pdfium is installed; elsewhere there's nothing to render with.
    #[test]
    fn renders_jpeg_when_pdfium_available() {
        let dev_dir = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../app/assets/pdfium"
        ));
        let Some(pdfium) = bind(std::slice::from_ref(&dev_dir)) else {
            let lib = Pdfium::pdfium_platform_library_name_at_path(&dev_dir);
            assert!(!lib.exists(), "{} exists but failed to load", lib.display());
            return;
        };
        let dir = TestDir::new("preview");
        let jpeg = first_page_jpeg(&pdfium, &make_pdf(&dir.0, "a.pdf", 1), 120).unwrap();
        assert_eq!(&jpeg[..2], &[0xFF, 0xD8]);
    }
}
