# ADR-0001: Cargo workspace with one crate per feature module

**Status:** Accepted
**Date:** 2026-11-20
**Deciders:** Shafat, Nahin

## Context
DocumentKit has 8 feature modules and 35 tools. Several developers will work on them at the same time. Modules will pull in heavy, very different dependencies: PDF engines, OCR, Office conversion, maybe ML models. We want parallel work without merge conflicts, fast incremental builds, and clear ownership.

## Decision
The repo is a Cargo workspace:
- `crates/core` (`dk-core`) holds the shared contract with no UI.
- `crates/modules/<module>` (`dk-<module>`) holds one crate per module, with one file per tool.
- `crates/app` (`documentkit`) is the Dioxus app that composes everything.

## Options considered

### A: Single crate with modules (`src/organize/…`)
| Dimension | Assessment |
| --- | --- |
| Complexity | Low |
| Build speed | Worse: every change recompiles everything |
| Ownership | Folder-level only |

**Pros:** simplest. **Cons:** shared `Cargo.toml`, so heavy dependencies hit everyone; frequent conflicts on shared files.

### B: Workspace, crate per module (chosen)
| Dimension | Assessment |
| --- | --- |
| Complexity | Medium (9 more `Cargo.toml` files) |
| Build speed | Better: `cargo test -p dk-edit` builds only what it needs |
| Ownership | CODEOWNERS per crate |

**Pros:** dependency isolation, parallel compilation, explicit boundaries enforced by the compiler. **Cons:** a bit more boilerplate per module.

## Consequences
- Adding a tool means one catalog line, one module file and one `tools()` entry. Tests enforce that all three match.
- Modules can't depend on each other unless that's declared in `Cargo.toml`, which keeps coupling visible.
- `dk-core` changes affect everyone, so they need lead review.
