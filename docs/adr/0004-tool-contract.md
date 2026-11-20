# ADR-0004: Synchronous `Tool` trait, run by an app-side job runner

**Status:** Accepted
**Date:** 2026-11-20
**Deciders:** Shafat, Nahin

## Context
35 tools will be written by different people. Each must be easy to write and test, impossible to freeze the UI with, and safe against hostile PDFs.

## Decision
```rust
pub trait Tool: Send + Sync {
    fn spec(&self) -> &'static ToolSpec;               // static catalog entry
    fn run(&self, job: Job, progress: &dyn Progress) -> Result<JobOutput>;
}
```
- **Synchronous and blocking by design.** Tool authors write plain Rust: no async, no UI types.
- The app's `jobs::runner` runs each job on its own thread and wraps it in `catch_unwind`, turning panics into `Error::Internal`. Progress goes through a bounded, throttled channel.
- `Job` carries a cancel flag. Tools check `job.is_cancelled()` per page.
- `ToolSpec` lives in the static `catalog`, so the UI can list tools before they're implemented, and the compiler checks each tool's reference to its spec.

## Options considered
| Option | Pros | Cons |
| --- | --- | --- |
| **Sync trait + runner (chosen)** | Simplest to implement; trivially testable; isolation in one place | One OS thread per running job (fine: jobs are few) |
| `async fn run` | Composes with async I/O | PDF work is CPU-bound anyway; forces async on every author |
| Separate process per job | Strongest isolation | Startup cost, IPC, harder on mobile |

## Consequences
- **Secrets:** passwords and keys arrive in `job.options`. Tools move them into `secrecy::SecretString` immediately. They are never logged and never written to SQLite. If they must persist, use the OS keychain (`keyring`) after a new ADR.
- **Memory:** tools must stream. Reviews reject reading whole inputs into memory.
- **Panics:** isolation needs `panic = "unwind"` in release. Do not switch to `abort`.
- If a future tool needs true process isolation (for example a native renderer that can segfault), add a sandboxed subprocess backend behind the same trait.
