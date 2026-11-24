//! Recent documents: every file DocumentKit produced, as a preview grid or a list.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use dioxus::prelude::*;
use dk_core::store::{RecentDoc, MAX_RECENT};

use crate::jobs::blocking;
use crate::router::Route;
use crate::state::use_store;
use crate::ui::{
    format_size, group_label, now_secs, EmptyState, Icon, Modal, RecentCard, RecentRow, Select,
};

/// Settings key for the remembered view ("grid" or "list").
const VIEW_KEY: &str = "recent_view";
/// Previews rendered per page visit; keeps CPU and memory bounded on long histories.
const MAX_THUMBS: usize = 60;

#[derive(Clone, Copy, PartialEq)]
enum Sort {
    Newest,
    Oldest,
    Name,
    Size,
}

impl Sort {
    const ALL: [(Sort, &'static str, &'static str); 4] = [
        (Sort::Newest, "newest", "Newest first"),
        (Sort::Oldest, "oldest", "Oldest first"),
        (Sort::Name, "name", "Name (A–Z)"),
        (Sort::Size, "size", "Largest first"),
    ];

    fn from_key(key: &str) -> Self {
        Self::ALL
            .iter()
            .find(|(_, k, _)| *k == key)
            .map_or(Sort::Newest, |(s, _, _)| *s)
    }

    /// Only date sorts are grouped under Today / This week / Earlier.
    fn grouped(self) -> bool {
        matches!(self, Sort::Newest | Sort::Oldest)
    }
}

/// Filter by name and sort. Returns `(doc, missing)` pairs.
fn arrange(list: &[(RecentDoc, bool)], query: &str, sort: Sort) -> Vec<(RecentDoc, bool)> {
    let needle = query.trim().to_lowercase();
    let mut out: Vec<_> = list
        .iter()
        .filter(|(d, _)| needle.is_empty() || d.name.to_lowercase().contains(&needle))
        .cloned()
        .collect();
    match sort {
        Sort::Newest => {} // already newest first from the store
        Sort::Oldest => out.reverse(),
        Sort::Name => out.sort_by_key(|(d, _)| d.name.to_lowercase()),
        Sort::Size => out.sort_by_key(|(d, _)| std::cmp::Reverse(d.size_bytes)),
    }
    out
}

