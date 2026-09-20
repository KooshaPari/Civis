#!/usr/bin/env python3
"""
FR-coverage gate for CI.

Regenerates docs/audits/fr-matrix.json from the current source/spec state and
fails the build if any of the gap-status counts rises above a stored snapshot.

Snapshot lives at docs/audits/.fr-snapshot.json and is committed alongside the
audit artifacts. The first run creates the snapshot; subsequent runs compare.

Run:
    python scripts/traceability/check-fr-coverage.py        # check + update snapshot
    python scripts/traceability/check-fr-coverage.py --no-write  # check only (CI mode)
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MATRIX = ROOT / "docs" / "audits" / "fr-matrix.json"
SNAPSHOT = ROOT / "docs" / "audits" / ".fr-snapshot.json"

# Allowed regression window. Pushes that bump any of these by more than N fail.
# A few extra are expected from time to time as audits catch up to new commits.
REGRESSION_BUDGET = {
    "SPEC-ONLY": 20,
    "TEST-NO-CODE-REF": 20,
    "IMPL-NO-TEST": 10,
    "STUB-TEST-ONLY": 5,
    "CODE-ONLY-no-spec": 2,
}

# Absolute floors. Even if the snapshot says 1000, we want to know if it ever
# balloons back to >1200.
ABSOLUTE_CEILINGS = {
    "SPEC-ONLY": 500,
    "TEST-NO-CODE-REF": 400,
    "IMPL-NO-TEST": 200,
    "STUB-TEST-ONLY": 30,
    "CODE-ONLY-no-spec": 5,
}


def regenerate_matrix() -> dict:
    """Run gen-fr-audit.py and load fr-matrix.json."""
    script = ROOT / "scripts" / "traceability" / "gen-fr-audit.py"
    if not script.exists():
        raise SystemExit(f"missing audit script: {script}")
    subprocess.run([sys.executable, str(script)], cwd=ROOT, check=True)
    return json.loads(MATRIX.read_text(encoding="utf-8"))


def count_statuses(matrix: dict) -> dict[str, int]:
    rows = matrix.get("rows") or []
    out: dict[str, int] = {}
    for r in rows:
        s = r.get("status", "UNKNOWN")
        out[s] = out.get(s, 0) + 1
    return out


def load_snapshot() -> dict[str, int]:
    if not SNAPSHOT.exists():
        return {}
    return json.loads(SNAPSHOT.read_text(encoding="utf-8"))


def save_snapshot(counts: dict[str, int]) -> None:
    SNAPSHOT.write_text(json.dumps(counts, indent=2, sort_keys=True), encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--no-write", action="store_true",
                    help="Do not update the snapshot (use in CI)")
    args = ap.parse_args()

    matrix = regenerate_matrix()
    counts = count_statuses(matrix)
    snapshot = load_snapshot()

    errors: list[str] = []

    # Check ceilings
    for status, ceiling in ABSOLUTE_CEILINGS.items():
        actual = counts.get(status, 0)
        if actual > ceiling:
            errors.append(
                f"{status}: {actual} exceeds absolute ceiling {ceiling}"
            )

    # Check regression budget
    for status, budget in REGRESSION_BUDGET.items():
        actual = counts.get(status, 0)
        prev = snapshot.get(status, actual)
        delta = actual - prev
        if delta > budget:
            errors.append(
                f"{status}: regressed by {delta} ({prev} -> {actual}), "
                f"budget is +{budget}"
            )

    print("FR-coverage status counts:")
    for status, count in sorted(counts.items(), key=lambda kv: -kv[1]):
        prev = snapshot.get(status, count)
        delta = count - prev
        arrow = "  " if delta == 0 else (f"+{delta}" if delta > 0 else f"{delta}")
        print(f"  {status:20s} {count:5d}  (prev {prev}, {arrow})")

    if errors:
        print("\nFR-coverage gate FAILED:")
        for e in errors:
            print(f"  - {e}")
        return 1

    print("\nFR-coverage gate OK.")
    if not args.no_write:
        save_snapshot(counts)
        print(f"Updated snapshot: {SNAPSHOT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
