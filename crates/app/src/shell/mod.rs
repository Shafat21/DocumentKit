//! Persistent app chrome: sidebar + top bar around the routed page.

mod sidebar;
mod topbar;

use dioxus::prelude::*;

use crate::router::Route;
use crate::state::theme::use_theme;
use sidebar::Sidebar;
use topbar::Topbar;

/// Layout for every route. Owns `data-theme`, so a theme change restyles everything.
#[component]
pub fn Shell() -> Element {
    let theme = use_theme().get();
    rsx! {
        div { class: "app", "data-theme": theme.as_str(),
            a { class: "skip-link", href: "#content", "Skip to content" }
            Sidebar {}
            div { class: "main",
                Topbar {}
                main { class: "content", id: "content", tabindex: "-1",
                    Outlet::<Route> {}
                }
            }
        }
    }
}
