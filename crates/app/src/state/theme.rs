//! Theme state: one signal at the root; `Shell` writes it to `data-theme`.

use dioxus::prelude::*;
use dk_core::store::Store;
use dk_core::Theme;

/// Copyable handle to the current theme.
#[derive(Clone, Copy)]
pub struct ThemeCtx(Signal<Theme>);

impl ThemeCtx {
    pub fn new(initial: Theme) -> Self {
        Self(Signal::new(initial))
    }

    /// Current theme. Reading subscribes the calling component to changes.
    pub fn get(&self) -> Theme {
        (self.0)()
    }

    /// Change theme and persist it. The write is a single-row upsert (sub-millisecond),
    /// so it stays on the UI thread.
    pub fn set(&self, theme: Theme, store: &Store) {
        let mut signal = self.0;
        signal.set(theme);
        if let Err(e) = store.set_setting(Theme::SETTING_KEY, theme.as_str()) {
            tracing::warn!("could not save theme: {e}");
        }
    }
}

/// Theme handle from context.
pub fn use_theme() -> ThemeCtx {
    use_context::<ThemeCtx>()
}

/// Saved theme, or the default (follow system) if none or unreadable.
pub fn load(store: &Store) -> Theme {
    match store.get_setting(Theme::SETTING_KEY) {
        Ok(Some(value)) => Theme::parse(&value),
        Ok(None) => Theme::default(),
        Err(e) => {
            tracing::warn!("could not read theme: {e}");
            Theme::default()
        }
    }
}
