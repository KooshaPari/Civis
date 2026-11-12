#!/usr/bin/env bash
set -euo pipefail

# Keep the pre-push command in a file so Lefthook uses one parser on Windows
# and Linux; the manifest verifier remains the cloud enforcement point.
if [[ "${CI:-}" == "true" || "${CI:-}" == "1" || "${SKIP_QUALITY_MANIFEST:-}" == "1" || "${SKIP_QUALITY:-}" == "1" ]]; then
  echo "quality-manifest: skipped (CI or SKIP_QUALITY*)"
  exit 0
fi

if command -v pwsh >/dev/null 2>&1; then
  pwsh -NoProfile -File scripts/quality/emit-quality-manifest.ps1
elif command -v powershell >/dev/null 2>&1; then
  powershell -NoProfile -File scripts/quality/emit-quality-manifest.ps1
else
  bash scripts/quality/emit-quality-manifest.sh
fi
