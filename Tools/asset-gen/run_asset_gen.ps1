<#
.SYNOPSIS
    Run the headless Blender actor-generation pipeline for Civis.

.DESCRIPTION
    Locates Blender 4.x, then invokes gen_actor.py twice (humanoid + herd)
    to produce CC0 .glb files into clients/bevy-ref/assets/models/.

.EXAMPLE
    pwsh -File tools/asset-gen/run_asset_gen.ps1
    pwsh -File tools/asset-gen/run_asset_gen.ps1 -Variant humanoid -Height 2.0 -LimbScale 1.1

.PARAMETER Variant
    Which variant to generate: 'humanoid', 'herd', or 'all' (default: all)

.PARAMETER Height
    Override body height in metres (optional; only meaningful with -Variant humanoid or herd)

.PARAMETER LimbScale
    Limb length multiplier (optional, default 1.0)
#>
param(
    [ValidateSet("humanoid", "herd", "all")]
    [string]$Variant = "all",
    [double]$Height  = 0,
    [double]$LimbScale = 1.0
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ---------------------------------------------------------------------------
# Locate Blender
# ---------------------------------------------------------------------------
$BlenderExe = $null

# 1. Check PATH
$onPath = Get-Command blender -ErrorAction SilentlyContinue
if ($onPath) {
    $BlenderExe = $onPath.Source
}

# 2. Scan common Windows install locations
if (-not $BlenderExe) {
    $LocalApp   = $env:LOCALAPPDATA
    $Candidates = [System.Collections.Generic.List[string]]::new()
    $Candidates.Add("C:/Program Files/Blender Foundation/Blender 4.5/blender.exe")
    $Candidates.Add("C:/Program Files/Blender Foundation/Blender 4.4/blender.exe")
    $Candidates.Add("C:/Program Files/Blender Foundation/Blender 4.3/blender.exe")
    $Candidates.Add("C:/Program Files/Blender Foundation/Blender 4.2/blender.exe")
    $Candidates.Add("C:/Program Files/Blender Foundation/Blender 4.1/blender.exe")
    $Candidates.Add("C:/Program Files/Blender Foundation/Blender 4.0/blender.exe")
    $Candidates.Add("$LocalApp/Programs/Blender Foundation/Blender 4.5/blender.exe")
    $Candidates.Add("$LocalApp/Programs/Blender Foundation/Blender 4.4/blender.exe")
    # Also try dynamic glob of the Blender Foundation folder
    $BFRoot = "C:/Program Files/Blender Foundation"
    if (Test-Path $BFRoot) {
        $found = Get-ChildItem -Path $BFRoot -Filter "blender.exe" -Recurse -ErrorAction SilentlyContinue |
                 Sort-Object -Property FullName -Descending |
                 Select-Object -First 1
        if ($found) { $Candidates.Insert(0, $found.FullName) }
    }

    foreach ($c in $Candidates) {
        if (Test-Path $c) { $BlenderExe = $c; break }
    }
}

if (-not $BlenderExe) {
    Write-Error @"
ERROR: Blender not found on PATH or common install locations.

Please install Blender 4.x LTS from:
    https://www.blender.org/download/lts/

Then either:
  a) Add the Blender directory to your PATH, OR
  b) Re-run this script (it will auto-detect the install).

Searched paths:
  - PATH (blender.exe)
  - C:/Program Files/Blender Foundation/Blender 4.x/blender.exe
  - $env:LOCALAPPDATA/Programs/Blender Foundation/...
"@
    exit 1
}

Write-Host "[asset-gen] Using Blender: $BlenderExe" -ForegroundColor Cyan

# ---------------------------------------------------------------------------
# Resolve paths
# ---------------------------------------------------------------------------
$RepoRoot  = (Resolve-Path "$PSScriptRoot/../..").Path
$Script    = Join-Path $PSScriptRoot "gen_actor.py"
$OutDir    = Join-Path $RepoRoot "clients/bevy-ref/assets/models"

if (-not (Test-Path $OutDir)) {
    New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
}

# ---------------------------------------------------------------------------
# Build extra arg lists
# ---------------------------------------------------------------------------
function Build-ExtraArgs([string]$v) {
    $extra = @("--variant", $v)
    if ($Height -gt 0) { $extra += @("--height", "$Height") }
    $extra += @("--limb-scale", "$LimbScale")
    return $extra
}

# ---------------------------------------------------------------------------
# Run Blender headless
# ---------------------------------------------------------------------------
function Invoke-Blender([string]$v) {
    $extraArgs = Build-ExtraArgs $v
    $allArgs   = @("-b", "-P", $Script, "--") + $extraArgs
    Write-Host "`n[asset-gen] Generating variant=$v ..." -ForegroundColor Yellow
    Write-Host "  CMD: $BlenderExe $($allArgs -join ' ')" -ForegroundColor DarkGray

    $proc = Start-Process -FilePath $BlenderExe `
                          -ArgumentList $allArgs `
                          -WorkingDirectory $RepoRoot `
                          -NoNewWindow -PassThru -Wait
    if ($proc.ExitCode -ne 0) {
        Write-Error "[asset-gen] Blender exited with code $($proc.ExitCode) for variant=$v"
        return $false
    }
    return $true
}

# ---------------------------------------------------------------------------
# Execute
# ---------------------------------------------------------------------------
$ok = $true
$variants = if ($Variant -eq "all") { @("humanoid", "herd") } else { @($Variant) }

foreach ($v in $variants) {
    if (-not (Invoke-Blender $v)) { $ok = $false }
}

# ---------------------------------------------------------------------------
# Report results
# ---------------------------------------------------------------------------
Write-Host "`n[asset-gen] Results:" -ForegroundColor Cyan
$glbFiles = Get-ChildItem -Path $OutDir -Filter "*_gen.glb" -ErrorAction SilentlyContinue
if ($glbFiles) {
    foreach ($f in $glbFiles) {
        $kb = [math]::Round($f.Length / 1KB, 1)
        Write-Host "  OK  $($f.Name)  ($kb KB)" -ForegroundColor Green
    }
} else {
    Write-Host "  (no *_gen.glb files found in $OutDir)" -ForegroundColor Red
}

if (-not $ok) {
    Write-Host "`n[asset-gen] One or more variants FAILED." -ForegroundColor Red
    exit 1
}

Write-Host "`n[asset-gen] Done. Files in: $OutDir" -ForegroundColor Green
