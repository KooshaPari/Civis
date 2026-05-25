#Requires -Version 5.1
<#
.SYNOPSIS
    Installs the DINOForge Desktop Companion (WinUI 3 GUI).

.DESCRIPTION
    Downloads the latest DINOForge.Companion release from GitHub and installs it to
    %LOCALAPPDATA%\DINOForge\Companion. Optionally creates a desktop shortcut.

.EXAMPLE
    irm https://raw.githubusercontent.com/KooshaPari/Dino/main/scripts/install-companion.ps1 | iex

.EXAMPLE
    # Install a specific version
    $env:DF_VERSION = "v0.11.0"
    irm https://raw.githubusercontent.com/KooshaPari/Dino/main/scripts/install-companion.ps1 | iex
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Repo      = "KooshaPari/Dino"
$AppName   = "DINOForge Companion"
$InstallDir = Join-Path $env:LOCALAPPDATA "DINOForge\Companion"
$ExeName   = "DINOForge.DesktopCompanion.exe"

function Write-Step([string]$msg) { Write-Host "  ==> $msg" -ForegroundColor Cyan }
function Write-OK([string]$msg)   { Write-Host "  [OK] $msg" -ForegroundColor Green }
function Write-Err([string]$msg)  { Write-Host "  [!!] $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "DINOForge Desktop Companion Installer" -ForegroundColor Magenta
Write-Host "======================================" -ForegroundColor Magenta
Write-Host ""

# ── Resolve version ────────────────────────────────────────────────────────────
$Version = $env:DF_VERSION
if (-not $Version) {
    Write-Step "Fetching latest release from GitHub..."
    try {
        $rel = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest" -ErrorAction Stop
        $Version = $rel.tag_name
    } catch {
        Write-Err "Could not reach GitHub API. Check your internet connection or set `$env:DF_VERSION manually."
    }
}
$VersionNum = $Version.TrimStart('v')
Write-OK "Target version: $Version"

# ── Check WindowsAppRuntime ────────────────────────────────────────────────────
Write-Step "Checking WindowsAppRuntime..."
$warReg = "HKLM:\SOFTWARE\Microsoft\WindowsAppSDK\MainChannel"
if (-not (Test-Path $warReg)) {
    Write-Host ""
    Write-Host "  WindowsAppRuntime is required but not installed." -ForegroundColor Yellow
    Write-Host "  Downloading installer (~5 MB)..." -ForegroundColor Yellow
    $warUrl = "https://aka.ms/windowsappruntimeinstall"
    $warTmp = Join-Path $env:TEMP "WindowsAppRuntimeInstall.exe"
    Invoke-WebRequest $warUrl -OutFile $warTmp -UseBasicParsing
    Start-Process $warTmp -ArgumentList "--quiet" -Wait
    Write-OK "WindowsAppRuntime installed"
} else {
    Write-OK "WindowsAppRuntime already installed"
}

# ── Download companion zip ─────────────────────────────────────────────────────
$ZipName = "DINOForge.Companion-v$VersionNum-win-x64.zip"
$ZipUrl  = "https://github.com/$Repo/releases/download/$Version/$ZipName"
$ZipTmp  = Join-Path $env:TEMP $ZipName

Write-Step "Downloading $ZipName..."
try {
    Invoke-WebRequest $ZipUrl -OutFile $ZipTmp -UseBasicParsing
} catch {
    Write-Err "Download failed: $_`nURL: $ZipUrl"
}
Write-OK "Downloaded $([math]::Round((Get-Item $ZipTmp).Length / 1MB, 1)) MB"

# ── Verify SHA256 ─────────────────────────────────────────────────────────────
$Sha256Url = "$ZipUrl.sha256"
try {
    $expectedLine = (Invoke-WebRequest $Sha256Url -UseBasicParsing).Content.Trim()
    $expectedHash = $expectedLine.Split(' ')[0].ToUpper()
    $actualHash   = (Get-FileHash $ZipTmp -Algorithm SHA256).Hash.ToUpper()
    if ($expectedHash -ne $actualHash) {
        Write-Err "SHA256 mismatch!`n  Expected: $expectedHash`n  Actual:   $actualHash"
    }
    Write-OK "SHA256 verified"
} catch {
    Write-Host "  [--] Could not verify SHA256 (skipping): $_" -ForegroundColor Yellow
}

# ── Extract ────────────────────────────────────────────────────────────────────
Write-Step "Installing to $InstallDir..."
if (Test-Path $InstallDir) {
    Remove-Item $InstallDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Expand-Archive -Path $ZipTmp -DestinationPath $InstallDir -Force
Remove-Item $ZipTmp -Force
Write-OK "Extracted to $InstallDir"

# ── Desktop shortcut ──────────────────────────────────────────────────────────
$ExePath = Join-Path $InstallDir $ExeName
if (Test-Path $ExePath) {
    $ShortcutPath = Join-Path ([Environment]::GetFolderPath("Desktop")) "DINOForge Companion.lnk"
    $WScript = New-Object -ComObject WScript.Shell
    $Shortcut = $WScript.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = $ExePath
    $Shortcut.WorkingDirectory = $InstallDir
    $Shortcut.Description = "DINOForge Desktop Companion"
    $Shortcut.Save()
    Write-OK "Desktop shortcut created"
}

# ── Done ───────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  DINOForge Companion $Version installed successfully!" -ForegroundColor Green
Write-Host "  Location : $InstallDir" -ForegroundColor Gray
Write-Host "  Launch   : $ExeName" -ForegroundColor Gray
Write-Host "  Shortcut : Desktop\DINOForge Companion.lnk" -ForegroundColor Gray
Write-Host ""
Write-Host "  On first run: Settings → set Packs Directory to your BepInEx\dinoforge_packs path" -ForegroundColor Yellow
Write-Host ""
