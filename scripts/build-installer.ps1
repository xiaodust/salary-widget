# Build the NSIS installer for Salary Widget.
# Usage: powershell -ExecutionPolicy Bypass -File scripts/build-installer.ps1

$ErrorActionPreference = "Stop"

$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

$TargetDir = if ($env:CARGO_TARGET_DIR) {
    $env:CARGO_TARGET_DIR
} else {
    Join-Path $ProjectRoot "src-tauri\target"
}

Write-Host "Building Salary Widget installer..."
npm.cmd run tauri build -- --bundles nsis

$setupDir = Join-Path $TargetDir "release\bundle\nsis"
$setupFiles = Get-ChildItem -LiteralPath $setupDir -Filter "*-setup.exe" -ErrorAction SilentlyContinue

if (-not $setupFiles) {
    throw "Installer not found under: $setupDir"
}

Write-Host "Build complete:"
$setupFiles | ForEach-Object { Write-Host "  $($_.FullName)" }
