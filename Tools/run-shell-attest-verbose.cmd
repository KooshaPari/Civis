@echo off
set PATH=C:\Users\koosh\.cargo\bin;%PATH%
set CARGO_TARGET_DIR=E:\civis-target-attest
set TMP=E:\tmp
set TEMP=E:\tmp
set CARGO_NET_OFFLINE=
set CARGO_HOME=%USERPROFILE%\.cargo
set CARGO_PROFILE_TEST_DEBUG=0
if not exist E:\tmp mkdir E:\tmp
cd /d C:\Users\koosh\Civis
echo START %DATE% %TIME% (verbose) > E:\tmp\shell_attest_latest.txt
C:\Users\koosh\.cargo\bin\cargo.exe test -p civ-bevy-ref --features bevy,egui --test shell_attest -j 1 -v -- --nocapture >> E:\tmp\shell_attest_latest.txt 2>&1
echo EXITCODE=%ERRORLEVEL% >> E:\tmp\shell_attest_latest.txt
