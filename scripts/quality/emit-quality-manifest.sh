#!/usr/bin/env bash
# Run full local quality gates and write `.ci/quality-manifest.json` for cloud CI verification.
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
  [[ "${status}" == "pass" ]] || FAIL=1
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

export MANIFEST_PATH="${MANIFEST}"
export QUALITY_GATE_RESULTS="$(printf '%s\n' "${RESULTS[@]}")"
python3 - <<'PY'
import hashlib
import json
import os
import subprocess
from datetime import datetime, timezone

root = subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip()
results = os.environ.get("QUALITY_GATE_RESULTS", "")
gates = {}
for row in results.splitlines():
    if not row.strip():
        continue
    parts = row.split("|", 2)
    name = parts[0]
    status = parts[1] if len(parts) > 1 else "fail"
    detail = parts[2] if len(parts) > 2 else ""
    gates[name] = {"status": status, "detail": detail}

git_sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
try:
    rust = subprocess.check_output(["rustc", "--version"], text=True).strip()
except Exception:
    rust = "unknown"
try:
    host = subprocess.check_output(["hostname"], text=True).strip()
except Exception:
    host = "unknown"

body = {
    "version": "1",
    "repo": "Civis",
    "git_sha": git_sha,
    "created_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "runner": {"host": host, "rust": rust},
    "gates": gates,
}
attestation = {
    "git_sha": git_sha,
    "gates": sorted(
        [{"key": k, "status": v["status"]} for k, v in gates.items()],
        key=lambda x: x["key"],
    ),
}
body["manifest_hash"] = hashlib.blake2b(
    json.dumps(attestation, separators=(",", ":")).encode(),
    digest_size=32,
).hexdigest()

path = os.environ["MANIFEST_PATH"]
with open(path, "w", encoding="utf-8") as f:
    json.dump(body, f, indent=2)
    f.write("\n")
print(f"Wrote {path} (git_sha={git_sha}, manifest_hash={body['manifest_hash']})")
failed = [k for k, v in gates.items() if v.get("status") != "pass"]
if failed:
    raise SystemExit(f"failed gates: {', '.join(failed)}")
PY

if [[ "${FAIL}" -ne 0 ]]; then
  exit 1
fi
