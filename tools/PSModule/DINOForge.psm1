#Requires -Version 5.1
<#
.SYNOPSIS
    DINOForge PowerShell Module - Native cmdlets for the DINO mod platform
.DESCRIPTION
    Provides a complete set of PowerShell cmdlets that wrap the dinoforge CLI,
    maintaining PowerShell verb-noun conventions and returning structured objects.
.NOTES
    Version: 0.26.0
    Author: KooshaPari
    Website: https://github.com/KooshaPari/Dino
#>

# Module-level variables
$script:DinoForgePath = $null
$script:GamePath = $null
$script:CacheDir = Join-Path $env:TEMP "DINOForge"

# Ensure cache directory exists
if (-not (Test-Path $script:CacheDir)) {
    New-Item -ItemType Directory -Path $script:CacheDir -Force | Out-Null
}

# ==================== HELPER FUNCTIONS ====================

<#
.INTERNAL
.SYNOPSIS
    Finds the dinoforge CLI executable location
#>
function Find-DINOForgeCli {
    [CmdletBinding()]
    param()

    # Check PATH first
    $cliPath = Get-Command dinoforge -ErrorAction SilentlyContinue
    if ($cliPath) {
        return $cliPath.Source
    }

    # Check common installation paths
    $commonPaths = @(
        (Join-Path $env:USERPROFILE ".dotnet" "tools" "dinoforge.exe"),
        (Join-Path $env:ProgramFiles "DINOForge" "dinoforge.exe"),
        (Join-Path $env:LocalAppData "DINOForge" "dinoforge.exe")
    )

    foreach ($path in $commonPaths) {
        if (Test-Path $path) {
            return $path
        }
    }

    throw "dinoforge CLI not found. Please install DINOForge first: dotnet tool install -g DINOForge.Tools.Cli"
}

<#
.INTERNAL
.SYNOPSIS
    Invokes dinoforge CLI and parses structured output
#>
function Invoke-DINOForgeCli {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Command,

        [Parameter()]
        [string[]]$Arguments,

        [Parameter()]
        [switch]$AsJson
    )

    if ($null -eq $script:DinoForgePath) {
        $script:DinoForgePath = Find-DINOForgeCli
    }

    $cliArgs = @($Command) + $Arguments
    if ($AsJson) {
        $cliArgs += "--json"
    }

    try {
        $output = & $script:DinoForgePath $cliArgs 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "dinoforge exited with code $LASTEXITCODE: $output"
        }

        if ($AsJson -and $output) {
            return $output | ConvertFrom-Json -ErrorAction SilentlyContinue
        }

        return $output
    }
    catch {
        Write-Error "Failed to execute dinoforge: $_"
        throw
    }
}

<#
.INTERNAL
.SYNOPSIS
    Detects the DINO game installation path
#>
function Find-DINOGamePath {
    [CmdletBinding()]
    param()

    # Check if already cached
    if ($script:GamePath) {
        return $script:GamePath
    }

    # Try Steam registry path
    try {
        $steamPath = Get-ItemProperty -Path "HKCU:\Software\Valve\Steam" -Name SteamPath -ErrorAction SilentlyContinue | Select-Object -ExpandProperty SteamPath
        if ($steamPath) {
            $dinoPath = Join-Path $steamPath "steamapps" "common" "Diplomacy is Not an Option"
            if (Test-Path $dinoPath) {
                $script:GamePath = $dinoPath
                return $dinoPath
            }
        }
    }
    catch {
        Write-Verbose "Could not find Steam registry path: $_"
    }

    # Try common library paths
    $commonPaths = @(
        "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option",
        "D:\SteamLibrary\steamapps\common\Diplomacy is Not an Option",
        "$env:ProgramFiles\Steam\steamapps\common\Diplomacy is Not an Option",
        "$env:ProgramFiles (x86)\Steam\steamapps\common\Diplomacy is Not an Option"
    )

    foreach ($path in $commonPaths) {
        if (Test-Path (Join-Path $path "Diplomacy is Not an Option.exe")) {
            $script:GamePath = $path
            return $path
        }
    }

    throw "DINO game installation not found. Please install Diplomacy is Not an Option via Steam."
}

<#
.INTERNAL
.SYNOPSIS
    Converts dinoforge CLI output to PSCustomObject
#>
function ConvertTo-DINOForgeObject {
    [CmdletBinding()]
    param(
        [Parameter(ValueFromPipeline)]
        [object]$InputObject,

        [Parameter()]
        [string]$ObjectType
    )

    process {
        if ($InputObject -is [string]) {
            return [PSCustomObject]@{
                Raw = $InputObject
                Type = $ObjectType
            }
        }

        return $InputObject
    }
}

