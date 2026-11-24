//! Merge PDF: combine multiple PDFs into one.
//!
//! Options (`job.options`), all optional except `output_name`:
//! - `output_name` (string): file name written inside `job.output_dir`.
//! - `bookmarks` (bool, default `true`): one bookmark per input file.
//! - `keep_page_sizes` (bool, default `true`): `false` scales every page to fit the
//!   first page's size.
//! - `rotations` (array of ints, default all 0): extra clockwise rotation per input,
//!   in multiples of 90.
//! - `toc` (bool, default `false`): table-of-contents page(s) first, linking to each file.
//! - `duplex` (bool, default `false`): blank page after odd-length parts, so each file
//!   starts on a front side when printed double-sided.
//! - `compress` (bool, default `true`): drop unused objects and compress streams.
//! - `title`, `author`, `subject`, `keywords` (strings): document properties.
//!   Creator, creation and modification dates are always set.

use std::path::{Path, PathBuf};

use dk_core::{catalog, Error, Job, JobOutput, Progress, Result, Tool, ToolSpec};
use lopdf::{Bookmark, Dictionary, Document, Object, ObjectId, Stream, StringFormat};

use crate::{pdf, toc};

/// shortcut: lopdf holds whole documents in memory, so total input is capped;
/// upgrade to a streaming writer if users need bigger merges.
pub const MAX_TOTAL_BYTES: u64 = 500 * 1024 * 1024;
const MAX_TEXT_CHARS: usize = 200;

/// Combines `job.inputs`, in order, into one PDF.
#[derive(Debug, Default, Clone, Copy)]
pub struct MergePdf;

impl Tool for MergePdf {
    fn spec(&self) -> &'static ToolSpec {
        &catalog::MERGE_PDF
    }

    fn run(&self, job: Job, progress: &dyn Progress) -> Result<JobOutput> {
        let options = Options::parse(&job)?;
        let dest = validate(&job, &options.name)?;
        let mut merged = merge(&job, &options, progress)?;

        if job.is_cancelled() {
            return Err(Error::Cancelled);
        }
        progress.report(100, "Saving");
        pdf::save_atomic(&mut merged, &dest)?;
        Ok(JobOutput { files: vec![dest] })
    }
}

struct Options {
    name: String,
    bookmarks: bool,
    keep_page_sizes: bool,
    rotations: Vec<i64>,
    toc: bool,
    duplex: bool,
    compress: bool,
    title: Option<String>,
    author: Option<String>,
    subject: Option<String>,
    keywords: Option<String>,
}

impl Options {
    fn parse(job: &Job) -> Result<Self> {
        let o = &job.options;
        let flag =
            |key: &str, default: bool| o.get(key).and_then(|v| v.as_bool()).unwrap_or(default);
        let text = |key: &str| -> Result<Option<String>> {
            match o
                .get(key)
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                Some(s) if s.chars().count() > MAX_TEXT_CHARS => {
                    Err(Error::Invalid(format!("The {key} is too long.")))
                }
                other => Ok(other.map(str::to_owned)),
            }
        };

        let name = text("output_name")?
            .ok_or_else(|| Error::Invalid("Choose a name for the merged file.".into()))?;
        let rotations = match o.get("rotations").and_then(|v| v.as_array()) {
            None => vec![0; job.inputs.len()],
            Some(values) => {
                let degrees: Option<Vec<i64>> = values
                    .iter()
                    .map(|v| v.as_i64().filter(|d| d % 90 == 0))
                    .collect();
                match degrees {
                    Some(d) if d.len() == job.inputs.len() => d,
                    _ => {
                        return Err(Error::Invalid(
                            "Rotations must be multiples of 90°, one per file.".into(),
                        ))
                    }
                }
            }
        };
        Ok(Self {
            name,
            bookmarks: flag("bookmarks", true),
            keep_page_sizes: flag("keep_page_sizes", true),
            rotations,
            toc: flag("toc", false),
            duplex: flag("duplex", false),
            compress: flag("compress", true),
            title: text("title")?,
            author: text("author")?,
            subject: text("subject")?,
            keywords: text("keywords")?,
        })
    }
}

