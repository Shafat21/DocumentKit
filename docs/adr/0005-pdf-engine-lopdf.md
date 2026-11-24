# ADR-0005: lopdf for structural PDF edits

**Status:** Accepted
**Date:** 2026-11-24
**Deciders:** Shafat

## Context
Merge PDF is the first real tool. The Organize tools (merge, split, remove/extract/reorder pages) only restructure PDFs; they don't render pages. The engine has to run on every target, mobile included, and fit the GPL-3.0 licence.

## Decision
Use **lopdf** (pure Rust, MIT) for structural edits. Shared helpers (`load`, `page_count`, `flattened_page`, `save_atomic`) live in `crates/modules/organize/src/pdf.rs`, and they move to a `dk-pdf` crate once a second module needs them.

Page **rendering** (thumbnails, PDF to image) is a separate decision. pdfium via `pdfium-render` is favoured and will get its own ADR.

## Options considered
| Option | Pros | Cons |
| --- | --- | --- |
| **lopdf (chosen)** | Pure Rust, no native libraries, all targets, MIT | Loads whole documents into memory; no rendering |
| pdfium-render | Renders pages; very robust parser | Native library per OS, FFI, bigger bundle |
| qpdf (FFI) | Excellent structural tooling | C++ dependency, FFI, Apache-2.0 + build complexity |

## Consequences
- **Memory:** merge input is capped at 500 MB in total (`MAX_TOTAL_BYTES`). Moving to a streaming writer would lift the cap.
- **Inherited page attributes:** `MediaBox`, `Resources`, `CropBox` and `Rotate` are copied onto each page before it is moved to a new page tree (`flattened_page`).
- **Atomic saves:** output is written to a temp file in the destination folder and then renamed, so a failed or cancelled job leaves nothing behind.
- **Encrypted inputs:** they are rejected with "Unlock it first". Decryption belongs to the Security module.
- **Not carried over:** source catalog extras such as forms and named destinations are not kept in a merge. Revisit this if users need forms preserved.
