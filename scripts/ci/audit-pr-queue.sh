#!/usr/bin/env bash
# scripts/ci/audit-pr-queue.sh
#
# Audit PR queue hygiene per FR-CIV-VERIFY-008
# (civ-016-devx-verify-harness-and-worktree-hygiene).
#
# Flags PRs open against main for more than 14 days without rebase. Read-only:
# never closes, comments on, or modifies PRs.
#
# Usage:
#   bash scripts/ci/audit-pr-queue.sh [--strict] [--stale-days N] [--json] [--base main]
#
# Exit codes:
#   0 — no stale PRs (or only soft warnings)
#   1 — strict mode + a stale PR is present
#
# Requires `gh` CLI authenticated with `repo` scope on the origin. If `gh` is
# missing or unauthenticated, falls back to reading `origin/<branch>` via
# `git log` only — which means PRs without a local branch cannot be inspected
# and a warning is emitted.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$REPO_ROOT" ]; then
  echo "ERROR: not inside a git worktree" >&2
  exit 2
fi

STRICT=0
STALE_DAYS=14
JSON=0
BASE_BRANCH="main"
USE_GH=1

while [ $# -gt 0 ]; do
  case "$1" in
    --strict) STRICT=1 ;;
    --stale-days) STALE_DAYS="${2:-14}"; shift ;;
    --json) JSON=1 ;;
    --base) BASE_BRANCH="${2:-main}"; shift ;;
    --no-gh) USE_GH=0 ;;
    -h|--help)
      sed -n '2,25p' "$0"
      exit 0
      ;;
    *) echo "Unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done

NOW_EPOCH="$(date +%s)"
STALE_THRESHOLD=$((NOW_EPOCH - STALE_DAYS * 86400))

# Decide whether `gh` is usable.
GH_OK=0
if [ "$USE_GH" -eq 1 ] && command -v gh >/dev/null 2>&1; then
  if gh auth status >/dev/null 2>&1; then
    GH_OK=1
  fi
fi

# Output buffers.
declare -a ROWS=()
declare -a JSON_ROWS=()
declare -a STRICT_FAIL_REASONS=()

if [ "$JSON" -eq 1 ]; then
  JSON_ROWS+=("{\"warning\":\"script_meta\"}")
fi

if [ "$GH_OK" -eq 0 ]; then
  WARN_LINE="WARN: gh CLI not authenticated — falling back to local git log only."
  echo "$WARN_LINE" >&2
  [ "$JSON" -eq 1 ] && JSON_ROWS+=("{\"severity\":\"warn\",\"reason\":\"gh_unavailable\",\"detail\":\"$WARN_LINE\"}")
fi

# Helper: convert ISO8601 -> epoch (Linux + macOS best-effort).
to_epoch() {
  iso="$1"
  if [ -z "$iso" ]; then
    echo "0"
    return
  fi
  date -d "$iso" +%s 2>/dev/null || python3 -c "import sys, datetime; print(int(datetime.datetime.fromisoformat(sys.argv[1].replace('Z','+00:00')).timestamp()))" "$iso" 2>/dev/null || echo "0"
}

# ─── Walk PRs ───────────────────────────────────────────────────────────────
if [ "$GH_OK" -eq 1 ]; then
  # Pull the last 100 open PRs against BASE_BRANCH (paged results are best-effort).
  pr_json="$(gh pr list --base "$BASE_BRANCH" --state open --limit 100 \
              --json number,title,baseRefName,headRefName,author,createdAt,updatedAt,isDraft,labels 2>/dev/null || echo "[]")"

  # Iterate via Python so JSON parsing is portable across BSD/GNU grep.
  python3 - "$pr_json" "$STALE_THRESHOLD" "$JSON" <<'PY' || true
import json, sys, os, subprocess, time
prs = json.loads(sys.argv[1])
threshold = int(sys.argv[2])
want_json = sys.argv[3] == "1"
now = int(time.time())
rows, jsons, fails = [], [], []

def emit_md(n, t, base, head, age, status):
    return f"| #{n} | {t[:48]} | {base} | {head} | {age} | {status} |"

def emit_json(n, t, base, head, age, status, severity, reason):
    obj = {
        "pr": n, "title": t, "base": base, "head": head,
        "age_days": age, "status": status,
        "severity": severity, "reason": reason,
    }
    return json.dumps(obj, ensure_ascii=False)

