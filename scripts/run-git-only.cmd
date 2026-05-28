@echo off
cd /d C:\Users\koosh\Dino
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\git-only.ps1 > "%TEMP%\git-only-out.log" 2>&1
echo EXIT=%ERRORLEVEL%>> "%TEMP%\git-only-out.log"
