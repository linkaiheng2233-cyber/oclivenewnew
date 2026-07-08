# Download the minimal checkpoint set needed by the local IndexTTS bridge.
# This targets the user-local tts-servers workspace used by chat-pro.

$ErrorActionPreference = "Stop"
$Workspace = if ($env:OCLIVE_TTS_SERVERS_DIR) { $env:OCLIVE_TTS_SERVERS_DIR } else { "D:/oclive-dev-artifacts/tts-servers" }
$IndexRoot = Join-Path $Workspace "index-tts"
$Py = Join-Path $IndexRoot ".venv\Scripts\python.exe"
$Target = Join-Path $IndexRoot "checkpoints"
$Repo = if ($env:OCLIVE_INDEXTTS_REPO) { $env:OCLIVE_INDEXTTS_REPO } else { "IndexTeam/IndexTTS-1.5" }

if (-not (Test-Path $Py)) {
  throw "Missing $Py — run scripts/setup-indextts-bridge.ps1 first"
}

New-Item -ItemType Directory -Force -Path $Target | Out-Null
Set-Location $IndexRoot

Write-Host "Downloading IndexTTS checkpoints from $Repo into $Target"
& $Py -m huggingface_hub download $Repo `
  config.yaml `
  bigvgan_discriminator.pth `
  bigvgan_generator.pth `
  bpe.model `
  dvae.pth `
  gpt.pth `
  unigram_12000.vocab `
  --local-dir $Target

Write-Host ""
Write-Host "Checkpoint download complete."
Write-Host "Start bridge: $Workspace/scripts/start-indextts-7860.ps1"