# ==================== PUBLIC CMDLETS ====================

<#
.SYNOPSIS
    Installs DINOForge into a DINO game installation
.DESCRIPTION
    Installs DINOForge BepInEx plugin and example packs to the game directory.
    Detects the game path automatically from Steam or accepts an explicit path.
.PARAMETER GamePath
    Explicit path to the DINO game installation. Auto-detected if not specified.
.PARAMETER SkipBackup
    Skip creating a backup before installation.
.EXAMPLE
    Install-DINOForge
.EXAMPLE
    Install-DINOForge -GamePath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
#>
function Install-DINOForge {
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter(ValueFromPipeline)]
        [ValidateScript({ Test-Path $_ })]
        [string]$GamePath,

        [Parameter()]
        [switch]$SkipBackup
    )

    process {
        if (-not $GamePath) {
            $GamePath = Find-DINOGamePath
        }

        Write-Verbose "Installing DINOForge to: $GamePath"

        if ($PSCmdlet.ShouldProcess($GamePath, "Install DINOForge")) {
            try {
                $result = Invoke-DINOForgeCli -Command "install" -Arguments @("status", $GamePath) -AsJson

                [PSCustomObject]@{
                    GamePath = $GamePath
                    Success = $true
                    Status = "Installed"
                    Timestamp = Get-Date
                    Details = $result
                }
            }
            catch {
                [PSCustomObject]@{
                    GamePath = $GamePath
                    Success = $false
                    Status = "Failed"
                    Error = $_.Exception.Message
                    Timestamp = Get-Date
                }
            }
        }
    }
}

<#
.SYNOPSIS
    Gets the current DINOForge status
.DESCRIPTION
    Queries the DINOForge runtime for status information including loaded packs,
    entity counts, and system health.
.PARAMETER GamePath
    Path to the game installation. Auto-detected if not specified.
.EXAMPLE
    Get-DINOForgeStatus
.EXAMPLE
    Get-DINOForgeStatus -GamePath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
#>
function Get-DINOForgeStatus {
    [CmdletBinding()]
    param(
        [Parameter(ValueFromPipeline)]
        [ValidateScript({ -not $_ -or (Test-Path $_) })]
        [string]$GamePath
    )

    process {
        if (-not $GamePath) {
            $GamePath = Find-DINOGamePath
        }

        Write-Verbose "Querying DINOForge status from: $GamePath"

        try {
            $result = Invoke-DINOForgeCli -Command "status" -Arguments @($GamePath) -AsJson

            [PSCustomObject]@{
                GamePath = $GamePath
                Success = $true
                Timestamp = Get-Date
                Data = $result
            }
        }
        catch {
            [PSCustomObject]@{
                GamePath = $GamePath
                Success = $false
                Error = $_.Exception.Message
                Timestamp = Get-Date
            }
        }
    }
}

<#
.SYNOPSIS
    Deploys a DINOForge pack to the game installation
.DESCRIPTION
    Builds, validates, and deploys a pack to the DINO game installation.
    Supports hot-reload if the game is running.
.PARAMETER PackName
    Name of the pack to deploy (directory under packs/)
.PARAMETER GamePath
    Path to the game installation. Auto-detected if not specified.
.PARAMETER NoValidate
    Skip pack validation before deployment.
.PARAMETER HotReload
    Attempt hot-reload if game is running.
.EXAMPLE
    Deploy-DINOForgePack -PackName warfare-starwars
.EXAMPLE
    Deploy-DINOForgePack -PackName economy-balanced -HotReload
#>
function Deploy-DINOForgePack {
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter(Mandatory, ValueFromPipeline)]
        [ValidateNotNullOrEmpty()]
        [string]$PackName,

        [Parameter()]
        [ValidateScript({ -not $_ -or (Test-Path $_) })]
        [string]$GamePath,

        [Parameter()]
        [switch]$NoValidate,

        [Parameter()]
        [switch]$HotReload
    )

    process {
        if (-not $GamePath) {
            $GamePath = Find-DINOGamePath
        }

        Write-Verbose "Deploying pack '$PackName' to: $GamePath"

        if ($PSCmdlet.ShouldProcess($PackName, "Deploy pack to $GamePath")) {
            try {
                $args = @("pack", "deploy", $PackName, $GamePath)
                if ($NoValidate) { $args += "--no-validate" }
                if ($HotReload) { $args += "--hot-reload" }

                $result = Invoke-DINOForgeCli -Command "pack" -Arguments $args -AsJson

                [PSCustomObject]@{
                    PackName = $PackName
                    GamePath = $GamePath
                    Success = $true
                    Status = "Deployed"
                    HotReloaded = $HotReload
                    Timestamp = Get-Date
                    Details = $result
                }
            }
            catch {
                [PSCustomObject]@{
                    PackName = $PackName
                    GamePath = $GamePath
                    Success = $false
                    Status = "Failed"
                    Error = $_.Exception.Message
                    Timestamp = Get-Date
                }
            }
        }
    }
}

