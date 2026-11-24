//! About / help dialog: version, links, and the people behind DocumentKit.

use std::path::Path;

use dioxus::prelude::*;
use dk_core::platform::OS_LABEL;

use crate::ui::{Icon, Modal};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LOGO_SVG: &str = include_str!("../../assets/logo.svg");

/// Main contributors, as listed in the README's Team section. Keep the two in sync.
const TEAM: [(&str, &str); 5] = [
    ("Shafat", "Lead Core Coding"),
    ("Nahin", "Core Coding"),
    ("Joynur", "UI / UX"),
    ("Ove", "UI / UX"),
    ("Samin", "QA Testing"),
];

/// Links open in the default browser. Only the fixed URLs below are ever passed.
const LINKS: [(&str, &str, &str); 3] = [
    (
        "Source code",
        "code",
        "https://github.com/Shafat21/DocumentKit",
    ),
    (
        "Report a problem",
        "circle-alert",
        "https://github.com/Shafat21/DocumentKit/issues/new/choose",
    ),
    (
        "License (GPL-3.0)",
        "scale",
        "https://github.com/Shafat21/DocumentKit/blob/main/LICENSE",
    ),
];

#[component]
pub fn AboutDialog(on_close: EventHandler<()>) -> Element {
    rsx! {
        Modal { title: "About DocumentKit", on_close,
            div { class: "about-head",
                span { class: "about-logo", "aria-hidden": "true", dangerous_inner_html: LOGO_SVG }
                div {
                    p { class: "about-name", "Document" span { class: "brand-kit", "Kit" } }
                    p { class: "muted", "Every PDF tool you need. One fast, private, open-source app." }
                    p { class: "hint", "Version {VERSION} · {OS_LABEL} · Your files never leave this device." }
                }
            }

            p { class: "section-label", "Main contributors" }
            ul { class: "team-list",
                for (name, role) in TEAM {
                    li { key: "{name}", class: "team-member",
                        span { class: "avatar", "aria-hidden": "true", "{&name[..1]}" }
                        span { class: "team-text",
                            span { class: "team-name", "{name}" }
                            span { class: "muted", "{role}" }
                        }
                    }
                }
            }

            p { class: "section-label", "Links" }
            div { class: "about-links",
                for (label, icon, url) in LINKS {
                    button {
                        key: "{label}",
                        class: "btn btn-secondary",
                        r#type: "button",
                        title: "{url}",
                        onclick: move |_| crate::os::open(Path::new(url)),
                        Icon { name: icon, size: 16 }
                        "{label}"
                        Icon { name: "external-link", size: 14 }
                    }
                }
            }
        }
    }
}
