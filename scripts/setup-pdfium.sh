#!/usr/bin/env sh
# Downloads pdfium (page previews) into crates/app/assets/pdfium/.
# Usage (from the repo root):  sh scripts/setup-pdfium.sh [release-tag]   (default: latest)
set -eu

release="${1:-latest}"
root="$(cd "$(dirname "$0")/.." && pwd)"
dest="$root/crates/app/assets/pdfium"

case "$(uname -s)-$(uname -m)" in
  Darwin-*) asset="pdfium-mac-univ.tgz"; lib="lib/libpdfium.dylib" ;;
  Linux-x86_64) asset="pdfium-linux-x64.tgz"; lib="lib/libpdfium.so" ;;
  Linux-aarch64 | Linux-arm64) asset="pdfium-linux-arm64.tgz"; lib="lib/libpdfium.so" ;;
  *) echo "Unsupported platform: $(uname -s)-$(uname -m)" >&2; exit 1 ;;
esac

if [ "$release" = "latest" ]; then
  url="https://github.com/bblanchon/pdfium-binaries/releases/latest/download/$asset"
else
  url="https://github.com/bblanchon/pdfium-binaries/releases/download/$release/$asset"
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
echo "Downloading $url"
curl -fsSL "$url" -o "$work/$asset"
tar -xzf "$work/$asset" -C "$work"
mkdir -p "$dest"
cp "$work/$lib" "$dest/"
[ -f "$work/LICENSE" ] && cp "$work/LICENSE" "$dest/PDFIUM-LICENSE.txt"
echo "Installed $dest/$(basename "$lib"). Re-add files or reopen previews in the app."
