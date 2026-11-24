//! Recent documents: every file DocumentKit produced, newest first.

use std::path::Path;

use dioxus::prelude::*;
use dk_core::store::{RecentDoc, MAX_RECENT};

use crate::jobs::blocking;
use crate::router::Route;
use crate::state::use_store;
use crate::ui::{EmptyState, Icon, Modal, RecentRow};

#[component]
pub fn Recent() -> Element {
    let store = use_store();
    // Bumped after remove/clear to reload the list.
    let mut refresh = use_signal(|| 0u32);
    let mut query = use_signal(String::new);
    let mut confirm_clear = use_signal(|| false);

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

    let remove_store = store.clone();
    // A Callback is Copy, so every row can hold it.
    let on_remove = use_callback(move |path: String| {
        let store = remove_store.clone();
        spawn(async move {
            if let Some(Err(e)) = blocking(move || store.remove_recent(&path)).await {
                tracing::warn!("could not remove recent document: {e}");
            }
            refresh += 1;
        });
    });

    let clear = move |_| {
        let store = store.clone();
        confirm_clear.set(false);
        spawn(async move {
            if let Some(Err(e)) = blocking(move || store.clear_recent()).await {
                tracing::warn!("could not clear recent documents: {e}");
            }
            refresh += 1;
        });
    };

    let loaded = match &*docs.read() {
        Some(Some(Ok(list))) => Some(list.clone()),
        Some(Some(Err(e))) => {
            tracing::warn!("recent documents: {e}");
            None
        }
        _ => None,
    };
    let failed = matches!(&*docs.read(), Some(Some(Err(_)) | None));
    let total = loaded.as_ref().map_or(0, Vec::len);
    let needle = query.read().trim().to_lowercase();
    let shown: Vec<(RecentDoc, bool)> = loaded
        .unwrap_or_default()
        .into_iter()
        .filter(|(d, _)| needle.is_empty() || d.name.to_lowercase().contains(&needle))
        .collect();

    let body = if docs.read().is_none() {
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
    } else if shown.is_empty() {
        rsx! {
            EmptyState {
                icon: "search",
                title: "No documents match \u{201c}{query}\u{201d}",
                body: "Try part of the file name.",
            }
        }
    } else {
        rsx! {
            ul { class: "recent-list",
                for (doc, missing) in shown {
                    RecentRow { key: "{doc.path}", doc, missing, on_remove }
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
                        " · newest first · kept on this device only"
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
                label { class: "search page-search",
                    Icon { name: "search", size: 16 }
                    input {
                        r#type: "search",
                        placeholder: "Search recent documents",
                        "aria-label": "Search recent documents",
                        value: "{query}",
                        oninput: move |e| query.set(e.value()),
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