/// Check inputs; returns the output path.
fn validate(job: &Job, name: &str) -> Result<PathBuf> {
    if job.inputs.len() < 2 {
        return Err(Error::Invalid("Add at least two PDFs to merge.".into()));
    }
    // A bare file name only: never let options steer the write outside output_dir.
    if Path::new(name).file_name() != Some(std::ffi::OsStr::new(name)) {
        return Err(Error::Invalid(
            "The output name can't contain folders.".into(),
        ));
    }
    let dest = job.output_dir.join(name);

    let dest_real = job.output_dir.canonicalize()?.join(name);
    let mut total = 0u64;
    for input in &job.inputs {
        let meta = std::fs::metadata(input).map_err(|_| {
            Error::Invalid(format!("{} can't be opened.", pdf::display_name(input)))
        })?;
        total += meta.len();
        if input.canonicalize()? == dest_real {
            return Err(Error::Invalid(format!(
                "The merged file can't replace {}. Choose another name.",
                pdf::display_name(input)
            )));
        }
    }
    if total > MAX_TOTAL_BYTES {
        return Err(Error::Invalid(
            "These files are too large to merge at once (500 MB limit).".into(),
        ));
    }
    Ok(dest)
}

/// Pages contributed by one input file.
struct Part {
    title: String,
    pages: Vec<ObjectId>,
}

/// Build the merged document: every input's objects, renumbered so ids don't clash,
/// under one new page tree and catalog.
fn merge(job: &Job, options: &Options, progress: &dyn Progress) -> Result<Document> {
    let mut out = Document::with_version("1.7");
    let parts = read_inputs(job, options, &mut out, progress)?;
    let target = parts
        .first()
        .and_then(|p| p.pages.first())
        .and_then(|id| out.get_dictionary(*id).ok())
        .map_or(pdf::DEFAULT_MEDIA_BOX, pdf::media_box);

    if !options.keep_page_sizes {
        for id in parts
            .iter()
            .flat_map(|p| p.pages.clone())
            .collect::<Vec<_>>()
        {
            fit_page(&mut out, id, target);
        }
    }

    // Page order: [TOC][blank?] part1 [blank?] part2 …
    let toc_pages = if options.toc {
        toc::pages_needed(parts.len(), target)
    } else {
        0
    };
    let mut kids: Vec<ObjectId> = Vec::new();
    let mut entries = Vec::new();
    let mut next_number = toc_pages + usize::from(options.duplex && toc_pages % 2 == 1) + 1;
    for (i, part) in parts.iter().enumerate() {
        entries.push(toc::Entry {
            title: part.title.clone(),
            page_number: next_number,
            target: part.pages[0],
        });
        next_number +=
            part.pages.len() + usize::from(needs_blank(options, part, i + 1 == parts.len()));
    }
    if options.toc {
        kids.extend(toc::build(&mut out, &entries, target));
        if options.duplex && toc_pages % 2 == 1 {
            kids.push(blank_page(&mut out, target));
        }
    }
    let last = parts.len().saturating_sub(1);
    for (i, part) in parts.iter().enumerate() {
        kids.extend(&part.pages);
        if needs_blank(options, part, i == last) {
            let size = part
                .pages
                .last()
                .and_then(|id| out.get_dictionary(*id).ok())
                .map_or(target, pdf::media_box);
            kids.push(blank_page(&mut out, size));
        }
    }

    let pages_id = out.new_object_id();
    for kid in &kids {
        if let Some(Object::Dictionary(page)) = out.objects.get_mut(kid) {
            page.set("Parent", Object::Reference(pages_id));
        }
    }
    let mut tree = Dictionary::new();
    tree.set("Type", Object::Name(b"Pages".to_vec()));
    tree.set("Count", Object::Integer(kids.len() as i64));
    tree.set(
        "Kids",
        Object::Array(kids.into_iter().map(Object::Reference).collect()),
    );
    out.objects.insert(pages_id, Object::Dictionary(tree));

    let mut catalog = Dictionary::new();
    catalog.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog.set("Pages", Object::Reference(pages_id));
    if options.bookmarks {
        for entry in &entries {
            out.add_bookmark(
                Bookmark::new(entry.title.clone(), [0.0, 0.0, 0.0], 0, entry.target),
                None,
            );
        }
        if let Some(outline) = out.build_outline() {
            catalog.set("Outlines", Object::Reference(outline));
            catalog.set("PageMode", Object::Name(b"UseOutlines".to_vec()));
        }
    }
    let catalog_id = out.add_object(Object::Dictionary(catalog));
    out.trailer.set("Root", Object::Reference(catalog_id));
    set_info(&mut out, options);

    if options.compress {
        let _ = out.prune_objects();
        out.compress();
    }
    Ok(out)
}

