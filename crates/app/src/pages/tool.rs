//! /tool/:id — sends each tool id to its page in `features/`, or a placeholder.

use dioxus::prelude::*;
use dk_core::catalog;

use crate::features::organize::MergePdf;
use crate::router::Route;
use crate::ui::{EmptyState, Icon};

#[component]
pub fn ToolPage(id: String) -> Element {
    match id.as_str() {
        "merge-pdf" => rsx! { MergePdf {} },
        _ => {
            let (title, body) = match catalog::by_id(&id) {
                Some(spec) => (
                    spec.name,
                    "This tool is being built. Check the development roadmap for progress.",
                ),
                None => ("Not found", "This tool doesn't exist."),
            };
            rsx! {
                div { class: "page",
                    BackLink {}
                    section { class: "hero",
                        div { h1 { "{title}" } }
                        span { class: "pill", "Coming soon" }
                    }
                    EmptyState { icon: "construction", title: "{title}", body: "{body}" }
                }
            }
        }
    }
}

/// "← All tools" link shown at the top of tool pages.
#[component]
pub fn BackLink() -> Element {
    rsx! {
        Link { class: "back-link", to: Route::AllTools {},
            Icon { name: "arrow-left", size: 16 }
            "All tools"
        }
    }
}
