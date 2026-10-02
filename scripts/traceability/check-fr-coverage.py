#!/usr/bin/env python3
"""
FR-coverage gate for CI.

Regenerates docs/audits/fr-matrix.json from the current source/spec state and
fails the build if any of the gap-status counts rises above a stored snapshot.

Three things must hold for a green run to mean anything, and each is enforced
below rather than assumed:

1. The evidence is regenerated. `gen-fr-audit.py` and `_build_matrix.py` both
   read `docs/audits/_id_inventory_v3.json` and neither regenerates it, so the
   gate rebuilds the inventory first. A missing generator is fatal, not a
   warning: the original version printed "matrix may be built on a stale
   inventory" and then exited 0.
2. Every status in the matrix is guarded. A status in neither REGRESSION_BUDGET
   nor ABSOLUTE_CEILINGS is invisible here, so its rows can vanish while the
   gate stays green. The two health statuses are guarded by floor, not budget.
3. No budget is wider than the count it guards, or losing every row it protects
   would still pass.

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
#
# Every budget must stay strictly below the current count of the status it
# guards. A budget equal to or wider than the count means the status could fall
# to zero and still pass, which is the one loss this gate exists to catch.
#
# REBASED 2026-10-02. 97 rows left COVERED and 96 entered TEST-NO-CODE-REF
# because `_gather_ids.py` had been crediting `// [unbound] <id>: NOT
# IMPLEMENTED` rationales as `in_code`, and a second pass found 74 more rows
# covered only by a per-id line inside a "Removed, with the reason each cannot
# be discharged here:" block. Both are comments asserting a requirement is NOT
# met, so crediting them asserted exactly what they deny.
#
# The budgets below are unchanged: the corrections moved COVERED 840 -> 669 and
# TEST-NO-CODE-REF 156 -> 326, and a 20-row regression budget still bites against
# the larger TEST-NO-CODE-REF base, so no widening was needed or justified. Only
# the COVERED floor, which had to follow a number that was itself wrong, moved.
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

# Health statuses get a floor instead of a regression budget. `COVERED` rising is
# good news and must never fail the build, while `COVERED` falling is exactly
# the regression that matters, so a symmetric budget is the wrong shape here.
# A floor is checked against the snapshot as an absolute minimum: if the count
# drops below it, rows were lost or silently reclassified, and the gate fails
# without needing to know by how much.
#
# These two carried 1068 of 1430 rows while sitting in no budget and no ceiling
# at all, so all of them could disappear with the gate reporting success.
#
# The COVERED floor moved 800 -> 600 on 2026-10-02 for the same reason as the
# budgets above: 97 rows were covered only by their own `[unbound] NOT
# IMPLEMENTED` comment. A second pass the same day found 74 more rows covered
# only by a line inside a "Removed, with the reason each cannot be discharged
# here:" block, which carries no `[unbound]` token and so escaped the first rule.
# Together the two passes took COVERED from 840 to 669. The floor is now 600,
# which keeps the guard meaningful against the corrected baseline rather than
# pinning a number already known to be inflated.
STATUS_FLOORS = {
    "COVERED": 600,
    "SELF-TEST-ONLY": 220,
}


def regenerate_matrix() -> dict:
    """Rebuild the inventory, then the matrix that reads it, and load the matrix.

    The inventory is NOT optional. `gen-fr-audit.py` and `_build_matrix.py` both
    read `docs/audits/_id_inventory_v3.json` and neither regenerates it, so
    running only the matrix builder lets the gate grade source that the inventory
    predates. That happened once and the gate passed on stale evidence: after
    re-tagging `crates/species/src/speciation.rs` with FR-CIV-SPECIES-300..304,
    `gen-fr-audit.py` reported all five still SPEC-ONLY, exactly as before the
    edit, because the inventory on disk was 14 minutes older than the change.

    Regenerating both steps makes a green gate mean what it claims.

    A missing `_gather_ids.py` is fatal. It used to print a warning and
    continue, which left the gate grading the stale inventory on disk and
    exiting 0 — the original defect, reachable by renaming one file.
    """
    gather = ROOT / "docs" / "audits" / "_gather_ids.py"
    script = ROOT / "scripts" / "traceability" / "gen-fr-audit.py"
    if not gather.exists():
        raise SystemExit(
            f"missing inventory generator: {gather}\n"
            "The matrix is built from docs/audits/_id_inventory_v3.json, which "
            "only this script writes. Without it the gate would grade a stale "
            "inventory and report success, which is the defect this check "
            "exists to prevent."
        )
    if not script.exists():
        raise SystemExit(f"missing audit script: {script}")
    subprocess.run([sys.executable, str(gather)], cwd=ROOT, check=True)
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

    # Check floors on the health statuses.
    #
    # A floor is checked against the snapshot, not just the live count, so a
    # status that is already below its floor fails even on a fresh checkout with
    # no snapshot. That matters because a dropped row usually arrives as a
    # lower count rather than as a jump in some gap status.
    for status, floor in STATUS_FLOORS.items():
        actual = counts.get(status, 0)
        if actual < floor:
            prev = snapshot.get(status)
            detail = f" (snapshot {prev})" if prev is not None else " (no snapshot)"
            errors.append(
                f"{status}: {actual} is below the floor of {floor}{detail}; "
                "rows were lost or silently reclassified"
            )

    # Every status that actually appears in the matrix must be guarded by one
    # of the three tables above. An unguarded status is invisible to this gate,
    # so its rows can be deleted, reclassified, or fabricated undetected.
    guarded = set(REGRESSION_BUDGET) | set(ABSOLUTE_CEILINGS) | set(STATUS_FLOORS)
    present = {r.get("status", "UNKNOWN") for r in matrix.get("rows") or []}
    unguarded = sorted(present - guarded)
    if unguarded:
        errors.append(
            f"unguarded statuses present in the matrix: {unguarded}; add each to "
            "REGRESSION_BUDGET, ABSOLUTE_CEILINGS, or STATUS_FLOORS"
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
