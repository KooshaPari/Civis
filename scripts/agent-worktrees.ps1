# Create parallel agent worktrees under ~/.cursor/worktrees/Dino
# Usage: powershell -NoProfile -File scripts\agent-worktrees.ps1 -BaseBranch main
param(
    [string]$BaseBranch = 'main',
    [string[]]$Names = @('wt-merge', 'wt-review', 'wt-gamelaunch')
)
$ErrorActionPreference = 'Stop'
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$WtRoot = Join-Path $env:USERPROFILE '.cursor\worktrees\Dino'
New-Item -ItemType Directory -Force -Path $WtRoot | Out-Null
Set-Location $RepoRoot
git fetch origin $BaseBranch 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { throw "git fetch origin $BaseBranch failed ($LASTEXITCODE)" }
foreach ($name in $Names) {
    $path = Join-Path $WtRoot $name
    if (Test-Path (Join-Path $path '.git')) {
        Write-Host "exists: $path"
        Set-Location $path
        git fetch origin
        git checkout $BaseBranch
        git pull --ff-only origin $BaseBranch 2>$null
    } else {
        git worktree add $path "origin/$BaseBranch" -b "agent/$name" 2>$null
        if ($LASTEXITCODE -ne 0) {
            git worktree add $path $BaseBranch
        }
        Write-Host "created: $path"
    }
    Set-Location $RepoRoot
}
git worktree list
