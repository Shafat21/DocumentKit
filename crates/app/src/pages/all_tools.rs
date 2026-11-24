//! All tools: the home screen.

use dioxus::prelude::*;
use dk_core::catalog::{self, matches};
use dk_core::platform::OS_LABEL;
use dk_core::{ModuleId, ToolSpec};

use crate::jobs::blocking;
use crate::router::Route;
use crate::state::{use_search, use_store};
use crate::ui::{EmptyState, Icon, RecentRow, ToolCard};

const RECENT_LIMIT: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Popular,
    Organize,
    Convert,
    EditSign,
    Ai,
    All,
}

impl Tab {
    const ALL: [Tab; 6] = [
        Tab::Popular,
        Tab::Organize,
        Tab::Convert,
        Tab::EditSign,
        Tab::Ai,
        Tab::All,
    ];

    const fn label(self) -> &'static str {
        match self {
            Tab::Popular => "Popular",
            Tab::Organize => "Organize",
            Tab::Convert => "Convert",
            Tab::EditSign => "Edit & sign",
            Tab::Ai => "AI tools",
            Tab::All => "All tools",
        }
    }

    fn tools(self) -> Vec<&'static ToolSpec> {
        let in_modules = |mods: &[ModuleId]| -> Vec<&'static ToolSpec> {
            catalog::TOOLS
                .iter()
                .copied()
                .filter(|t| mods.contains(&t.module))
                .collect()
        };
        match self {
            Tab::Popular => catalog::POPULAR.to_vec(),
            Tab::Organize => in_modules(&[ModuleId::Organize]),
            Tab::Convert => in_modules(&[ModuleId::ConvertTo, ModuleId::ConvertFrom]),
            Tab::EditSign => in_modules(&[ModuleId::Edit, ModuleId::Security]),
            Tab::Ai => in_modules(&[ModuleId::Intelligence]),
            Tab::All => catalog::TOOLS.to_vec(),
        }
    }
}

#[component]
pub fn AllTools() -> Element {
    let mut tab = use_signal(|| Tab::Popular);
    let mut search = use_search();

    // Recomputed only when the tab or the query changes, not on every render.
    let tools = use_memo(move || {
        let query = search.read();
        if query.trim().is_empty() {
            tab().tools()
        } else {
            catalog::TOOLS
                .iter()
                .copied()
                .filter(|t| matches(t, &query))
                .collect()
        }
    });
    let searching = !search.read().trim().is_empty();

    rsx! {
        div { class: "page",
            section { class: "hero",
                div {
                    h1 { "Every document. One toolkit." }
                    p { class: "muted", "Your everyday PDF tools, all in one place." }
                }
                span { class: "pill pill-accent", "Designed for {OS_LABEL}" }
            }

            DropCard {}

            section { class: "section", "aria-labelledby": "essentials-title",
                div { class: "section-head",
                    h2 { id: "essentials-title", "Your PDF essentials" }
                    button {
                        class: "link-btn",
                        r#type: "button",
                        onclick: move |_| {
                            search.set(String::new());
                            tab.set(Tab::All);
                        },
                        "Explore all {catalog::TOOLS.len()} tools"
                    }
                }

                if searching {
                    p { class: "muted", "Results for \u{201c}{search}\u{201d}" }
                } else {
                    div { class: "tabs", role: "tablist", "aria-label": "Tool categories",
                        for t in Tab::ALL {
                            button {
                                key: "{t.label()}",
                                class: if tab() == t { "tab active" } else { "tab" },
                                r#type: "button",
                                role: "tab",
                                "aria-selected": if tab() == t { "true" } else { "false" },
                                onclick: move |_| tab.set(t),
                                "{t.label()}"
                            }
                        }
                    }
                }

                if tools.read().is_empty() {
                    EmptyState {
                        icon: "search",
                        title: "No tools match \u{201c}{search}\u{201d}",
                        body: "Try a shorter word, like \u{201c}merge\u{201d} or \u{201c}word\u{201d}.",
                    }
                } else {
                    div { class: "tool-grid", role: if searching { "list" } else { "tabpanel" },
                        for spec in tools.read().iter().copied() {
                            ToolCard { key: "{spec.id}", spec }
                        }
                    }
                }
            }

            RecentDocuments {}

            footer { class: "page-footer muted",
                span {
                    "A little less paperwork. A lot more possibility. "
                    Link { class: "link-btn", to: Route::Roadmap {}, "See what's coming →" }
                }
                span { "{catalog::TOOLS.len()} tools · {ModuleId::ALL.len()} modules · Local workspace" }
            }
        }
    }
}

