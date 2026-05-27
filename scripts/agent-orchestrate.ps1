# Agent PR orchestration: worktrees -> CodeRabbit -> poll -> merge -> sync main
# Usage:
#   powershell -NoProfile -File scripts\agent-orchestrate.ps1 -PrNumber 221
#   powershell -NoProfile -File scripts\agent-orchestrate.ps1 -Step worktrees
#   powershell -NoProfile -File scripts\agent-orchestrate.ps1 -Step poll -PrNumber 221
param(
    [string]$BaseBranch = 'main',
    [string]$Repo = 'KooshaPari/Dino',
    [int]$PrNumber = 0,
    [ValidateSet('worktrees', 'coderabbit', 'poll', 'merge', 'workflow', 'sync', 'all')]
    [string]$Step = 'all',
    [int]$PollMinutes = 5,
    [int]$PollIntervalSec = 60
)

$ErrorActionPreference = 'Stop'
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$WtRoot = Join-Path $env:USERPROFILE '.cursor\worktrees\Dino'
$WtNames = @('wt-merge', 'wt-review', 'wt-gamelaunch')
$WorktreesScript = Join-Path $PSScriptRoot 'agent-worktrees.ps1'

function Invoke-AgentWorktrees {
    if (-not (Test-Path $WorktreesScript)) { throw "Missing $WorktreesScript" }
    & $WorktreesScript -BaseBranch $BaseBranch -Names $WtNames
    if ($LASTEXITCODE -ne 0) { throw "agent-worktrees.ps1 failed ($LASTEXITCODE)" }
}

function Write-WorktreePaths {
    Write-Host '=== parallel subagent worktrees (~/.cursor/worktrees/Dino) ==='
    foreach ($name in $WtNames) {
        Write-Host (Join-Path $WtRoot $name)
    }
    Set-Location $RepoRoot
    git worktree list
}

function Invoke-CodeRabbitReview {
    param([int]$Number)
    if ($Number -le 0) { throw 'PrNumber required for CodeRabbit step' }
    $body = @(
        '@coderabbitai review'
        '@coderabbitai approve'
    ) -join "`n"
    $bodyFile = Join-Path $env:TEMP "coderabbit-pr$Number.txt"
    Set-Content -Path $bodyFile -Value $body -Encoding utf8NoBOM
    Write-Host "=== CodeRabbit trigger (body-file) PR #$Number ==="
    gh pr comment $Number --repo $Repo --body-file $bodyFile
    if ($LASTEXITCODE -ne 0) { throw "gh pr comment failed ($LASTEXITCODE)" }
}

function Wait-PrApproved {
    param(
        [int]$Number,
        [int]$Minutes = 5,
        [int]$IntervalSec = 60
    )
    if ($Number -le 0) { throw 'PrNumber required for poll step' }
    $deadline = (Get-Date).AddMinutes($Minutes)
    $i = 0
    Write-Host "=== poll PR #$Number until reviewDecision APPROVED (max ${Minutes}m) ==="
    while ((Get-Date) -lt $deadline) {
        $i++
        $j = gh pr view $Number --repo $Repo --json reviewDecision,mergeStateStatus | ConvertFrom-Json
        Write-Host "[$i] reviewDecision=$($j.reviewDecision) mergeStateStatus=$($j.mergeStateStatus)"
        if ($j.reviewDecision -eq 'APPROVED') { return $true }
        $r = gh api "repos/$Repo/pulls/$Number/reviews" | ConvertFrom-Json
        $approved = $r | Where-Object { $_.state -eq 'APPROVED' }
        if ($approved) {
            $approved | ForEach-Object { Write-Host "APPROVED by $($_.user.login)" }
            return $true
        }
        Start-Sleep -Seconds $IntervalSec
    }
    throw "Timed out waiting for PR #$Number approval"
}

function Invoke-PrMerge {
    param([int]$Number)
    if ($Number -le 0) { throw 'PrNumber required for merge step' }
    Write-Host "=== gh pr merge PR #$Number ==="
    gh pr merge $Number --repo $Repo --merge
    return ($LASTEXITCODE -eq 0)
}

function Invoke-AgentMergeWorkflow {
    param([int]$Number)
    if ($Number -le 0) { throw 'PrNumber required for workflow step' }
    Write-Host "=== fallback: workflow_dispatch agent-merge-on-bot-approve.yml PR #$Number ==="
    gh workflow run agent-merge-on-bot-approve.yml --repo $Repo --ref $BaseBranch -f "pr_number=$Number"
    if ($LASTEXITCODE -ne 0) { throw "gh workflow run failed ($LASTEXITCODE)" }
}

function Invoke-PrMergeWithFallback {
    param([int]$Number)
    if (Invoke-PrMerge -Number $Number) { return }
    Write-Warning 'gh pr merge failed; using workflow_dispatch fallback'
    Invoke-AgentMergeWorkflow -Number $Number
}

function Sync-MainWorktree {
    Write-Host '=== sync main worktree ==='
    Set-Location $RepoRoot
    git checkout main
    if ($LASTEXITCODE -ne 0) { throw "git checkout main failed ($LASTEXITCODE)" }
    git pull
    if ($LASTEXITCODE -ne 0) { throw "git pull failed ($LASTEXITCODE)" }
}

Set-Location $RepoRoot

switch ($Step) {
    'worktrees' {
        Invoke-AgentWorktrees
        Write-WorktreePaths
    }
    'coderabbit' { Invoke-CodeRabbitReview -Number $PrNumber }
    'poll'       { Wait-PrApproved -Number $PrNumber -Minutes $PollMinutes -IntervalSec $PollIntervalSec | Out-Null }
    'merge'      { Invoke-PrMergeWithFallback -Number $PrNumber }
    'workflow'   { Invoke-AgentMergeWorkflow -Number $PrNumber }
    'sync'       { Sync-MainWorktree }
    'all' {
        Invoke-AgentWorktrees
        Write-WorktreePaths
        if ($PrNumber -gt 0) {
            Invoke-CodeRabbitReview -Number $PrNumber
            Wait-PrApproved -Number $PrNumber -Minutes $PollMinutes -IntervalSec $PollIntervalSec | Out-Null
            Invoke-PrMergeWithFallback -Number $PrNumber
        } else {
            Write-Host 'PrNumber not set; skipping coderabbit/poll/merge (pass -PrNumber N)'
        }
        Sync-MainWorktree
    }
}

Write-Host '=== agent-orchestrate done ==='
