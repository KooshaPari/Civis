#Requires -Version 7.0
<#
.SYNOPSIS
PreToolUse hook to block `git stash` and `git stash drop` commands.
Allow: stash list, stash show, stash pop, stash apply, stash branch (auto-routed).
#>

param()

# Read from pipeline stdin
$input = $null
if ($PSVersionTable.PSVersion.Major -ge 6) {
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
    if ($cmd -match '^\s*git\s+stash\s+(list|show|pop|apply|branch)\b') {
        exit 0
    }

    # Block ALL other `git stash` invocations:
    # - bare `git stash` (default = push)
    # - `git stash push ...`
    # - `git stash save ...`
    # - `git stash create`
    # - `git stash store ...`
    # - `git stash drop ...`
    # - `git stash clear`
    if ($cmd -match '^\s*git\s+stash(\s|$)') {
        Write-Error "❌ git stash is BLOCKED per CLAUDE.md governance (feedback_never_git_stash). Use 'git switch -c stash/auto-$(Get-Date -Format yyyy-MM-dd-HHmm)-<reason>' to route changes to a dated branch instead."
        exit 2
    }

    exit 0
} catch {
    Write-Error "Hook error: $_"
    exit 1
}
