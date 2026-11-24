//! The Merge page's file list: adding, validating, reading and reordering PDFs.

use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use dioxus::prelude::*;

use crate::jobs::blocking;
use crate::ui::error_text::user_message;

pub(super) const MAX_FILES: usize = 200;
pub(super) const MAX_NAME_CHARS: usize = 200;
/// Card preview width in pixels: cards are ~170 CSS px wide, doubled for high-DPI screens.
const THUMB_WIDTH: u16 = 340;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Pages {
    Counting,
    Ok(u32),
    Unreadable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PdfFile {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub pages: Pages,
    /// First page as a `data:image/jpeg` URL; `None` until rendered or if pdfium is missing.
    pub thumb: Option<String>,
    /// Extra clockwise rotation applied when merging: 0, 90, 180 or 270.
    pub rotation: i64,
}

impl PdfFile {
    pub fn page_count(&self) -> Option<u32> {
        match self.pages {
            Pages::Ok(n) => Some(n),
            _ => None,
        }
    }
}

pub(super) fn is_pdf(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
}

pub(super) fn jpeg_data_url(jpeg: Vec<u8>) -> String {
    format!("data:image/jpeg;base64,{}", STANDARD.encode(jpeg))
}

/// CSS that shows a preview the way it will be merged. Sideways pages are shrunk to fit.
pub(super) fn rotation_style(degrees: i64) -> String {
    match degrees.rem_euclid(360) {
        0 => String::new(),
        d if d % 180 == 0 => format!("transform: rotate({d}deg)"),
        d => format!("transform: rotate({d}deg) scale(0.75)"),
    }
}

/// Append new PDFs, skipping non-PDFs, duplicates and anything past `MAX_FILES`.
/// Returns how many were skipped.
pub(super) fn add_unique(
    list: &mut Vec<PdfFile>,
    incoming: impl IntoIterator<Item = PdfFile>,
) -> usize {
    let mut skipped = 0;
    for file in incoming {
        if !is_pdf(&file.path)
            || list.len() >= MAX_FILES
            || list.iter().any(|f| f.path == file.path)
        {
            skipped += 1;
        } else {
            list.push(file);
        }
    }
    skipped
}

/// Move the file at `from` to `to`. Returns false when out of range or unchanged.
pub(super) fn move_file(list: &mut Vec<PdfFile>, from: usize, to: usize) -> bool {
    if from == to || from >= list.len() || to >= list.len() {
        return false;
    }
    let file = list.remove(from);
    list.insert(to, file);
    true
}

/// Validate the user's output name and make sure it ends in `.pdf`.
pub(super) fn output_file_name(raw: &str) -> Result<String, &'static str> {
    let name = raw.trim();
    if name.is_empty() {
        return Err("Enter a file name.");
    }
    if name
        .chars()
        .any(|c| c.is_control() || r#"/\:*?"<>|"#.contains(c))
    {
        return Err(r#"A file name can't contain / \ : * ? " < > |"#);
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return Err("That file name is too long.");
    }
    Ok(if name.to_lowercase().ends_with(".pdf") {
        name.to_owned()
    } else {
        format!("{name}.pdf")
    })
}

/// Size, page count and preview, read off the UI thread.
fn inspect(path: &Path) -> (u64, Pages, Option<String>) {
    let size = std::fs::metadata(path).map_or(0, |m| m.len());
    let pages = match dk_organize::pdf::page_count(path) {
        Ok(n) => Pages::Ok(n),
        Err(e) => return (size, Pages::Unreadable(user_message(&e)), None),
    };
    // shortcut: pdfium is bound per file (cheap, the OS caches the library); keep one
    // instance alive if previews of many files become slow.
    let dirs = crate::os::pdfium_dirs();
    let Some(pdfium) = dk_organize::preview::bind(&dirs) else {
        static WARNED: std::sync::Once = std::sync::Once::new();
        WARNED.call_once(|| {
            tracing::warn!("pdfium not found in {dirs:?} or system path; previews disabled")
        });
        return (size, pages, None);
    };
    let thumb = match dk_organize::preview::first_page_jpeg(&pdfium, path, THUMB_WIDTH) {
        Ok(jpeg) => Some(jpeg_data_url(jpeg)),
        Err(e) => {
            tracing::warn!("preview failed: {e}");
            None
        }
    };
    (size, pages, thumb)
}

/// Add files to the list, then read them one by one in the background so cards fill in
/// progressively (one worker thread at a time, however many files are added).
pub(super) fn add_files(
    mut files: Signal<Vec<PdfFile>>,
    mut notice: Signal<Option<String>>,
    incoming: Vec<(PathBuf, String)>,
) {
    let new: Vec<PdfFile> = incoming
        .into_iter()
        .map(|(path, name)| PdfFile {
            path,
            name,
            size: 0,
            pages: Pages::Counting,
            thumb: None,
            rotation: 0,
        })
        .collect();
    let skipped = add_unique(&mut files.write(), new);
    notice.set(
        (skipped > 0)
            .then(|| format!("Skipped {skipped} file(s). Only new PDF files can be added.")),
    );

    let pending: Vec<PathBuf> = files
        .read()
        .iter()
        .filter(|f| f.pages == Pages::Counting)
        .map(|f| f.path.clone())
        .collect();
    if pending.is_empty() {
        return;
    }
    spawn(async move {
        for path in pending {
            let p = path.clone();
            let Some((size, pages, thumb)) = blocking(move || inspect(&p)).await else {
                continue;
            };
            // The file may have been removed while it was being read.
            if let Some(file) = files.write().iter_mut().find(|f| f.path == path) {
                file.size = size;
                file.pages = pages;
                file.thumb = thumb;
            }
        }
    });
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(super) async fn pick_pdfs() -> Vec<(PathBuf, String)> {
    rfd::AsyncFileDialog::new()
        .set_title("Add PDFs")
        .add_filter("PDF", &["pdf"])
        .pick_files()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|h| (h.path().to_path_buf(), h.file_name()))
        .collect()
}

// shortcut: no native file picker on mobile yet; files can't be added there.
#[cfg(any(target_os = "android", target_os = "ios"))]
pub(super) async fn pick_pdfs() -> Vec<(PathBuf, String)> {
    Vec::new()
}

/// Where to save. `None` = user cancelled.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(super) async fn choose_destination(name: &str, first_input: &Path) -> Option<PathBuf> {
    let mut dialog = rfd::AsyncFileDialog::new()
        .set_title("Save merged PDF")
        .set_file_name(name)
        .add_filter("PDF", &["pdf"]);
    if let Some(dir) = first_input.parent() {
        dialog = dialog.set_directory(dir);
    }
    let path = dialog.save_file().await?.path().to_path_buf();
    Some(if is_pdf(&path) {
        path
    } else {
        let mut with_ext = path.into_os_string();
        with_ext.push(".pdf");
        PathBuf::from(with_ext)
    })
}

// shortcut: no save dialog on mobile yet; saves next to the first file.
#[cfg(any(target_os = "android", target_os = "ios"))]
pub(super) async fn choose_destination(name: &str, first_input: &Path) -> Option<PathBuf> {
    Some(first_input.parent()?.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pdf(path: &str) -> PdfFile {
        PdfFile {
            path: PathBuf::from(path),
            name: path.to_owned(),
            size: 1,
            pages: Pages::Counting,
            thumb: None,
            rotation: 0,
        }
    }

    #[test]
    fn adds_only_new_pdfs() {
        let mut list = vec![pdf("a.pdf")];
        let skipped = add_unique(
            &mut list,
            [pdf("a.pdf"), pdf("b.PDF"), pdf("c.docx"), pdf("noext")],
        );
        assert_eq!(skipped, 3);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn moves_files() {
        let mut list = vec![pdf("a.pdf"), pdf("b.pdf"), pdf("c.pdf")];
        assert!(move_file(&mut list, 0, 2));
        let names: Vec<_> = list.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["b.pdf", "c.pdf", "a.pdf"]);
        assert!(!move_file(&mut list, 1, 1));
        assert!(!move_file(&mut list, 0, 9));
    }

    #[test]
    fn caps_file_count() {
        let mut list = Vec::new();
        let skipped = add_unique(
            &mut list,
            (0..MAX_FILES + 5).map(|i| pdf(&format!("{i}.pdf"))),
        );
        assert_eq!((list.len(), skipped), (MAX_FILES, 5));
    }

    #[test]
    fn validates_output_name() {
        assert_eq!(output_file_name(" report "), Ok("report.pdf".into()));
        assert_eq!(output_file_name("Report.PDF"), Ok("Report.PDF".into()));
        assert!(output_file_name("").is_err());
        assert!(output_file_name("../evil").is_err());
        assert!(output_file_name("a:b").is_err());
        assert!(output_file_name(&"x".repeat(MAX_NAME_CHARS + 1)).is_err());
    }

    #[test]
    fn rotation_style_shrinks_sideways_pages() {
        assert_eq!(rotation_style(0), "");
        assert_eq!(rotation_style(180), "transform: rotate(180deg)");
        assert_eq!(rotation_style(-90), "transform: rotate(270deg) scale(0.75)");
    }

    #[test]
    fn unreadable_file_reports_reason() {
        let path = std::env::temp_dir().join(format!("dk-app-bad-{}.pdf", std::process::id()));
        std::fs::write(&path, b"junk").unwrap();
        let (size, pages, thumb) = inspect(&path);
        let _ = std::fs::remove_file(&path);
        assert_eq!(size, 4);
        assert!(matches!(pages, Pages::Unreadable(_)));
        assert!(thumb.is_none());
    }
}
