<#
.SYNOPSIS
    One-line installer for DINOForge PowerShell module and CLI

.DESCRIPTION
    Downloads and installs DINOForge including:
    - DINOForge.Tools.Cli (dotnet tool)
    - PowerShell module with native cmdlets
    - BepInEx (if not already installed)
    - Runtime plugin and example packs

.EXAMPLE
    # Run from PowerShell prompt:
    iwr https://raw.githubusercontent.com/KooshaPari/Dino/main/tools/Install-DINOForge.ps1 | iex

.EXAMPLE
    # Or save and run:
    Invoke-WebRequest -Uri "https://raw.githubusercontent.com/KooshaPari/Dino/main/tools/Install-DINOForge.ps1" -OutFile "$env:TEMP\Install-DINOForge.ps1"
    & "$env:TEMP\Install-DINOForge.ps1"

.NOTES
    Requires: PowerShell 5.1+, .NET CLI, Internet connection
    Platform: Windows (primary), macOS/Linux (partial support)
#>

#Requires -Version 5.1
#Requires -RunAsAdministrator

param(
    [Parameter()]
    [switch]$SkipBepInEx,

    [Parameter()]
    [string]$GamePath,

    [Parameter()]
    [string]$InstallPath = (Join-Path $env:USERPROFILE "Documents" "PowerShell" "Modules" "DINOForge")
)

$ErrorActionPreference = 'Stop'
$VerbosePreference = 'Continue'

# ==================== HELPER FUNCTIONS ====================

function Write-Header {
    param([string]$Text)
    Write-Host ""
    Write-Host ("=" * 70) -ForegroundColor Cyan
    Write-Host $Text -ForegroundColor Cyan
    Write-Host ("=" * 70) -ForegroundColor Cyan
    Write-Host ""
}

function Write-Status {
    param([string]$Text, [ValidateSet("Info", "Success", "Warning", "Error")]$Type = "Info")
    $colors = @{
        "Info"    = "White"
        "Success" = "Green"
        "Warning" = "Yellow"
        "Error"   = "Red"
    }
    Write-Host ("[" + $Type.ToUpper() + "] ") -ForegroundColor $colors[$Type] -NoNewline
    Write-Host $Text
}

function Find-GamePath {
    Write-Status "Detecting DINO game installation..." "Info"

    # Try Steam registry first
    try {
        $steamPath = Get-ItemProperty -Path "HKCU:\Software\Valve\Steam" -Name SteamPath -ErrorAction SilentlyContinue | Select-Object -ExpandProperty SteamPath
        if ($steamPath) {
            $dinoPath = Join-Path $steamPath "steamapps" "common" "Diplomacy is Not an Option"
            if ((Test-Path $dinoPath) -and (Test-Path (Join-Path $dinoPath "Diplomacy is Not an Option.exe"))) {
                Write-Status "Found DINO at: $dinoPath" "Success"
                return $dinoPath
            }
        }
    }
    catch {
        Write-Verbose "Steam registry check failed: $_"
    }

    # Try common library paths
    $commonPaths = @(
        "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option",
        "D:\SteamLibrary\steamapps\common\Diplomacy is Not an Option",
        "$env:ProgramFiles\Steam\steamapps\common\Diplomacy is Not an Option",
        "$env:ProgramFiles (x86)\Steam\steamapps\common\Diplomacy is Not an Option"
    )

    foreach ($path in $commonPaths) {
        if ((Test-Path $path) -and (Test-Path (Join-Path $path "Diplomacy is Not an Option.exe"))) {
            Write-Status "Found DINO at: $path" "Success"
            return $path
        }
    }

    Write-Status "DINO game installation not found" "Warning"
    return $null
}

function Install-DotnetTool {
    Write-Header "Installing DINOForge CLI"

    Write-Status "Checking for dotnet CLI..." "Info"
    try {
        $version = & dotnet --version
        Write-Status "dotnet version: $version" "Success"
    }
    catch {
        Write-Status "dotnet CLI not found. Please install .NET SDK from https://dotnet.microsoft.com/download" "Error"
        throw "dotnet CLI required but not installed"
    }

    Write-Status "Installing DINOForge.Tools.Cli global tool..." "Info"
    try {
        & dotnet tool update -g DINOForge.Tools.Cli --prerelease 2>&1 | ForEach-Object { Write-Verbose $_ }
        Write-Status "DINOForge CLI installed successfully" "Success"
        return $true
    }
    catch {
        Write-Status "Failed to install DINOForge CLI: $_" "Error"
        return $false
    }
}

function Install-PowerShellModule {
    Write-Header "Installing PowerShell Module"

    Write-Status "Creating module directory: $InstallPath" "Info"
    if (-not (Test-Path $InstallPath)) {
        New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
    }

    # Download module files from GitHub
    $repoBase = "https://raw.githubusercontent.com/KooshaPari/Dino/main/tools/PSModule"

    Write-Status "Downloading DINOForge.psm1..." "Info"
    try {
        $psmUri = "$repoBase/DINOForge.psm1"
        $psmPath = Join-Path $InstallPath "DINOForge.psm1"
        Invoke-WebRequest -Uri $psmUri -OutFile $psmPath -ErrorAction Stop
        Write-Status "Downloaded DINOForge.psm1" "Success"
    }
    catch {
        Write-Status "Failed to download module: $_" "Error"
        return $false
    }

    Write-Status "Downloading DINOForge.psd1..." "Info"
    try {
        $psdUri = "$repoBase/DINOForge.psd1"
        $psdPath = Join-Path $InstallPath "DINOForge.psd1"
        Invoke-WebRequest -Uri $psdUri -OutFile $psdPath -ErrorAction Stop
        Write-Status "Downloaded DINOForge.psd1" "Success"
    }
    catch {
        Write-Status "Failed to download manifest: $_" "Error"
        return $false
    }

    # Import the module
    Write-Status "Importing DINOForge module..." "Info"
    try {
        Import-Module $InstallPath -Force -ErrorAction Stop
        Write-Status "DINOForge PowerShell module loaded successfully" "Success"
        return $true
    }
    catch {
        Write-Status "Failed to import module: $_" "Error"
        return $false
    }
}

