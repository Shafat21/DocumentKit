//! App-wide state shared through Dioxus context.

pub mod theme;

use std::sync::Arc;

use dioxus::prelude::*;
use dk_core::store::Store;
use dk_core::Theme;

pub use theme::ThemeCtx;

/// Built once in `main` before launch; handed to every component via context.
#[derive(Clone)]
pub struct AppCtx {
    pub store: Arc<Store>,
    pub initial_theme: Theme,
}

/// Text in the top-bar search box; filters the All tools grid.
#[derive(Clone, Copy)]
pub struct SearchCtx(pub Signal<String>);

impl SearchCtx {
    pub fn new() -> Self {
        Self(Signal::new(String::new()))
    }
}

/// Shared database handle.
pub fn use_store() -> Arc<Store> {
    use_context::<AppCtx>().store
}

/// Shared search text.
pub fn use_search() -> Signal<String> {
    use_context::<SearchCtx>().0
}
