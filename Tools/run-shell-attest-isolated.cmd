@echo off
set PATH=C:\Users\koosh\.cargo\bin;%PATH%
rem Any value of CARGO_NET_OFFLINE enables offline; clear user env override.
set CARGO_NET_OFFLINE=
set CARGO_HOME=E:\civis-cargo-home
set CARGO_TARGET_DIR=E:\civis-target-attest
set TMP=E:\tmp
set TEMP=E:\tmp
if not exist E:\tmp mkdir E:\tmp
cd /d C:\Users\koosh\Civis
echo START %DATE% %TIME% > E:\tmp\shell_attest_out10.txt
C:\Users\koosh\.cargo\bin\cargo.exe test -p civ-bevy-ref --features bevy,egui --test shell_attest -j 1 -- --nocapture >> E:\tmp\shell_attest_out10.txt 2>&1
echo EXITCODE=%ERRORLEVEL% >> E:\tmp\shell_attest_out10.txt
