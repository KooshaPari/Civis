# Build mods/example-policy/mod.wasm from civlab-sdk (wasm32-wasi).
param(
    [string]$OutPath = (Join-Path $PSScriptRoot "..\mods\example-policy\mod.wasm")
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$SdkDir = Join-Path $Root "crates\civlab-sdk"

Push-Location $SdkDir
try {
    rustup target add wasm32-wasi 2>$null | Out-Null
    cargo build --release --target wasm32-wasi
    $Built = Join-Path $SdkDir "target\wasm32-wasi\release\civlab_sdk.wasm"
    if (-not (Test-Path $Built)) {
        throw "Expected wasm artifact at $Built"
    }
    $Dest = Resolve-Path (Split-Path $OutPath -Parent) -ErrorAction SilentlyContinue
    if (-not $Dest) {
        New-Item -ItemType Directory -Force -Path (Split-Path $OutPath -Parent) | Out-Null
    }
    Copy-Item -Force $Built $OutPath
    Write-Host "Wrote $OutPath"
}
finally {
    Pop-Location
}
