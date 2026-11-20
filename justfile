# One command surface for every OS. Install: `cargo install just`.
# `just` lists recipes.

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default:
    @just --list

# Run the desktop app (no dx needed).
dev:
    cargo run -p documentkit

# Run with dx (hot reload).
serve:
    cd crates/app; dx serve --desktop

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
    cd crates/app; dx bundle --desktop --release

# Android emulator/device (needs Android SDK + NDK, see CONTRIBUTING.md).
android:
    cd crates/app; dx serve --android

# iOS simulator (macOS + Xcode only).
ios:
    cd crates/app; dx serve --ios
