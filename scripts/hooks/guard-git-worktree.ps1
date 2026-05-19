#Requires -Version 7.0
<#
.SYNOPSIS
PreToolUse hook to guard `git worktree remove --force` on fix/* branches.
Allow: `git worktree remove` without --force, `git worktree remove .claude/worktrees/*`.
Block: `git worktree remove --force <path>` on active branches.
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

    # Block: git worktree remove --force <anything except .claude/worktrees>
    if ($cmd -match '^\s*git\s+worktree\s+remove\s+--force\b' -and $cmd -notmatch '\.claude[/\\]worktrees') {
        Write-Error "❌ git worktree remove --force is BLOCKED on active branches per CLAUDE.md governance. Use `git worktree remove` without --force instead."
        exit 2
    }

    # Allow: everything else (remove without --force, remove .claude/worktrees/*, etc.)
    exit 0
} catch {
    Write-Error "Hook error: $_"
    exit 1
}
