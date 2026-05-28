#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Install DumpTools CLI tool for DINOForge

.DESCRIPTION
    Downloads the latest DumpTools standalone executable from GitHub releases
    and installs it to Program Files or /usr/local/bin (macOS/Linux via WSL).

.PARAMETER Version
    Specific version to install (e.g., v0.20.0). Defaults to latest.

.PARAMETER NoAddPath
    Skip adding to PATH (for manual management).

.EXAMPLE
    ./install-DumpTools.ps1
    ./install-DumpTools.ps1 -Version v0.20.0
#>

param(
    [string]$Version = "latest",
    [switch]$NoAddPath = $false
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$repo = "KooshaPari/Dino"
$toolName = "DumpTools"
$executableName = "DINOForge.Tools.DumpTools.exe"

Write-Host "Installing $toolName..." -ForegroundColor Cyan

# Detect OS
$isWindows = $PSVersionTable.Platform -eq 'Win32NT' -or $PSVersionTable.PSVersion.Major -lt 6
$isLinux = $PSVersionTable.OS -match 'Linux'
$isMacOS = $PSVersionTable.OS -match 'Darwin'

if ($isWindows) {
    $rid = "win-x64"
    $installPath = "$env:ProgramFiles\DINOForge\bin"
} elseif ($isLinux -or $isMacOS) {
    $rid = if ($isMacOS -and [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -eq 'Arm64') { "osx-arm64" } elseif ($isMacOS) { "osx-x64" } elseif ([System.Runtime.InteropServices.RuntimeInformation]::ProcessorCount -gt 4) { "linux-arm64" } else { "linux-x64" }
    $installPath = "/usr/local/bin"
} else {
    Write-Error "Unsupported platform"
    exit 1
}

# Get release info
Write-Host "Fetching latest release from GitHub..."
$apiUrl = "https://api.github.com/repos/$repo/releases"
if ($Version -ne "latest") {
    $apiUrl += "/tags/$Version"
} else {
    $apiUrl += "/latest"
}

try {
    $release = Invoke-RestMethod -Uri $apiUrl -Headers @{ 'User-Agent' = 'PowerShell' }
} catch {
    Write-Error "Failed to fetch release info: $_"
    exit 1
}

$tagName = $release.tag_name
$downloadUrl = $release.assets | Where-Object { $_.name -match "DumpTools-$rid" } | Select-Object -ExpandProperty browser_download_url

if (-not $downloadUrl) {
    Write-Error "No binary found for platform: $rid"
    exit 1
}

Write-Host "Tag: $tagName | Platform: $rid"
Write-Host "Download URL: $downloadUrl"

# Download
$tempDir = New-Item -ItemType Directory -Path "$env:TEMP\DINOForge-DumpTools-$([guid]::NewGuid())" -Force
$zipPath = Join-Path $tempDir "DumpTools.zip"

Write-Host "Downloading..."
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -UseBasicParsing
} catch {
    Write-Error "Download failed: $_"
    exit 1
}

# Extract
Write-Host "Extracting..."
Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force

# Install
if ($isWindows) {
    New-Item -ItemType Directory -Path $installPath -Force | Out-Null
    $exePath = Join-Path $tempDir $executableName
    if (-not (Test-Path $exePath)) {
        $exePath = Get-ChildItem $tempDir -Filter "*.exe" | Select-Object -First 1 -ExpandProperty FullName
    }

    Copy-Item $exePath "$installPath\DumpTools.exe" -Force
    Write-Host "Installed to: $installPath\DumpTools.exe"

    if (-not $NoAddPath) {
        # Add to PATH if not already present
        $envPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        if (-not ($envPath -like "*$installPath*")) {
            [Environment]::SetEnvironmentVariable('Path', "$envPath;$installPath", 'User')
            Write-Host "✓ Added to PATH (restart terminal to take effect)"
        }
    }
} else {
    $exePath = Get-ChildItem $tempDir -Executable | Select-Object -First 1 -ExpandProperty FullName
    sudo cp $exePath /usr/local/bin/DumpTools
    sudo chmod +x /usr/local/bin/DumpTools
    Write-Host "Installed to: /usr/local/bin/DumpTools"
}

# Verify
Write-Host "Verifying installation..."
if ($isWindows) {
    & "$installPath\DumpTools.exe" --version
} else {
    /usr/local/bin/DumpTools --version
}

Write-Host "✓ $toolName installed successfully" -ForegroundColor Green

# Cleanup
Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue # remove-item-ok: temp-cleanup-ok: installer download temp dir in $env:TEMP