/// Load each input, renumber it into `out`, and collect its (flattened, rotated) pages.
fn read_inputs(
    job: &Job,
    options: &Options,
    out: &mut Document,
    progress: &dyn Progress,
) -> Result<Vec<Part>> {
    let count = job.inputs.len();
    let mut next_id = 1;
    let mut parts = Vec::with_capacity(count);

    for (i, path) in job.inputs.iter().enumerate() {
        if job.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let name = pdf::display_name(path);
        progress.report(
            (i * 90 / count) as u8,
            &format!("Reading {name} ({} of {count})", i + 1),
        );

        let mut doc = pdf::load(path)?;
        doc.renumber_objects_with(next_id);
        next_id = doc.max_id + 1;

        // get_pages is ordered by page number.
        let page_ids: Vec<ObjectId> = doc.get_pages().into_values().collect();
        if page_ids.is_empty() {
            return Err(Error::Invalid(format!("{name} has no pages.")));
        }
        let mut pages = Vec::with_capacity(page_ids.len());
        for &id in &page_ids {
            let mut page = pdf::flattened_page(&doc, id)?;
            rotate(&mut page, options.rotations[i]);
            pages.push((id, page));
        }

        // Keep content, fonts, images…; drop the old document structure (rebuilt in `merge`).
        for (id, object) in doc.objects {
            if !is_structure(&object) {
                out.objects.insert(id, object);
            }
        }
        for (id, page) in pages {
            out.objects.insert(id, Object::Dictionary(page));
        }
        let title = path
            .file_stem()
            .map_or(name, |s| s.to_string_lossy().into_owned());
        parts.push(Part {
            title,
            pages: page_ids,
        });
    }
    out.max_id = next_id - 1;
    Ok(parts)
}

fn needs_blank(options: &Options, part: &Part, is_last: bool) -> bool {
    options.duplex && !is_last && part.pages.len() % 2 == 1
}

/// Add `degrees` (multiple of 90) to the page's own rotation.
fn rotate(page: &mut Dictionary, degrees: i64) {
    if degrees % 360 == 0 {
        return;
    }
    let current = page.get(b"Rotate").and_then(Object::as_i64).unwrap_or(0);
    page.set(
        "Rotate",
        Object::Integer((current + degrees).rem_euclid(360)),
    );
}

