#!/usr/bin/env bash
# Run full local quality gates and write `.ci/quality-manifest.json` for cloud CI verification.
# Optional Unreal tier: unreal_preflight, unreal_build (see scripts/quality/README.md).
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "${ROOT}"
MANIFEST="${ROOT}/.ci/quality-manifest.json"
mkdir -p "${ROOT}/.ci"

if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: python3 is required to emit the quality manifest" >&2
  exit 1
fi

RESULTS=()
FAIL=0

record() {
  local name="$1"
  local status="$2"
  local detail="${3:-}"
  RESULTS+=("${name}|${status}|${detail}")
  if [[ "${status}" == "pass" ]]; then
    return 0
  fi
  if [[ "${status}" == "skip" && "${name}" == unreal_* ]]; then
    return 0
  fi
  FAIL=1
}

run_gate() {
  local name="$1"
  shift
  local out=""
  if out="$("$@" 2>&1)"; then
    record "${name}" "pass" ""
  else
    record "${name}" "fail" "$(echo "${out}" | tail -n 20 | tr '\n' ' ')"
    return 1
  fi
}

echo "==> civis quality manifest (local gates)"

if command -v just >/dev/null 2>&1; then
  run_gate civis_3d_verify just civis-3d-verify || true
else
  run_gate rust_fmt cargo fmt --check || true
  run_gate rust_clippy cargo clippy --workspace --all-targets -- -D warnings || true
  run_gate rust_test cargo test --workspace || true
  run_gate godot_test bash -lc 'cd clients/godot-ref/rust && cargo test' || true
fi
run_gate web_test bash -lc 'cd web && npm test' || true
run_gate dashboard_typecheck bash -lc 'cd web/dashboard && bun install --frozen-lockfile && bun run typecheck' || true

# Optional Unreal tier (skip when no UE; never fails machines without UE)
optional_unreal_gate() {
  if [[ "${CIVIS_QUALITY_UNREAL:-}" != "1" ]]; then
    if command -v pwsh >/dev/null 2>&1; then
      pwsh -NoProfile -File "${ROOT}/clients/unreal-show/scripts/detect-ue.ps1" >/dev/null 2>&1 || return 0
    else
      return 0
    fi
  fi
  local verify="${ROOT}/clients/unreal-show/scripts/verify-unreal-ready.ps1"
  local build="${ROOT}/clients/unreal-show/scripts/build.ps1"
  if [[ -f "${verify}" ]]; then
    if command -v pwsh >/dev/null 2>&1; then
      run_gate unreal_preflight pwsh -NoProfile -File "${verify}" || true
    else
      run_gate unreal_preflight powershell -NoProfile -File "${verify}" || true
    fi
  fi
  if command -v pwsh >/dev/null 2>&1 && pwsh -NoProfile -File "${ROOT}/clients/unreal-show/scripts/detect-ue.ps1" >/dev/null 2>&1; then
    if [[ -f "${build}" ]]; then
      run_gate unreal_build pwsh -NoProfile -File "${build}" || true
    fi
  else
    record unreal_build skip "no UE_ROOT/UBT"
  fi
}
optional_unreal_gate || true

export MANIFEST_PATH="${MANIFEST}"
export QUALITY_GATE_RESULTS="$(printf '%s\n' "${RESULTS[@]}")"
GATES_JSON="$(python3 - <<'PY'
import json
import os

gates = {}
for row in os.environ.get("QUALITY_GATE_RESULTS", "").splitlines():
    if not row.strip():
        continue
    parts = row.split("|", 2)
    name = parts[0]
    status = parts[1] if len(parts) > 1 else "fail"
    detail = parts[2] if len(parts) > 2 else ""
    gates[name] = {"status": status, "detail": detail}
print(json.dumps(gates))
PY
)"
export QUALITY_GATES_JSON="${GATES_JSON}"
python3 "${ROOT}/scripts/quality/write-quality-manifest.py"

if [[ "${FAIL}" -ne 0 ]]; then
  exit 1
fi
