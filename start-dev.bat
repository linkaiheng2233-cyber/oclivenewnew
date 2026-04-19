@echo off
chcp 65001 >nul
setlocal
cd /d "%~dp0"
echo [start-dev] scripts\start-dev.ps1 -^> npm run tauri:dev
echo [start-dev] Skip verify: set OCLIVE_SKIP_VERIFY=1
echo.
if not exist "%~dp0scripts\start-dev.ps1" (
  echo ERROR: missing scripts\start-dev.ps1
  exit /b 1
)
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\start-dev.ps1"
exit /b %ERRORLEVEL%
