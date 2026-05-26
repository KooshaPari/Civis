#Requires -Version 7.0
<#
.SYNOPSIS
PreToolUse hook to block `git stash` and `git stash drop` commands.
Allow: stash list, stash show, stash pop, stash apply, stash branch (auto-routed).
#>

param()

# Read from pipeline stdin only if redirected (avoid blocking in interactive shell)
$input = $null
if ([Console]::IsInputRedirected) {
    $input = [Console]::In.ReadToEnd()
}
if ([string]::IsNullOrWhiteSpace($input)) {
    $input = $env:CLAUDE_TOOL_INPUT
}

try {
    if ($input -like '*"command"*') {
        $payload = $input | ConvertFrom-Json
        $cmd = $payload.tool_input.command
    } else {
        $cmd = $input
    }

    if ($null -eq $cmd) {
        exit 0
    }

    # Allow ONLY read-only / recovery subcommands. Everything else is denied.
    # Allowed: stash list, stash show, stash pop, stash apply, stash branch
    # Unanchored: catches env-var prefixes (LEFTHOOK=0 git stash), -C path forms,
    # compound (cmd ; git stash), and wrappers (bash -c "git stash").
    if ($cmd -match '\bgit(\s+-[A-Za-z]\S*(\s+\S+)?)*\s+stash\s+(list|show|pop|apply|branch)\b') {
        exit 0
    }

    # Block ALL other `git stash` invocations:
    # - bare `git stash` (default = push)
    # - `git stash push|save|create|store|drop|clear`
    # - `LEFTHOOK=0 git stash`, `git -C /path stash`, `cmd ; git stash`, `bash -c "git stash"`
    if ($cmd -match '\bgit(\s+-[A-Za-z]\S*(\s+\S+)?)*\s+stash\b') {
        Write-Error "git stash is BLOCKED per CLAUDE.md governance (feedback_never_git_stash). Use 'git switch -c stash/auto-$(Get-Date -Format yyyy-MM-dd-HHmm)-<reason>' to route changes to a dated branch instead."
        exit 2
    }

    exit 0
} catch {
    Write-Error "Hook error: $_"
    exit 1
}

# Debug self-test: pwsh ./block-git-stash.ps1 -Debug-Self-Test
if ($args -contains '-Debug-Self-Test') {
    $cases = @(
        @{ cmd = 'LEFTHOOK=0 git stash';            expected = 'BLOCK' },
        @{ cmd = 'git -C /some/path stash';          expected = 'BLOCK' },
        @{ cmd = 'cmd1 ; git stash';                 expected = 'BLOCK' },
        @{ cmd = 'bash -c "git stash"';              expected = 'BLOCK' },
        @{ cmd = 'git stash list';                   expected = 'ALLOW' },
        @{ cmd = 'git stash pop';                    expected = 'ALLOW' }
    )
    foreach ($c in $cases) {
        $allow = $c.cmd -match '\bgit(\s+-[A-Za-z]\S*(\s+\S+)?)*\s+stash\s+(list|show|pop|apply|branch)\b'
        $block = (-not $allow) -and ($c.cmd -match '\bgit(\s+-[A-Za-z]\S*(\s+\S+)?)*\s+stash\b')
        $verdict = if ($block) { 'BLOCK' } elseif ($allow) { 'ALLOW' } else { 'PASSTHROUGH' }
        $pf = if ($verdict -eq $c.expected) { 'PASS' } else { 'FAIL' }
        Write-Host "$pf [$verdict] $($c.cmd)"
    }
}
