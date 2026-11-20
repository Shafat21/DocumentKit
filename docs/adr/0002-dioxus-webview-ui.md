# ADR-0002: Dioxus 0.7 with the system WebView for the UI

**Status:** Accepted
**Date:** 2026-11-20
**Deciders:** Shafat, Nahin, Joynur

## Context
The UI must run on Windows, macOS, Linux, iOS and Android from one codebase, fully in Rust, and support a web target later. The designer (UI/UX) works in standard CSS concepts.

## Decision
We use **Dioxus 0.7** (`desktop` and `mobile` features) rendering into the OS WebView (wry): WebView2 on Windows, WKWebView on Apple platforms, WebKitGTK on Linux. Styling is plain CSS with design tokens as CSS variables (`crates/app/assets/main.css`). Icons come from `lucide-dioxus`.

## Options considered
| Option | Pros | Cons |
| --- | --- | --- |
| **Dioxus (chosen)** | One Rust codebase for desktop, mobile and web; React-like model; CSS styling; `dx` CLI bundles every OS | WebView memory overhead; mobile support is younger |
| Tauri + JS frontend | Mature desktop bundling | UI not in Rust; two languages |
| egui / iced / Slint | Native rendering, low RAM | Custom styling far from the mockups; weaker web story |

## Consequences
- Idle RAM includes a WebView (around 60–100 MB). Heavy work must stay off the UI thread (`jobs::runner`).
- Linux users need WebKitGTK 4.1, which is declared in `Dioxus.toml` `[bundle.deb]`.
- `include_str!` embeds the CSS, so `cargo run` works without `dx`.
