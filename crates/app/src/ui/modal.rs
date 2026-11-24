use dioxus::html::Key;
use dioxus::prelude::*;

use super::Icon;

/// Centred dialog over a dimmed backdrop. Esc, the × button or a backdrop click closes it.
/// Focus moves into the dialog when it opens.
// shortcut: no focus trap; Tab can reach the page behind. Add one if dialogs get forms with many fields.
#[component]
pub fn Modal(
    title: String,
    on_close: EventHandler<()>,
    #[props(default)] wide: bool,
    children: Element,
) -> Element {
    rsx! {
        div { class: "modal-backdrop", onclick: move |_| on_close.call(()),
            div {
                class: if wide { "modal wide" } else { "modal" },
                role: "dialog",
                "aria-modal": "true",
                "aria-label": "{title}",
                tabindex: "-1",
                onclick: move |e| e.stop_propagation(),
                onkeydown: move |e| {
                    if e.key() == Key::Escape {
                        on_close.call(());
                    }
                },
                onmounted: move |e| async move {
                    let _ = e.set_focus(true).await;
                },
                header { class: "modal-head",
                    h2 { "{title}" }
                    button {
                        class: "icon-btn",
                        r#type: "button",
                        "aria-label": "Close",
                        onclick: move |_| on_close.call(()),
                        Icon { name: "x", size: 18 }
                    }
                }
                div { class: "modal-body", {children} }
            }
        }
    }
}
