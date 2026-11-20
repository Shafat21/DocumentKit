# ADR-0003: Bundled SQLite for local app data

**Status:** Accepted
**Date:** 2026-11-20
**Deciders:** Shafat, Nahin

## Context
The app persists settings (theme), recent documents, and later favorites, presets and a roadmap cache. The data must live locally, work offline on every OS, and survive crashes.

## Decision
We use `rusqlite` with the `bundled` feature (SQLite compiled in, so there is no system dependency). The database lives at `<data_dir>/documentkit.db`, and the `Store` in `dk-core` owns it.
- A `Mutex<Connection>` inside `Store` makes it `Send + Sync`, and it is shared as `Arc<Store>`.
- WAL journal, `foreign_keys = ON`, `synchronous = NORMAL`.
- Schema migrations are a `const` list keyed by `PRAGMA user_version`. They are append-only and never edited once shipped.
- Queries are parameterised only.
- The store sits behind a `sqlite` cargo feature so a future web build can swap the backend.

## Options considered
| Option | Pros | Cons |
| --- | --- | --- |
| **SQLite (chosen)** | Transactions, queries, crash-safe, tiny | C dependency (bundled); no wasm |
| JSON/TOML file | Trivial | No concurrency safety; rewrites the whole file; corrupts on crash |
| `sled` / `redb` | Pure Rust | No SQL; harder ad-hoc queries |

## Consequences
- A single mutex serialises DB access. That's fine for app metadata, but bulk work must never go through it in a hot loop.
- If the file can't be opened, the app falls back to an in-memory store and keeps running. The user loses persistence, not the app.
- Secrets are never stored here (see ADR-0004).
