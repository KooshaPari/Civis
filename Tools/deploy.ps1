<#
.SYNOPSIS
    Build release artifacts and package the Civis standalone client.
#>
[CmdletBinding()]
param(
    [string]$OutDir = 'dist'
)

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent $PSScriptRoot
$OutPath = Join-Path $RepoRoot $OutDir
$Stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$ZipName = "civis-standalone-windows-$Stamp.zip"
$ZipPath = Join-Path $OutPath $ZipName
$Staging = Join-Path $OutPath "stage-$Stamp"

New-Item -ItemType Directory -Force -Path $OutPath, $Staging | Out-Null

Write-Host "[deploy] Building release..." -ForegroundColor Cyan
Push-Location $RepoRoot
try {
    & cargo build --release -p civ-bevy-ref --features bevy,egui --bin civ-standalone
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally { Pop-Location }

$exe = Join-Path $RepoRoot 'target/release/civ-standalone.exe'
if (-not (Test-Path $exe)) { Write-Host "[deploy] Missing exe: $exe" -ForegroundColor Red; exit 1 }

Copy-Item $exe -Destination $Staging
if (Test-Path (Join-Path $RepoRoot 'assets')) {
    Copy-Item -Recurse (Join-Path $RepoRoot 'assets') -Destination (Join-Path $Staging 'assets')
}
if (Test-Path (Join-Path $RepoRoot 'README.md')) {
    Copy-Item (Join-Path $RepoRoot 'README.md') -Destination $Staging
}

Write-Host "[deploy] Packaging $ZipName ..." -ForegroundColor Cyan
if (Test-Path $ZipPath) { Remove-Item $ZipPath -Force }
Compress-Archive -Path "$Staging\*" -DestinationPath $ZipPath -CompressionLevel Optimal
Remove-Item -Recurse -Force $Staging

$size = (Get-Item $ZipPath).Length / 1MB
Write-Host ("[deploy] Done: {0} ({1:N2} MB)" -f $ZipPath, $size) -ForegroundColor Green
