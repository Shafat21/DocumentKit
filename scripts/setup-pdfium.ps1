# Downloads pdfium (page previews) into crates/app/assets/pdfium/.
# Usage (from the repo root):  powershell -ExecutionPolicy Bypass -File scripts\setup-pdfium.ps1
# Optional: -Release chromium/7350   (default: latest build from bblanchon/pdfium-binaries)

param([string]$Release = "latest")

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$dest = Join-Path $root "crates\app\assets\pdfium"
$arch = if ([Environment]::Is64BitOperatingSystem) { if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "x64" } } else { "x86" }
$asset = "pdfium-win-$arch.tgz"
$url = if ($Release -eq "latest") {
    "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/$asset"
} else {
    "https://github.com/bblanchon/pdfium-binaries/releases/download/$Release/$asset"
}

$work = Join-Path ([IO.Path]::GetTempPath()) ("dk-pdfium-" + [Guid]::NewGuid())
New-Item -ItemType Directory -Force $work | Out-Null
try {
    Write-Host "Downloading $url"
    Invoke-WebRequest $url -OutFile (Join-Path $work $asset)
    tar -xzf (Join-Path $work $asset) -C $work
    New-Item -ItemType Directory -Force $dest | Out-Null
    Copy-Item (Join-Path $work "bin\pdfium.dll") $dest -Force
    Copy-Item (Join-Path $work "LICENSE") (Join-Path $dest "PDFIUM-LICENSE.txt") -Force -ErrorAction SilentlyContinue
    Write-Host "Installed $(Join-Path $dest 'pdfium.dll'). Re-add files or reopen previews in the app."
} finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}
