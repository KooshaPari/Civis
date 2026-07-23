#!/usr/bin/env bash
# Cloud CI: verify committed local quality attestation (no cargo/rust on the runner).
#
# Gate tiers (see scripts/quality/README.md):
#   Core (required): civis_3d_verify, bevy_egui_check, web_test, dashboard_typecheck, dashboard_build, rust_*, godot_test
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
ERROR: .ci/quality-manifest.json not found
Local-quality attestation is required before merge. Run local gates and commit the manifest:

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
if not isinstance(attested, str) or not attested:
    raise SystemExit("manifest git_sha is required and must identify HEAD or HEAD^")
if attested != head:
    content_matches = False
    try:
        parent = subprocess.check_output(["git", "rev-parse", "HEAD^"], text=True).strip()
    except subprocess.CalledProcessError:
        parent = ""
    if attested != parent:
        attested_exists = subprocess.run(
            ["git", "cat-file", "-e", f"{attested}^{{commit}}"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        ).returncode == 0
        if isinstance(body.get("content_hash"), str):
            entries = subprocess.check_output(
                ["git", "ls-tree", "-r", "--full-tree", "HEAD"], text=True
            ).splitlines()
            current_hash = hashlib.blake2b(
                "\n".join(
                    entry
                    for entry in entries
                    if not entry.endswith("\t.ci/quality-manifest.json")
                ).encode(),
                digest_size=32,
            ).hexdigest()
            content_matches = body["content_hash"] == current_hash
        elif attested_exists:
            try:
                subprocess.run(
                    [
                        "git",
                        "diff",
                        "--quiet",
                        attested,
                        "HEAD",
                        "--",
                        ".",
                        ":(exclude).ci/quality-manifest.json",
                    ],
                    check=True,
                )
                content_matches = True
            except subprocess.CalledProcessError:
                content_matches = False
    if attested != parent and not content_matches:
        raise SystemExit(
            f"stale manifest: git_sha {attested} != HEAD {head}"
            + (f" or parent {parent}" if parent else "")
            + " or matching non-manifest content"
            + "\nRe-run: lefthook run pre-push && commit .ci/quality-manifest.json"
        )

OPTIONAL_GATE_PREFIXES = ("unreal_", "extra_")

def gate_ok(key: str, status: str) -> bool:
    if status == "pass":
        return True
    if key.startswith(OPTIONAL_GATE_PREFIXES):
        # Optional Unreal tier: pass/skip/fail never block cloud verify.
        return status in ("pass", "skip", "fail")
    return False

gates = body.get("gates") or {}
if not isinstance(gates, dict):
    raise SystemExit("manifest gates must be an object")

required = {
    "civis_3d_verify",
    "bevy_egui_check",
    "web_test",
    "dashboard_typecheck",
    "dashboard_build",
}
missing = sorted(required - set(gates))
if missing:
    raise SystemExit(f"manifest is missing required gates: {', '.join(missing)}")

failed = [k for k, v in gates.items() if not gate_ok(k, v.get("status", ""))]
if failed:
    raise SystemExit(f"manifest records failed gates: {', '.join(failed)}")

attestation = {
    "git_sha": body["git_sha"],
    "content_hash": body.get("content_hash", ""),
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
