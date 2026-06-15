# OCLive Windows dev environment quick check
# Usage: .\scripts\setup-dev.ps1

$ErrorActionPreference = "Stop"

Write-Host "=== OCLive dev setup check ===" -ForegroundColor Cyan

function Test-Command($name) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        Write-Host "MISSING: $name" -ForegroundColor Red
        return $false
    }
    return $true
}

$ok = $true
if (-not (Test-Command node)) { $ok = $false } else {
    $nodeVer = node -v
    Write-Host "Node: $nodeVer"
    $major = [int]($nodeVer -replace '^v(\d+).*','$1')
    if ($major -lt 20) {
        Write-Host "WARN: Node >= 20 required (see package.json engines)" -ForegroundColor Yellow
    }
}
if (-not (Test-Command rustc)) { $ok = $false } else { Write-Host "Rust: $(rustc --version)" }
if (-not (Test-Command cargo)) { $ok = $false }

if (Test-Path ".cargo\config.toml") {
    Write-Host "Cargo target-dir: external (see .cargo/config.toml) — keeps repo lean" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "MSVC: required for Windows Tauri builds — see human-docs/10_SETUP_WINDOWS.md" -ForegroundColor DarkGray
Write-Host ""
if ($ok) {
    Write-Host "Recommended next steps:" -ForegroundColor Green
    Write-Host "  npm install"
    Write-Host "  npm run tauri:dev"
    Write-Host "  npm run check"
} else {
    Write-Host "Install missing tools, then re-run this script." -ForegroundColor Red
    exit 1
}
