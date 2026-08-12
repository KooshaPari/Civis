#!/usr/bin/env bash
# Drive the Civis worktree cleanup + PR churn to a clean main.
# Idempotent, AFK-friendly driver. Run from /Users/kooshapari/CodeProjects/Phenotype/repos.
set -uo pipefail
REPO="/Users/kooshapari/CodeProjects/Phenotype/repos/Civis"
LOG="/tmp/civis_drive.log"
exec > >(tee -a "$LOG") 2>&1
cd "$REPO" || { echo "FATAL: cd $REPO"; exit 1; }
ts() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }
step() { echo; echo "===== [$(ts)] $* ====="; }

step "preflight: fetch"
git fetch --all --prune --tags 2>&1 | tail -5
echo "origin/main = $(git rev-parse origin/main)"

step "drop merged/empty/detached worktrees"
DROP_WTS=(
  "Civis/worktrees/Civis/main-stabilize-20260729"
  "Civis/worktrees/Civis/postmerge-ci-1453-20260730"
  "worktrees/Civis/Civis-rebase"
  "Civis/worktrees/Civis/Civis-first-splash-artifact"
  "Civis/worktrees/Civis/mergify-repair-20260802"
  "Civis/worktrees/Civis/postmerge-dogfood-20260729"
  "Civis/worktrees/Civis/precommit-rust-hooks-20260802"
  "Civis/worktrees/Civis/nightly-menu-command-20260801"
  "worktrees/Civis/Civis-asset-pipeline-recovery"
  "worktrees/Civis/Civis-current-hardening"
  "worktrees/Civis/Civis-first-playable-asset"
  "worktrees/Civis/Civis-playability-next"
  "worktrees/Civis/Civis-playability-splash"
)
for wt in "${DROP_WTS[@]}"; do
  abs="/Users/kooshapari/CodeProjects/Phenotype/repos/$wt"
  if [ -d "$abs" ]; then
    (cd "$abs" && git rebase --abort 2>/dev/null || true)
    echo "removing worktree: $wt"
    git worktree remove --force "$abs" 2>&1 || echo "  (already gone?)"
  fi
done
git worktree prune

step "delete already-merged / superseded branches"
DELETE_BRANCHES=(
  "wip/civis-first-splash-artifact-20260728"
  "chore/repair-mergify-config-20260802"
  "fix/bevy-window-menu-command"
  "fix/precommit-rust-hooks-20260802"
  "fix/nightly-menu-command-20260801"
  "wip/civis-asset-pipeline-recovery-20260727"
  "wip/civis-notification-expiry-20260726"
  "wip/civis-first-playable-asset-20260729"
  "wip/civis-mods-a11y-20260727"
  "wip/civis-first-splash-artifact-20260729"
  "wip/civis-replay-route-hardening-20260726"
)
for b in "${DELETE_BRANCHES[@]}"; do
  if git show-ref --verify --quiet "refs/heads/$b"; then
    echo "deleting local: $b"
    git branch -D "$b" 2>&1
  fi
  if git ls-remote --heads origin "$b" 2>/dev/null | grep -q "$b"; then
    echo "deleting remote: origin/$b"
    git push origin --delete "$b" 2>&1 | head -3
  fi
done

step "main worktree: drop daemon commits, re-add clean"
MAIN_WT="/Users/kooshapari/CodeProjects/Phenotype/repos/Civis"
if [ -d "$MAIN_WT/.git" ]; then
  (cd "$MAIN_WT" && git rebase --abort 2>/dev/null || true)
  git worktree remove --force "$MAIN_WT" 2>&1 || true
fi
git worktree prune
git worktree add "$MAIN_WT" origin/main 2>&1
cd "$MAIN_WT"
git checkout -B main origin/main 2>&1
git log --oneline -3

step "post-prune state"
git worktree list
echo
git status
