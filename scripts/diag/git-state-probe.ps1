<#
.SYNOPSIS
Captures git working-tree + branch + stash + deploy-state in JSON for autonomous orchestrator assessment.

.DESCRIPTION
Analogue of `scripts/diag/game-state-probe.ps1` (game runtime) but for the developer-side state.
Next session can call this to assess: what branch are we on? What's uncommitted? Any pending stashes?
Is the deployed DLL fresh? Without asking the user.

Run from repo root.

.PARAMETER Json
Output as compact JSON (default: True). When False, outputs a human-readable summary.

.EXAMPLE
pwsh scripts/diag/git-state-probe.ps1 -Json
#>
[CmdletBinding()]
param([bool]$Json = $true)

$ErrorActionPreference = 'SilentlyContinue'

$repoRoot = git rev-parse --show-toplevel 2>$null
if (-not $repoRoot) { Write-Error 'not a git repo'; exit 1 }

$probe = [ordered]@{
    timestamp_utc      = (Get-Date).ToUniversalTime().ToString('o')
    repo_root          = $repoRoot
    branch             = (git branch --show-current 2>$null)
    head_commit        = (git rev-parse --short HEAD 2>$null)
    head_message       = (git log -1 --format=%s 2>$null)
    is_clean           = ((git status --porcelain 2>$null | Measure-Object).Count -eq 0)
    modified_count     = ((git diff --name-only 2>$null | Measure-Object).Count)
    staged_count       = ((git diff --cached --name-only 2>$null | Measure-Object).Count)
    untracked_count    = ((git ls-files --others --exclude-standard 2>$null | Measure-Object).Count)
    stash_count        = ((git stash list 2>$null | Measure-Object).Count)
    ahead_of_main      = $null
    behind_main        = $null
    deployed_dll       = $null
    deployed_dll_mtime = $null
    deployed_dll_age_minutes = $null
}

# Compare to main
$base = git merge-base HEAD main 2>$null
if ($base) {
    $probe.ahead_of_main  = ((git rev-list "$base..HEAD" 2>$null | Measure-Object).Count)
    $probe.behind_main    = ((git rev-list "HEAD..main" 2>$null | Measure-Object).Count)
}

# Deployed DLL state (DINOForge.Runtime in BepInEx)
$deployedDll = 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\DINOForge.Runtime.dll'
if (Test-Path $deployedDll) {
    $info = Get-Item $deployedDll
    $probe.deployed_dll = $deployedDll
    $probe.deployed_dll_mtime = $info.LastWriteTime.ToString('o')
    $probe.deployed_dll_age_minutes = [math]::Round(((Get-Date) - $info.LastWriteTime).TotalMinutes, 1)
}

if ($Json) {
    $probe | ConvertTo-Json -Depth 4 -Compress
} else {
    "branch:         $($probe.branch)"
    "head:           $($probe.head_commit) — $($probe.head_message)"
    "clean:          $($probe.is_clean)"
    "modified:       $($probe.modified_count)"
    "staged:         $($probe.staged_count)"
    "untracked:      $($probe.untracked_count)"
    "stashes:        $($probe.stash_count)"
    "ahead/behind:   $($probe.ahead_of_main)/$($probe.behind_main) vs main"
    "deployed mtime: $($probe.deployed_dll_mtime)"
    "deploy age:     $($probe.deployed_dll_age_minutes) min"
}
