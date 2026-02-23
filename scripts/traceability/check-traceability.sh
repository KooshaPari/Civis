#!/usr/bin/env bash
set -euo pipefail

missing=0
for id in CIV-CORE-1 CIV-POLICY-1 CIV-METRICS-1 CIV-EVENT-1; do
  if ! rg -q "$id" docs/traceability/TRACEABILITY_MATRIX.md; then
    echo "Missing traceability ID: $id"
    missing=1
  fi
done

if [ "$missing" -ne 0 ]; then
  echo "Traceability check failed"
  exit 1
fi

echo "Traceability check passed"
