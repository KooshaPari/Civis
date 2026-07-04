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
    raw_gates = json.loads(gates_raw)
    # Normalise: PowerShell ConvertTo-Json may serialise a hashtable as a list of
    # {"Key": k, "Value": v} pairs when pipeline semantics unwrap the object.
    if isinstance(raw_gates, list):
        gates: dict[str, dict[str, str]] = {}
        for item in raw_gates:
            if isinstance(item, dict) and "Key" in item and "Value" in item:
                val = item["Value"]
                if isinstance(val, dict):
                    gates[item["Key"]] = val
                else:
                    gates[item["Key"]] = {"status": str(val), "detail": ""}
            elif isinstance(item, dict) and "key" in item:
                gates[item["key"]] = {
                    "status": item.get("status", "fail"),
                    "detail": item.get("detail", ""),
                }
    elif isinstance(raw_gates, dict):
        gates = {}
        for k, v in raw_gates.items():
            if isinstance(v, dict):
                gates[k] = v
            elif isinstance(v, list):
                # PS may serialise a small hashtable as a list of {Key,Value} pairs
                entry: dict[str, str] = {"status": "fail", "detail": ""}
                for pair in v:
                    if isinstance(pair, dict):
                        pk = str(pair.get("Key") or pair.get("key") or "").lower()
                        pv = str(
                            pair.get("Value")
                            if pair.get("Value") is not None
                            else pair.get("value", "")
                        )
                        if pk:  # skip pairs with empty key
                            entry[pk] = pv
                gates[k] = entry
            else:
                gates[k] = {"status": str(v), "detail": ""}
    else:
        gates = {}

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

    # Unreal + opt-in extras gates are optional infrastructure gates; they
    # never block the push.
    optional_gate_prefixes = ("unreal_", "extra_")
    failed = [
        k
        for k, v in gates.items()
        if v.get("status") not in ("pass", "skip")
        and not any(k.startswith(p) for p in optional_gate_prefixes)
    ]
    print(
        f"Wrote {manifest_path} (git_sha={git_sha}, manifest_hash={body['manifest_hash']})"
    )
    if failed:
        print(f"failed gates: {', '.join(failed)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
