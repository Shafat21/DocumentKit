# Contributing to DocumentKit

Architecture and standards are in [docs/PLAN.md](docs/PLAN.md). Ideas and open questions go in [docs/BRAINSTORM.md](docs/BRAINSTORM.md). Decisions are recorded in [docs/adr/](docs/adr/).

## Setup

1. Install Rust stable: <https://rustup.rs>. `rust-toolchain.toml` picks the version for you.
2. Install the system WebView for your OS:
   - **Windows:** WebView2 is already part of Windows 10/11.
   - **macOS:** nothing extra.
   - **Linux (Debian/Ubuntu):**
     `sudo apt install libwebkit2gtk-4.1-dev build-essential libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
3. The Inter font is already committed at `crates/app/assets/fonts/InterVariable.woff2`. If it's missing, see `crates/app/assets/fonts/README.md`.
4. Optional tools:
   - `cargo install just` (shortcut commands)
   - `cargo binstall dioxus-cli` or `cargo install dioxus-cli` (`dx`: hot reload, installers, mobile)
   - `cargo install cargo-deny` (license and advisory checks)

## Everyday commands

| Command | What it does |
| :--- | :--- |
| `just dev` / `cargo run -p documentkit` | Run the desktop app |
| `just serve` | Run with hot reload (`dx`) |
| `just test` | Test the whole workspace |
| `just lint` | fmt check + clippy `-D warnings` + cargo-deny |
| `just bundle` | Build an installer for this OS |
| `just android` / `just ios` | Run on a mobile emulator (see below) |

## Repository layout

```
crates/
  core/                  dk-core: Tool contract, catalog, SQLite store, theme. No UI.
  modules/<module>/      one crate per feature module; one file per tool
  app/                   Dioxus app: shell, pages, feature UI, job runner
docs/                    PLAN, BRAINSTORM, ADRs
```

You own a module: you work in `crates/modules/<module>/` and `crates/app/src/features/<module>/`. You rarely need to touch other folders.

## Implementing a tool

Example: `merge-pdf` in the Organize module.

1. **Engine.** Edit `crates/modules/organize/src/merge_pdf.rs` and replace the `NotImplemented` body of `run`:
   - read inputs from `job.inputs` and stream them, never reading a whole file into a `Vec<u8>`,
   - write only inside `job.output_dir`, and never overwrite an input,
   - call `progress.report(pct, "Page 3 of 40")` per page,
   - check `job.is_cancelled()` per page and return `Err(Error::Cancelled)` when it is set,
   - return `Err(Error::Invalid(..))` for bad user input. Don't use `unwrap`, `expect` or `panic!`.
2. **Dependencies.** Add PDF crates to *this module's* `Cargo.toml` only. Check the license is GPL-compatible (`cargo deny check`).
3. **Tests.** Write tests in the same file (`#[cfg(test)] mod tests`). Put fixture PDFs in `crates/modules/organize/tests/fixtures/`.
4. **UI.** Create `crates/app/src/features/organize/merge_pdf.rs` with a `#[component]`. Start the job with `jobs::runner::start(registry::find("merge-pdf")?, job)` and render `handle.state`. Add a route in `crates/app/src/router.rs` and wire up the tool card.
5. Open a PR. The template has the checklist.

New tool that isn't in the catalog yet? Add one line to `crates/core/src/catalog.rs`, then a file in the module crate, and register it in that module's `lib.rs` `tools()`. The tests fail until all three match.

## Updating the roadmap

The app's **Development roadmap** screen reads [`roadmap.json`](roadmap.json) from the `main` branch on GitHub, so everyone's app shows the same progress.

- To change a task's progress, set `"status"` to `"pending"`, `"in_progress"` or `"done"`.
- To assign someone, set `"assignee"` to their name, or `null` for nobody.
- Commit the change to `main`. Apps pick it up when the Roadmap screen opens or someone clicks **Refresh**.
- When offline, the app shows the last copy it synced, or the copy built into the app.
- Task ids must match `crates/core/src/catalog.rs`. A test enforces this.

## Code standards (short version)

- **Never block the UI thread.** Tools run through `jobs::runner`. Other blocking work (DB reads, file stats) goes through `jobs::blocking`.
- **Bounded memory.** Stream files, process page by page, and use bounded channels. Big results go in an `Arc`, not cloned per render.
- **Errors, not panics.** `unwrap`, `expect` and `panic!` are denied outside tests by clippy. Every error is a `dk_core::Error`.
- **No `unsafe`.** It is forbidden workspace-wide. FFI goes in its own reviewed crate, with an ADR.
- **Secrets.** Passwords and keys are never logged and never stored in SQLite.
- **SQL.** Parameterised queries only (`params![]`).
- **Both themes.** Style with the CSS variables in `crates/app/assets/main.css`, never with hard-coded colors. The brand red is `#DE0036`, from the logo.
- **Typography.** Inter only. It is embedded from `crates/app/assets/fonts/`, so don't add other fonts.
- **Accessibility.** Every icon-only button needs an `aria-label`, and focus must stay visible.

## Mobile

- **Android:** install Android Studio with the SDK, NDK and CMake. Set `JAVA_HOME`, `ANDROID_HOME` and `NDK_HOME`, then run
  `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`, then `just android`.
- **iOS (macOS only):** install Xcode, run `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`, then `just ios`.

## Branches and commits

- Branches: `feat/<module>/<tool>`, `fix/<area>`, `docs/<topic>`.
- Commits: imperative mood, ≤ 72-char subject, e.g. `organize: implement merge-pdf`.
