//! First-page previews as `data:` URLs, shared by every page that shows PDF cards.
//! Blocking: call through `jobs::blocking`.

use std::path::Path;

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;

/// Card preview width in pixels: cards are ~170 CSS px wide, doubled for high-DPI screens.
pub const CARD_WIDTH: u16 = 340;

pub fn jpeg_data_url(jpeg: Vec<u8>) -> String {
    format!("data:image/jpeg;base64,{}", STANDARD.encode(jpeg))
}

/// First page as a data URL, or `None` if pdfium is missing or the page can't render.
// shortcut: pdfium is bound per call (cheap, the OS caches the library); keep one
// instance alive if previews of many files become slow.
pub fn first_page(path: &Path, width: u16) -> Option<String> {
    let dirs = crate::os::pdfium_dirs();
    let Some(pdfium) = dk_organize::preview::bind(&dirs) else {
        static WARNED: std::sync::Once = std::sync::Once::new();
        WARNED.call_once(|| {
            tracing::warn!("pdfium not found in {dirs:?} or system path; previews disabled")
        });
        return None;
    };
    match dk_organize::preview::first_page_jpeg(&pdfium, path, width) {
        Ok(jpeg) => Some(jpeg_data_url(jpeg)),
        Err(e) => {
            tracing::warn!("preview failed: {e}");
            None
        }
    }
}
