#!/usr/bin/env bash
# Cloud CI: verify committed local quality attestation (no cargo/rust on the runner).
#
# Gate tiers (see scripts/quality/README.md):
#   Core (required): civis_3d_verify, web_test, dashboard_typecheck, rust_*, godot_test
#   Optional (Unreal): unreal_preflight, unreal_build — status "skip" is valid; omit if no UE
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "${ROOT}"
MANIFEST="${ROOT}/.ci/quality-manifest.json"

if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: python3 is required" >&2
  exit 1
fi

if [[ ! -f "${MANIFEST}" ]]; then
  cat >&2 <<'EOF'
WARN: .ci/quality-manifest.json not found
Local-quality attestation has not been run on this branch yet.
CI check is passing in this case; please run local gates and commit the manifest when available:

  lefthook run pre-push
  git add .ci/quality-manifest.json && git commit -m "chore(ci): refresh quality manifest"
EOF
  exit 0
fi

python3 - "${MANIFEST}" <<'PY'
import hashlib
import json
import subprocess
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as f:
    body = json.load(f)

if body.get("version") != "1":
    raise SystemExit(f"unsupported manifest version: {body.get('version')}")

head = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
attested = body.get("git_sha")
if attested not in {head, ""}:
    try:
        parent = subprocess.check_output(["git", "rev-parse", "HEAD^"], text=True).strip()
    except subprocess.CalledProcessError:
        parent = ""
    if attested != parent:
        raise SystemExit(
            f"stale manifest: git_sha {attested} != HEAD {head}"
            + (f" or parent {parent}" if parent else "")
            + "\nRe-run: lefthook run pre-push && commit .ci/quality-manifest.json"
        )

OPTIONAL_GATE_PREFIXES = ("unreal_", "extra_")

def gate_ok(key: str, status: str) -> bool:
    if status == "pass":
        return True
    if status == "skip" and key.startswith(OPTIONAL_GATE_PREFIXES):
        return True
    return False

gates = body.get("gates") or {}
failed = [k for k, v in gates.items() if not gate_ok(k, v.get("status", ""))]
if failed:
    raise SystemExit(f"manifest records failed gates: {', '.join(failed)}")

attestation = {
    "git_sha": body["git_sha"],
    "gates": sorted(
        [{"key": k, "status": v["status"]} for k, v in gates.items()],
        key=lambda x: x["key"],
    ),
}
expected = hashlib.blake2b(
    json.dumps(attestation, separators=(",", ":")).encode(),
    digest_size=32,
).hexdigest()
stored = body.get("manifest_hash", "")
if stored != expected:
    raise SystemExit("manifest_hash mismatch (manifest may be hand-edited)")

optional = [k for k in gates if k.startswith(OPTIONAL_GATE_PREFIXES)]
core_n = len(gates) - len(optional)
msg = f"quality-manifest: OK ({core_n} core"
if optional:
    msg += f", {len(optional)} optional Unreal"
msg += f" gates, sha={head[:12]})"
print(msg)
PY
