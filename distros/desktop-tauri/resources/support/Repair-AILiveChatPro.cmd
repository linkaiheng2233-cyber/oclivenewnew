@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Repair-AILiveChatPro.ps1" %*
exit /b %ERRORLEVEL%
