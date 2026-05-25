#!/usr/bin/env python3
"""Refresh timestamp_utc fields in a proof bundle JSON for CI freshness checks."""
from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: refresh_proof_bundle_timestamps.py <bundle.json>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    data = json.loads(path.read_text(encoding="utf-8"))
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    for key in ("judges", "bridges", "judge_receipts", "bridge_receipts"):
        for item in data.get(key, []):
            if isinstance(item, dict):
                item["timestamp_utc"] = ts

    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    print(f"refreshed timestamps in {path} -> {ts}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
