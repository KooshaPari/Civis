<#
.SYNOPSIS
    DINOForge Installer for Windows.
    Installs BepInEx 5.4.23.5 and DINOForge into Diplomacy is Not an Option.

.PARAMETER GamePath
    Manual override for the DINO game installation path.

.PARAMETER Dev
    Includes development tools: SDK DLL, PackCompiler, DumpTools, schemas.

.PARAMETER SkipBepInEx
    Skip BepInEx installation (use if already installed).

.EXAMPLE
    .\Install-DINOForge.ps1
    .\Install-DINOForge.ps1 -GamePath "D:\Games\Diplomacy is Not an Option"
    .\Install-DINOForge.ps1 -Dev
#>

[CmdletBinding()]
param(
    [string]$GamePath,
    [switch]$Dev,
    [switch]$SkipBepInEx
)

$ErrorActionPreference = "Stop"

$BepInExVersion = "5.4.23.2"
$BepInExUrl = "https://github.com/BepInEx/BepInEx/releases/download/v$BepInExVersion/BepInEx_win_x64_$BepInExVersion.zip"
$DinoAppId = 1272320
$DinoDirName = "Diplomacy is Not an Option"

# --- Helper Functions ---

function Write-Step {
    param([string]$Message)
    Write-Host "[*] " -ForegroundColor Cyan -NoNewline
    Write-Host $Message
}

function Write-Success {
    param([string]$Message)
    Write-Host "[+] " -ForegroundColor Green -NoNewline
    Write-Host $Message
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[!] " -ForegroundColor Yellow -NoNewline
    Write-Host $Message
}

function Write-Err {
    param([string]$Message)
    Write-Host "[-] " -ForegroundColor Red -NoNewline
    Write-Host $Message
}

function Find-SteamPath {
    # Try registry
    $regPaths = @(
        "HKLM:\SOFTWARE\WOW6432Node\Valve\Steam",
        "HKLM:\SOFTWARE\Valve\Steam"
    )
    foreach ($regPath in $regPaths) {
        try {
            $key = Get-ItemProperty -Path $regPath -ErrorAction SilentlyContinue
            if ($key -and $key.InstallPath -and (Test-Path $key.InstallPath)) {
                return $key.InstallPath
            }
        } catch { }
    }

    # Fallback defaults
    $defaults = @(
        "${env:ProgramFiles(x86)}\Steam",
        "$env:ProgramFiles\Steam"
    )
    foreach ($d in $defaults) {
        if (Test-Path $d) { return $d }
    }
    return $null
}

function Get-LibraryFolders {
    param([string]$SteamPath)

    $folders = @($SteamPath)
    $vdfPath = Join-Path $SteamPath "steamapps\libraryfolders.vdf"
    if (-not (Test-Path $vdfPath)) {
        $vdfPath = Join-Path $SteamPath "config\libraryfolders.vdf"
    }
    if (-not (Test-Path $vdfPath)) { return $folders }

    $content = Get-Content $vdfPath -Raw
    $matches = [regex]::Matches($content, '"path"\s+"([^"]+)"')
    foreach ($m in $matches) {
        $path = $m.Groups[1].Value -replace '\\\\', '\'
        if ($path -and ($path -notin $folders)) {
            $folders += $path
        }
    }
    return $folders
}

function Find-DinoPath {
    $steamPath = Find-SteamPath
    if (-not $steamPath) { return $null }

    Write-Step "Steam found at: $steamPath"
    $libraries = Get-LibraryFolders $steamPath

    foreach ($lib in $libraries) {
        $commonPath = Join-Path $lib "steamapps\common\$DinoDirName"
        if (Test-Path $commonPath) { return $commonPath }

        # Check ACF manifest
        $acf = Join-Path $lib "steamapps\appmanifest_$DinoAppId.acf"
        if (Test-Path $acf) {
            $acfContent = Get-Content $acf -Raw
            $m = [regex]::Match($acfContent, '"installdir"\s+"([^"]+)"')
            if ($m.Success) {
                $installDir = $m.Groups[1].Value
                $gamePath = Join-Path $lib "steamapps\common\$installDir"
                if (Test-Path $gamePath) { return $gamePath }
            }
        }
    }
    return $null
}

# --- Main ---

Write-Host ""
Write-Host "========================================" -ForegroundColor Magenta
Write-Host "  DINOForge Installer" -ForegroundColor Magenta
Write-Host "========================================" -ForegroundColor Magenta
Write-Host ""

# 1. Find game path
if ($GamePath) {
    if (-not (Test-Path $GamePath)) {
        Write-Err "Specified game path does not exist: $GamePath"
        exit 1
    }
    Write-Step "Using specified game path: $GamePath"
} else {
    Write-Step "Auto-detecting DINO installation..."
    $GamePath = Find-DinoPath
    if (-not $GamePath) {
        Write-Warn "Could not auto-detect DINO installation."
        $GamePath = Read-Host "Enter the full path to the DINO game directory"
        if (-not (Test-Path $GamePath)) {
            Write-Err "Path does not exist: $GamePath"
            exit 1
        }
    } else {
        Write-Success "Found DINO at: $GamePath"
    }
}

# Verify game exe
$gameExe = Join-Path $GamePath "$DinoDirName.exe"
if (-not (Test-Path $gameExe)) {
    Write-Warn "Game executable not found at expected path. Continuing anyway..."
}