<#
.SYNOPSIS
    Creates a new DINOForge pack
.DESCRIPTION
    Scaffolds a new mod pack with templates for content, schema, and manifest.
.PARAMETER PackName
    Name of the pack (must be kebab-case)
.PARAMETER PackType
    Type of pack: content, balance, ruleset, scenario, total_conversion, utility
.PARAMETER Author
    Author name (defaults to current user)
.PARAMETER OutputPath
    Directory where to create the pack (defaults to packs/)
.EXAMPLE
    New-DINOForgePack -PackName my-custom-mod -PackType content
.EXAMPLE
    New-DINOForgePack -PackName warcraft-theme -PackType total_conversion -Author "MyName"
#>
function New-DINOForgePack {
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter(Mandatory, ValueFromPipeline)]
        [ValidatePattern("^[a-z0-9-]+$")]
        [string]$PackName,

        [Parameter()]
        [ValidateSet("content", "balance", "ruleset", "scenario", "total_conversion", "utility")]
        [string]$PackType = "content",

        [Parameter()]
        [string]$Author = $env:USERNAME,

        [Parameter()]
        [string]$OutputPath
    )

    process {
        if (-not $OutputPath) {
            $OutputPath = "packs"
        }

        Write-Verbose "Creating new pack '$PackName' of type '$PackType' in: $OutputPath"

        if ($PSCmdlet.ShouldProcess($PackName, "Create new DINOForge pack")) {
            try {
                $args = @("new", $PackName, "--type", $PackType, "--author", $Author, "--output", $OutputPath)

                $result = Invoke-DINOForgeCli -Command "new" -Arguments $args -AsJson

                [PSCustomObject]@{
                    PackName = $PackName
                    PackType = $PackType
                    Author = $Author
                    OutputPath = $OutputPath
                    Success = $true
                    Timestamp = Get-Date
                    Details = $result
                }
            }
            catch {
                [PSCustomObject]@{
                    PackName = $PackName
                    Success = $false
                    Error = $_.Exception.Message
                    Timestamp = Get-Date
                }
            }
        }
    }
}

<#
.SYNOPSIS
    Runs DINOForge smoke tests
.DESCRIPTION
    Executes the smoke test suite to verify mod pack functionality.
.PARAMETER Scenario
    Specific scenario to test (all, smoke, integration). Defaults to smoke.
.PARAMETER GamePath
    Path to the game installation. Auto-detected if not specified.
.EXAMPLE
    Invoke-DINOForgeSmoke
.EXAMPLE
    Invoke-DINOForgeSmoke -Scenario integration
#>
function Invoke-DINOForgeSmoke {
    [CmdletBinding()]
    param(
        [Parameter()]
        [ValidateSet("smoke", "integration", "all")]
        [string]$Scenario = "smoke",

        [Parameter()]
        [ValidateScript({ -not $_ -or (Test-Path $_) })]
        [string]$GamePath
    )

    process {
        if (-not $GamePath) {
            try {
                $GamePath = Find-DINOGamePath
            }
            catch {
                Write-Warning "Game path not found; tests may run in sandbox mode"
                $GamePath = $null
            }
        }

        Write-Verbose "Running smoke tests (scenario: $Scenario)"

        try {
            $args = @("--scenario", $Scenario)
            if ($GamePath) { $args += $GamePath }

            $result = Invoke-DINOForgeCli -Command "smoke" -Arguments $args

            [PSCustomObject]@{
                Scenario = $Scenario
                Success = $true
                Output = $result
                Timestamp = Get-Date
            }
        }
        catch {
            [PSCustomObject]@{
                Scenario = $Scenario
                Success = $false
                Error = $_.Exception.Message
                Timestamp = Get-Date
            }
        }
    }
}

<#
.SYNOPSIS
    Updates DINOForge to the latest version
