# pdfium (page previews)

DocumentKit renders page previews with [pdfium](https://pdfium.googlesource.com/pdfium/), the PDF engine used in Chrome. pdfium is a native library that the app loads at runtime. If the library is missing, the app still works and shows a file icon instead of a preview.

## Dev setup (one time)

Run this from the repo root:

- **Windows:** `powershell -ExecutionPolicy Bypass -File scripts\setup-pdfium.ps1`
- **macOS / Linux:** `sh scripts/setup-pdfium.sh`

Both accept a release tag (e.g. `chromium/7350`) to pin a version. To set it up by hand instead:

1. Download the build for your OS from <https://github.com/bblanchon/pdfium-binaries/releases/latest>:
   - **Windows:** `pdfium-win-x64.tgz`
   - **macOS:** `pdfium-mac-univ.tgz`
   - **Linux:** `pdfium-linux-x64.tgz`
2. Extract it and copy the library into this folder:
   - **Windows:** `bin/pdfium.dll`
   - **macOS:** `lib/libpdfium.dylib`
   - **Linux:** `lib/libpdfium.so`
3. Restart the app.

Debug builds look in this folder, and every build also looks next to the app's executable and in the system library path.

The binaries are git-ignored because each is about 5 MB per OS. Bundling them into installers is tracked in `docs/PLAN.md` under Known gaps.

pdfium is © The PDFium Authors, licensed under BSD-3-Clause and Apache-2.0.
