$ErrorActionPreference = "Continue"
Set-StrictMode -Version Latest
$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot
if ($env:OCLIVE_SKIP_VERIFY -ne "1") {
  npm run verify:ui
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
$LogDir = Join-Path $RepoRoot "logs"
$LogFile = Join-Path $LogDir "dev-start.log"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
"`n=== $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') tauri:dev ===" | Add-Content -Path $LogFile -Encoding utf8
Write-Host "[start-dev] repo: $RepoRoot"
Write-Host "[start-dev] log:  $LogFile"
npm run tauri:dev *>&1 | Tee-Object -FilePath $LogFile -Append
