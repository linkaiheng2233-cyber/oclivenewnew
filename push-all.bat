@echo off
chcp 65001 >nul
setlocal
cd /d "%~dp0"

if not exist ".git" (
  echo ERROR: not a git repo
  exit /b 1
)

echo === git remote ===
git remote -v
echo.
echo === push main ===
git push origin main
if errorlevel 1 (
  echo.
  echo FAILED: git push origin main
  echo Tip: check network / VPN / firewall; try SSH: git remote set-url origin git@github.com:linkaiheng2233-cyber/oclivenewnew.git
  exit /b 1
)

echo.
echo === push tags (if any) ===
git push origin --tags
REM tags push may "fail" with no tags - ignore

echo.
echo Done.
exit /b 0
