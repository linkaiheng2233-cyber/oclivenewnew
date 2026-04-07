@echo off
chcp 65001 >nul
setlocal
cd /d "%~dp0"

echo [start-dev] 将打开两个窗口：1) Vite  2) Tauri（需已在本目录执行过 npm install）
echo.

start "oclivenewnew - Vite" cmd /k cd /d "%CD%" ^&^& npm run dev
timeout /t 3 /nobreak >nul
start "oclivenewnew - Tauri" cmd /k cd /d "%CD%\src-tauri" ^&^& npx --no-install tauri dev -c tauri.vite-external.json

echo 已启动（本窗口可关闭）。
exit /b 0
