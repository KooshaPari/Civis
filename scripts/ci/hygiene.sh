#!/usr/bin/env bash
# scripts/ci/hygiene.sh
#
# Single entry point for repository hygiene checks
# (civ-016-devx-verify-harness-and-worktree-hygiene).
#
# Runs:
#   * audit-worktrees.sh — worktree path/branch/staleness audit
#   * audit-pr-queue.sh  — PR queue staleness audit (falls back to local git
#                          if `gh` is not authenticated)
#   * with-cargo-target.sh — env probe (does not invoke cargo unless -- is
#                            passed as the trailing separator)
#
# Usage:
#   bash scripts/ci/hygiene.sh [--strict] [--stale-days N] [--json]
#
# Exit codes:
#   0 — all hygiene checks passed (or only soft warnings in non-strict mode)
#   1 — strict mode + a hygiene violation is present
#   2 — a sub-script failed to run (script missing / bash error)
#
# Flags:
#   --strict         Promote any soft violation (stale worktree, stale branch,
#                    bad branch prefix, canonical repo off main) to a hard FAIL
#   --stale-days N   Override the staleness threshold (default: 30)
#   --json           Emit machine-readable output (newline-delimited JSON)

set -eu

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$REPO_ROOT" ]; then
  echo "ERROR: not inside a git worktree" >&2
  exit 2
fi

STRICT=0
STALE_DAYS=30
JSON=0

while [ $# -gt 0 ]; do
  case "$1" in
    --strict) STRICT=1 ;;
    --stale-days) STALE_DAYS="${2:-30}"; shift ;;
    --json) JSON=1 ;;
    -h|--help)
      sed -n '2,25p' "$0"
      exit 0
      ;;
    *) echo "Unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done

# Build the args to forward to the sub-scripts.
FORWARD_ARGS=()
if [ "$STRICT" -eq 1 ]; then FORWARD_ARGS+=("--strict"); fi
FORWARD_ARGS+=("--stale-days" "$STALE_DAYS")
if [ "$JSON" -eq 1 ]; then FORWARD_ARGS+=("--json"); fi

echo "==> Hygiene: worktree audit" >&2
echo "==> Hygiene: PR queue audit" >&2
echo "==> Hygiene: cargo-target env probe" >&2

if [ "$JSON" -eq 1 ]; then
  echo "{\"gate\":\"hygiene\",\"strict\":$STRICT,\"stale_days\":$STALE_DAYS}"
fi

# 1) Worktree audit.
echo "--- audit-worktrees ---" >&2
if ! bash "$SCRIPT_DIR/audit-worktrees.sh" "${FORWARD_ARGS[@]}"; then
  WT_RC=$?
  echo "hygiene: audit-worktrees failed (rc=$WT_RC)" >&2
  exit "$WT_RC"
fi

# 2) PR queue audit.
echo "--- audit-pr-queue ---" >&2
if ! bash "$SCRIPT_DIR/audit-pr-queue.sh" "${FORWARD_ARGS[@]}"; then
  PR_RC=$?
  echo "hygiene: audit-pr-queue failed (rc=$PR_RC)" >&2
  exit "$PR_RC"
fi

# 3) CARGO_TARGET_DIR probe. We never *run* cargo here — just verify the
#    env resolves to a writable directory and matches the convention.
echo "--- with-cargo-target (env probe) ---" >&2
probe_out="$(bash "$SCRIPT_DIR/with-cargo-target.sh" 2>&1 || true)"
printf '%s\n' "$probe_out"
probe_dir="$(echo "$probe_out" | sed -n 's/^CARGO_TARGET_DIR=//p' | head -1)"
if [ -z "$probe_dir" ]; then
  echo "hygiene: with-cargo-target did not export CARGO_TARGET_DIR" >&2
  exit 2
fi
if [ ! -d "$probe_dir" ]; then
  echo "hygiene: CARGO_TARGET_DIR '$probe_dir' is not a directory (env probe failed)" >&2
  if [ "$STRICT" -eq 1 ]; then
    exit 1
  fi
fi

if [ "$JSON" -eq 1 ]; then
  echo "{\"gate\":\"hygiene\",\"result\":\"pass\",\"cargo_target_dir\":\"$probe_dir\"}"
else
  echo "hygiene: OK" >&2
fi
exit 0