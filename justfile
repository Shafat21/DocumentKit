# One command surface for every OS. Install: `cargo install just`.
# `just` lists recipes.

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default:
    @just --list

# One-time: download pdfium for page previews.
[windows]
setup:
    powershell -ExecutionPolicy Bypass -File scripts\setup-pdfium.ps1

[unix]
setup:
    sh scripts/setup-pdfium.sh

# Run the desktop app (no dx needed).
dev:
    cargo run -p documentkit

# Run with dx (hot reload).
serve:
    cd crates/app; dx serve --desktop --package documentkit

# All tests in the workspace.
test:
    cargo test --workspace

# Format check + clippy (warnings are errors) + dependency audit.
lint:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo deny check

# Auto-format.
fmt:
    cargo fmt --all

# Release installer for this OS -> target/dx/documentkit/bundle/
bundle:
    cd crates/app; dx bundle --desktop --release --package documentkit

# Android emulator/device (needs Android SDK + NDK, see CONTRIBUTING.md).
android:
    cd crates/app; dx serve --android --package documentkit

# iOS simulator (macOS + Xcode only).
ios:
    cd crates/app; dx serve --ios --package documentkit
