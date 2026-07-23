param(
    [ValidateRange(1, 600)]
    [int]$Frames = 5
)

$ErrorActionPreference = "Stop"
$env:CIVIS_SMOKE_FRAMES = $Frames.ToString()
$env:BEVY_ASSET_ROOT = Join-Path (Get-Location) "clients/bevy-ref"

$root = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path (Get-Location) "target" }
$candidates = @(
    (Join-Path $root "debug/civ-standalone.exe"),
    (Join-Path $root "release/civ-standalone.exe")
)
$exe = $candidates | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1

if (-not $exe) {
    throw "Standalone binary not found under $root (tried debug then release)"
}
Write-Host "[smoke] using $exe"

& $exe
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
