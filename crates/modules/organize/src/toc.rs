//! Table-of-contents pages for merged documents: one clickable line per source file.

use lopdf::{Dictionary, Document, Object, ObjectId, Stream};

const MARGIN: f32 = 56.0;
/// Distance from the page top to the first entry's baseline (room for the title).
const TOP: f32 = 116.0;
const LINE: f32 = 20.0;
const FONT_SIZE: f32 = 11.0;
const TITLE_SIZE: f32 = 22.0;
/// Rough Helvetica advance width (em) for fitting text; exact metrics aren't needed here.
const AVG_CHAR_EM: f32 = 0.5;
const DIGIT_EM: f32 = 0.556;

/// One line of the table of contents.
pub(crate) struct Entry {
    pub title: String,
    /// 1-based page number in the final document.
    pub page_number: usize,
    /// Page the line links to.
    pub target: ObjectId,
}

fn lines_per_page(media: [f32; 4]) -> usize {
    let height = media[3] - media[1];
    (((height - TOP - MARGIN) / LINE).floor() as usize).max(1)
}

/// How many TOC pages `entries` lines need on pages of size `media`.
pub(crate) fn pages_needed(entries: usize, media: [f32; 4]) -> usize {
    entries.div_ceil(lines_per_page(media)).max(1)
}

/// Escape text for a PDF literal string and cap its length.
// shortcut: Helvetica/WinAnsi only, so non-Latin characters show as "?"; embed a Unicode
// font (e.g. Inter) when TOC titles in other scripts matter.
fn pdf_text(text: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (i, c) in text.chars().enumerate() {
        if i == max_chars {
            out.push_str("...");
            break;
        }
        match c {
            '(' | ')' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            ' '..='~' => out.push(c),
            _ => out.push('?'),
        }
    }
    out
}

fn font(doc: &mut Document, base: &str) -> ObjectId {
    let mut f = Dictionary::new();
    f.set("Type", Object::Name(b"Font".to_vec()));
    f.set("Subtype", Object::Name(b"Type1".to_vec()));
    f.set("BaseFont", Object::Name(base.as_bytes().to_vec()));
    f.set("Encoding", Object::Name(b"WinAnsiEncoding".to_vec()));
    doc.add_object(Object::Dictionary(f))
}

fn rect(values: [f32; 4]) -> Object {
    Object::Array(values.into_iter().map(Object::Real).collect())
}

/// Build the TOC pages (without `Parent`; the caller attaches them to the page tree).
pub(crate) fn build(doc: &mut Document, entries: &[Entry], media: [f32; 4]) -> Vec<ObjectId> {
    let regular = font(doc, "Helvetica");
    let bold = font(doc, "Helvetica-Bold");
    let mut fonts = Dictionary::new();
    fonts.set("F1", Object::Reference(regular));
    fonts.set("F2", Object::Reference(bold));
    let mut resources = Dictionary::new();
    resources.set("Font", Object::Dictionary(fonts));

    let [x0, _, x1, y1] = media;
    let left = x0 + MARGIN;
    let right = x1 - MARGIN;
    let max_chars = ((right - left - 60.0) / (FONT_SIZE * AVG_CHAR_EM)).max(10.0) as usize;
    let per_page = lines_per_page(media);

    let mut pages = Vec::new();
    for (chunk_index, chunk) in entries.chunks(per_page).enumerate() {
        let mut content = String::new();
        if chunk_index == 0 {
            content.push_str(&format!(
                "BT /F2 {TITLE_SIZE} Tf {left} {} Td (Contents) Tj ET\n",
                y1 - MARGIN - TITLE_SIZE
            ));
        }
        let mut annots = Vec::new();
        for (line, entry) in chunk.iter().enumerate() {
            let y = y1 - TOP - line as f32 * LINE;
            let number = entry.page_number.to_string();
            let number_x = right - number.len() as f32 * FONT_SIZE * DIGIT_EM;
            content.push_str(&format!(
                "BT /F1 {FONT_SIZE} Tf {left} {y} Td ({}) Tj ET\nBT /F1 {FONT_SIZE} Tf {number_x} {y} Td ({number}) Tj ET\n",
                pdf_text(&entry.title, max_chars)
            ));

            let mut link = Dictionary::new();
            link.set("Type", Object::Name(b"Annot".to_vec()));
            link.set("Subtype", Object::Name(b"Link".to_vec()));
            link.set("Rect", rect([left, y - 4.0, right, y + FONT_SIZE + 2.0]));
            link.set("Border", Object::Array(vec![Object::Integer(0); 3]));
            link.set(
                "Dest",
                Object::Array(vec![
                    Object::Reference(entry.target),
                    Object::Name(b"Fit".to_vec()),
                ]),
            );
            annots.push(Object::Reference(doc.add_object(Object::Dictionary(link))));
        }

        let stream = doc.add_object(Object::Stream(Stream::new(
            Dictionary::new(),
            content.into_bytes(),
        )));
        let mut page = Dictionary::new();
        page.set("Type", Object::Name(b"Page".to_vec()));
        page.set("MediaBox", rect(media));
        page.set("Resources", Object::Dictionary(resources.clone()));
        page.set("Contents", Object::Reference(stream));
        page.set("Annots", Object::Array(annots));
        pages.push(doc.add_object(Object::Dictionary(page)));
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_and_truncates_text() {
        assert_eq!(pdf_text("a(b)\\c", 50), "a\\(b\\)\\\\c");
        assert_eq!(pdf_text("héllo", 50), "h?llo");
        assert_eq!(pdf_text("abcdef", 3), "abc...");
    }

    #[test]
    fn page_count_grows_with_entries() {
        let a4 = [0.0, 0.0, 595.0, 842.0];
        let per = lines_per_page(a4);
        assert_eq!(pages_needed(1, a4), 1);
        assert_eq!(pages_needed(per, a4), 1);
        assert_eq!(pages_needed(per + 1, a4), 2);
    }
}