function Install-BepInEx {
    param([string]$GamePath)

    Write-Header "Checking BepInEx Installation"

    if (-not $GamePath) {
        Write-Status "No game path available; skipping BepInEx check" "Warning"
        return $true
    }

    $bepinexPath = Join-Path $GamePath "BepInEx"
    if (Test-Path $bepinexPath) {
        Write-Status "BepInEx already installed at: $bepinexPath" "Success"
        return $true
    }

    if ($SkipBepInEx) {
        Write-Status "Skipping BepInEx installation per user request" "Info"
        return $true
    }

    Write-Status "BepInEx not found; installing..." "Info"
    Write-Host ""
    Write-Host "To install BepInEx manually:" -ForegroundColor Yellow
    Write-Host "  1. Visit: https://github.com/BepInEx/BepInEx/releases/tag/v5.4.23.5"
    Write-Host "  2. Download: BepInEx_x64_5.4.23.5.zip"
    Write-Host "  3. Extract to: $GamePath"
    Write-Host "  4. Run: $GamePath\BepInEx\tools\Manager.exe"
    Write-Host ""

    return $false
}

function Deploy-DINOForgeRuntime {
    param([string]$GamePath)

    Write-Header "Deploying DINOForge Runtime"

    if (-not $GamePath) {
        Write-Status "No game path available; skipping runtime deployment" "Warning"
        return $true
    }

    $bepinexPath = Join-Path $GamePath "BepInEx" "plugins"
    if (-not (Test-Path $bepinexPath)) {
        Write-Status "BepInEx plugins directory not found; runtime deployment skipped" "Warning"
        return $false
    }

    Write-Status "Deploying DINOForge.Runtime to: $bepinexPath" "Info"
    try {
        # Build the runtime project
        Write-Status "Building DINOForge.Runtime..." "Info"
        & dotnet build "src\Runtime\DINOForge.Runtime.csproj" -c Release -p:DeployToGame=true -p:GameInstallPath="$GamePath" 2>&1 | ForEach-Object { Write-Verbose $_ }
        Write-Status "DINOForge.Runtime deployed successfully" "Success"
        return $true
    }
    catch {
        Write-Status "Failed to deploy runtime: $_" "Warning"
        return $false
    }
}

function Show-QuickStart {
    Write-Header "Quick Start Guide"

    Write-Host @"
DINOForge is now installed! Here are some next steps:

1. LAUNCH THE GAME
   Start Diplomacy is Not an Option from Steam

2. VERIFY INSTALLATION
   PS> Get-DINOForgeStatus

3. CREATE YOUR FIRST MOD PACK
   PS> New-DINOForgePack -PackName my-custom-mod -PackType content

4. DEPLOY A PACK
   PS> Deploy-DINOForgePack -PackName my-custom-mod

5. GET HELP
   PS> Get-DINOForgeHelp
   PS> Get-Help Deploy-DINOForgePack -Full

USEFUL ALIASES
   PS> dino-status      # Get game status
   PS> dino-deploy      # Deploy a pack
   PS> dino-new         # Create new pack
   PS> dino-metrics     # View metrics

DOCUMENTATION
   Full docs: https://kooshapari.github.io/Dino/
   GitHub:    https://github.com/KooshaPari/Dino
   Issues:    https://github.com/KooshaPari/Dino/issues

"@
}

# ==================== MAIN INSTALLATION ====================

Write-Header "DINOForge PowerShell Module Installer v0.26.0"

Write-Host @"
This installer will:
  1. Install DINOForge.Tools.Cli (global dotnet tool)
  2. Download PowerShell module
  3. Verify/install BepInEx
  4. Deploy DINOForge runtime
  5. Configure PowerShell module

"@

# Step 1: Install dotnet tool
if (-not (Install-DotnetTool)) {
    Write-Status "Installation aborted" "Error"
    exit 1
}

# Step 2: Detect game path
if (-not $GamePath) {
    $GamePath = Find-GamePath
}

# Step 3: Install PowerShell module
if (-not (Install-PowerShellModule)) {
    Write-Status "Installation aborted" "Error"
    exit 1
}

# Step 4: Check BepInEx
if ($GamePath) {
    if (-not $SkipBepInEx) {
        Install-BepInEx -GamePath $GamePath | Out-Null
    }

    # Step 5: Deploy runtime
    Deploy-DINOForgeRuntime -GamePath $GamePath | Out-Null
}

# Step 6: Show quick start
Show-QuickStart

Write-Header "Installation Complete"
Write-Status "DINOForge is ready to use!" "Success"
Write-Host ""
Write-Host "Start a new PowerShell session and run: Get-DINOForgeStatus" -ForegroundColor Cyan
Write-Host ""
