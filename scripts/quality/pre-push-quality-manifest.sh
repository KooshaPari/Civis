#!/usr/bin/env bash
set -euo pipefail

# Keep the pre-push command in a file so Lefthook uses one parser on Windows
# and Linux; the manifest verifier remains the cloud enforcement point.
if [[ "${CI:-}" == "true" || "${CI:-}" == "1" || "${SKIP_QUALITY_MANIFEST:-}" == "1" || "${SKIP_QUALITY:-}" == "1" ]]; then
  echo "quality-manifest: skipped (CI or SKIP_QUALITY*)"
  exit 0
fi

manifest_path=".ci/quality-manifest.json"
if [[ -f "$manifest_path" ]] && command -v python3 >/dev/null 2>&1 && \
  python3 -c 'import os, sys, time; raise SystemExit(0 if time.time() - os.path.getmtime(sys.argv[1]) < 3600 else 1)' "$manifest_path"; then
  echo "quality-manifest: skipped (recent manifest < 1h old)"
  exit 0
fi

if command -v pwsh >/dev/null 2>&1; then
  pwsh -NoProfile -File scripts/quality/emit-quality-manifest.ps1
elif command -v powershell >/dev/null 2>&1; then
  powershell -NoProfile -File scripts/quality/emit-quality-manifest.ps1
else
  bash scripts/quality/emit-quality-manifest.sh
fi