/// Scale a page's content to fit `target`, centred, and give it `target`'s size.
// shortcut: annotation rectangles (links, form fields) aren't transformed; they may sit
// off their visuals on rescaled pages.
fn fit_page(out: &mut Document, id: ObjectId, target: [f32; 4]) {
    let Ok(page) = out.get_dictionary(id) else {
        return;
    };
    let [x0, y0, x1, y1] = pdf::media_box(page);
    let (w, h) = (x1 - x0, y1 - y0);
    let (tw, th) = (target[2] - target[0], target[3] - target[1]);
    if w < 1.0 || h < 1.0 || ((w - tw).abs() < 0.5 && (h - th).abs() < 0.5) {
        return;
    }
    let contents: Vec<Object> = match page.get(b"Contents") {
        Ok(Object::Reference(r)) => vec![Object::Reference(*r)],
        Ok(Object::Array(items)) => items.clone(),
        _ => Vec::new(),
    };

    let s = (tw / w).min(th / h);
    let tx = target[0] + (tw - w * s) / 2.0 - x0 * s;
    let ty = target[1] + (th - h * s) / 2.0 - y0 * s;
    let pre = out.add_object(Object::Stream(Stream::new(
        Dictionary::new(),
        format!("q {s} 0 0 {s} {tx} {ty} cm\n").into_bytes(),
    )));
    let post = out.add_object(Object::Stream(Stream::new(
        Dictionary::new(),
        b"\nQ\n".to_vec(),
    )));

    if let Some(Object::Dictionary(page)) = out.objects.get_mut(&id) {
        let mut stack = vec![Object::Reference(pre)];
        stack.extend(contents);
        stack.push(Object::Reference(post));
        page.set("Contents", Object::Array(stack));
        page.set(
            "MediaBox",
            Object::Array(target.into_iter().map(Object::Real).collect()),
        );
        for key in [&b"CropBox"[..], b"TrimBox", b"BleedBox", b"ArtBox"] {
            page.remove(key);
        }
    }
}

fn blank_page(out: &mut Document, media: [f32; 4]) -> ObjectId {
    let mut page = Dictionary::new();
    page.set("Type", Object::Name(b"Page".to_vec()));
    page.set(
        "MediaBox",
        Object::Array(media.into_iter().map(Object::Real).collect()),
    );
    page.set("Resources", Object::Dictionary(Dictionary::new()));
    out.add_object(Object::Dictionary(page))
}

/// PDF text string as UTF-16BE with BOM, so any language survives.
fn utf16_string(text: &str) -> Object {
    let mut bytes = vec![0xFE, 0xFF];
    bytes.extend(text.encode_utf16().flat_map(u16::to_be_bytes));
    Object::String(bytes, StringFormat::Hexadecimal)
}

