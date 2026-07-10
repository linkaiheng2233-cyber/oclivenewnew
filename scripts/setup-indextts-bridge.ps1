# Bootstrap helper for the user-local IndexTTS bridge used by chat-pro.
# This only prepares the local runtime; the heavy model weights still need to
# exist under D:/oclive-dev-artifacts/tts-servers/index-tts/checkpoints.

$ErrorActionPreference = "Stop"
$Workspace = if ($env:OCLIVE_TTS_SERVERS_DIR) { $env:OCLIVE_TTS_SERVERS_DIR } else { "D:/oclive-dev-artifacts/tts-servers" }
$IndexRoot = Join-Path $Workspace "index-tts"
$Venv = Join-Path $IndexRoot ".venv"

if (-not (Test-Path $IndexRoot)) {
  throw "IndexTTS root not found: $IndexRoot"
}

Write-Host "Preparing IndexTTS bridge runtime in $IndexRoot"
Set-Location $IndexRoot

if (-not (Get-Command uv -ErrorAction SilentlyContinue)) {
  throw "uv is required. Install from https://docs.astral.sh/uv/"
}

if (-not (Test-Path $Venv)) {
  Write-Host "Creating .venv with Python 3.11..."
  uv venv --python 3.11 $Venv
}

Write-Host "Syncing IndexTTS dependencies (this may take a while)..."
uv sync --python $Venv --extra webui

Write-Host ""
Write-Host "Bridge runtime prepared."
Write-Host "Start script: $Workspace/scripts/start-indextts-7860.ps1"
