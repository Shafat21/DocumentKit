# DocumentKit — Brainstorm

_Last updated: 2026-11-20. A living document: add ideas freely. When an idea becomes a decision, write an ADR in [adr/](adr/) and link it here._

## Open questions

| # | Question | Options | Decide before |
| :-: | :--- | :--- | :--- |
| Q1 | How does the Intelligence module stay private? | (a) Local models only (candle / llama.cpp, a few hundred MB download). (b) Opt-in cloud API with explicit per-file consent. (c) Both, with local as the default. | Any code in `dk-intelligence` |
| Q2 | Which PDF engine? | `lopdf` (pure Rust, low-level), `pdfium-render` (Chromium's engine, needs a native lib and FFI crate), `mupdf` (AGPL, which is a license issue), or a mix | M2 (Merge/Split) |
| Q3 | Which OCR engine? | Tesseract via FFI, `ocrs` (pure Rust), or a platform OCR API (Windows.Media.Ocr, Apple Vision) | OCR work |
| Q4 | How do Office conversions work? | Bundle LibreOffice headless (large), a pure-Rust subset, or platform APIs | Convert-to/-from work |
| Q5 | Which storage for the web target? | IndexedDB, `sqlite-wasm` with OPFS, or server-side | Web milestone |
| Q6 | How do updates ship? | A custom updater against GitHub Releases, OS stores (MS Store, Mac App Store, Flathub), or both | M4 |
| Q7 | Which Linux packaging? | AppImage, Flatpak, `.deb`/`.rpm`, or all of them | M4 |
| Q8 | Should DocumentKit be the default `.pdf` app? | Register a file association with an "Open with DocumentKit" prompt, or opt in during setup | M4 |
| Q9 | Single instance or multiple windows? | Opening a second file focuses the existing window, or opens a new window | M2 |

## Feature ideas beyond the README

- **Batch mode:** apply one tool to a whole folder, with a queue and per-file progress.
- **Pipelines:** chain tools, for example Scan → Deskew → OCR → Compress, and save the chain as a preset.
- **Command palette** (Ctrl/Cmd + K): jump to any tool or recent file.
- **CLI:** a `documentkit` binary reusing `dk-core` and the modules, for scripting and CI use.
- **Favorites:** pin tools to the sidebar.
- **Recent documents:** thumbnails cached in `<data_dir>/thumbs/` and capped by size.
- **Watch folder:** auto-process PDFs dropped into a chosen folder.
- **Accessibility checker:** report missing tags and alt text, and check PDF/UA compliance.
- **Metadata editor:** title, author and keywords; strip metadata for privacy.
- **Mobile:** camera scanner with edge detection, and a share-sheet integration ("Send to DocumentKit").

## UX notes

- The mockups have a red accent in light mode and an indigo primary button in dark mode. The tokens in `crates/app/assets/main.css` follow them.
- Below 900 px the sidebar collapses to an icon rail. On phones we may want a bottom tab bar instead; test this on real devices.
- Every tool page should share one layout: inputs → options → run → result, with the same progress and cancel UI.

## Performance budget (targets to measure)

| Metric | Target |
| :--- | :--- |
| Cold start to first paint (desktop) | < 500 ms |
| Idle RAM (desktop) | < 120 MB including the WebView |
| Merge of 2 × 100-page PDFs | < 2 s, with flat memory |
| UI frame time during a job | < 16 ms (60 fps) |

## Monetisation

None. GPL-3.0, free forever. Donations (GitHub Sponsors) could come later.
