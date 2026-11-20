use dioxus::prelude::*;
use dk_core::ToolSpec;

use super::Icon;

/// Card for one tool in the All tools grid.
#[component]
pub fn ToolCard(spec: &'static ToolSpec) -> Element {
    rsx! {
        // Tool pages land with each module; until then the card is inert.
        button {
            class: "tool-card",
            r#type: "button",
            title: "Coming soon",
            "aria-label": "{spec.name}: {spec.desc} (coming soon)",
            span { class: "tool-icon", Icon { name: spec.icon } }
            span { class: "tool-text",
                span { class: "tool-name", "{spec.name}" }
                span { class: "tool-desc", "{spec.desc}" }
            }
            span { class: "tool-chevron", Icon { name: "chevron-right", size: 16 } }
        }
    }
}
