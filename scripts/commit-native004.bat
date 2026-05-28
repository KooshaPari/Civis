@echo off
set LOG=%TEMP%\commit-native004.log
echo === %DATE% %TIME% === > "%LOG%"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0git-only.ps1" >> "%LOG%" 2>&1
echo EXIT=%ERRORLEVEL%>> "%LOG%"
type "%LOG%"
exit /b %ERRORLEVEL%
