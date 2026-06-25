#!/usr/bin/env bash
# Run full local quality gates and write `.ci/quality-manifest.json` for cloud CI verification.
# Optional Unreal tier: unreal_preflight, unreal_build (see scripts/quality/README.md).
# Optional Extras tier (opt-in via CIVIS_QUALITY_EXTRAS=1): cargo-audit, cargo-deny,
# cargo-machete, cargo-semver-checks, trufflehog, fr-coverage, docs:check. These
# run only when the corresponding tools are on PATH; otherwise the gate is
# recorded as `skip` with a clear reason. CI never runs them.
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
  if [[ "${status}" == "skip" && ( "${name}" == unreal_* || "${name}" == extra_* ) ]]; then
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
  if [ "${SKIP_CIVIS_3D_VERIFY:-0}" = "1" ] || [ "${SKIP_QUALITY_MANIFEST:-0}" = "1" ] || [ "${SKIP_QUALITY:-0}" = "1" ]; then
    record "civis_3d_verify" "skip" "SKIP_CIVIS_3D_VERIFY/SKIP_QUALITY_MANIFEST set"
  else
    run_gate civis_3d_verify just civis-3d-verify || true
  fi
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

# Optional Extras tier (opt-in; gated behind CIVIS_QUALITY_EXTRAS=1 so the
# default path is the 5–7 second manifest emission, not a multi-minute sweep).
# Each gate that is `skip` is recorded as such and the manifest hash treats it
# as informational. CI's verify-quality-manifest.sh still passes for skipped
# extras (the script's gate_ok treats any status as long as it's not "fail" for
# non-optional gates; extras are added to the optional prefix list below).
optional_extras_gate() {
  if [[ "${CIVIS_QUALITY_EXTRAS:-}" != "1" ]]; then
    record extra_cargo_audit skip "CIVIS_QUALITY_EXTRAS not set (opt-in)"
    record extra_cargo_deny skip "CIVIS_QUALITY_EXTRAS not set (opt-in)"
    record extra_cargo_machete skip "CIVIS_QUALITY_EXTRAS not set (opt-in)"
    record extra_cargo_semver skip "CIVIS_QUALITY_EXTRAS not set (opt-in)"
    record extra_trufflehog skip "CIVIS_QUALITY_EXTRAS not set (opt-in)"
    record extra_fr_coverage skip "CIVIS_QUALITY_EXTRAS not set (opt-in)"
    record extra_docs_check skip "CIVIS_QUALITY_EXTRAS not set (opt-in)"
    record extra_security_guard skip "CIVIS_QUALITY_EXTRAS not set (opt-in)"
    return 0
  fi
  if command -v cargo-audit >/dev/null 2>&1; then
    run_gate extra_cargo_audit cargo audit --quiet || true
  else
    record extra_cargo_audit skip "cargo-audit not installed"
  fi
  if command -v cargo-deny >/dev/null 2>&1; then
    run_gate extra_cargo_deny cargo deny check || true
  else
    record extra_cargo_deny skip "cargo-deny not installed"
  fi
  if command -v cargo-machete >/dev/null 2>&1; then
    run_gate extra_cargo_machete cargo machete || true
  else
    record extra_cargo_machete skip "cargo-machete not installed"
  fi
  if command -v cargo-semver-checks >/dev/null 2>&1; then
    run_gate extra_cargo_semver cargo semver-checks || true
  else
    record extra_cargo_semver skip "cargo-semver-checks not installed"
  fi
  if command -v trufflehog >/dev/null 2>&1; then
    run_gate extra_trufflehog trufflehog filesystem . --no-update --only-verified || true
  else
    record extra_trufflehog skip "trufflehog not installed"
  fi
  if [[ -f "${ROOT}/scripts/fr-coverage/run-fr-coverage.sh" ]]; then
    run_gate extra_fr_coverage bash "${ROOT}/scripts/fr-coverage/run-fr-coverage.sh" || true
  else
    record extra_fr_coverage skip "scripts/fr-coverage/run-fr-coverage.sh missing"
  fi
  if [[ -d "${ROOT}/docs" ]] && command -v bun >/dev/null 2>&1; then
    run_gate extra_docs_check bash -lc 'cd docs && bun run docs:check' || true
  else
    record extra_docs_check skip "docs/ missing or bun not installed"
  fi
  if [[ -x "${ROOT}/.github/hooks/security-guard.sh" ]]; then
    run_gate extra_security_guard bash "${ROOT}/.github/hooks/security-guard.sh" || true
  else
    record extra_security_guard skip ".github/hooks/security-guard.sh missing"
  fi
}
optional_extras_gate || true

export MANIFEST_PATH="${MANIFEST}"
QUALITY_GATE_RESULTS="$(printf '%s\n' "${RESULTS[@]}")"
export QUALITY_GATE_RESULTS
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
