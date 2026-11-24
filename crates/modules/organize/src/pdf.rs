//! PDF helpers shared by the Organize tools (load, count, flatten, save).
//!
//! Move to a `dk-pdf` crate once a second module needs them.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use dk_core::{Error, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

/// Page attributes a page may inherit from its parent `Pages` nodes (PDF 32000-1, 7.7.3.4).
const INHERITABLE: [&[u8]; 4] = [b"Resources", b"MediaBox", b"CropBox", b"Rotate"];
/// Guards against cyclic `Parent` chains in malformed files.
const MAX_TREE_DEPTH: usize = 64;

pub(crate) fn display_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// Open a PDF for editing. Errors name the file so the user knows which one failed.
pub fn load(path: &Path) -> Result<Document> {
    let name = display_name(path);
    let doc = Document::load(path)
        .map_err(|e| Error::Invalid(format!("{name} isn't a valid PDF ({e}).")))?;
    if doc.is_encrypted() {
        return Err(Error::Invalid(format!(
            "{name} is password-protected. Unlock it first."
        )));
    }
    Ok(doc)
}

/// Number of pages in a PDF.
pub fn page_count(path: &Path) -> Result<u32> {
    let pages = load(path)?.get_pages().len();
    u32::try_from(pages)
        .map_err(|_| Error::Invalid(format!("{} has too many pages.", display_name(path))))
}

/// A page's dictionary with inherited attributes copied in, so the page stays correct
/// after it is moved under a different `Pages` parent.
pub(crate) fn flattened_page(doc: &Document, page_id: ObjectId) -> Result<Dictionary> {
    let mut page = doc
        .get_dictionary(page_id)
        .map_err(|e| Error::Invalid(format!("broken page object: {e}")))?
        .clone();
    let mut parent = page.get(b"Parent").and_then(Object::as_reference).ok();
    for _ in 0..MAX_TREE_DEPTH {
        let Some(node) = parent.and_then(|id| doc.get_dictionary(id).ok()) else {
            break;
        };
        for key in INHERITABLE {
            if !page.has(key) {
                if let Ok(value) = node.get(key) {
                    page.set(key.to_vec(), value.clone());
                }
            }
        }
        parent = node.get(b"Parent").and_then(Object::as_reference).ok();
    }
    Ok(page)
}

/// US Letter, the PDF default when a page declares no MediaBox.
pub(crate) const DEFAULT_MEDIA_BOX: [f32; 4] = [0.0, 0.0, 612.0, 792.0];

/// A page's MediaBox as `[x0, y0, x1, y1]`, normalised so x0 < x1 and y0 < y1.
pub(crate) fn media_box(page: &Dictionary) -> [f32; 4] {
    let values: Option<Vec<f32>> = page
        .get(b"MediaBox")
        .and_then(Object::as_array)
        .ok()
        .map(|a| a.iter().filter_map(|v| v.as_float().ok()).collect());
    match values.as_deref() {
        Some(&[a, b, c, d]) => [a.min(c), b.min(d), a.max(c), b.max(d)],
        _ => DEFAULT_MEDIA_BOX,
    }
}

/// Write `doc` to `dest` atomically: temp file in the same folder, then rename.
/// A failed or interrupted save never leaves a half-written PDF behind.
pub fn save_atomic(doc: &mut Document, dest: &Path) -> Result<()> {
    let tmp = TempFile::next_to(dest);
    {
        let mut out = BufWriter::new(File::create(&tmp.0)?);
        doc.save_to(&mut out)?;
        out.flush()?;
    }
    std::fs::rename(&tmp.0, dest)?;
    tmp.keep();
    Ok(())
}

/// Deletes the file on drop unless `keep` was called.
struct TempFile(PathBuf);

impl TempFile {
    fn next_to(dest: &Path) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        let name = format!(".{}.{}-{nanos}.tmp", display_name(dest), std::process::id());
        Self(dest.with_file_name(name))
    }

    fn keep(self) {
        std::mem::forget(self);
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Test helpers: build small PDFs in code so the repo needs no binary fixtures.
#[cfg(test)]
pub(crate) mod testing {
    use super::*;

    /// Unique empty folder under the system temp dir; removed on drop.
    pub struct TestDir(pub PathBuf);

    impl TestDir {
        pub fn new(tag: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir =
                std::env::temp_dir().join(format!("dk-test-{tag}-{}-{nanos}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        pub fn files(&self) -> Vec<String> {
            let mut names: Vec<_> = std::fs::read_dir(&self.0)
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn name(n: &str) -> Object {
        Object::Name(n.as_bytes().to_vec())
    }

    /// PDF with `pages` blank A4 pages. MediaBox sits on the `Pages` node so tests
    /// also cover inherited attributes.
    pub fn make_pdf(dir: &Path, file: &str, pages: usize) -> PathBuf {
        make_pdf_sized(dir, file, pages, 595, 842)
    }

    /// Like [`make_pdf`] with a custom page size in points.
    pub fn make_pdf_sized(
        dir: &Path,
        file: &str,
        pages: usize,
        width: i64,
        height: i64,
    ) -> PathBuf {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let kids: Vec<Object> = (0..pages)
            .map(|_| {
                let mut page = Dictionary::new();
                page.set("Type", name("Page"));
                page.set("Parent", Object::Reference(pages_id));
                Object::Reference(doc.add_object(Object::Dictionary(page)))
            })
            .collect();
        let mut tree = Dictionary::new();
        tree.set("Type", name("Pages"));
        tree.set("Count", Object::Integer(pages as i64));
        tree.set("Kids", Object::Array(kids));
        let media_box = [0, 0, width, height]
            .into_iter()
            .map(Object::Integer)
            .collect();
        tree.set("MediaBox", Object::Array(media_box));
        doc.objects.insert(pages_id, Object::Dictionary(tree));

        let mut catalog = Dictionary::new();
        catalog.set("Type", name("Catalog"));
        catalog.set("Pages", Object::Reference(pages_id));
        let catalog_id = doc.add_object(Object::Dictionary(catalog));
        doc.trailer.set("Root", Object::Reference(catalog_id));

        let path = dir.join(file);
        doc.save(&path).unwrap();
        path
    }
}

#[cfg(test)]
mod tests {
    use super::testing::*;
    use super::*;

    #[test]
    fn counts_pages() {
        let dir = TestDir::new("count");
        assert_eq!(page_count(&make_pdf(&dir.0, "a.pdf", 3)).unwrap(), 3);
    }

    #[test]
    fn rejects_non_pdf() {
        let dir = TestDir::new("bad");
        let path = dir.0.join("fake.pdf");
        std::fs::write(&path, b"not a pdf").unwrap();
        assert!(matches!(page_count(&path), Err(Error::Invalid(m)) if m.contains("fake.pdf")));
    }

    #[test]
    fn flattening_copies_inherited_media_box() {
        let dir = TestDir::new("flat");
        let doc = load(&make_pdf(&dir.0, "a.pdf", 1)).unwrap();
        let page_id = *doc.get_pages().values().next().unwrap();
        assert!(!doc.get_dictionary(page_id).unwrap().has(b"MediaBox"));
        assert!(flattened_page(&doc, page_id).unwrap().has(b"MediaBox"));
    }

    #[test]
    fn save_atomic_leaves_no_temp_file() {
        let dir = TestDir::new("save");
        let mut doc = load(&make_pdf(&dir.0, "a.pdf", 1)).unwrap();
        save_atomic(&mut doc, &dir.0.join("out.pdf")).unwrap();
        assert_eq!(dir.files(), vec!["a.pdf", "out.pdf"]);
    }
}
