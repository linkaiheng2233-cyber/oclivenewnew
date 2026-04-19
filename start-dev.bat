@echo off
setlocal
cd /d "%~dp0"

if not exist "package.json" (
  echo [start-dev] ERROR: package.json not found. Place start-dev.bat in repo root.
  exit /b 1
)

REM Single process: Tauri runs Vite via src-tauri/tauri.conf.json beforeDevCommand.
echo [start-dev] npm run tauri:dev
echo [start-dev] Optional first: npm run verify:ui
echo.

call npm run tauri:dev
exit /b %ERRORLEVEL%