.DESCRIPTION
    Checks for and installs the latest version of DINOForge from NuGet.
.PARAMETER PreRelease
    Include pre-release versions.
.EXAMPLE
    Update-DINOForge
.EXAMPLE
    Update-DINOForge -PreRelease
#>
function Update-DINOForge {
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter()]
        [switch]$PreRelease
    )

    process {
        Write-Verbose "Checking for DINOForge updates"

        if ($PSCmdlet.ShouldProcess("DINOForge", "Update to latest version")) {
            try {
                $toolArgs = @("update", "-g", "DINOForge.Tools.Cli")
                if ($PreRelease) {
                    $toolArgs += "--include-prerelease"
                }

                # Use dotnet tool to update
                $result = & dotnet tool $toolArgs 2>&1

                [PSCustomObject]@{
                    Success = $true
                    Status = "Updated"
                    Output = $result
                    Timestamp = Get-Date
                }
            }
            catch {
                [PSCustomObject]@{
                    Success = $false
                    Error = $_.Exception.Message
                    Timestamp = Get-Date
                }
            }
        }
    }
}

<#
.SYNOPSIS
    Gets DINOForge telemetry and metrics
.DESCRIPTION
    Retrieves performance metrics, pack load times, and system diagnostics.
.PARAMETER MetricType
    Type of metrics to retrieve: all, performance, packs, entities, memory
.PARAMETER GamePath
    Path to the game installation. Auto-detected if not specified.
.EXAMPLE
    Get-DINOForgeMetrics
.EXAMPLE
    Get-DINOForgeMetrics -MetricType performance
#>
function Get-DINOForgeMetrics {
    [CmdletBinding()]
    param(
        [Parameter()]
        [ValidateSet("all", "performance", "packs", "entities", "memory")]
        [string]$MetricType = "all",

        [Parameter()]
        [ValidateScript({ -not $_ -or (Test-Path $_) })]
        [string]$GamePath
    )

    process {
        if (-not $GamePath) {
            try {
                $GamePath = Find-DINOGamePath
            }
            catch {
                Write-Warning "Game path not found; using default"
                $GamePath = $null
            }
        }

        Write-Verbose "Retrieving $MetricType metrics"

        try {
            $args = @("metrics", "--type", $MetricType)
            if ($GamePath) { $args += $GamePath }

            $result = Invoke-DINOForgeCli -Command "metrics" -Arguments $args -AsJson

            [PSCustomObject]@{
                MetricType = $MetricType
                Success = $true
                GamePath = $GamePath
                Timestamp = Get-Date
                Data = $result
            }
        }
        catch {
            [PSCustomObject]@{
                MetricType = $MetricType
                Success = $false
                Error = $_.Exception.Message
                Timestamp = Get-Date
            }
        }
    }
}

<#
.SYNOPSIS
    Gets help for a DINOForge CLI command
.DESCRIPTION
    Displays help information for specific dinoforge commands.
.PARAMETER Command
    Command name to get help for. Lists all commands if not specified.
.EXAMPLE
    Get-DINOForgeHelp
.EXAMPLE
    Get-DINOForgeHelp -Command pack
#>
function Get-DINOForgeHelp {
    [CmdletBinding()]
    param(
        [Parameter(ValueFromPipeline)]
        [string]$Command
    )

    process {
        try {
            if ($Command) {
                Invoke-DINOForgeCli -Command $Command -Arguments @("--help")
            }
            else {
                Invoke-DINOForgeCli -Command "--help"
            }
        }
        catch {
            Write-Error "Failed to retrieve help: $_"
        }
    }
}

# ==================== EXPORTED FUNCTIONS ====================

# Explicitly export public cmdlets
Export-ModuleMember -Function @(
    'Install-DINOForge',
    'Get-DINOForgeStatus',
    'Deploy-DINOForgePack',
    'New-DINOForgePack',
    'Invoke-DINOForgeSmoke',
    'Update-DINOForge',
    'Get-DINOForgeMetrics',
    'Get-DINOForgeHelp'
)

# Aliases for convenience
New-Alias -Name dino-status -Value Get-DINOForgeStatus -Force
New-Alias -Name dino-deploy -Value Deploy-DINOForgePack -Force
New-Alias -Name dino-new -Value New-DINOForgePack -Force
New-Alias -Name dino-metrics -Value Get-DINOForgeMetrics -Force

Export-ModuleMember -Alias @('dino-status', 'dino-deploy', 'dino-new', 'dino-metrics')

Write-Verbose "DINOForge PowerShell module loaded successfully"
