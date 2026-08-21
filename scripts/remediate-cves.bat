@echo off
REM Civis remediation script — runs after the build environment is ready.
REM Bumps transitive CVE-affected crates, removes unused deps, and re-runs gates.
REM Each step is idempotent; abort on first failure with `set -e` semantics.

setlocal EnableExtensions EnableDelayedExpansion

set ROOT=%~dp0..\..\
cd /d "%ROOT%" || (echo [ERR] failed to cd to %ROOT% & exit /b 1)

set CARGO=%USERPROFILE%\.cargo\bin\cargo.exe
if not exist "%CARGO%" set CARGO=cargo

echo === [1/8] Bumping transitive deps with known upstream fixes ===
"%CARGO%" update -p wasmtime@45.0.3 --precise 46.0.2    || goto :err
"%CARGO%" update -p h2@0.4.15 --precise 0.4.16          || goto :err
"%CARGO%" update -p webbrowser@1.2.1 --precise 1.2.2    || goto :err
"%CARGO%" update -p quick-xml@0.39.4 --precise 0.41.0   || goto :err
REM protobuf 2.x → 3.x is a breaking change; requires manual upgrade of consumers.
REM See crates/server/Cargo.toml (opentelemetry stack) and crates/civis-cli if present.

echo === [2/8] Running cargo fmt --check ===
"%CARGO%" fmt --check || goto :err

echo === [3/8] Running cargo clippy --workspace ===
"%CARGO%" clippy --workspace --all-targets -- -D warnings || goto :err

echo === [4/8] Running cargo test --workspace ===
"%CARGO%" test --workspace --no-fail-fast || goto :err

echo === [5/8] Running cargo deny check ===
"%CARGO%" deny check || goto :err

echo === [6/8] Running cargo audit ===
"%CARGO%" audit || goto :err

echo === [7/8] Running cargo machete (unused deps) ===
"%CARGO%" machete --fix || "%CARGO%" machete || goto :err

echo === [8/8] Final cargo test --workspace ===
"%CARGO%" test --workspace || goto :err

echo === ALL GATES GREEN ===
exit /b 0

:err
echo.
echo [FAIL] Step above exited non-zero. Inspect cargo output and re-run.
exit /b 1
