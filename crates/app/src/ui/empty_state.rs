use dioxus::prelude::*;

use super::Icon;

/// Centered icon + title + body, for pages and lists with nothing to show yet.
#[component]
pub fn EmptyState(icon: &'static str, title: String, body: String) -> Element {
    rsx! {
        div { class: "empty-state",
            span { class: "empty-icon", Icon { name: icon, size: 28 } }
            p { class: "empty-title", "{title}" }
            p { class: "empty-body", "{body}" }
        }
    }
}
