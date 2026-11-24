//! Placeholder for screens that aren't built yet, and the 404 fallback.

use dioxus::prelude::*;

use crate::ui::EmptyState;

fn title_for(segments: &[String]) -> &'static str {
    match segments {
        [page] => match page.as_str() {
            "favorites" => "Favorites",
            "settings" => "Settings",
            _ => "Not found",
        },
        _ => "Not found",
    }
}

#[component]
pub fn ComingSoon(segments: Vec<String>) -> Element {
    let title = title_for(&segments);
    let body = if title == "Not found" {
        "This page doesn't exist."
    } else {
        "This part of DocumentKit is being built. Check the development roadmap for progress."
    };
    rsx! {
        div { class: "page",
            section { class: "hero",
                div { h1 { "{title}" } }
                span { class: "pill", "Coming soon" }
            }
            EmptyState { icon: "construction", title: "{title}", body: "{body}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segs(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn titles() {
        assert_eq!(title_for(&segs(&["settings"])), "Settings");
        assert_eq!(title_for(&segs(&["nope", "x"])), "Not found");
        assert_eq!(title_for(&segs(&[])), "Not found");
    }
}
