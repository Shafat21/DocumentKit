# DocumentKit — Engineering Plan

_Last updated: 2026-11-20 · Owner: Shafat (Lead Developer)_

## Goal

DocumentKit is a fast, private, open-source PDF toolkit for Windows, macOS, Linux, iOS and Android. A web version comes later. It is written fully in Rust, with a Dioxus UI and local SQLite storage. Files never leave the device unless the user explicitly opts in.

## Milestones

| # | Milestone | Status |
| :-: | :--- | :--- |
| M0 | Workspace, tool contract, theme system, All tools screen, packaging and CI | 🚧 Code written; first build and review pending |
| M1 | Development roadmap screen, fed live from `roadmap.json` on GitHub | 🚧 Built; needs `roadmap.json` pushed to `main` |
| M2 | First real tools: Merge, Split (Organize), plus the file dialog and drag-and-drop | Planned |
| M3 | Remaining modules, in parallel, one owner per module | Planned |
| M4 | Signed installers, auto-update, mobile release | Planned |

## Architecture

```
┌──────────────────────── crates/app (documentkit) ────────────────────────┐
│ main.rs ── platform launch (desktop / mobile), window, logging, Store     │
│ app.rs ─── contexts: AppCtx(Arc<Store>), ThemeCtx, SearchCtx; Router      │
│ shell/ ─── Sidebar · Topbar · Outlet           pages/ ── AllTools, Roadmap │
│ features/<module>/ ── tool pages (UI only)     ui/ ──── Icon, ToolCard …   │
│ jobs/ ──── runner: worker thread, progress, cancel, panic isolation       │
│ registry ─ every Tool impl, collected from module crates                  │
└───────────────┬───────────────────────────────────────────┬──────────────┘
                │ depends on                                │
   ┌────────────▼────────────┐        ┌─────────────────────▼──────────────┐
   │ crates/modules/* (8)     │──────▶│ crates/core (dk-core)               │
   │ dk-organize, dk-optimize │ uses  │ Tool trait · ToolSpec · Job         │
   │ dk-convert-to/-from      │       │ catalog (35 tools, 8 modules)       │
   │ dk-edit, dk-security     │       │ Store (SQLite) · Theme · Error      │
   │ dk-intelligence, dk-scan │       │ platform (paths, OS label)          │
   └──────────────────────────┘       └─────────────────────────────────────┘
```

- **dk-core** has no UI dependency. It is the contract between everyone ([ADR-0004](adr/0004-tool-contract.md)).
- **Module crates** each own one feature area. Each tool is one file containing one struct that implements `Tool` ([ADR-0001](adr/0001-workspace-per-module-crates.md)).
- **App** is the only crate that knows about Dioxus, threads or the platform ([ADR-0002](adr/0002-dioxus-webview-ui.md)).
- **Store** is SQLite behind a `Mutex`, shared as `Arc<Store>` ([ADR-0003](adr/0003-sqlite-store.md)).

### How a tool run flows

1. A UI page builds a `Job` (inputs, output dir, options) and calls `jobs::runner::start(tool, job)`.
2. The runner spawns a named thread, `job-<id>`, which calls `execute`. `execute` wraps `tool.run` in `catch_unwind`, so a panic becomes `Error::Internal`.
3. The tool reports progress into a bounded channel. Reports are throttled to at most 20 per second, and extra ones are dropped, never queued.
4. An async task on the UI side drains the channel into a `Signal<JobState>`, so only the progress widget re-renders.
5. Cancel sets an `AtomicBool`. The tool checks it per page and returns `Error::Cancelled`.

### State and reactivity

- `ThemeCtx(Signal<Theme>)` is read by `Shell`, which writes it to `data-theme`. CSS variables restyle the whole tree, and the choice is persisted in `settings`.
- `SearchCtx(Signal<String>)` holds the search text from the top bar. The All tools grid is a `use_memo` over (tab, query) and recomputes only when one of those changes.
- The catalog is `&'static` data, so rendering it allocates nothing.

## Code standards

**Performance (no RAM hangs, no heavy load)**
- The UI thread never does I/O or CPU-heavy work. Use `jobs::runner` for tools and `jobs::blocking` for small blocking calls.
- Stream files and process page by page. Never load a whole PDF into a `Vec<u8>`.
- Use bounded channels and throttled progress. Big results go in an `Arc`, never cloned per render.
- Temp files go in `<data_dir>/tmp/<job-id>/` and are removed when the job ends.
- Release profile: `lto = "fat"`, `codegen-units = 1`, `opt-level = 3`, `strip`. `panic = "unwind"` is kept so the runner can isolate panics.

**Security**
- `unsafe_code = "forbid"` applies workspace-wide.
- Clippy denies `unwrap`, `expect`, `panic!`, `dbg!` and `print*!` outside tests.
- Typed errors (`dk_core::Error`) are mapped to user text in one place (`ui/error_text.rs`). Internal detail goes to the log only.
- SQL is always parameterised. The database runs with WAL and `foreign_keys = ON`.
- Secrets (PDF passwords, signing keys) are never logged or persisted. They are wrapped in `secrecy::SecretString` when the Security module lands.
- Untrusted PDFs are parsed on worker threads under `catch_unwind`. Each parser gets a fuzz target.
- cargo-deny blocks advisories, unknown sources and GPL-incompatible licenses.

**Clean code**
- One tool per file and one struct per tool. Functions stay around 60 lines or fewer; split them as validate → process → write.
- Public items get a one-line `///` doc explaining why. `missing_docs` is a warning in dk-core.
- No glob re-exports across crates. Tests sit next to the code.

## Platforms and packaging

| Target | Build | Output |
| :--- | :--- | :--- |
| Windows | `just bundle` on Windows | `.msi` / `.exe` |
| macOS | `just bundle` on macOS | `.app` / `.dmg` |
| Linux | `just bundle` on Linux | `.deb` / `.rpm` / `.AppImage` |
| Android | `just android` | `.apk` |
| iOS | `just ios` (macOS only) | `.app` |

Each OS bundles natively, and CI's `bundle` job runs a matrix over the three desktop OSes on tags or manual dispatch.

## Known gaps (tracked)

- Bundle icons: export `crates/app/assets/logo.svg` (the official mark) to PNG, ICO and ICNS, then enable `icon` in `Dioxus.toml`.
- Font: `crates/app/assets/fonts/InterVariable.woff2` must be present to build (see the README in that folder).
- Mobile data dir: `dirs::data_dir()` is `None` on Android and iOS, so the app falls back to the working directory. This needs the platform API.
- `lucide-dioxus` is built with `all-icons`. Trim it to category features to cut compile time.
- CODEOWNERS entries need real GitHub usernames.

## Open questions

See [BRAINSTORM.md](BRAINSTORM.md#open-questions).
