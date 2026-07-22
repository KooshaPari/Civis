param(
    [ValidateRange(1, 600)]
    [int]$Frames = 5
)

$ErrorActionPreference = "Stop"
$env:CIVIS_SMOKE_FRAMES = $Frames.ToString()
$env:BEVY_ASSET_ROOT = Join-Path (Get-Location) "clients/bevy-ref"

$exe = if ($env:CARGO_TARGET_DIR) {
    Join-Path $env:CARGO_TARGET_DIR "debug/civ-standalone.exe"
} else {
    Join-Path (Get-Location) "target/debug/civ-standalone.exe"
}

if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) {
    throw "Standalone binary not found: $exe"
}

& $exe
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
