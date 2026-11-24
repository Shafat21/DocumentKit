//! DocumentKit app entry point. The only file that knows which platform it runs on.

// Release builds on Windows run without a console window.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod features;
mod jobs;
mod logging;
mod os;
mod pages;
mod registry;
mod roadmap_source;
mod router;
mod shell;
mod state;
mod thumbs;
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

/// Make sure the window is visible, on a monitor, and in front. Call once after first paint.
///
/// Dioxus dev builds restore the last window position from a temp file; if the app was
/// closed while minimized that position is (-32000, -32000), i.e. off every screen.
/// Dioxus also shows the window without focusing it, leaving it behind the terminal.
#[cfg(feature = "desktop")]
pub fn bring_window_on_screen() {
    use dioxus::desktop::tao::dpi::PhysicalPosition;
    use dioxus::desktop::LogicalSize;

    let window = dioxus::desktop::window();
    window.set_minimized(false);

    let on_screen = window.outer_position().is_ok_and(|pos| {
        window.available_monitors().any(|m| {
            let (origin, size) = (m.position(), m.size());
            // Allow a small margin: the frame may sit slightly outside the work area.
            pos.x >= origin.x - 50
                && pos.y >= origin.y - 50
                && pos.x < origin.x + size.width as i32 - 50
                && pos.y < origin.y + size.height as i32 - 50
        })
    });

    if !on_screen {
        let monitor = window
            .primary_monitor()
            .or_else(|| window.available_monitors().next());
        if let Some(monitor) = monitor {
            window.set_inner_size(LogicalSize::new(1280.0, 820.0));
            let (origin, area, size) = (monitor.position(), monitor.size(), window.outer_size());
            let x = origin.x + (area.width.saturating_sub(size.width) / 2) as i32;
            let y = origin.y + (area.height.saturating_sub(size.height) / 2) as i32;
            window.set_outer_position(PhysicalPosition::new(x, y));
        }
    }

    window.set_focus();
}

#[cfg(not(feature = "desktop"))]
fn launch(ctx: state::AppCtx, _data_dir: Option<&Path>) {
    dioxus::LaunchBuilder::new()
        .with_context(ctx)
        .launch(app::App);
}
