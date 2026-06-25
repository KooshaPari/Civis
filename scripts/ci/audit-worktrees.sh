#!/usr/bin/env bash
# scripts/ci/audit-worktrees.sh
#
# Audit worktree hygiene per FR-CIV-VERIFY-009 (civ-016-devx-verify-harness-and-worktree-hygiene).
#
# Lists worktrees whose underlying branch has no commits ahead of origin/main for
# more than 30 days. Read-only: never prunes, deletes, or modifies worktrees.
#
# Conventions checked:
#   * Canonical repo SHALL stay on `main` (no other branches checked out at
#     <root>).
#   * Worktree paths SHOULD live under `<disk>:/civis-wt-<SHORTNAME>` (or the
#     equivalent under .worktrees/ in this checkout). Worktrees outside those
#     roots are reported but not failed.
#   * Branches SHALL use a `feat/`, `fix/`, `chore/`, or `wt/` prefix
#     (FR-CIV-VERIFY-007).
#   * Detached HEAD worktrees are reported (warn-only by default).
#
# Usage:
#   bash scripts/ci/audit-worktrees.sh [--strict] [--stale-days N] [--json]
#
# Exit codes:
#   0 — no problems (or only soft warnings)
#   1 — strict mode + a violation is present (stale branch, wrong branch prefix,
#       canonical repo off main)
#
# Flags:
#   --strict         Fail on stale branches and wrong branch prefixes
#                    (default: warn-only for soft violations)
#   --stale-days N   Override the staleness threshold (default: 30)
#   --json           Emit JSON (one object per line) instead of a markdown table

set -euo pipefail

# Resolve repo root (works from anywhere inside the worktree).
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
      sed -n '2,30p' "$0"
      exit 0
      ;;
    *) echo "Unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done

# Detect canonical main worktree (the one whose path equals REPO_ROOT).
CANONICAL_BRANCH="$(git -C "$REPO_ROOT" symbolic-ref --short HEAD 2>/dev/null || echo "DETACHED")"

# Fetch origin/main if reachable; tolerate offline runs.
if git -C "$REPO_ROOT" fetch origin main --depth=1 2>/dev/null; then
  MAIN_TIP="$(git -C "$REPO_ROOT" rev-parse origin/main 2>/dev/null || echo "")"
else
  MAIN_TIP=""
fi

NOW_EPOCH="$(date +%s)"
STALE_THRESHOLD=$((NOW_EPOCH - STALE_DAYS * 86400))

# Output buffers (for non-JSON / JSON modes).
ROW_FMT="| %s | %s | %s | %s | %s |"
HEADER=$(printf "$ROW_FMT" "Worktree" "Branch" "Last-Commit" "Age(Days)" "Status")
SEPARATOR="|---|---|---|---|---|"

declare -a ROWS=()
declare -a JSON_ROWS=()

# Header for canonical repo row.
if [ "$CANONICAL_BRANCH" != "main" ]; then
  MSG="CANONICAL OFF MAIN (branch=$CANONICAL_BRANCH)"
  ROWS+=( "$(printf "$ROW_FMT" "$REPO_ROOT" "$CANONICAL_BRANCH" "-" "-" "$MSG")" )
  if [ "$JSON" -eq 1 ]; then
    JSON_ROWS+=("{\"worktree\":\"$REPO_ROOT\",\"branch\":\"$CANONICAL_BRANCH\",\"severity\":\"error\",\"reason\":\"canonical_off_main\"}")
  fi
else
  ROWS+=( "$(printf "$ROW_FMT" "$REPO_ROOT" "main" "-" "-" "canonical (ok)")" )
  if [ "$JSON" -eq 1 ]; then
    JSON_ROWS+=("{\"worktree\":\"$REPO_ROOT\",\"branch\":\"main\",\"severity\":\"ok\",\"reason\":\"canonical\"}")
  fi
fi

