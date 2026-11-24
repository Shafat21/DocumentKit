//! Merge PDF page: pick PDFs, put them in order, choose settings, run the merge job.
//!
//! The engine is `dk_organize::MergePdf` (see its docs for the options sent), run through
//! `jobs::runner` so the UI never blocks.

mod dialogs;
mod files;

use std::path::PathBuf;

use dioxus::html::{HasFileData, Key, Modifiers, ModifiersInteraction};
use dioxus::prelude::*;
use dk_core::{catalog, Job};

use crate::jobs::blocking;
use crate::jobs::runner::{self, JobHandle, JobState};
use crate::pages::BackLink;
use crate::registry;
use crate::state::use_store;
use crate::ui::error_text::user_message;
use crate::ui::{format_size, Icon, Toggle};
use dialogs::{use_advanced, AdvancedDialog, PreviewDialog};
use files::{
    add_files, choose_destination, move_file, output_file_name, pick_pdfs, rotation_style, Pages,
    PdfFile,
};

const DEFAULT_NAME: &str = "Merged_Document.pdf";

fn card_class(pages: &Pages, dragged: bool) -> &'static str {
    match (matches!(pages, Pages::Unreadable(_)), dragged) {
        (true, _) => "file-card bad",
        (false, true) => "file-card dragging",
        (false, false) => "file-card",
    }
}

