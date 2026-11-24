//! Module workspace: every tool in one module, e.g. /module/organize.

use dioxus::prelude::*;
use dk_core::{catalog, ModuleId};

use crate::ui::{EmptyState, ToolCard};

#[component]
pub fn ModulePage(module: String) -> Element {
    let Some(module) = ModuleId::ALL.into_iter().find(|m| m.key() == module) else {
        return rsx! {
            div { class: "page",
                EmptyState { icon: "construction", title: "Not found", body: "This module doesn't exist." }
            }
        };
    };
    let count = catalog::tools_in(module).count();

    rsx! {
        div { class: "page",
            section { class: "hero",
                div {
                    h1 { "{module.name()}" }
                    p { class: "muted", "{count} tools in this workspace." }
                }
            }
            div { class: "tool-grid",
                for spec in catalog::tools_in(module) {
                    ToolCard { key: "{spec.id}", spec }
                }
            }
        }
    }
}
