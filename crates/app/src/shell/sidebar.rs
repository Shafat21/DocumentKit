use dioxus::prelude::*;
use dk_core::platform::OS_LABEL;
use dk_core::{catalog, ModuleId};

use crate::router::Route;
use crate::ui::Icon;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LOGO_SVG: &str = include_str!("../../assets/logo.svg");

fn coming_soon(path: &[&str]) -> Route {
    Route::ComingSoon {
        segments: path.iter().map(|s| (*s).to_owned()).collect(),
    }
}

#[component]
pub fn Sidebar() -> Element {
    // A module stays highlighted on its own page and on each of its tools' pages.
    let route = use_route::<Route>();
    let active_module = match &route {
        Route::Module { module } => Some(module.as_str()),
        Route::Tool { id } => catalog::by_id(id).map(|t| t.module.key()),
        _ => None,
    };

    rsx! {
        nav { class: "sidebar", "aria-label": "Main",
            div { class: "brand",
                // Trusted, compile-time SVG from our own assets.
                span { class: "brand-mark", "aria-hidden": "true", dangerous_inner_html: LOGO_SVG }
                span { class: "brand-text",
                    span { class: "brand-name", "Document" span { class: "brand-kit", "Kit" } }
                    span { class: "brand-tagline", "PDF, without the friction" }
                }
            }

            NavItem { to: Route::AllTools {}, icon: "layout-grid", label: "All tools" }
            NavItem { to: coming_soon(&["recent"]), icon: "clock", label: "Recent documents" }
            NavItem { to: coming_soon(&["favorites"]), icon: "star", label: "Favorites" }

            p { class: "nav-heading", "Workspace" }
            for module in ModuleId::ALL {
                NavItem {
                    key: "{module.key()}",
                    to: Route::Module { module: module.key().to_owned() },
                    icon: module.icon(),
                    label: module.name(),
                    active: active_module == Some(module.key()),
                }
            }

            hr { class: "nav-divider" }
            NavItem { to: Route::Roadmap {}, icon: "map", label: "Development roadmap" }

            div { class: "sidebar-spacer" }

            div { class: "privacy-card",
                span { class: "privacy-icon", Icon { name: "shield-check", size: 18 } }
                span {
                    span { class: "privacy-title", "Your files. Your device." }
                    span { class: "privacy-body", "A workspace built for {OS_LABEL}." }
                }
            }
            NavItem { to: coming_soon(&["settings"]), icon: "settings", label: "Settings" }
            p { class: "sidebar-version", "v{VERSION} · {OS_LABEL}" }
        }
    }
}

#[component]
fn NavItem(
    to: Route,
    icon: &'static str,
    label: &'static str,
    #[props(default)] active: bool,
) -> Element {
    let class = if active {
        "nav-item active"
    } else {
        "nav-item"
    };
    rsx! {
        Link { class, active_class: "active", to,
            Icon { name: icon, size: 18 }
            span { class: "nav-label", "{label}" }
        }
    }
}
