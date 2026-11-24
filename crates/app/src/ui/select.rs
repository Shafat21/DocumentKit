use dioxus::html::Key;
use dioxus::prelude::*;

use super::Icon;

/// Themed dropdown (the native `<select>` popup can't be styled).
///
/// Keyboard: Enter/Space/↓ opens; ↑/↓ move; Home/End jump; Enter picks; Esc or Tab closes.
/// Clicking outside closes it. `options` are `(value, label)` pairs.
#[component]
pub fn Select(label: String, options: Vec<(String, String)>, value: Signal<String>) -> Element {
    let mut value = value;
    let mut open = use_signal(|| false);
    // Highlighted row while open.
    let mut active = use_signal(|| 0usize);

    let current = options.iter().position(|(v, _)| *v == value()).unwrap_or(0);
    let shown = options
        .get(current)
        .map_or_else(String::new, |(_, l)| l.clone());
    let count = options.len();
    let values: Vec<String> = options.iter().map(|(v, _)| v.clone()).collect();

    let mut choose = move |index: usize, values: &[String]| {
        if let Some(v) = values.get(index) {
            value.set(v.clone());
        }
        open.set(false);
    };

    let keys = values.clone();
    let onkeydown = move |e: KeyboardEvent| {
        let last = count.saturating_sub(1);
        match (open(), e.key()) {
            (false, Key::Enter | Key::ArrowDown | Key::ArrowUp) => {
                active.set(current);
                open.set(true);
            }
            (false, Key::Character(c)) if c == " " => {
                active.set(current);
                open.set(true);
            }
            (true, Key::ArrowDown) => active.set((active() + 1).min(last)),
            (true, Key::ArrowUp) => active.set(active().saturating_sub(1)),
            (true, Key::Home) => active.set(0),
            (true, Key::End) => active.set(last),
            (true, Key::Enter) => choose(active(), &keys),
            (true, Key::Character(c)) if c == " " => choose(active(), &keys),
            (true, Key::Escape | Key::Tab) => {
                open.set(false);
                return;
            }
            _ => return,
        }
        e.prevent_default();
    };

    rsx! {
        div { class: if open() { "select open" } else { "select" },
            if open() {
                // Invisible layer that closes the list on any outside click.
                div { class: "select-backdrop", onclick: move |_| open.set(false) }
            }
            button {
                class: "select-trigger",
                r#type: "button",
                "aria-haspopup": "listbox",
                "aria-expanded": if open() { "true" } else { "false" },
                "aria-label": "{label}: {shown}",
                onclick: move |_| {
                    active.set(current);
                    open.toggle();
                },
                onkeydown,
                span { "{shown}" }
                span { class: "select-chevron", Icon { name: "chevron-down", size: 16 } }
            }
            if open() {
                ul { class: "select-list", role: "listbox", "aria-label": "{label}",
                    for (i, (_, text)) in options.iter().enumerate() {
                        li {
                            key: "{i}",
                            role: "option",
                            "aria-selected": if i == current { "true" } else { "false" },
                            class: match (i == current, i == active()) {
                                (true, true) => "select-option selected active",
                                (true, false) => "select-option selected",
                                (false, true) => "select-option active",
                                (false, false) => "select-option",
                            },
                            onmouseenter: move |_| active.set(i),
                            onclick: {
                                let values = values.clone();
                                move |_| choose(i, &values)
                            },
                            span { "{text}" }
                            if i == current {
                                Icon { name: "check", size: 16 }
                            }
                        }
                    }
                }
            }
        }
    }
}
