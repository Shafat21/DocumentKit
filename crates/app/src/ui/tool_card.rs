use dioxus::prelude::*;
use dk_core::ToolSpec;

use super::Icon;
use crate::router::Route;

/// Card for one tool; opens the tool's page.
#[component]
pub fn ToolCard(spec: &'static ToolSpec) -> Element {
    rsx! {
        Link {
            class: "tool-card",
            to: Route::Tool { id: spec.id.to_owned() },
            span { class: "tool-icon", Icon { name: spec.icon } }
            span { class: "tool-text",
                span { class: "tool-name", "{spec.name}" }
                span { class: "tool-desc", "{spec.desc}" }
            }
            span { class: "tool-chevron", Icon { name: "chevron-right", size: 16 } }
        }
    }
}
