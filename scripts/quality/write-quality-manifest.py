"""Write `.ci/quality-manifest.json` from gate results (shared by bash + PowerShell emitters)."""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
from datetime import datetime, timezone


def main() -> int:
    gates_raw = os.environ.get("QUALITY_GATES_JSON", "{}")
    manifest_path = os.environ.get("MANIFEST_PATH", ".ci/quality-manifest.json")
    gates = json.loads(gates_raw)

    git_sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    try:
        rust = subprocess.check_output(["rustc", "--version"], text=True).strip()
    except Exception:
        rust = "unknown"
    host = os.environ.get("COMPUTERNAME") or os.environ.get("HOSTNAME") or "unknown"

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

    os.makedirs(os.path.dirname(manifest_path) or ".", exist_ok=True)
    with open(manifest_path, "w", encoding="utf-8") as f:
        json.dump(body, f, indent=2)
        f.write("\n")

    failed = [k for k, v in gates.items() if v.get("status") != "pass"]
    print(f"Wrote {manifest_path} (git_sha={git_sha}, manifest_hash={body['manifest_hash']})")
    if failed:
        print(f"failed gates: {', '.join(failed)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
