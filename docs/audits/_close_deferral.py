"""Remove an implemented requirement from the manual deferral ledger.

The ledger at `docs/audits/spec-only-deferrals.json` lists IDs a human has
judged unimplementable or out of scope. Once a requirement is actually
implemented and tagged, leaving it in the ledger produces a contradiction:
the matrix says COVERED and the ledger says deferred. This script drops the ID
and recomputes the totals so the two files cannot drift silently.

Usage:  python docs/audits/_close_deferral.py FR-SAVE-006 [...]
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
LEDGER = REPO / "docs/audits/spec-only-deferrals.json"


def main(argv: list[str]) -> int:
    if not argv:
        print(__doc__)
        return 2

    data = json.loads(LEDGER.read_text(encoding="utf-8"))
    buckets = data["ids"]

    removed: list[tuple[str, str]] = []
    for target in argv:
        found = False
        for bucket, entries in buckets.items():
            keep = [e for e in entries if e.get("id") != target]
            if len(keep) != len(entries):
                removed.append((target, bucket))
                buckets[bucket] = keep
                found = True
        if not found:
            print(f"{target}: NOT in the ledger (already closed, or never listed)")

    if not removed:
        print("nothing to remove")
        return 0

    for target, bucket in removed:
        print(f"{target}: removed from '{bucket}'")

    data["counts"] = {k: len(v) for k, v in buckets.items()}
    data["spec_only_total"] = sum(len(v) for v in buckets.values())
    LEDGER.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    print(f"\nspec_only_total: {data['spec_only_total']}")
    for k, v in sorted(data["counts"].items()):
        print(f"  {k}: {v}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
