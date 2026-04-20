@echo off
setlocal
cd /d "%~dp0"

if not exist "package.json" (
  echo [start-dev] ERROR: package.json not found in repo root.
  exit /b 1
)

echo [start-dev] repo: %CD%
echo.

set "VITE_PID="
for /f %%p in ('powershell -NoProfile -Command "(Get-NetTCPConnection -LocalPort 1420 -State Listen -ErrorAction SilentlyContinue ^| Select-Object -First 1 -ExpandProperty OwningProcess)"') do set "VITE_PID=%%p"

if defined VITE_PID (
  echo [start-dev] detected existing Vite on 1420 ^(PID %VITE_PID%^), reuse it.
) else (
  echo [start-dev] starting Vite dev server in a new window...
  start "oclive-vite" cmd /c "cd /d \"%CD%\" && npm run dev"
  timeout /t 2 /nobreak >nul
)

echo [start-dev] launching Tauri against external Vite...
call npm run dev:tauri
exit /b %ERRORLEVEL%
