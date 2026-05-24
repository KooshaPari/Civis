<#
.SYNOPSIS
    SPEC-005 supersession verification: boot.config single-instance=0 on launch-game installs.

.DESCRIPTION
    Asserts boot.config files referenced by .claude/commands/launch-game.md (step 0)
    contain single-instance=0 so Unity's native single-instance check stays disabled.

    Paths (must match launch-game.md):
      - Main:  ...\Diplomacy is Not an Option\Diplomacy is Not an Option_Data\boot.config
      - _TEST: ...\Diplomacy is Not an Option_TEST\Diplomacy is Not an Option_Data\boot.config

    Run:
      Invoke-Pester -Path .\tests\unit\Test-BootConfigSingleInstance.ps1

    Requires Pester (Install-Module Pester -Scope CurrentUser).
#>

# Same paths as launch-game.md step 0 (auto-repair loop)
$script:LaunchGameBootConfigPaths = @(
    'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option_Data\boot.config',
    'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST\Diplomacy is Not an Option_Data\boot.config'
)

$script:SingleInstancePattern = 'single-instance\s*=\s*0'

Describe 'SPEC-005 supersession (boot.config single-instance=0)' {
    Context 'launch-game boot.config installs' {
        foreach ($bootConfigPath in $script:LaunchGameBootConfigPaths) {
            $installLabel = (Split-Path (Split-Path $bootConfigPath -Parent) -Parent | Split-Path -Leaf)

            It "[$installLabel] boot.config exists and sets single-instance=0" {
                if (-not (Test-Path -LiteralPath $bootConfigPath)) {
                    Set-TestInconclusive "boot.config not found (game not installed): $bootConfigPath"
                    return
                }

                $bootContent = Get-Content -LiteralPath $bootConfigPath -Raw
                $bootContent | Should Match $script:SingleInstancePattern
            }
        }
    }

    It 'defines at least one launch-game boot.config path' {
        $script:LaunchGameBootConfigPaths.Count | Should BeGreaterThan 0
    }
}