# Walk every worktree.
# Use porcelain output (multi-line) so we can read `branch refs/heads/<name>`
# directly without invoking `git symbolic-ref` per worktree.
wt_path=""
wt_sha=""
wt_branch=""
emit_wt_row() {
  # Last-commit age for the worktree HEAD.
  last_iso="$(git -C "$wt_path" log -1 --format=%cI "$wt_sha" 2>/dev/null || echo "")"
  last_epoch=0
  if [ -n "$last_iso" ]; then
    last_epoch="$(date -d "$last_iso" +%s 2>/dev/null || echo 0)"
  fi
  age_days="-"
  if [ "$last_epoch" -gt 0 ]; then
    age_days="$(( (NOW_EPOCH - last_epoch) / 86400 ))"
  fi

  # Ahead-of-main counts (only if we have origin/main).
  ahead="-"
  if [ -n "$MAIN_TIP" ]; then
    ahead="$(git -C "$wt_path" rev-list --count "${MAIN_TIP}..${wt_sha}" 2>/dev/null || echo "-")"
  fi

  # Severity decision.
  severity="ok"
  reason="clean"
  if [ -z "$wt_branch" ]; then
    severity="warn"
    reason="detached_head"
  elif ! echo "$wt_branch" | grep -qE '^(feat|fix|chore|wt)/'; then
    severity="warn"
    reason="bad_branch_prefix"
  fi

  # Stale: ahead==0 AND last-commit-epoch older than threshold.
  if [ "$age_days" != "-" ] && [ "$age_days" -gt "$STALE_DAYS" ] && [ "$ahead" = "0" ]; then
    severity="warn"
    reason="stale_branch_no_commits_ahead"
  fi

  # Path convention check (soft).
  case "$wt_path" in
    */.worktrees/wt-*|*/civis-wt-*|*/.cursor/worktrees/*) : ;;
    *) reason="${reason}+unconventional_path" ;;
  esac

  ROWS+=( "$(printf "$ROW_FMT" "$wt_path" "${wt_branch:-DETACHED}" "${last_iso:-?}" "$age_days" "$reason")" )

  branch_json="${wt_branch:-DETACHED}"
  if [ "$JSON" -eq 1 ]; then
    JSON_ROWS+=("{\"worktree\":\"$wt_path\",\"branch\":\"$branch_json\",\"last_commit\":\"${last_iso:-}\",\"age_days\":\"$age_days\",\"ahead_of_main\":\"$ahead\",\"severity\":\"$severity\",\"reason\":\"$reason\"}")
  fi
  # Guard the function's exit status so callers don't trip set -e.
  return 0
}

while IFS= read -r raw_line; do
  case "$raw_line" in
    "worktree "*)
      # Emit previous row if any (skip the canonical row — already printed).
      if [ -n "$wt_path" ] && [ "$wt_path" != "$REPO_ROOT" ]; then
        emit_wt_row
      fi
      wt_path="${raw_line#worktree }"
      wt_sha=""
      wt_branch=""
      ;;
    "HEAD "*)
      wt_sha="${raw_line#HEAD }"
      ;;
    "branch "*)
      wt_branch="${raw_line#branch refs/heads/}"
      ;;
    "detached")
      wt_branch=""
      ;;
    "")
      # Blank line separates records — nothing to do; emit happens on next record.
      ;;
    *)
      # Ignore other keys (locked, prunable, etc.) — they don't affect severity.
      ;;
  esac
done < <(git -C "$REPO_ROOT" worktree list --porcelain; printf '\n')

# Emit the final record.
if [ -n "$wt_path" ] && [ "$wt_path" != "$REPO_ROOT" ]; then
  emit_wt_row
fi

# Emit.
if [ "$JSON" -eq 1 ]; then
  printf '%s\n' "${JSON_ROWS[@]}"
else
  echo "## Worktree hygiene audit"
  echo
  echo "Repo: $REPO_ROOT"
  echo "Canonical branch: $CANONICAL_BRANCH"
  echo "Stale threshold: $STALE_DAYS days"
  echo
  echo "$HEADER"
  echo "$SEPARATOR"
  for r in "${ROWS[@]}"; do
    echo "$r"
  done
fi

# Decide exit code.
if [ "$STRICT" -eq 1 ]; then
  for r in "${JSON_ROWS[@]}"; do
    if echo "$r" | grep -q '"severity":"warn"' 2>/dev/null; then
      exit 1
    fi
    if echo "$r" | grep -q '"severity":"error"' 2>/dev/null; then
      exit 1
    fi
  done
fi
exit 0