#[component]
pub fn MergePdf() -> Element {
    let mut files = use_signal(Vec::<PdfFile>::new);
    let bookmarks = use_signal(|| true);
    let keep_sizes = use_signal(|| true);
    let advanced = use_advanced();
    let mut out_name = use_signal(|| DEFAULT_NAME.to_owned());
    let notice = use_signal(|| None::<String>);
    let mut job = use_signal(|| None::<JobHandle>);
    // Index of the card being dragged. Mouse events, not HTML drag-and-drop: on Windows
    // Dioxus's file-drop handler swallows HTML drag events.
    let mut dragging = use_signal(|| None::<usize>);
    let mut previewing = use_signal(|| None::<usize>);
    let mut show_advanced = use_signal(|| false);
    let store = use_store();

    // Record a finished merge in "Recent documents". touch_recent is an upsert, so reruns are harmless.
    use_effect(move || {
        let Some(handle) = job.read().clone() else {
            return;
        };
        let JobState::Done(result) = handle.state.read().clone() else {
            return;
        };
        let Ok(output) = &*result else { return };
        let Some(path) = output.files.first().cloned() else {
            return;
        };
        let pages: u32 = files.peek().iter().filter_map(PdfFile::page_count).sum();
        let store = store.clone();
        spawn(async move {
            let saved = blocking(move || {
                let size = std::fs::metadata(&path).map_or(0, |m| m.len());
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                store.touch_recent(&path.to_string_lossy(), &name, pages, size)
            })
            .await;
            if let Some(Err(e)) = saved {
                tracing::warn!("could not record recent document: {e}");
            }
        });
    });

    let state = job.read().as_ref().map(|h| h.state.read().clone());
    let running = matches!(state, Some(JobState::Running { .. }));
    let count = files.read().len();
    let total_size = files.read().iter().map(|f| f.size).sum::<u64>();
    let total_pages: u32 = files.read().iter().filter_map(PdfFile::page_count).sum();
    let all_ready = files.read().iter().all(|f| f.page_count().is_some());
    let any_unreadable = files
        .read()
        .iter()
        .any(|f| matches!(f.pages, Pages::Unreadable(_)));
    let name_check = output_file_name(&out_name.read());
    let can_merge = count >= 2 && all_ready && name_check.is_ok() && !running;
    let extras = advanced.changed().join(" · ");

    let browse = move |_: MouseEvent| {
        spawn(async move {
            let picked = pick_pdfs().await;
            if !picked.is_empty() {
                add_files(files, notice, picked);
            }
        });
    };

    let start_merge = move |_: MouseEvent| {
        let Ok(name) = output_file_name(&out_name.read()) else {
            return;
        };
        let (inputs, rotations): (Vec<PathBuf>, Vec<i64>) = files
            .read()
            .iter()
            .map(|f| (f.path.clone(), f.rotation))
            .unzip();
        let Some(first) = inputs.first().cloned() else {
            return;
        };
        let mut options = serde_json::json!({
            "bookmarks": bookmarks(),
            "keep_page_sizes": keep_sizes(),
            "rotations": rotations,
            "compress": (advanced.compress)(),
            "toc": (advanced.toc)(),
            "duplex": (advanced.duplex)(),
        });
        for (key, value) in advanced.properties() {
            options[key] = value.into();
        }
        spawn(async move {
            let Some(dest) = choose_destination(&name, &first).await else {
                return;
            };
            let (Some(dir), Some(file_name)) = (dest.parent(), dest.file_name()) else {
                return;
            };
            let Some(tool) = registry::find(catalog::MERGE_PDF.id) else {
                return;
            };
            let mut merge = Job::new(inputs, dir.to_path_buf());
            merge.options = options;
            merge.options["output_name"] = file_name.to_string_lossy().into();
            job.set(Some(runner::start(tool, merge)));
        });
    };

    let summary_title = if any_unreadable {
        "Remove unreadable files to continue"
    } else if count < 2 {
        "Add at least 2 PDFs"
    } else if !all_ready {
        "Reading files…"
    } else {
        "Ready to merge"
    };

    let preview = previewing().and_then(|i| files.read().get(i).cloned());

    rsx! {
        div { class: "page",
            BackLink {}
            section { class: "hero",
                div {
                    h1 { "Merge PDF" }
                    p { class: "muted", "Combine multiple PDFs into one unified document in your preferred order." }
                }
                button { class: "btn btn-secondary", r#type: "button", disabled: running, onclick: browse,
                    Icon { name: "plus", size: 18 }
                    "Add more files"
                }
            }

            div { class: "tool-layout",
                section {
                    class: "panel drop-target",
                    "aria-labelledby": "files-title",
                    onmouseup: move |_| dragging.set(None),
                    onmouseleave: move |_| dragging.set(None),
                    ondragover: move |e| e.prevent_default(),
                    ondrop: move |e| {
                        e.prevent_default();
                        if !running {
                            add_files(files, notice, e.files().into_iter().map(|f| (f.path(), f.name())).collect());
                        }
                    },
                    div { class: "panel-head",
                        h2 { id: "files-title", "Files to merge" span { class: "count-badge", "{count}" } }
                    }
                    if let Some(msg) = notice() {
                        p { class: "notice", role: "status", "{msg}" }
                    }
                    div { class: "file-grid",
                        for (i, file) in files.read().iter().enumerate() {
                            article {
                                key: "{file.path.display()}",
                                class: card_class(&file.pages, dragging() == Some(i)),
                                tabindex: "0",
                                "aria-label": "{i + 1}. {file.name}. Press Alt and the arrow keys to reorder.",
                                onmousedown: move |_| { if !running { dragging.set(Some(i)) } },
                                onmouseenter: move |_| {
                                    if let Some(from) = dragging() {
                                        if move_file(&mut files.write(), from, i) {
                                            dragging.set(Some(i));
                                        }
                                    }
                                },
                                onkeydown: move |e| {
                                    if running || !e.modifiers().contains(Modifiers::ALT) {
                                        return;
                                    }
                                    let to = match e.key() {
                                        Key::ArrowLeft | Key::ArrowUp => i.checked_sub(1),
                                        Key::ArrowRight | Key::ArrowDown => Some(i + 1),
                                        _ => None,
                                    };
                                    if let Some(to) = to {
                                        e.prevent_default();
                                        move_file(&mut files.write(), i, to);
                                    }
                                },
                                div { class: "file-card-top",
                                    span { class: "file-index", "{i + 1}" }
                                    div { class: "file-actions",
                                        button {
                                            class: "icon-btn sm", r#type: "button",
                                            title: "Rotate left",
                                            "aria-label": "Rotate {file.name} left",
                                            disabled: running,
                                            onclick: move |_| {
                                                if let Some(f) = files.write().get_mut(i) {
                                                    f.rotation = (f.rotation + 270) % 360;
                                                }
                                            },
                                            Icon { name: "rotate-ccw", size: 15 }
                                        }
                                        button {
                                            class: "icon-btn sm", r#type: "button",
                                            title: "Preview",
                                            "aria-label": "Preview {file.name}",
                                            disabled: file.page_count().is_none(),
                                            onclick: move |_| previewing.set(Some(i)),
                                            Icon { name: "eye", size: 15 }
                                        }
                                        button {
                                            class: "icon-btn sm", r#type: "button",
                                            title: "Remove",
                                            "aria-label": "Remove {file.name}",
                                            disabled: running,
                                            onclick: move |_| {
                                                let mut list = files.write();
                                                if i < list.len() { list.remove(i); }
                                            },
                                            Icon { name: "x", size: 15 }
                                        }
                                    }
                                }
                                div { class: "file-thumb",
                                    if let Some(src) = &file.thumb {
                                        img { src: "{src}", alt: "", draggable: "false", style: rotation_style(file.rotation) }
                                    } else {
                                        Icon { name: "file-text", size: 36 }
                                    }
                                }
                                p { class: "file-name", title: "{file.name}", "{file.name}" }
                                p { class: "file-meta muted",
                                    {match &file.pages {
                                        Pages::Counting => "Reading…".to_owned(),
                                        Pages::Ok(n) => format!("{n} pages · {}", format_size(file.size)),
                                        Pages::Unreadable(why) => why.clone(),
                                    }}
                                }
                            }
                        }
                        button { class: "add-tile", r#type: "button", disabled: running, onclick: browse,
                            span { class: "add-tile-icon", Icon { name: "plus", size: 22 } }
                            span { class: "add-tile-title", "Drop more PDFs here" }
                            span { class: "muted", "or click to browse" }
                        }
                    }
                    p { class: "panel-foot muted",
                        "Files will be merged in this order. Drag cards to reorder, or press Alt and the arrow keys."
                    }
                }

                aside { class: "panel settings", "aria-labelledby": "settings-title",
                    div { class: "panel-head",
                        h2 { id: "settings-title", "Merge Settings" }
                        button {
                            class: if extras.is_empty() { "icon-btn" } else { "icon-btn has-dot" },
                            r#type: "button",
                            title: "Advanced settings",
                            "aria-label": "Advanced settings",
                            disabled: running,
                            onclick: move |_| show_advanced.set(true),
                            Icon { name: "sliders-horizontal", size: 18 }
                        }
                    }
                    Toggle { label: "Add bookmarks for each merged file", hint: "Jump between documents easily.", on: bookmarks, disabled: running }
                    Toggle { label: "Maintain original page sizes", hint: "Keep each page exactly as it is. Off: fit all pages to the first page's size.", on: keep_sizes, disabled: running }
                    if !extras.is_empty() {
                        p { class: "hint", "Also: {extras}" }
                    }

                    hr { class: "divider" }
                    p { class: "section-label", "Output" }
                    label { class: "field",
                        span { class: "setting-label", "Output filename" }
                        input {
                            r#type: "text",
                            value: "{out_name}",
                            disabled: running,
                            "aria-invalid": if name_check.is_err() { "true" } else { "false" },
                            oninput: move |e| out_name.set(e.value()),
                        }
                        if let Err(why) = &name_check {
                            span { class: "field-error", "{why}" }
                        } else {
                            span { class: "hint", "Choose where to save after merging." }
                        }
                    }

                    div { class: "settings-spacer" }
                    div { class: "summary",
                        Icon { name: "circle-check", size: 18 }
                        div {
                            p { class: "summary-title", "{summary_title}" }
                            p { class: "muted", "{count} files · {total_pages} total pages · ~{format_size(total_size)}" }
                        }
                    }

                    if running {
                        button {
                            class: "btn btn-secondary btn-block", r#type: "button",
                            onclick: move |_| { if let Some(h) = job.read().as_ref() { h.cancel() } },
                            "Cancel"
                        }
                    } else {
                        button { class: "btn btn-primary btn-block btn-lg", r#type: "button", disabled: !can_merge, onclick: start_merge,
                            "Merge {count} PDFs"
                            Icon { name: "arrow-right", size: 18 }
                        }
                    }

                    div { role: "status", "aria-live": "polite",
                        {match state {
                            Some(JobState::Running { pct, msg }) => rsx! {
                                div { class: "progress", div { class: "progress-fill", style: "width: {pct}%" } }
                                p { class: "job-status muted", "{msg}" }
                            },
                            Some(JobState::Done(result)) => match &*result {
                                Ok(out) => match out.files.first().cloned() {
                                    Some(saved) => rsx! { SavedResult { path: saved } },
                                    None => rsx! {},
                                },
                                Err(e) => rsx! { p { class: "job-status error", "{user_message(e)}" } },
                            },
                            None => rsx! {},
                        }}
                    }
                    p { class: "privacy-note muted",
                        Icon { name: "lock", size: 14 }
                        "Secure, offline processing"
                    }
                }
            }

            if let Some(file) = preview {
                PreviewDialog {
                    key: "{file.path.display()}",
                    path: file.path.clone(),
                    name: file.name.clone(),
                    pages: file.page_count().unwrap_or(0),
                    rotation: file.rotation,
                    on_close: move |_| previewing.set(None),
                }
            }
            if show_advanced() {
                AdvancedDialog { advanced, on_close: move |_| show_advanced.set(false) }
            }
        }
    }
}

/// Success box: file name, its folder, and a button to show it in the file manager.
#[component]
fn SavedResult(path: PathBuf) -> Element {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let folder = path
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    rsx! {
        div { class: "result-card",
            span { class: "result-icon", Icon { name: "circle-check", size: 18 } }
            div { class: "result-text",
                p { class: "summary-title", "Merged successfully" }
                p { class: "result-name", title: "{path.display()}", "{name}" }
                p { class: "result-folder muted", title: "{folder}", "in {folder}" }
            }
        }
        button {
            class: "btn btn-secondary btn-block",
            r#type: "button",
            onclick: move |_| crate::os::reveal(&path),
            Icon { name: "folder-open", size: 16 }
            "Show in folder"
        }
    }
}
