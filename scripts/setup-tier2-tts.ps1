# Tier-2 TTS servers for OCLive voice.asr HTTP profiles (user-local, outside kernel).
# Workspace: D:/oclive-dev-artifacts/tts-servers/
# Requires: Python 3.12+, NVIDIA GPU, network (hf-mirror / modelscope in CN).

$ErrorActionPreference = "Stop"
$Workspace = if ($env:OCLIVE_TTS_SERVERS_DIR) { $env:OCLIVE_TTS_SERVERS_DIR } else { "D:/oclive-dev-artifacts/tts-servers" }
$RepoRoot = Split-Path $PSScriptRoot -Parent
$ModelsRoot = Join-Path $env:APPDATA "OCLive/models/tts"

New-Item -ItemType Directory -Force -Path $Workspace, "$Workspace/scripts", "$Workspace/qwen3", "$Workspace/gpt-sovits", "$Workspace/fish-speech", "$Workspace/index-tts" | Out-Null

function Write-ManifestStub {
  param([string]$Id, [string]$Engine, [hashtable]$Extra = @{})
  $dir = Join-Path $ModelsRoot $Id
  New-Item -ItemType Directory -Force -Path $dir | Out-Null
  $manifest = @{
    id = $Id
    engine = $Engine
    label = $Id
  }
  foreach ($k in $Extra.Keys) { $manifest[$k] = $Extra[$k] }
  $manifest | ConvertTo-Json -Depth 5 | ForEach-Object {
    [System.IO.File]::WriteAllText((Join-Path $dir "MANIFEST.json"), $_, (New-Object System.Text.UTF8Encoding $false))
  }
}

Write-Host "=== 1/5 edge-tts (OCLive voice-loop .venv) ==="
$VoiceVenv = Join-Path $RepoRoot "examples/voice-loop-minimal/.venv/Scripts/pip.exe"
if (Test-Path $VoiceVenv) {
  & $VoiceVenv install "edge-tts>=7.0.0" -q
  Write-Host "edge-tts installed in voice-loop .venv"
} else {
  Write-Warning "voice-loop .venv missing — pip install edge-tts manually"
}

Write-Host "=== 2/5 Qwen3-TTS (faster-qwen3-tts, port 8080) ==="
$QwenPy = Join-Path $Workspace "qwen3/.venv/Scripts/python.exe"
if (-not (Test-Path $QwenPy)) {
  py -3.12 -m venv (Join-Path $Workspace "qwen3/.venv")
  $env:HF_ENDPOINT = "https://hf-mirror.com"
  & (Join-Path $Workspace "qwen3/.venv/Scripts/pip.exe") install --upgrade pip
  & (Join-Path $Workspace "qwen3/.venv/Scripts/pip.exe") install torch torchaudio --index-url https://download.pytorch.org/whl/cu128
  & (Join-Path $Workspace "qwen3/.venv/Scripts/pip.exe") install "faster-qwen3-tts[demo]"
}
Write-ManifestStub -Id "local-qwen3-tts-http" -Engine "qwen3-tts-http" -Extra @{
  sidecar_endpoint = "http://127.0.0.1:8080"
  voice = "Vivian"
  language = "Chinese"
}

Write-Host "=== 3/5 GPT-SoVITS (ModelScope 7z, port 9880) ==="
$GsvPackage = Join-Path $Workspace "gpt-sovits/GPT-SoVITS-v2pro-20250604-nvidia50.7z"
$GsvDir = Join-Path $Workspace "gpt-sovits/GPT-SoVITS"
$SevenZip = @("C:/Program Files/7-Zip/7z.exe", "C:/Program Files (x86)/7-Zip/7z.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not (Test-Path (Join-Path $GsvDir "api_v2.py"))) {
  if (-not (Test-Path $GsvPackage)) {
    Write-Host "Downloading GPT-SoVITS nvidia50 package (~8.8GB) from ModelScope..."
    $url = "https://www.modelscope.cn/models/FlowerCry/gpt-sovits-7z-pacakges/resolve/master/GPT-SoVITS-v2pro-20250604-nvidia50.7z"
    New-Item -ItemType Directory -Force -Path (Split-Path $GsvPackage) | Out-Null
    curl.exe -L -o $GsvPackage $url
  }
  if (-not $SevenZip) { throw "7-Zip required to extract GPT-SoVITS package. Install: winget install 7zip.7zip" }
  Write-Host "Extracting GPT-SoVITS..."
  & $SevenZip x $GsvPackage "-o$GsvDir" -y
}
Write-ManifestStub -Id "local-gpt-sovits-http" -Engine "gpt-sovits-http" -Extra @{
  sidecar_endpoint = "http://127.0.0.1:9880"
  synthesize_path = "/tts"
  text_language = "zh"
  prompt_language = "zh"
}

Write-Host "=== 4/5 Fish Speech (git / pip, port 9881) — manual if git blocked ==="
Write-ManifestStub -Id "local-fish-speech-http" -Engine "fish-speech-http" -Extra @{
  sidecar_endpoint = "http://127.0.0.1:9881"
  api_style = "openai-speech-v1"
}

Write-Host "=== 5/5 IndexTTS (port 7860) — manual if git blocked ==="
Write-ManifestStub -Id "local-indextts-http" -Engine "indextts-http" -Extra @{
  sidecar_endpoint = "http://127.0.0.1:7860"
  probe_path = "/health"
  synthesize_path = "/infer"
}

Write-ManifestStub -Id "edge-tts-zh" -Engine "edge-tts" -Extra @{ voice = "zh-CN-XiaoxiaoNeural" }

Write-Host ""
Write-Host "Done. Start servers:"
Write-Host "  Qwen3:      $Workspace/scripts/start-qwen3-8080.ps1"
Write-Host "  GPT-SoVITS: $Workspace/scripts/start-gptsovits-9880.ps1"
Write-Host "Then switch TTS profile in OCLive Voice Settings."
