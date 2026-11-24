//! Development roadmap: live from `roadmap.json` on GitHub.
//!
//! Shows the cached (or built-in) copy instantly, then syncs in the background.
//! Network and DB work run on worker threads via `jobs::blocking`.

use std::sync::Arc;

use dioxus::prelude::*;
use dk_core::roadmap::{RoadmapModule, Status, Task};
use dk_core::store::Store;
use dk_core::ModuleId;

use crate::jobs::blocking;
use crate::roadmap_source::{self, format_timestamp, Loaded, Origin, FILE_URL};
use crate::state::use_store;
use crate::ui::{EmptyState, Icon, Select};

const ALL_MODULES: &str = "all";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncState {
    Loading,
    Syncing,
    Synced,
    Offline,
}

/// Show local data first (cache, else built-in), then try GitHub.
async fn refresh(
    store: Arc<Store>,
    mut view: Signal<Option<Loaded>>,
    mut state: Signal<SyncState>,
) {
    if view.peek().is_none() {
        let s = Arc::clone(&store);
        let local = blocking(move || roadmap_source::cached(&s)).await.flatten();
        view.set(Some(local.unwrap_or_else(roadmap_source::bundled)));
    }
    state.set(SyncState::Syncing);
    match blocking(move || roadmap_source::fetch_and_cache(&store)).await {
        Some(Ok(live)) => {
            view.set(Some(live));
            state.set(SyncState::Synced);
        }
        Some(Err(e)) => {
            tracing::warn!("roadmap sync failed: {e}");
            state.set(SyncState::Offline);
        }
        None => state.set(SyncState::Offline),
    }
}

#[component]
pub fn Roadmap() -> Element {
    let store = use_store();
    let view = use_signal(|| None::<Loaded>);
    let state = use_signal(|| SyncState::Loading);
    let filter = use_signal(|| ALL_MODULES.to_owned());

    let first_store = Arc::clone(&store);
    use_future(move || refresh(Arc::clone(&first_store), view, state));

    // Clone the Arc out so the signal isn't borrowed while rendering.
    let Some(Loaded { roadmap, origin }) = view.read().clone() else {
        return rsx! {
            div { class: "page", p { class: "muted", "Loading roadmap…" } }
        };
    };

    let stats = roadmap.stats();
    let syncing = matches!(state(), SyncState::Loading | SyncState::Syncing);
    let offline = state() == SyncState::Offline;
    let sync_text = match (state(), &origin) {
        (SyncState::Loading | SyncState::Syncing, _) => "Syncing with GitHub…".to_owned(),
        (_, Origin::Live { fetched_at }) => format!("Synced {}", format_timestamp(fetched_at)),
        (_, Origin::Cached { fetched_at }) => {
            format!(
                "Offline · showing copy from {}",
                format_timestamp(fetched_at)
            )
        }
        (_, Origin::Bundled) => "Offline · showing built-in copy".to_owned(),
    };

    let selected = filter();
    let filter_options: Vec<(String, String)> =
        std::iter::once((ALL_MODULES.to_owned(), "All modules".to_owned()))
            .chain(
                roadmap
                    .modules
                    .iter()
                    .map(|m| (m.key.clone(), m.name.clone())),
            )
            .collect();
    let modules: Vec<&RoadmapModule> = roadmap
        .modules
        .iter()
        .filter(|m| selected == ALL_MODULES || m.key == selected)
        .collect();

    rsx! {
        div { class: "page",
            section { class: "hero",
                div {
                    h1 { "Development roadmap" }
                    p { class: "muted",
                        "One toolkit. {stats.modules} modules. A clear path from idea to implementation."
                    }
                }
                if !roadmap.phase.is_empty() {
                    span { class: "pill pill-accent",
                        span { class: "dot", "aria-hidden": "true" }
                        "{roadmap.phase}"
                    }
                }
            }

            div { class: "sync-bar", role: "status", "aria-live": "polite",
                span { class: if offline { "sync-text offline" } else { "sync-text" },
                    if offline { Icon { name: "cloud-off", size: 16 } }
                    "{sync_text}"
                }
                div { class: "sync-actions",
                    a {
                        class: "link-btn",
                        href: FILE_URL,
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "Edit on GitHub"
                        Icon { name: "external-link", size: 14 }
                    }
                    button {
                        class: "btn btn-secondary",
                        r#type: "button",
                        disabled: syncing,
                        onclick: move |_| {
                            spawn(refresh(Arc::clone(&store), view, state));
                        },
                        span { class: if syncing { "spin" } else { "" }, Icon { name: "refresh-cw", size: 16 } }
                        if syncing { "Syncing…" } else { "Refresh" }
                    }
                }
            }

            div { class: "stat-grid",
                Stat { value: stats.planned, label: "Features planned", hint: "The complete feature set to build" }
                Stat { value: stats.modules, label: "Focused modules", hint: "From organize to intelligence" }
                Stat { value: stats.done, label: "Features completed", hint: "{stats.in_progress} in progress right now" }
            }

            section { class: "section", "aria-labelledby": "checklist-title",
                div { class: "section-head",
                    h2 { id: "checklist-title", "Feature implementation checklist" }
                    Select { label: "Filter by module", options: filter_options, value: filter }
                }

                if modules.is_empty() {
                    EmptyState {
                        icon: "search",
                        title: "No modules to show",
                        body: "Pick another module from the filter.",
                    }
                } else {
                    div { class: "module-grid",
                        for m in modules {
                            ModuleCard { key: "{m.key}", module: m.clone() }
                        }
                    }
                }
            }

            footer { class: "page-footer muted",
                span { "Edit roadmap.json on GitHub to update this checklist for everyone." }
                if !roadmap.updated.is_empty() {
                    span { "Roadmap last edited {roadmap.updated}" }
                }
            }
        }
    }
}

