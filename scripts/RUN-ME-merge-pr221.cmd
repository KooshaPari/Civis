@echo off
setlocal EnableDelayedExpansion
title DINOForge merge PR 221
cd /d C:\Users\koosh\Dino

reg import C:\Users\koosh\.cursor\fix-cursor-shell.reg >nul 2>&1
set "MIC_LD_LIBRARY_PATH="
set "INTEL_DEV_REDIST="

echo [1/2] CodeRabbit config on main + merge PR 221...
call C:\Users\koosh\Dino\scripts\coderabbit-main-config.bat
set ERR=!ERRORLEVEL!

echo.
echo [2/2] Result written to scripts\coderabbit-orchestration.log
if exist scripts\coderabbit-orchestration.log type scripts\coderabbit-orchestration.log

if !ERR! neq 0 (
  echo.
  echo Fallback: approve and merge manually:
  echo   gh pr review 221 --repo KooshaPari/Dino --approve
  echo   gh pr merge 221 --merge --repo KooshaPari/Dino
)

pause
exit /b !ERR!
