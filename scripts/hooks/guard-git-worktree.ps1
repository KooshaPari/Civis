#Requires -Version 7.0
<#
.SYNOPSIS
PreToolUse hook to guard `git worktree remove --force` on fix/* branches.
Allow: `git worktree remove` without --force, `git worktree remove .claude/worktrees/*`.
Block: `git worktree remove --force <path>` on active branches.
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

    # Block: git worktree remove --force <anything except .claude/worktrees>
    if ($cmd -match '\bgit(\s+-[A-Za-z]\S*(\s+\S+)?)*\s+worktree\s+remove\s+--force\b' -and $cmd -notmatch '\.claude[/\\]worktrees') {
        Write-Error "git worktree remove --force is BLOCKED on active branches per CLAUDE.md governance. Use 'git worktree remove' without --force instead."
        exit 2
    }

    # Block: git worktree prune --expire=now (more aggressive than default; can drop in-progress worktrees)
    if ($cmd -match '\bgit(\s+-[A-Za-z]\S*(\s+\S+)?)*\s+worktree\s+prune\b.*--expire(=|\s+)now\b') {
        Write-Error "git worktree prune --expire=now is BLOCKED per CLAUDE.md governance (feedback_worktree_boundary). Run 'git worktree prune' without --expire=now, or use the default expiry window."
        exit 2
    }

    # Block: git worktree remove on a dirty target worktree (unless --force, handled above, OR .claude/worktrees scope)
    if ($cmd -match '\bgit(\s+-[A-Za-z]\S*(\s+\S+)?)*\s+worktree\s+remove\s+(?!--force)(\S+)' -and $cmd -notmatch '\.claude[/\\]worktrees') {
        $target = $Matches[3]
        if ($target -and (Test-Path $target)) {
            try {
                $statusOut = & git -C $target status --porcelain 2>$null
                if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace(($statusOut -join "`n"))) {
                    Write-Error "git worktree remove is BLOCKED: target worktree '$target' has uncommitted changes. Commit or push to a dated branch first (feedback_worktree_boundary)."
                    exit 2
                }
            } catch {
                # If git status fails, fall through (don't false-positive block)
            }
        }
    }

    # Allow: everything else (remove without --force on clean tree, remove .claude/worktrees/*, etc.)
    exit 0
} catch {
    Write-Error "Hook error: $_"
    exit 1
}