/// PDF date string (`D:YYYYMMDDHHmmSSZ`, UTC) for a Unix timestamp.
fn pdf_date(unix_secs: u64) -> String {
    let days = (unix_secs / 86_400) as i64;
    let secs = unix_secs % 86_400;
    // Days since 1970-01-01 → civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "D:{year:04}{month:02}{day:02}{:02}{:02}{:02}Z",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

fn set_info(out: &mut Document, options: &Options) {
    let mut info = Dictionary::new();
    info.set("Producer", utf16_string("DocumentKit"));
    info.set(
        "Creator",
        utf16_string(concat!("DocumentKit ", env!("CARGO_PKG_VERSION"))),
    );
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let date = Object::string_literal(pdf_date(now));
    info.set("CreationDate", date.clone());
    info.set("ModDate", date);
    for (key, value) in [
        ("Title", &options.title),
        ("Author", &options.author),
        ("Subject", &options.subject),
        ("Keywords", &options.keywords),
    ] {
        if let Some(value) = value {
            info.set(key, utf16_string(value));
        }
    }
    let info_id = out.add_object(Object::Dictionary(info));
    out.trailer.set("Info", Object::Reference(info_id));
}

/// Objects that make up a document's structure rather than its content.
fn is_structure(object: &Object) -> bool {
    let kind = object
        .as_dict()
        .ok()
        .and_then(|d| d.get(b"Type").ok())
        .and_then(|t| t.as_name().ok());
    matches!(kind, Some(b"Catalog" | b"Pages" | b"Page" | b"Outlines"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::testing::{make_pdf, make_pdf_sized, TestDir};
    use dk_core::NoProgress;
    use serde_json::json;

    fn run(dir: &TestDir, inputs: Vec<PathBuf>, options: serde_json::Value) -> Result<JobOutput> {
        let mut job = Job::new(inputs, dir.0.clone());
        job.options = options;
        MergePdf.run(job, &NoProgress)
    }

    fn pages_of(path: &Path) -> (Document, Vec<ObjectId>) {
        let doc = Document::load(path).unwrap();
        let ids = doc.get_pages().into_values().collect();
        (doc, ids)
    }

    fn reference(dict: &Dictionary, key: &[u8]) -> Option<ObjectId> {
        dict.get(key).and_then(Object::as_reference).ok()
    }

    /// Top-level bookmarks: walk Catalog → Outlines → First → Next….
    fn outline_count(path: &Path) -> usize {
        let doc = Document::load(path).unwrap();
        let catalog = doc
            .get_dictionary(reference(&doc.trailer, b"Root").unwrap())
            .unwrap();
        let Some(outlines) = reference(catalog, b"Outlines") else {
            return 0;
        };
        let mut item = reference(doc.get_dictionary(outlines).unwrap(), b"First");
        let mut count = 0;
        while let Some(id) = item {
            count += 1;
            item = reference(doc.get_dictionary(id).unwrap(), b"Next");
        }
        count
    }

    #[test]
    fn merges_pages_in_order() {
        let dir = TestDir::new("merge");
        let a = make_pdf(&dir.0, "a.pdf", 2);
        let b = make_pdf(&dir.0, "b.pdf", 3);
        let out = run(&dir, vec![a, b], json!({ "output_name": "out.pdf" })).unwrap();
        assert_eq!(out.files, vec![dir.0.join("out.pdf")]);
        let (doc, pages) = pages_of(&out.files[0]);
        assert_eq!(pages.len(), 5);
        // Inherited MediaBox survives the move to the new page tree.
        assert!(doc.get_dictionary(pages[0]).unwrap().has(b"MediaBox"));
    }

    #[test]
    fn bookmarks_follow_option() {
        let dir = TestDir::new("bookmarks");
        let a = make_pdf(&dir.0, "a.pdf", 1);
        let b = make_pdf(&dir.0, "b.pdf", 1);
        run(
            &dir,
            vec![a.clone(), b.clone()],
            json!({ "output_name": "on.pdf" }),
        )
        .unwrap();
        run(
            &dir,
            vec![a, b],
            json!({ "output_name": "off.pdf", "bookmarks": false }),
        )
        .unwrap();
        assert_eq!(outline_count(&dir.0.join("on.pdf")), 2);
        assert_eq!(outline_count(&dir.0.join("off.pdf")), 0);
    }

    #[test]
    fn rotates_only_the_chosen_file() {
        let dir = TestDir::new("rotate");
        let a = make_pdf(&dir.0, "a.pdf", 1);
        let b = make_pdf(&dir.0, "b.pdf", 2);
        let out = run(
            &dir,
            vec![a, b],
            json!({ "output_name": "o.pdf", "rotations": [0, 270] }),
        )
        .unwrap();
        let (doc, pages) = pages_of(&out.files[0]);
        let rotation = |id| {
            doc.get_dictionary(id)
                .unwrap()
                .get(b"Rotate")
                .and_then(Object::as_i64)
                .unwrap_or(0)
        };
        assert_eq!(
            [rotation(pages[0]), rotation(pages[1]), rotation(pages[2])],
            [0, 270, 270]
        );
    }

    #[test]
    fn normalises_page_sizes_when_asked() {
        let dir = TestDir::new("sizes");
        let a = make_pdf(&dir.0, "a.pdf", 1);
        let letter = make_pdf_sized(&dir.0, "b.pdf", 1, 612, 792);
        let out = run(
            &dir,
            vec![a, letter],
            json!({ "output_name": "o.pdf", "keep_page_sizes": false }),
        )
        .unwrap();
        let (doc, pages) = pages_of(&out.files[0]);
        let size = |id| pdf::media_box(doc.get_dictionary(id).unwrap());
        assert_eq!(size(pages[1]), size(pages[0]));
    }

    #[test]
    fn toc_and_duplex_add_pages() {
        let dir = TestDir::new("toc");
        let a = make_pdf(&dir.0, "a.pdf", 1);
        let b = make_pdf(&dir.0, "b.pdf", 2);
        // TOC(1) + blank(1, duplex) + a(1) + blank(1) + b(2) = 6
        let out = run(
            &dir,
            vec![a, b],
            json!({ "output_name": "o.pdf", "toc": true, "duplex": true }),
        )
        .unwrap();
        let (doc, pages) = pages_of(&out.files[0]);
        assert_eq!(pages.len(), 6);
        let annots = doc
            .get_dictionary(pages[0])
            .unwrap()
            .get(b"Annots")
            .and_then(Object::as_array)
            .unwrap();
        assert_eq!(annots.len(), 2);
    }

    #[test]
    fn writes_document_info() {
        let dir = TestDir::new("info");
        let a = make_pdf(&dir.0, "a.pdf", 1);
        let b = make_pdf(&dir.0, "b.pdf", 1);
        let out = run(
            &dir,
            vec![a, b],
            json!({
                "output_name": "o.pdf",
                "title": "Report",
                "author": "Shafat",
                "subject": "Q4",
                "keywords": "finance, report",
            }),
        )
        .unwrap();
        let doc = Document::load(&out.files[0]).unwrap();
        let info = doc
            .get_dictionary(reference(&doc.trailer, b"Info").unwrap())
            .unwrap();
        for key in [
            &b"Title"[..],
            b"Author",
            b"Subject",
            b"Keywords",
            b"Creator",
            b"CreationDate",
            b"ModDate",
        ] {
            assert!(info.has(key), "missing {}", String::from_utf8_lossy(key));
        }
    }

    #[test]
    fn formats_pdf_dates() {
        assert_eq!(pdf_date(0), "D:19700101000000Z");
        assert_eq!(pdf_date(951_782_400), "D:20000229000000Z");
        assert_eq!(pdf_date(1_795_514_400 + 3_723), "D:20261124110203Z");
    }

    #[test]
    fn rejects_bad_jobs() {
        let dir = TestDir::new("reject");
        let a = make_pdf(&dir.0, "a.pdf", 1);
        let b = make_pdf(&dir.0, "b.pdf", 1);
        let garbage = dir.0.join("junk.pdf");
        std::fs::write(&garbage, b"junk").unwrap();
        let ab = || vec![a.clone(), b.clone()];

        let invalid = |r: Result<JobOutput>| matches!(r, Err(Error::Invalid(_)));
        assert!(invalid(run(
            &dir,
            vec![a.clone()],
            json!({ "output_name": "o.pdf" })
        )));
        assert!(invalid(run(&dir, ab(), json!({ "output_name": "a.pdf" }))));
        assert!(invalid(run(
            &dir,
            ab(),
            json!({ "output_name": "../o.pdf" })
        )));
        assert!(invalid(run(&dir, ab(), json!({ "output_name": "" }))));
        assert!(invalid(run(
            &dir,
            ab(),
            json!({ "output_name": "o.pdf", "rotations": [45, 0] })
        )));
        assert!(invalid(run(
            &dir,
            ab(),
            json!({ "output_name": "o.pdf", "rotations": [90] })
        )));
        assert!(invalid(run(
            &dir,
            vec![a.clone(), garbage],
            json!({ "output_name": "o.pdf" })
        )));
        assert!(!dir.0.join("o.pdf").exists());
    }

    #[test]
    fn cancelled_job_writes_nothing() {
        let dir = TestDir::new("cancel");
        let a = make_pdf(&dir.0, "a.pdf", 1);
        let b = make_pdf(&dir.0, "b.pdf", 1);
        let mut job = Job::new(vec![a, b], dir.0.clone());
        job.options = json!({ "output_name": "o.pdf" });
        job.cancel_handle()
            .store(true, std::sync::atomic::Ordering::Relaxed);
        assert!(matches!(
            MergePdf.run(job, &NoProgress),
            Err(Error::Cancelled)
        ));
        assert_eq!(dir.files(), vec!["a.pdf", "b.pdf"]);
    }
}
