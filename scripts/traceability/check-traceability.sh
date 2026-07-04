#!/usr/bin/env bash
set -euo pipefail

ROOT="${ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT"

missing=0

search_contains() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q "$pattern" "$file"
  else
    grep -qF "$pattern" "$file"
  fi
}

extract_fr_civ_ids() {
  local file="$1"
  if command -v rg >/dev/null 2>&1; then
    rg -o 'FR-CIV-[A-Z0-9]+-[0-9]+' "$file" | sort -u
  else
    grep -oE 'FR-CIV-[A-Z0-9]+-[0-9]+' "$file" | sort -u
  fi
}

id_in_any_file() {
  local id="$1"
  shift
  for f in "$@"; do
    if [ -f "$f" ] && search_contains "$id" "$f"; then
      return 0
    fi
  done
  return 1
}

MATRIX="docs/traceability/TRACEABILITY_MATRIX.md"
FR_3D_DOC="docs/development-guide/fr-3d-additions.md"
FR_3D_MATRIX="docs/traceability/fr-3d-matrix.md"

# Legacy CIV traceability IDs (strategic matrix)
for id in CIV-CORE-1 CIV-POLICY-1 CIV-METRICS-1 CIV-EVENT-1; do
  if ! id_in_any_file "$id" "$MATRIX"; then
    echo "Missing traceability ID: $id (expected in $MATRIX)"
    missing=1
  fi
done

# FR-CIV-* IDs from 3D extension requirements
if [ ! -f "$FR_3D_DOC" ]; then
  echo "Missing source doc: $FR_3D_DOC"
  exit 1
fi

fr_3d_targets=("$MATRIX")
if [ -f "$FR_3D_MATRIX" ]; then
  fr_3d_targets+=("$FR_3D_MATRIX")
fi

while IFS= read -r id; do
  [ -n "$id" ] || continue
  if ! id_in_any_file "$id" "${fr_3d_targets[@]}"; then
    echo "Missing FR-CIV traceability ID: $id (expected in $MATRIX or $FR_3D_MATRIX)"
    missing=1
  fi
done < <(extract_fr_civ_ids "$FR_3D_DOC")

if [ "$missing" -ne 0 ]; then
  echo "Traceability check failed"
  exit 1
fi

echo "Traceability check passed"
