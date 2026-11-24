use dioxus::prelude::*;

use super::about::AboutDialog;
use crate::router::Route;
use crate::state::theme::use_theme;
use crate::state::{use_search, use_store};
use crate::ui::Icon;

#[component]
pub fn Topbar() -> Element {
    let mut search = use_search();
    let theme = use_theme();
    let store = use_store();
    let route = use_route::<Route>();
    let nav = navigator();
    let mut show_about = use_signal(|| false);

    let current = theme.get();
    let theme_icon = match current {
        dk_core::Theme::Light => "sun",
        dk_core::Theme::Dark => "moon",
        dk_core::Theme::System => "monitor",
    };
    let theme_label = format!("{} (click to switch)", current.label());

    rsx! {
        header { class: "topbar",
            label { class: "search",
                Icon { name: "search", size: 16 }
                input {
                    r#type: "search",
                    placeholder: "Search tools, e.g. \u{201c}compress PDF\u{201d}",
                    "aria-label": "Search tools",
                    value: "{search}",
                    oninput: move |e| {
                        search.set(e.value());
                        // Results live on the All tools page; jump there while typing.
                        if !matches!(route, Route::AllTools {}) {
                            nav.push(Route::AllTools {});
                        }
                    },
                }
            }
            div { class: "topbar-actions",
                span { class: "workspace-pill",
                    Icon { name: "hard-drive", size: 16 }
                    span { "Local workspace" }
                }
                button {
                    class: "icon-btn",
                    r#type: "button",
                    title: "{theme_label}",
                    "aria-label": "{theme_label}",
                    onclick: move |_| theme.set(current.next(), &store),
                    Icon { name: theme_icon, size: 18 }
                }
                button {
                    class: "icon-btn",
                    r#type: "button",
                    title: "About DocumentKit",
                    "aria-label": "About DocumentKit",
                    onclick: move |_| show_about.set(true),
                    Icon { name: "circle-help", size: 18 }
                }
            }
        }
        if show_about() {
            AboutDialog { on_close: move |_| show_about.set(false) }
        }
    }
}
