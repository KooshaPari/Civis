@{
    # Module metadata
    RootModule              = 'DINOForge.psm1'
    ModuleVersion          = '0.26.0'
    GUID                   = 'b7f4a2c1-5e8d-4c3b-9a1d-2f6e8c0a4b5d'
    Author                 = 'KooshaPari'
    CompanyName            = 'DINOForge Project'
    Copyright              = '(c) 2026 KooshaPari. Licensed under MIT.'
    Description            = 'PowerShell module for DINOForge - the mod platform for Diplomacy is Not an Option. Provides native cmdlets for pack management, deployment, and game automation.'
    PowerShellVersion      = '5.1'
    CompatiblePSEditions   = @('Desktop', 'Core')

    # Exported cmdlets
    CmdletsToExport        = @(
        'Install-DINOForge',
        'Get-DINOForgeStatus',
        'Deploy-DINOForgePack',
        'New-DINOForgePack',
        'Invoke-DINOForgeSmoke',
        'Update-DINOForge',
        'Get-DINOForgeMetrics',
        'Get-DINOForgeHelp'
    )

    # Exported aliases
    AliasesToExport        = @(
        'dino-status',
        'dino-deploy',
        'dino-new',
        'dino-metrics'
    )

    # Functions to export (internal helpers are not exported)
    FunctionsToExport      = @(
        'Install-DINOForge',
        'Get-DINOForgeStatus',
        'Deploy-DINOForgePack',
        'New-DINOForgePack',
        'Invoke-DINOForgeSmoke',
        'Update-DINOForge',
        'Get-DINOForgeMetrics',
        'Get-DINOForgeHelp'
    )

    # No variables exported
    VariablesToExport      = @()

    # External dependencies
    RequiredModules        = @()
    RequiredAssemblies     = @()

    # Script module processing
    ScriptsToProcess       = @()
    TypesToProcess         = @()
    FormatsToProcess       = @()

    # Private data for the module
    PrivateData            = @{
        PSData = @{
            # Tags for the module (used by PowerShell Gallery)
            Tags                   = @(
                'DINO',
                'DINOForge',
                'Mod',
                'GameDev',
                'ModPlatform',
                'CLI',
                'Diplomacy-is-Not-an-Option'
            )

            # Link to project repository
            ProjectUri             = 'https://github.com/KooshaPari/Dino'

            # Link to license
            LicenseUri             = 'https://github.com/KooshaPari/Dino/blob/main/LICENSE'

            # Link to documentation
            HelpInfoUri            = 'https://kooshapari.github.io/Dino/'

            # Release notes
            ReleaseNotes           = @'
# DINOForge PowerShell Module v0.26.0

## New Features
- Native PowerShell cmdlets for all DINOForge CLI commands
- Verb-Noun naming convention (Install-DINOForge, Deploy-DINOForgePack, etc.)
- Pipeline support for pack deployment operations
- Structured output as PSCustomObjects (not raw text)
- Automatic game path detection via Steam registry
- One-line installer for quick setup

## Cmdlets
- Install-DINOForge: Install DINOForge to game directory
- Get-DINOForgeStatus: Query DINOForge runtime status
- Deploy-DINOForgePack: Build, validate, and deploy packs
- New-DINOForgePack: Scaffold new mod packs
- Invoke-DINOForgeSmoke: Run smoke tests
- Update-DINOForge: Update to latest version
- Get-DINOForgeMetrics: Retrieve telemetry and metrics
- Get-DINOForgeHelp: Display command help

## Aliases
- dino-status: Alias for Get-DINOForgeStatus
- dino-deploy: Alias for Deploy-DINOForgePack
- dino-new: Alias for New-DINOForgePack
- dino-metrics: Alias for Get-DINOForgeMetrics

## Requirements
- PowerShell 5.1 or later (Desktop or Core edition)
- .NET CLI (dotnet command)
- DINOForge CLI (dotnet tool install -g DINOForge.Tools.Cli)

## Installation
- Manual: Copy DINOForge.psm1 and DINOForge.psd1 to $PROFILE\Modules\DINOForge\
- One-liner: iwr https://raw.githubusercontent.com/KooshaPari/Dino/main/tools/Install-DINOForge.ps1 | iex

For detailed docs, visit: https://kooshapari.github.io/Dino/
'@

            # Is prerelease
            Prerelease             = $false

            # Minimum version of PowerShellGet required
            PowerShellGetFormatVersion = '2'

            # Module icon (when published to Gallery)
            IconUri                = 'https://raw.githubusercontent.com/KooshaPari/Dino/main/docs/logo.png'
        }
    }
}
