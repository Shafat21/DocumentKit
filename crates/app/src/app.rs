//! Root component: provides app-wide state, loads styles, mounts the router.

use std::sync::LazyLock;

use base64::Engine as _;
use dioxus::prelude::*;

use crate::router::Route;
use crate::state::{AppCtx, SearchCtx, ThemeCtx};

/// Embedded at compile time, so `cargo run` works without the `dx` asset pipeline.
static CSS: &str = include_str!("../assets/main.css");

/// Inter variable font (OFL-1.1), embedded so the UI looks identical on every OS.
static INTER: &[u8] = include_bytes!("../assets/fonts/InterVariable.woff2");

/// `@font-face` with the font inlined as a data URL. Encoded once (~0.5 MB), on first render.
static FONT_CSS: LazyLock<String> = LazyLock::new(|| {
    let data = base64::engine::general_purpose::STANDARD.encode(INTER);
    format!(
        "@font-face{{font-family:\"Inter\";font-style:normal;font-weight:100 900;\
         font-display:block;src:url(data:font/woff2;base64,{data}) format(\"woff2\");}}"
    )
});

#[component]
pub fn App() -> Element {
    let ctx = use_context::<AppCtx>();
    use_context_provider(|| ThemeCtx::new(ctx.initial_theme));
    use_context_provider(SearchCtx::new);

    // Effects run after Dioxus shows the window, so fix position/focus here once.
    // (cfg is compile-time, so hook order stays fixed.)
    #[cfg(feature = "desktop")]
    use_effect(crate::bring_window_on_screen);

    rsx! {
        style { {FONT_CSS.as_str()} }
        style { {CSS} }
        Router::<Route> {}
    }
}
