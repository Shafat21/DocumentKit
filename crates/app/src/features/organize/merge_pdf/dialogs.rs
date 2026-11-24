//! Dialogs on the Merge page: page preview and advanced settings.

use std::path::PathBuf;

use dioxus::prelude::*;

use super::files::rotation_style;
use crate::jobs::blocking;
use crate::thumbs::jpeg_data_url;
use crate::ui::{EmptyState, Modal, Toggle};

const PREVIEW_WIDTH: u16 = 480;
/// Caps memory: ~100 KB per rendered page.
const MAX_PREVIEW_PAGES: u32 = 100;
/// Pages rendered per background call, so the dialog fills in as it goes.
const BATCH: u16 = 4;
const MAX_TEXT_CHARS: &str = "200";

/// Shown when pdfium can't be loaded. Dev builds say how to install it.
const MISSING_PDFIUM: &str = if cfg!(debug_assertions) {
    "Dev setup: run `powershell -ExecutionPolicy Bypass -File scripts\\setup-pdfium.ps1` (or `sh scripts/setup-pdfium.sh`) from the repo root, then reopen this preview."
} else {
    "Page previews need the pdfium library. Merging still works."
};

/// Every page of one file, rendered in the background a few at a time.
/// Rendering stops when the dialog closes (its task is dropped with it).
#[component]
pub(super) fn PreviewDialog(
    path: PathBuf,
    name: String,
    pages: u32,
    rotation: i64,
    on_close: EventHandler<()>,
) -> Element {
    let shown = pages.min(MAX_PREVIEW_PAGES) as u16;
    let mut images = use_signal(Vec::<Option<String>>::new);
    let mut unavailable = use_signal(|| false);

    use_future(move || {
        let path = path.clone();
        async move {
            let mut start = 0;
            while start < shown {
                let end = (start + BATCH).min(shown);
                let p = path.clone();
                let batch = blocking(move || {
                    let pdfium = dk_organize::preview::bind(&crate::os::pdfium_dirs())?;
                    dk_organize::preview::render_pages(&pdfium, &p, start..end, PREVIEW_WIDTH).ok()
                })
                .await
                .flatten();
                let Some(batch) = batch else {
                    unavailable.set(true);
                    return;
                };
                images
                    .write()
                    .extend(batch.into_iter().map(|page| page.ok().map(jpeg_data_url)));
                start = end;
            }
        }
    });

    let style = rotation_style(rotation);
    let limited = pages > MAX_PREVIEW_PAGES;

    rsx! {
        Modal { title: name, wide: true, on_close,
            p { class: "muted",
                "{pages} pages"
                if limited { " · showing the first {MAX_PREVIEW_PAGES}" }
                if rotation % 360 != 0 { " · shown as it will be merged" }
            }
            if unavailable() {
                EmptyState {
                    icon: "eye",
                    title: "Previews aren't available",
                    body: MISSING_PDFIUM.to_string(),
                }
            } else {
                div { class: "preview-grid",
                    for i in 0..shown {
                        figure { key: "{i}", class: "preview-page",
                            div { class: "preview-frame",
                                {match images.read().get(usize::from(i)) {
                                    Some(Some(src)) => rsx! { img { src: "{src}", alt: "Page {i + 1}", style: "{style}" } },
                                    Some(None) => rsx! { span { class: "muted", "Can't show this page" } },
                                    None => rsx! { span { class: "skeleton", "aria-hidden": "true" } },
                                }}
                            }
                            figcaption { class: "muted", "Page {i + 1}" }
                        }
                    }
                }
            }
        }
    }
}

/// Extra merge options, kept on the page so they survive closing the dialog.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Advanced {
    pub compress: Signal<bool>,
    pub toc: Signal<bool>,
    pub duplex: Signal<bool>,
    pub title: Signal<String>,
    pub author: Signal<String>,
    pub subject: Signal<String>,
    pub keywords: Signal<String>,
}

/// Hook: create the advanced options with their defaults.
pub(super) fn use_advanced() -> Advanced {
    Advanced {
        compress: use_signal(|| true),
        toc: use_signal(|| false),
        duplex: use_signal(|| false),
        title: use_signal(String::new),
        author: use_signal(String::new),
        subject: use_signal(String::new),
        keywords: use_signal(String::new),
    }
}

impl Advanced {
    /// Short labels for the options that differ from the defaults.
    pub fn changed(&self) -> Vec<&'static str> {
        let mut labels = Vec::new();
        if !(self.compress)() {
            labels.push("No compression");
        }
        if (self.toc)() {
            labels.push("Table of contents");
        }
        if (self.duplex)() {
            labels.push("Double-sided");
        }
        let properties = [self.title, self.author, self.subject, self.keywords];
        if properties.iter().any(|p| !p.read().trim().is_empty()) {
            labels.push("Properties");
        }
        labels
    }

    /// Document-property options for the merge job; the engine ignores empty values.
    pub fn properties(&self) -> [(&'static str, String); 4] {
        let get = |s: Signal<String>| s.read().trim().to_owned();
        [
            ("title", get(self.title)),
            ("author", get(self.author)),
            ("subject", get(self.subject)),
            ("keywords", get(self.keywords)),
        ]
    }
}

#[component]
pub(super) fn AdvancedDialog(advanced: Advanced, on_close: EventHandler<()>) -> Element {
    let Advanced {
        compress,
        toc,
        duplex,
        title,
        author,
        subject,
        keywords,
    } = advanced;
    rsx! {
        Modal { title: "Advanced settings", on_close,
            div { class: "settings-stack",
                Toggle { label: "Compress output", hint: "Remove unused data and compress page content.", on: compress }
                Toggle { label: "Add a table of contents", hint: "Starts with a page listing each file, with clickable links.", on: toc }
                Toggle { label: "Ready for double-sided printing", hint: "Adds blank pages so each file starts on a new sheet.", on: duplex }

                p { class: "section-label", "Document properties" }
                p { class: "hint", "Shown in PDF readers. Created and modified dates are added automatically." }
                TextField { label: "Title", value: title }
                TextField { label: "Author", value: author }
                TextField { label: "Subject", value: subject }
                TextField { label: "Keywords", value: keywords, placeholder: "Separate with commas" }
                button { class: "btn btn-primary btn-block", r#type: "button", onclick: move |_| on_close.call(()), "Done" }
            }
        }
    }
}

#[component]
fn TextField(
    label: &'static str,
    value: Signal<String>,
    #[props(default)] placeholder: &'static str,
) -> Element {
    let mut value = value;
    rsx! {
        label { class: "field",
            span { class: "field-label", "{label}" }
            input {
                r#type: "text",
                maxlength: MAX_TEXT_CHARS,
                placeholder,
                value: "{value}",
                oninput: move |e| value.set(e.value()),
            }
        }
    }
}