#[component]
fn DropCard() -> Element {
    rsx! {
        section { class: "drop-card", "aria-label": "Open files",
            span { class: "drop-icon", Icon { name: "upload", size: 22 } }
            div { class: "drop-text",
                h2 { "Big plans for your PDFs?" }
                p { class: "muted", "Drop your documents here to get started." }
                p { class: "hint", "PDF, Word, Excel, PowerPoint & images" }
            }
            div { class: "drop-actions",
                // Enabled with the first tool engine (file dialog + drag-and-drop).
                button {
                    class: "btn btn-primary",
                    r#type: "button",
                    disabled: true,
                    title: "Coming soon",
                    Icon { name: "plus", size: 18 }
                    "Open files"
                }
                span { class: "hint", "or drag and drop anywhere" }
            }
        }
    }
}

#[component]
fn RecentDocuments() -> Element {
    let store = use_store();
    // Database read runs off the UI thread.
    let recent = use_resource(move || {
        let store = store.clone();
        async move { blocking(move || store.recent_documents(RECENT_LIMIT)).await }
    });

    let has_recent = matches!(&*recent.read(), Some(Some(Ok(docs))) if !docs.is_empty());
    let body = match &*recent.read() {
        None => rsx! { p { class: "muted", "Loading…" } },
        Some(Some(Ok(docs))) if !docs.is_empty() => rsx! {
            ul { class: "recent-list",
                for doc in docs.iter() {
                    RecentRow { key: "{doc.path}", doc: doc.clone() }
                }
            }
        },
        Some(Some(Ok(_))) => rsx! {
            EmptyState {
                icon: "clock",
                title: "No recent documents yet",
                body: "Files you open with any tool will show up here.",
            }
        },
        Some(Some(Err(e))) => {
            tracing::warn!("recent documents: {e}");
            rsx! { p { class: "muted", "Recent documents are unavailable right now." } }
        }
        Some(None) => rsx! { p { class: "muted", "Recent documents are unavailable right now." } },
    };

    rsx! {
        section { class: "section", "aria-labelledby": "recent-title",
            div { class: "section-head",
                h2 { id: "recent-title", "Pick up where you left off" }
                if has_recent {
                    Link { class: "link-btn", to: Route::Recent {}, "View all →" }
                }
            }
            {body}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_reachable_from_all_tab() {
        assert_eq!(Tab::All.tools().len(), catalog::TOOLS.len());
    }

    /// Renders the page headless (no window) and checks the catalog reaches the UI.
    #[test]
    fn renders_popular_tools() {
        use std::sync::Arc;

        use crate::state::{AppCtx, SearchCtx, ThemeCtx};
        use dk_core::store::Store;

        // Full shell through the router (memory history starts at "/").
        fn harness() -> Element {
            use_context_provider(SearchCtx::new);
            use_context_provider(|| ThemeCtx::new(dk_core::Theme::Light));
            rsx! { Router::<Route> {} }
        }

        let ctx = AppCtx {
            store: Arc::new(Store::in_memory().unwrap()),
            initial_theme: dk_core::Theme::Light,
        };
        let mut dom = VirtualDom::new(harness).with_root_context(ctx);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("Every document. One toolkit."));
        assert!(html.contains("data-theme=\"light\""));
        assert!(html.contains("Development roadmap"));
        for tool in catalog::POPULAR {
            assert!(html.contains(tool.name), "missing {}", tool.name);
        }
    }
}
