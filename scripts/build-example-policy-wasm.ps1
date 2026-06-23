# Build mods/example-policy/mod.wasm from civlab-sdk (wasm32-unknown-unknown).
# Prefer: just civis-3d-mod-wasm
param(
    [string]$OutPath = (Join-Path $PSScriptRoot "..\mods\example-policy\mod.wasm")
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$Target = "wasm32-unknown-unknown"
$Wasm = Join-Path $Root "target\$Target\release\civlab_sdk.wasm"

Push-Location (Join-Path $Root "crates\civlab-sdk")
try {
    cmd /c "rustup target add $Target >nul 2>&1"
    cargo rustc --release --target $Target --crate-type cdylib
    if ($LASTEXITCODE -ne 0) { throw "cargo rustc failed" }
}
finally {
    Pop-Location
}

if (-not (Test-Path -LiteralPath $Wasm)) {
    throw "missing $Wasm (run from repo root via just civis-3d-mod-wasm)"
}
$Parent = Split-Path $OutPath -Parent
if (-not (Test-Path -LiteralPath $Parent)) {
    New-Item -ItemType Directory -Force -Path $Parent | Out-Null
}
Copy-Item -Force $Wasm $OutPath
Write-Host "Wrote $OutPath"
