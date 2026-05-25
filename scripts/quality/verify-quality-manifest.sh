#!/usr/bin/env bash
# Cloud CI: verify committed local quality attestation (no cargo/rust on the runner).
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
ERROR: missing .ci/quality-manifest.json

Run local gates and commit the manifest:
  lefthook install
  lefthook run pre-push
  git add .ci/quality-manifest.json && git commit -m "chore(ci): refresh quality manifest"
EOF
  exit 1
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

gates = body.get("gates") or {}
failed = [k for k, v in gates.items() if v.get("status") != "pass"]
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

print(f"quality-manifest: OK ({len(gates)} gates, sha={head[:12]})")
PY
