#!/usr/bin/env bash
# Phase 2: rebase the 6 open CONFLICTING PRs onto origin/main, force-push,
# open PRs for the 5 no-PR branches, drive CI green, approve, merge.
set -uo pipefail
REPO="/Users/kooshapari/CodeProjects/Phenotype/repos/Civis"
LOG="/tmp/civis_phase2.log"
exec >> "$LOG" 2>&1
cd "$REPO" || exit 1
ts() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }
log() { echo "[$(ts)] $*"; }

# -------- 1. Rebase 6 open CONFLICTING PRs -----------------------------------
# Each lives in a known worktree dir.
declare -A REBASE_WTS=(
  [fix/ci-mypy-hook]="worktrees/Civis/ci-repair-20260729"
  [feat/civis-client-ws-resilience-preserve]="worktrees/Civis/client-ws-resilience"
  [wip/civis-dashboard-a11y-20260729]="worktrees/Civis/dashboard-a11y"
  [wip/civis-frame-perf-20260729]="worktrees/Civis/frame-perf"
  [chore/repair-precommit-mypy-20260802]="worktrees/Civis/precommit-mypy-language-20260802"
  [wip/civis-origin-main-hardening-20260723]="worktrees/Civis/Civis-hardening"
)

log "=== Phase 2a: rebase 6 open CONFLICTING PRs onto origin/main ==="
for branch in "${!REBASE_WTS[@]}"; do
  wt="${REBASE_WTS[$branch]}"
  wt_abs="/Users/kooshapari/CodeProjects/Phenotype/repos/Civis/$wt"
  log ""
  log "--- $branch (wt: $wt) ---"
  if [ ! -d "$wt_abs" ]; then
    log "  MISSING worktree, skipping"
    continue
  fi
  (cd "$wt_abs" && {
    git rebase --abort 2>/dev/null || true
    git checkout "$branch" 2>&1 | tail -3
    git fetch origin main 2>&1 | tail -3
    log "  rebasing onto origin/main..."
    if git rebase origin/main 2>&1 | tail -20; then
      log "  rebase OK"
    else
      log "  rebase CONFLICT — will need manual fix"
    fi
  })
done

log ""
log "=== Phase 2a done. Status: ==="
for branch in "${!REBASE_WTS[@]}"; do
  wt="${REBASE_WTS[$branch]}"
  wt_abs="/Users/kooshapari/CodeProjects/Phenotype/repos/Civis/$wt"
  (cd "$wt_abs" && {
    log "  $branch: rev=$(git rev-parse HEAD), dirty=$(git status --porcelain | wc -l | tr -d ' '), rebase-in-progress=$([ -d .git/rebase-merge ] || [ -d .git/rebase-apply ] && echo yes || echo no)"
  })
done
