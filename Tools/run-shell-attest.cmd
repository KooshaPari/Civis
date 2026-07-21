@echo off
set PATH=C:\Users\koosh\.cargo\bin;%PATH%
set CARGO_TARGET_DIR=E:\civis-target-attest
set TMP=E:\tmp
set TEMP=E:\tmp
rem User env may set CARGO_NET_OFFLINE=true; any value enables offline and breaks resolve.
set CARGO_NET_OFFLINE=
set CARGO_HOME=%USERPROFILE%\.cargo
rem Avoid hung rust-lld on huge Bevy binaries (debuginfo bloated).
rem Bins built beside tests use the dev profile; tests use the test profile.
set CARGO_PROFILE_TEST_DEBUG=0
set CARGO_PROFILE_DEV_DEBUG=0
rem Belt-and-suspenders: force no debuginfo even if a profile setting is missed.
set RUSTFLAGS=-C debuginfo=0
if not exist E:\tmp mkdir E:\tmp
cd /d C:\Users\koosh\Civis
echo START %DATE% %TIME% > E:\tmp\shell_attest_latest.txt
C:\Users\koosh\.cargo\bin\cargo.exe test -p civ-bevy-ref --features bevy,egui --test shell_attest -j 1 -- --nocapture >> E:\tmp\shell_attest_latest.txt 2>&1
echo EXITCODE=%ERRORLEVEL% >> E:\tmp\shell_attest_latest.txt
