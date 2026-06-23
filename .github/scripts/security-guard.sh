#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

if command -v ggshield >/dev/null 2>&1; then
  GGSHIELD=(ggshield)
elif command -v uvx >/dev/null 2>&1; then
  GGSHIELD=(uvx ggshield)
elif command -v uv >/dev/null 2>&1; then
  GGSHIELD=(uv tool run ggshield)
else
  echo "ERROR: ggshield not installed. Install with: pipx install ggshield or uv tool install ggshield" >&2
  exit 1
fi

echo "[security-guard] Running ggshield secret scan"
# ggshield exit-code contract (verified against ggshield >=1.x):
#   0 — clean scan, no incidents
#   1 — scan ran and found incidents (secrets detected)        -> MUST fail the hook
#   2 — scan errored (bad config, network, parse failure, etc.) -> MUST fail the hook
#   3 — unauthenticated / no GITGUARDIAN_API_KEY               -> treat as skip so
#                                                                  local dev commits
#                                                                  aren't blocked
# Critically, exit 1 is NOT an auth failure — it is "secrets found". Treating it
# as a skip would let pre-commit pass while the hook printed "skipping" — exactly
# the failure mode Bugbot flagged. Only exit 3 may short-circuit the hook.
if ! "${GGSHIELD[@]}" secret scan pre-commit; then
  exit_code=$?
  if [ "$exit_code" -eq 3 ]; then
    echo "[security-guard] ggshield not authenticated (exit $exit_code) — skipping secret scan. Set GITGUARDIAN_API_KEY to enable." >&2
  else
    echo "[security-guard] ggshield secret scan failed with exit code $exit_code" >&2
    exit "$exit_code"
  fi
fi

if command -v codespell >/dev/null 2>&1; then
  changed_files=$(git diff --cached --name-only --diff-filter=ACM || true)
  if [ -z "${changed_files}" ]; then
    changed_files=$(git diff --name-only HEAD~1..HEAD 2>/dev/null || true)
  fi

  if [ -n "${changed_files}" ]; then
    echo "[security-guard] Running optional codespell fast pass"
    echo "${changed_files}" |       grep -E '\.(md|txt|py|ts|tsx|js|go|rs|kt|java|yaml|yml)$' |       xargs -r codespell -q 2 -L "hte,teh" || true
  fi
fi