#[component]
fn Stat(value: usize, label: &'static str, hint: String) -> Element {
    let shown = format!("{value:02}");
    rsx! {
        div { class: "stat",
            span { class: "stat-value", "{shown}" }
            span { class: "stat-label", "{label}" }
            span { class: "stat-hint", "{hint}" }
        }
    }
}

#[component]
fn ModuleCard(module: RoadmapModule) -> Element {
    let total = module.tasks.len();
    let done = module.done();
    let pct = (done * 100).checked_div(total).unwrap_or(0);
    let icon = ModuleId::ALL
        .iter()
        .find(|m| m.key() == module.key)
        .map_or("layers", |m| m.icon());

    rsx! {
        article { class: "module-card",
            header { class: "module-head",
                span { class: "tool-icon", Icon { name: icon } }
                div { class: "module-title",
                    h3 { "{module.name}" }
                    span { class: "muted", "{total} features" }
                }
            }
            ul { class: "task-list",
                for task in module.tasks.iter() {
                    TaskRow { key: "{task.id}", task: task.clone() }
                }
            }
            footer { class: "module-foot",
                div {
                    class: "progress",
                    role: "progressbar",
                    "aria-label": "{module.name} progress",
                    "aria-valuemin": "0",
                    "aria-valuemax": "{total}",
                    "aria-valuenow": "{done}",
                    div { class: "progress-fill", style: "width: {pct}%" }
                }
                span { class: "muted", "{done} of {total} completed" }
            }
        }
    }
}

#[component]
fn TaskRow(task: Task) -> Element {
    let (icon, class) = match task.status {
        Status::Done => ("square-check", "task done"),
        Status::InProgress => ("circle-dashed", "task in-progress"),
        Status::Pending => ("square", "task"),
    };
    let assignee = task.assignee.as_deref().unwrap_or("Unassigned");
    let assignee_class = if task.assignee.is_some() {
        "task-assignee"
    } else {
        "task-assignee unassigned"
    };
    let status = task.status.label();

    rsx! {
        li { class,
            span { class: "task-status", Icon { name: icon, size: 16 } }
            span { class: "task-name", "{task.name}" }
            span { class: "sr-only", "({status})" }
            span { class: assignee_class, "{assignee}" }
        }
    }
}
