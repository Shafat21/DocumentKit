use dioxus::prelude::*;

/// Labelled on/off switch bound to a signal.
#[component]
pub fn Toggle(
    label: &'static str,
    hint: &'static str,
    on: Signal<bool>,
    #[props(default)] disabled: bool,
) -> Element {
    let mut on = on;
    rsx! {
        div { class: "setting",
            div {
                p { class: "setting-label", "{label}" }
                p { class: "hint", "{hint}" }
            }
            button {
                class: "switch",
                r#type: "button",
                role: "switch",
                "aria-checked": if on() { "true" } else { "false" },
                "aria-label": label,
                disabled,
                onclick: move |_| on.toggle(),
                span { class: "switch-knob" }
            }
        }
    }
}
