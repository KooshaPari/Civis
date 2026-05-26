#Requires -Version 5.1
# SPEC-003: minimal unit checks for scripts/game/capture-feature-clips.ps1 (no game required)

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$scriptRel = 'scripts\game\capture-feature-clips.ps1'
$scriptPath = Join-Path $repoRoot $scriptRel

function Get-ScriptLevelParamBlockAst {
    param([string]$Path)

    $tokens = $null
    $parseErrors = $null
    $resolved = (Resolve-Path -LiteralPath $Path).Path
    $ast = [System.Management.Automation.Language.Parser]::ParseFile(
        $resolved, [ref]$tokens, [ref]$parseErrors)

  if ($parseErrors.Count -gt 0) {
        throw "Failed to parse '$Path': $($parseErrors[0].Message)"
    }

    return $ast.FindAll(
        {
            $node = $args[0]
            $node -is [System.Management.Automation.Language.ParamBlockAst] -and
                $node.Parent -eq $ast
        },
        $false)
}

function Invoke-CaptureScriptWithArgs {
    param(
        [string[]]$ExtraArgs,
        [string]$FilePath = $scriptPath,
        [int]$TimeoutMs = 8000
    )

    $argList = @(
        '-NoProfile',
        '-NonInteractive',
        '-ExecutionPolicy', 'Bypass',
        '-File', $FilePath
    ) + $ExtraArgs

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = 'powershell.exe'
    $psi.Arguments = ($argList | ForEach-Object {
            if ($_ -match '\s') { "`"$_`"" } else { $_ }
        }) -join ' '
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true

    $proc = [System.Diagnostics.Process]::Start($psi)
    if (-not $proc.WaitForExit($TimeoutMs)) {
        try { $proc.Kill() } catch { }
        return @{ TimedOut = $true; ExitCode = $null }
    }

    return @{ TimedOut = $false; ExitCode = $proc.ExitCode }
}

Describe 'SPEC-003 capture-feature-clips.ps1' {
    It 'exists at scripts/game/capture-feature-clips.ps1' {
        Join-Path $repoRoot $scriptRel | Should Exist
        $scriptPath | Should Exist
    }

    It 'has a script-level param block or exits non-zero when given invalid arguments' {
        $scriptParam = Get-ScriptLevelParamBlockAst -Path $scriptPath
        if ($scriptParam) {
            $scriptParam.Count | Should BeGreaterThan 0
            return
        }

        $run = Invoke-CaptureScriptWithArgs -ExtraArgs @('-__PesterInvalidSwitch__')
        $run.TimedOut | Should Be $false -Because (
            'No script-level param block; script did not exit within timeout on invalid args. ' +
            'Add a script-level param block or fail fast on unknown parameters.'
        )
        $run.ExitCode | Should Not Be 0
    }

    It 'declares [CmdletBinding()] on the script-level param block' {
        $paramBlock = Get-ScriptLevelParamBlockAst -Path $scriptPath
        $paramBlock | Should Not BeNullOrEmpty
        $hasCmdletBinding = $paramBlock.Attributes | Where-Object {
            $_.TypeName.Name -eq 'CmdletBinding'
        }
        @($hasCmdletBinding).Count | Should BeGreaterThan 0
    }

    It 'references edge-tts or Remotion paths under scripts/video' {
        $content = Get-Content -LiteralPath $scriptPath -Raw
        $referencesPipeline = ($content -match 'edge-tts') -or
            ($content -match 'Remotion') -or
            ($content -match 'scripts[/\\]video')
        $referencesPipeline | Should Be $true
    }

    It 'exits non-zero when bootstrap debug log path is missing' {
        $missingLog = Join-Path $TestDrive 'bootstrap-missing-dinoforge_debug.log'
        $content = Get-Content -LiteralPath $scriptPath -Raw
        $content = $content.Replace(
            '$debugLog  = "$gameDir\BepInEx\dinoforge_debug.log"',
            "`$debugLog  = '$($missingLog -replace "'", "''")'"
        )
        $content = $content.Replace(
            'Start-Process -FilePath $gameExe -WorkingDirectory $gameDir',
            '# game launch skipped for bootstrap log unit test'
        )
        $content = $content.Replace(
            'Wait-ForLog -LogPath $debugLog -Pattern "Awake completed" -TimeoutSec 30',
            'Wait-ForLog -LogPath $debugLog -Pattern "Awake completed" -TimeoutSec 4'
        )
        $content = $content -replace '(?ms)(function Wait-ForLog \{.*?Start-Sleep -Seconds )2', '${1}1'

        $patchedPath = Join-Path $TestDrive 'capture-feature-clips.bootstrap-missing.ps1'
        Set-Content -LiteralPath $patchedPath -Value $content -Encoding UTF8

        $run = Invoke-CaptureScriptWithArgs -FilePath $patchedPath -TimeoutMs 20000
        $run.TimedOut | Should Be $false -Because 'bootstrap wait should finish within 20s'
        $run.ExitCode | Should Be 1 -Because 'missing bootstrap log should fail Awake wait'
    }
}
