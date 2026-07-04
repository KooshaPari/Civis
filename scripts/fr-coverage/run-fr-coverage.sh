#!/usr/bin/env bash
set -euo pipefail

ROOT="${ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT"

echo "==> Traceability check (matrix + FR-CIV-3D)"
bash scripts/traceability/check-traceability.sh

echo "==> civ-engine determinism tests"
cargo test -p civ-engine determinism

echo "==> civ-engine replay tests"
cargo test -p civ-engine replay

echo "==> civ-engine hash chain tests"
cargo test -p civ-engine hash_chain

echo "==> civ-engine proptest (determinism_proptest)"
cargo test -p civ-engine --test determinism_proptest

echo "==> civ-engine proptest (invariants_proptest)"
cargo test -p civ-engine --test invariants_proptest

echo "==> civ-research tests"
cargo test -p civ-research

echo "==> web spectator tests (ADR-009)"
if command -v node >/dev/null 2>&1; then
  (cd web && npm test)
else
  echo "skip: node not installed"
fi
