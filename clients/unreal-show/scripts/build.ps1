#Requires -Version 5.1
<#
.SYNOPSIS
  Build CivShow (rust-shim + Unreal Editor target) from the CLI.

.EXIT CODES
  0  Success
  1  Build failed (cargo, UBT, or copy step)
  2  Unreal Engine 5.4 not found (install UE or set UE_ROOT)
#>
[CmdletBinding()]
param(
    [ValidateSet('Development', 'DebugGame', 'Shipping')]
    [string] $Configuration = 'Development',

    [switch] $SkipUe,
    [switch] $SkipRust
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ProjectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$Uproject = Join-Path $ProjectRoot 'CivShow.uproject'
$RustShimDir = Join-Path $ProjectRoot 'Source\Civis\rust-shim'
$LibDir = Join-Path $ProjectRoot 'Source\Civis\lib'
$LibName = 'civis_unreal_ffi.lib'
$UeVersion = '5.4'
$UeFolder = "UE_$UeVersion"
$EditorTarget = 'CivShowEditor'
$Platform = 'Win64'

function Write-Step([string] $Message) {
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Get-EpicLauncherUeRoots {
    $paths = [System.Collections.Generic.List[string]]::new()
    $manifest = Join-Path $env:PROGRAMDATA 'Epic\UnrealEngineLauncher\LauncherInstalled.dat'
    if (-not (Test-Path -LiteralPath $manifest)) {
        return $paths
    }
    try {
        $json = Get-Content -LiteralPath $manifest -Raw -Encoding UTF8 | ConvertFrom-Json
        foreach ($entry in @($json.InstallationList)) {
            $loc = [string]$entry.InstallLocation
            if ([string]::IsNullOrWhiteSpace($loc)) { continue }
            $parent = Split-Path -Parent $loc.TrimEnd('\', '/')
            $candidate = Join-Path $parent $UeFolder
            if (Test-Path -LiteralPath $candidate) {
                $paths.Add((Resolve-Path -LiteralPath $candidate).Path)
            }
        }
    }
    catch {
        Write-Verbose "Could not parse Epic launcher manifest: $_"
    }
    return $paths
}

function Get-UeRoot {
    if ($env:UE_ROOT -and (Test-Path -LiteralPath $env:UE_ROOT)) {
        return (Resolve-Path -LiteralPath $env:UE_ROOT).Path
    }

    $candidates = @(
        (Join-Path ${env:ProgramFiles} "Epic Games\$UeFolder"),
        (Join-Path ${env:ProgramFiles(x86)} "Epic Games\$UeFolder"),
        (Join-Path $env:LOCALAPPDATA "EpicGamesLauncher\Engine\$UeFolder")
    )

    foreach ($path in $candidates) {
        if ($path -and (Test-Path -LiteralPath $path)) {
            return (Resolve-Path -LiteralPath $path).Path
        }
    }

    foreach ($path in (Get-EpicLauncherUeRoots)) {
        if (Test-Path -LiteralPath $path) {
            return $path
        }
    }

    return $null
}

function Get-UnrealBuildTool([string] $UeRoot) {
    $ubt = Join-Path $UeRoot 'Engine\Binaries\DotNET\UnrealBuildTool\UnrealBuildTool.exe'
    if (Test-Path -LiteralPath $ubt) { return $ubt }

    $buildBat = Join-Path $UeRoot 'Engine\Build\BatchFiles\Build.bat'
    if (Test-Path -LiteralPath $buildBat) { return $buildBat }

    return $null
}

function Invoke-RustShimBuild {
    Write-Step "Building rust-shim (release)"
    Push-Location $RustShimDir
    try {
        & cargo build --release
        if ($LASTEXITCODE -ne 0) {
            throw "cargo build failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }

    $builtLib = Join-Path $RustShimDir "target\release\$LibName"
    if (-not (Test-Path -LiteralPath $builtLib)) {
        throw "Expected static library not found: $builtLib"
    }

    New-Item -ItemType Directory -Force -Path $LibDir | Out-Null
    Copy-Item -LiteralPath $builtLib -Destination (Join-Path $LibDir $LibName) -Force
    Write-Host "Copied $LibName -> $LibDir"
}

function Invoke-GenerateProjectFiles([string] $UeRoot) {
    $targetFile = Join-Path $ProjectRoot 'Source\CivShowEditor.Target.cs'
    if (Test-Path -LiteralPath $targetFile) {
        return
    }

    Write-Step 'Generating Visual Studio / UBT target files (first-time)'
    $buildBat = Join-Path $UeRoot 'Engine\Build\BatchFiles\Build.bat'
    if (-not (Test-Path -LiteralPath $buildBat)) {
        Write-Warning 'Build.bat not found; skipping -projectfiles (UBT may fail without Target.cs)'
        return
    }

    & $buildBat -projectfiles -project="$Uproject" -game -engine -progress
    if ($LASTEXITCODE -ne 0) {
        throw "Generate project files failed with exit code $LASTEXITCODE"
    }
}

function Invoke-UnrealBuild([string] $UeRoot, [string] $UbtOrBuildBat) {
    Write-Step "Building $EditorTarget $Platform $Configuration"
    Invoke-GenerateProjectFiles -UeRoot $UeRoot

    $projectArg = "-Project=`"$Uproject`""

    if ($UbtOrBuildBat -like '*.exe') {
        & $UbtOrBuildBat $EditorTarget $Platform $Configuration $projectArg '-WaitMutex'
    }
    else {
        & $UbtOrBuildBat $EditorTarget $Platform $Configuration $projectArg '-WaitMutex'
    }

    if ($LASTEXITCODE -ne 0) {
        throw "Unreal build failed with exit code $LASTEXITCODE"
    }
}

# --- main ---
Write-Step "CivShow automated build (project: $ProjectRoot)"

if (-not (Test-Path -LiteralPath $Uproject)) {
    Write-Error "Missing uproject: $Uproject"
    exit 1
}

if (-not $SkipRust) {
    try {
        Invoke-RustShimBuild
    }
    catch {
        Write-Error $_
        exit 1
    }
}
else {
    Write-Host 'Skipping rust-shim (--SkipRust)' -ForegroundColor Yellow
}

if ($SkipUe) {
    Write-Host 'Skipping Unreal build (--SkipUe)' -ForegroundColor Yellow
    exit 0
}

$ueRoot = Get-UeRoot
if (-not $ueRoot) {
    Write-Host @"

Unreal Engine $UeVersion was not found.

Checked:
  - Environment variable UE_ROOT
  - C:\Program Files\Epic Games\$UeFolder
  - Epic Launcher manifest ($env:PROGRAMDATA\Epic\UnrealEngineLauncher\LauncherInstalled.dat)

Install UE $UeVersion from Epic Games Launcher, or set UE_ROOT to your engine directory.
Rust shim was built successfully; only the UE compile step was skipped.

"@ -ForegroundColor Yellow
    exit 2
}

Write-Host "Using UE_ROOT: $ueRoot" -ForegroundColor Green

$ubt = Get-UnrealBuildTool -UeRoot $ueRoot
if (-not $ubt) {
    Write-Host "UE directory found but UnrealBuildTool / Build.bat missing under: $ueRoot" -ForegroundColor Red
    exit 2
}

try {
    Invoke-UnrealBuild -UeRoot $ueRoot -UbtOrBuildBat $ubt
}
catch {
    Write-Error $_
    exit 1
}

Write-Step 'Build finished successfully'
exit 0
