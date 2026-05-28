<#
.SYNOPSIS
Install DINOForge optional development tools (UnityExplorer, etc.)

.DESCRIPTION
Downloads and installs UnityExplorer BepInEx plugin to the game's plugins directory.
Idempotent: safe to run multiple times. Skips if already installed.

.PARAMETER GamePath
Path to the DINO game installation. Defaults to value from Directory.Build.props or GIT_GAME_INSTALL_PATH env var.

.PARAMETER Force
If specified, reinstall even if already present.

.EXAMPLE
./install-dev-tools.ps1
./install-dev-tools.ps1 -Force
./install-dev-tools.ps1 -GamePath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
#>

param(
    [string]$GamePath = "",
    [switch]$Force
)

$ErrorActionPreference = "Continue"

# Resolve game path
if ([string]::IsNullOrWhiteSpace($GamePath)) {
    # Try environment variable first
    $GamePath = $env:GIT_GAME_INSTALL_PATH

    # Fall back to Directory.Build.props
    if ([string]::IsNullOrWhiteSpace($GamePath)) {
        $propsFile = Join-Path $PSScriptRoot ".." "Directory.Build.props"
        if (Test-Path $propsFile) {
            [xml]$xml = Get-Content $propsFile
            $GamePath = $xml.Project.PropertyGroup.GameInstallPath
        }
    }
}

if ([string]::IsNullOrWhiteSpace($GamePath) -or -not (Test-Path $GamePath)) {
    Write-Warning @"
Could not resolve game install path. Please specify -GamePath or set GIT_GAME_INSTALL_PATH env var.
Expected: "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
"@
    exit 1
}

# Set up target directory
$BepInExPath = Join-Path $GamePath "BepInEx"
$PluginsDir = Join-Path $BepInExPath "plugins"
$DevToolsDir = Join-Path $PluginsDir "dev"
$UnityExplorerDir = Join-Path $DevToolsDir "UnityExplorer"

Write-Host "DINOForge Dev Tools Installer"
Write-Host "==============================="
Write-Host "Game Path: $GamePath"
Write-Host "Target: $UnityExplorerDir"

# Check if already installed
if ((Test-Path $UnityExplorerDir) -and -not $Force) {
    Write-Host "`nUnityExplorer is already installed at $UnityExplorerDir" -ForegroundColor Green
    Write-Host "Use -Force flag to reinstall."
    exit 0
}

# Create dev directory if needed
if (-not (Test-Path $DevToolsDir)) {
    New-Item -ItemType Directory -Path $DevToolsDir -Force | Out-Null
    Write-Host "Created $DevToolsDir"
}

# Download latest release
Write-Host "`nDownloading UnityExplorer from GitHub..."
$tempZip = Join-Path $env:TEMP "UnityExplorer.BepInEx.Mono.zip"

try {
    $releaseUrl = "https://github.com/sinai-dev/UnityExplorer/releases/download/4.8.2/UnityExplorer.BepInEx.Mono.zip"

    # Try to get latest release dynamically
    try {
        $releases = Invoke-RestMethod -Uri "https://api.github.com/repos/sinai-dev/UnityExplorer/releases/latest" `
            -TimeoutSec 10 -ErrorAction SilentlyContinue
        if ($releases -and $releases.assets) {
            $asset = $releases.assets | Where-Object { $_.name -match "BepInEx.Mono" } | Select-Object -First 1
            if ($asset) {
                $releaseUrl = $asset.browser_download_url
            }
        }
    }
    catch {
        Write-Warning "Could not fetch latest release info, using fallback URL"
    }

    Write-Host "Downloading from: $releaseUrl"
    Invoke-WebRequest -Uri $releaseUrl -OutFile $tempZip -TimeoutSec 30

    if (-not (Test-Path $tempZip)) {
        Write-Warning "Download failed. Please check your internet connection."
        exit 1
    }

    Write-Host "Downloaded successfully" -ForegroundColor Green
}
catch {
    Write-Warning "Failed to download UnityExplorer: $_"
    Write-Warning "Please install manually from https://github.com/sinai-dev/UnityExplorer/releases"
    exit 1
}

# Extract
Write-Host "Extracting to $UnityExplorerDir..."
try {
    if (Test-Path $UnityExplorerDir) {
        Remove-Item $UnityExplorerDir -Recurse -Force
    }

    Expand-Archive -Path $tempZip -DestinationPath $UnityExplorerDir -Force
    Write-Host "Extracted successfully" -ForegroundColor Green
}
catch {
    Write-Warning "Failed to extract: $_"
    exit 1
}
finally {
    Remove-Item $tempZip -Force -ErrorAction SilentlyContinue
}

Write-Host "`n✓ UnityExplorer installed successfully!"
Write-Host "`nUsage:"
Write-Host "- Launch the game"
Write-Host "- Press F7 to toggle UnityExplorer"
Write-Host "- Use the hierarchy inspector and C# REPL"
Write-Host "`nDocumentation: https://github.com/sinai-dev/UnityExplorer"