# 2. Install BepInEx
if (-not $SkipBepInEx) {
    $winhttpDll = Join-Path $GamePath "winhttp.dll"
    if (Test-Path $winhttpDll) {
        Write-Warn "BepInEx appears to be already installed (winhttp.dll exists). Skipping."
    } else {
        Write-Step "Downloading BepInEx $BepInExVersion..."
        $tempZip = Join-Path $env:TEMP "BepInEx_$BepInExVersion.zip"
        try {
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
            Invoke-WebRequest -Uri $BepInExUrl -OutFile $tempZip -UseBasicParsing
            Write-Success "Downloaded BepInEx."
        } catch {
            Write-Err "Failed to download BepInEx: $_"
            exit 1
        }

        Write-Step "Extracting BepInEx to game directory..."
        try {
            Expand-Archive -Path $tempZip -DestinationPath $GamePath -Force
            Write-Success "BepInEx extracted."
        } catch {
            Write-Err "Failed to extract BepInEx: $_"
            exit 1
        }

        Remove-Item $tempZip -ErrorAction SilentlyContinue
    }
}

# 3. Create plugins directory if needed
$pluginsDir = Join-Path $GamePath "BepInEx\plugins"
if (-not (Test-Path $pluginsDir)) {
    New-Item -Path $pluginsDir -ItemType Directory -Force | Out-Null
    Write-Step "Created BepInEx\plugins\ directory."
}

# 4. Copy DINOForge Runtime DLL
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$runtimeDll = Join-Path $scriptDir "..\..\..\Runtime\bin\Release\netstandard2.0\DINOForge.Runtime.dll"
if (-not (Test-Path $runtimeDll)) {
    $runtimeDll = Join-Path $scriptDir "..\..\..\Runtime\bin\Debug\netstandard2.0\DINOForge.Runtime.dll"
}
if (Test-Path $runtimeDll) {
    Copy-Item $runtimeDll -Destination $pluginsDir -Force
    Write-Success "Copied DINOForge.Runtime.dll to BepInEx\plugins\"
} else {
    Write-Warn "DINOForge.Runtime.dll not found. Build the Runtime project first, then copy manually."
}

# 5. Create packs directory with example pack
$packsDir = Join-Path $GamePath "packs"
if (-not (Test-Path $packsDir)) {
    New-Item -Path $packsDir -ItemType Directory -Force | Out-Null
    Write-Step "Created packs\ directory."
}

$examplePackDir = Join-Path $packsDir "example-balance"
if (-not (Test-Path $examplePackDir)) {
    New-Item -Path $examplePackDir -ItemType Directory -Force | Out-Null
    $exampleManifest = @"
id: example-balance
name: Example Balance Pack
version: 0.1.0
framework_version: ">=0.1.0 <1.0.0"
author: DINOForge Agents
type: balance
description: An example balance pack demonstrating DINOForge pack structure.
depends_on: []
conflicts_with: []
loads:
  units: []
  buildings: []
"@
    Set-Content -Path (Join-Path $examplePackDir "pack.yaml") -Value $exampleManifest -Encoding UTF8
    Write-Success "Created example pack at packs\example-balance\"
}

# 6. Dev mode extras
if ($Dev) {
    Write-Step "Installing dev tools..."

    $sdkDll = Join-Path $scriptDir "..\..\..\SDK\bin\Release\netstandard2.0\DINOForge.SDK.dll"
    if (-not (Test-Path $sdkDll)) {
        $sdkDll = Join-Path $scriptDir "..\..\..\SDK\bin\Debug\netstandard2.0\DINOForge.SDK.dll"
    }
    if (Test-Path $sdkDll) {
        Copy-Item $sdkDll -Destination $pluginsDir -Force
        Write-Success "Copied DINOForge.SDK.dll"
    } else {
        Write-Warn "SDK DLL not found. Build the SDK project first."
    }

    # Copy schemas
    $schemasSource = Join-Path $scriptDir "..\..\..\..\schemas"
    $schemasTarget = Join-Path $GamePath "DINOForge\schemas"
    if (Test-Path $schemasSource) {
        New-Item -Path $schemasTarget -ItemType Directory -Force | Out-Null
        Copy-Item "$schemasSource\*" -Destination $schemasTarget -Recurse -Force
        Write-Success "Copied schemas to DINOForge\schemas\"
    }

    Write-Success "Dev tools installed."
}

# 7. Verify installation
Write-Host ""
Write-Step "Verifying installation..."
$allGood = $true

$checks = @(
    @{ Path = (Join-Path $GamePath "winhttp.dll"); Name = "BepInEx Doorstop (winhttp.dll)" },
    @{ Path = (Join-Path $GamePath "doorstop_config.ini"); Name = "Doorstop config" },
    @{ Path = (Join-Path $GamePath "BepInEx"); Name = "BepInEx directory" },
    @{ Path = (Join-Path $GamePath "BepInEx\plugins"); Name = "BepInEx plugins directory" },
    @{ Path = (Join-Path $GamePath "packs"); Name = "Packs directory" }
)

foreach ($check in $checks) {
    if (Test-Path $check.Path) {
        Write-Success "$($check.Name) ... OK"
    } else {
        Write-Err "$($check.Name) ... MISSING"
        $allGood = $false
    }
}

$runtimeCheck = Join-Path $GamePath "BepInEx\plugins\DINOForge.Runtime.dll"
if (Test-Path $runtimeCheck) {
    Write-Success "DINOForge Runtime DLL ... OK"
} else {
    Write-Warn "DINOForge Runtime DLL ... NOT FOUND (build and copy manually)"
}

Write-Host ""
if ($allGood) {
    Write-Success "DINOForge installation complete!"
    Write-Host "  Game path: $GamePath" -ForegroundColor Gray
    Write-Host "  Run the game to generate BepInEx config." -ForegroundColor Gray
} else {
    Write-Warn "Installation has issues. Review the messages above."
}
Write-Host ""