for pr in prs:
    n = pr["number"]
    t = pr["title"]
    base = pr["baseRefName"]
    head = pr["headRefName"]
    author = (pr.get("author") or {}).get("login", "?")
    created = pr.get("createdAt") or ""
    updated = pr.get("updatedAt") or ""
    draft = pr.get("isDraft", False)
    labels = [l["name"] for l in (pr.get("labels") or [])]

    def parse(iso):
        if not iso: return 0
        try:
            from datetime import datetime, timezone
            s = iso.replace("Z","+00:00")
            return int(datetime.fromisoformat(s).timestamp())
        except Exception:
            return 0

    created_epoch = parse(created)
    updated_epoch = parse(updated)
    age_days = (now - created_epoch) // 86400 if created_epoch else -1
    staleness = (now - updated_epoch) // 86400 if updated_epoch else -1

    severity = "ok"
    reason = "fresh"
    status_parts = []
    if age_days < 0:
        status_parts.append("?")
    if draft:
        status_parts.append("draft")
    if age_days >= 14:
        status_parts.append(f"open {age_days}d")
    if staleness >= 14:
        severity = "warn"
        reason = "stale_no_rebase"
        status_parts.append(f"no-rebase {staleness}d")
    label_block = ",".join(labels)
    if label_block:
        status_parts.append(f"labels={label_block}")

    status = ",".join(status_parts) or "ok"

    rows.append(emit_md(n, t, base, head, age_days, status))
    jsons.append(emit_json(n, t, base, head, age_days, status, severity, reason))
    if severity == "warn":
        fails.append(f"PR #{n} stale {staleness}d without rebase")

print("## PR queue hygiene audit")
print()
print(f"Base branch: {base_branch := sys.argv[0]}")
print(f"Stale threshold: {sys.argv[1]} days")
print()
print("| PR | Title | Base | Head | Age(Days) | Status |")
print("|---|---|---|---|---|---|")
for r in rows:
    print(r)
print()
if not rows:
    print("(no open PRs against base)")
print()

if want_json:
    for j in jsons:
        print(j)

if fails:
    print("STRICT_FAIL:", file=sys.stderr)
    for f in fails:
        print(" -", f, file=sys.stderr)
    if os.environ.get("STRICT", "0") == "1":
        sys.exit(2)
PY
  STRICT_RC=$?
  if [ "$STRICT" -eq 1 ] && [ "$STRICT_RC" -ne 0 ]; then
    exit 1
  fi
  exit 0
fi

# Fallback: no gh. Inspect every local worktree branch and look at its last
# commit epoch against the BASE_BRANCH tip.
if git -C "$REPO_ROOT" fetch origin "$BASE_BRANCH" --depth=1 >/dev/null 2>&1; then
  BASE_TIP="$(git -C "$REPO_ROOT" rev-parse "origin/$BASE_BRANCH" 2>/dev/null || echo "")"
else
  BASE_TIP=""
fi

if [ "$JSON" -eq 1 ]; then
  echo "{\"severity\":\"warn\",\"reason\":\"no_gh\",\"detail\":\"local-only mode\"}"
else
  echo "## PR queue hygiene audit (local-only — no gh CLI)"
  echo
  echo "Base branch: $BASE_BRANCH"
  echo "Stale threshold: $STALE_DAYS days"
  echo
  echo "(no remote PR data; use --no-gh deliberately if you only want local branches)"
  echo
  echo "| Branch | Last-Commit | Age(Days) | Ahead | Status |"
  echo "|---|---|---|---|---|"
fi

while IFS= read -r ref; do
  branch="${ref#refs/heads/}"
  case "$branch" in
    "$BASE_BRANCH"|"HEAD") continue ;;
  esac
  last_iso="$(git -C "$REPO_ROOT" log -1 --format=%cI "$branch" 2>/dev/null || echo "")"
  last_epoch="$(to_epoch "$last_iso")"
  age_days="-"
  if [ "$last_epoch" -gt 0 ]; then
    age_days="$(( (NOW_EPOCH - last_epoch) / 86400 ))"
  fi
  ahead="-"
  if [ -n "$BASE_TIP" ]; then
    ahead="$(git -C "$REPO_ROOT" rev-list --count "${BASE_TIP}..${branch}" 2>/dev/null || echo "-")"
  fi
  severity="ok"
  reason="fresh"
  if [ "$age_days" != "-" ] && [ "$age_days" -gt "$STALE_DAYS" ] && [ "$ahead" = "0" ]; then
    severity="warn"
    reason="stale_no_ahead"
  fi
  if [ "$JSON" -eq 1 ]; then
    JSON_ROWS+=("{\"branch\":\"$branch\",\"last_commit\":\"${last_iso:-}\",\"age_days\":\"$age_days\",\"ahead_of_base\":\"$ahead\",\"severity\":\"$severity\",\"reason\":\"$reason\"}")
  else
    ROWS+=( "$(printf "| %s | %s | %s | %s | %s |" "$branch" "${last_iso:-?}" "$age_days" "$ahead" "$reason")" )
  fi
  if [ "$severity" = "warn" ]; then
    STRICT_FAIL_REASONS+=("branch $branch stale $age_days d, no commits ahead of $BASE_BRANCH")
  fi
done < <(git -C "$REPO_ROOT" for-each-ref --format='%(refname)' refs/heads/ 2>/dev/null)

if [ "$JSON" -eq 1 ]; then
  printf '%s\n' "${JSON_ROWS[@]}"
else
  for r in "${ROWS[@]}"; do echo "$r"; done
fi

if [ "$STRICT" -eq 1 ] && [ "${#STRICT_FAIL_REASONS[@]}" -gt 0 ]; then
  for r in "${STRICT_FAIL_REASONS[@]}"; do
    echo "STRICT_FAIL: $r" >&2
  done
  exit 1
fi
exit 0