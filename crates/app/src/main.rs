//! DocumentKit app entry point. The only file that knows which platform it runs on.

// Release builds on Windows run without a console window.
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod app;
mod features;
mod jobs;
mod logging;
mod pages;
mod registry;
mod roadmap_source;
mod router;
mod shell;
mod state;
mod ui;

use std::path::Path;
use std::sync::Arc;

use dk_core::store::Store;

fn main() -> dk_core::Result<()> {
    let data_dir = dk_core::platform::data_dir().ok();
    // Keep the guard alive for the whole run so buffered log lines get flushed.
    let _log_guard = data_dir.as_deref().and_then(logging::init);

    let store = Arc::new(open_store(data_dir.as_deref())?);
    let ctx = state::AppCtx {
        initial_theme: state::theme::load(&store),
        store,
    };

    launch(ctx, data_dir.as_deref());
    Ok(())
}

/// Open the on-disk store; fall back to in-memory so the app still starts
/// (settings just won't persist) if the data dir is unusable.
fn open_store(data_dir: Option<&Path>) -> dk_core::Result<Store> {
    if let Some(dir) = data_dir {
        match Store::open(&dir.join("documentkit.db")) {
            Ok(store) => return Ok(store),
            Err(e) => tracing::error!("cannot open database, using in-memory store: {e}"),
        }
    }
    Store::in_memory()
}

/// 256 px is the largest size Windows shows in the taskbar/Alt-Tab; decoding it takes a few ms.
#[cfg(feature = "desktop")]
static APP_ICON: &[u8] = include_bytes!("../assets/icons/256x256.png");

#[cfg(feature = "desktop")]
fn launch(ctx: state::AppCtx, data_dir: Option<&Path>) {
    use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

    // Paint the window in the theme's background before the WebView loads (no white flash).
    let background = match ctx.initial_theme {
        dk_core::Theme::Dark => (15, 20, 36, 255),
        _ => (246, 247, 250, 255),
    };

    let window = WindowBuilder::new()
        .with_title("DocumentKit")
        .with_inner_size(LogicalSize::new(1280.0, 820.0))
        .with_min_inner_size(LogicalSize::new(960.0, 640.0));

    let mut config = Config::new()
        .with_window(window)
        .with_menu(None)
        .with_background_color(background);
    // Window + taskbar icon. Must come after `with_window`, which replaces the builder.
    match dioxus::desktop::icon_from_memory::<dioxus::desktop::tao::window::Icon>(APP_ICON) {
        Ok(icon) => config = config.with_icon(icon),
        Err(e) => tracing::warn!("app icon not loaded: {e}"),
    }
    if let Some(dir) = data_dir {
        config = config.with_data_directory(dir.join("webview"));
    }

    dioxus::LaunchBuilder::new()
        .with_cfg(config)
        .with_context(ctx)
        .launch(app::App);
}

#[cfg(not(feature = "desktop"))]
fn launch(ctx: state::AppCtx, _data_dir: Option<&Path>) {
    dioxus::LaunchBuilder::new().with_context(ctx).launch(app::App);
}