/// Split into consecutive groups by recency label (input must be date-sorted).
fn group(items: Vec<(RecentDoc, bool)>, now: u64) -> Vec<(&'static str, Vec<(RecentDoc, bool)>)> {
    let mut groups: Vec<(&'static str, Vec<(RecentDoc, bool)>)> = Vec::new();
    for item in items {
        let label = group_label(&item.0.opened_at, now);
        match groups.last_mut() {
            Some((last, docs)) if *last == label => docs.push(item),
            _ => groups.push((label, vec![item])),
        }
    }
    groups
}

#[component]
pub fn Recent() -> Element {
    let store = use_store();
    // Bumped after remove/clear to reload the list.
    let mut refresh = use_signal(|| 0u32);
    let mut query = use_signal(String::new);
    let sort_key = use_signal(|| "newest".to_owned());
    let mut confirm_clear = use_signal(|| false);
    let mut thumbs = use_signal(HashMap::<String, Option<String>>::new);

    let view_store = store.clone();
    let mut grid = use_signal(move || {
        view_store.get_setting(VIEW_KEY).ok().flatten().as_deref() != Some("list")
    });

    let load_store = store.clone();
    let docs = use_resource(move || {
        let _ = refresh();
        let store = load_store.clone();
        // One background pass: read the list and check each file still exists.
        async move {
            blocking(move || {
                store.recent_documents(MAX_RECENT).map(|docs| {
                    docs.into_iter()
                        .map(|d| {
                            let missing = !Path::new(&d.path).exists();
                            (d, missing)
                        })
                        .collect::<Vec<(RecentDoc, bool)>>()
                })
            })
            .await
        }
    });

    // Render previews for files that still exist, one at a time, newest first.
    use_effect(move || {
        let Some(Some(Ok(list))) = &*docs.read() else {
            return;
        };
        let pending: Vec<PathBuf> = list
            .iter()
            .filter(|(d, missing)| !missing && !thumbs.peek().contains_key(&d.path))
            .take(MAX_THUMBS)
            .map(|(d, _)| PathBuf::from(&d.path))
            .collect();
        spawn(async move {
            for path in pending {
                let p = path.clone();
                let thumb =
                    blocking(move || crate::thumbs::first_page(&p, crate::thumbs::CARD_WIDTH))
                        .await
                        .flatten();
                thumbs
                    .write()
                    .insert(path.to_string_lossy().into_owned(), thumb);
            }
        });
    });

    let remove_store = store.clone();
    // A Callback is Copy, so every row and card can hold it.
    let on_remove = use_callback(move |path: String| {
        let store = remove_store.clone();
        spawn(async move {
            if let Some(Err(e)) = blocking(move || store.remove_recent(&path)).await {
                tracing::warn!("could not remove recent document: {e}");
            }
            refresh += 1;
        });
    });

    let clear_store = store.clone();
    let clear = move |_| {
        let store = clear_store.clone();
        confirm_clear.set(false);
        spawn(async move {
            if let Some(Err(e)) = blocking(move || store.clear_recent()).await {
                tracing::warn!("could not clear recent documents: {e}");
            }
            refresh += 1;
        });
    };

    let set_view = use_callback(move |as_grid: bool| {
        grid.set(as_grid);
        let store = store.clone();
        spawn(async move {
            let value = if as_grid { "grid" } else { "list" };
            if let Some(Err(e)) = blocking(move || store.set_setting(VIEW_KEY, value)).await {
                tracing::warn!("could not save view: {e}");
            }
        });
    });

    let state = docs.read();
    let list: &[(RecentDoc, bool)] = match &*state {
        Some(Some(Ok(list))) => list,
        _ => &[],
    };
    let failed = matches!(&*state, Some(Some(Err(_)) | None));
    let total = list.len();
    let total_size: u64 = list.iter().map(|(d, _)| d.size_bytes).sum();
    let missing_count = list.iter().filter(|(_, m)| *m).count();
    let sort = Sort::from_key(&sort_key.read());
    let shown = arrange(list, &query.read(), sort);
    let groups = if sort.grouped() {
        group(shown, now_secs())
    } else {
        vec![("", shown)]
    };
    let nothing_shown = groups.iter().all(|(_, g)| g.is_empty());
    let loading = state.is_none();
    drop(state);

    let sort_options: Vec<(String, String)> = Sort::ALL
        .iter()
        .map(|(_, k, label)| ((*k).to_owned(), (*label).to_owned()))
        .collect();

    let body = if loading {
        rsx! { p { class: "muted", "Loading…" } }
    } else if failed {
        rsx! { p { class: "muted", "Recent documents are unavailable right now." } }
    } else if total == 0 {
        rsx! {
            EmptyState {
                icon: "clock",
                title: "No recent documents yet",
                body: "Files you create with any tool will show up here.",
            }
            Link { class: "link-btn center", to: Route::AllTools {}, "Browse all tools →" }
        }
    } else if nothing_shown {
        rsx! {
            EmptyState {
                icon: "search",
                title: "No documents match \u{201c}{query}\u{201d}",
                body: "Try part of the file name.",
            }
        }
    } else {
        rsx! {
            for (label, items) in groups {
                section { key: "{label}", class: "doc-group",
                    if !label.is_empty() {
                        h2 { class: "group-title", "{label}" span { class: "count-badge", "{items.len()}" } }
                    }
                    if grid() {
                        div { class: "doc-grid",
                            for (doc, missing) in items {
                                RecentCard {
                                    key: "{doc.path}",
                                    thumb: thumbs.read().get(&doc.path).cloned().flatten(),
                                    doc,
                                    missing,
                                    on_remove,
                                }
                            }
                        }
                    } else {
                        ul { class: "recent-list",
                            for (doc, missing) in items {
                                RecentRow { key: "{doc.path}", doc, missing, on_remove }
                            }
                        }
                    }
                }
            }
        }
    };

    rsx! {
        div { class: "page",
            section { class: "hero",
                div {
                    h1 { "Recent documents" }
                    p { class: "muted",
                        if total == 1 { "1 document" } else { "{total} documents" }
                        " · {format_size(total_size)}"
                        if missing_count > 0 { " · {missing_count} not found" }
                        " · kept on this device only"
                    }
                }
                if total > 0 {
                    button {
                        class: "btn btn-secondary",
                        r#type: "button",
                        onclick: move |_| confirm_clear.set(true),
                        Icon { name: "trash", size: 16 }
                        "Clear history"
                    }
                }
            }

            if total > 0 {
                div { class: "toolbar",
                    label { class: "search toolbar-search",
                        Icon { name: "search", size: 16 }
                        input {
                            r#type: "search",
                            placeholder: "Search by file name",
                            "aria-label": "Search recent documents",
                            value: "{query}",
                            oninput: move |e| query.set(e.value()),
                        }
                    }
                    div { class: "toolbar-right",
                        Select { label: "Sort documents", options: sort_options, value: sort_key }
                        div { class: "segmented", role: "group", "aria-label": "View",
                            button {
                                class: if grid() { "seg active" } else { "seg" },
                                r#type: "button",
                                "data-tip": "Grid view",
                                "data-tip-align": "end",
                                "aria-label": "Grid view",
                                "aria-pressed": if grid() { "true" } else { "false" },
                                onclick: move |_| set_view.call(true),
                                Icon { name: "layout-grid", size: 16 }
                            }
                            button {
                                class: if grid() { "seg" } else { "seg active" },
                                r#type: "button",
                                "data-tip": "List view",
                                "data-tip-align": "end",
                                "aria-label": "List view",
                                "aria-pressed": if grid() { "false" } else { "true" },
                                onclick: move |_| set_view.call(false),
                                Icon { name: "layout-list", size: 16 }
                            }
                        }
                    }
                }
            }

            {body}

            if confirm_clear() {
                Modal { title: "Clear history?", on_close: move |_| confirm_clear.set(false),
                    p { "This removes all {total} documents from this list. The files themselves are not deleted." }
                    div { class: "modal-actions",
                        button {
                            class: "btn btn-secondary",
                            r#type: "button",
                            onclick: move |_| confirm_clear.set(false),
                            "Cancel"
                        }
                        button { class: "btn btn-primary", r#type: "button", onclick: clear, "Clear history" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(name: &str, size: u64, opened_at: &str) -> (RecentDoc, bool) {
        let d = RecentDoc {
            path: format!("/{name}"),
            name: name.to_owned(),
            pages: 1,
            size_bytes: size,
            opened_at: opened_at.to_owned(),
        };
        (d, false)
    }

    fn names(items: &[(RecentDoc, bool)]) -> Vec<&str> {
        items.iter().map(|(d, _)| d.name.as_str()).collect()
    }

    #[test]
    fn filters_and_sorts() {
        let list = [
            doc("b.pdf", 30, "2026-11-24T10:00:00Z"),
            doc("A report.pdf", 10, "2026-11-23T10:00:00Z"),
            doc("c.pdf", 20, "2026-11-01T10:00:00Z"),
        ];
        assert_eq!(
            names(&arrange(&list, "", Sort::Newest)),
            ["b.pdf", "A report.pdf", "c.pdf"]
        );
        assert_eq!(
            names(&arrange(&list, "", Sort::Oldest)),
            ["c.pdf", "A report.pdf", "b.pdf"]
        );
        assert_eq!(
            names(&arrange(&list, "", Sort::Name)),
            ["A report.pdf", "b.pdf", "c.pdf"]
        );
        assert_eq!(
            names(&arrange(&list, "", Sort::Size)),
            ["b.pdf", "c.pdf", "A report.pdf"]
        );
        assert_eq!(
            names(&arrange(&list, " REPORT ", Sort::Newest)),
            ["A report.pdf"]
        );
    }

    #[test]
    fn groups_consecutive_labels() {
        let now = 1_795_514_400 + 3_600; // 2026-11-24T11:00:00Z
        let list = vec![
            doc("a", 1, "2026-11-24T10:00:00Z"),
            doc("b", 1, "2026-11-21T10:00:00Z"),
            doc("c", 1, "2026-10-01T10:00:00Z"),
        ];
        let labels: Vec<_> = group(list, now)
            .into_iter()
            .map(|(l, g)| (l, g.len()))
            .collect();
        assert_eq!(labels, [("Today", 1), ("This week", 1), ("Earlier", 1)]);
    }
}
