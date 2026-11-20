## What

<!-- One or two sentences. Link the issue: Closes #123 -->

## Module / tool

<!-- e.g. organize / merge-pdf, or "core", "app shell" -->

## Checklist

- [ ] `just lint` passes (fmt, clippy `-D warnings`, cargo-deny)
- [ ] `just test` passes; new logic has a test
- [ ] Heavy work runs through `jobs::runner` / `jobs::blocking`, never on the UI thread
- [ ] Files are streamed, not read whole into memory
- [ ] No `unwrap`/`expect`/`panic!` outside tests; errors are `dk_core::Error`
- [ ] No secrets or file contents in logs
- [ ] Both light and dark themes checked (if UI changed)
- [ ] ADR added in `docs/adr/` (if this changes architecture or adds a major dependency)
