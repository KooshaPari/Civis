#!/usr/bin/env python3
"""Regenerate the FR coverage matrix and audit report from the ID inventory.

Usage:
    python scripts/traceability/gen-fr-audit.py

Inputs:
    docs/audits/_id_inventory_v3.json  (produced by docs/audits/_gather_ids.py)

Outputs:
    docs/audits/fr-matrix.json
    docs/audits/fr-coverage-audit-<date>.md

Classification:
    COVERED          — spec/trace reference + code reference + test reference
    IMPL-NO-TEST     — spec/trace reference + code reference, no test reference
    SPEC-ONLY        — spec/trace reference only, no code reference
    CODE-ONLY-no-spec — code reference only, no spec/trace reference
"""

import json
import sys
from datetime import date
from pathlib import Path
from collections import defaultdict

ROOT = Path(__file__).resolve().parents[2]
INVENTORY = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
OUT_JSON = ROOT / "docs" / "audits" / "fr-matrix.json"
OUT_MD = ROOT / "docs" / "audits" / f"fr-coverage-audit-{date.today().isoformat()}.md"

STATUS_ORDER = ["COVERED", "IMPL-NO-TEST", "SPEC-ONLY", "CODE-ONLY-no-spec"]
STATUS_LEGEND = {
    "COVERED": "spec/trace + code + test all present",
    "IMPL-NO-TEST": "spec/trace + code present, no test reference",
    "SPEC-ONLY": "spec/trace present, no implementing code found",
    "CODE-ONLY-no-spec": "code present, no spec/traceability reference",
}


def epic_from_id(eid: str) -> str:
    """Return the epic/family prefix of an ID.

    Examples:
        FR-AI-001          -> FR-AI
        FR-CIV-EMERG-001   -> FR-CIV-EMERG
        NFR-CIV-PERF-001   -> NFR-CIV-PERF
    """
    parts = eid.split("-")
    # Drop the trailing numeric token (and any empty trailing tokens).
    while parts and (parts[-1].isdigit() or parts[-1] == ""):
        parts.pop()
    return "-".join(parts)


def classify(row: dict) -> str:
    has_spec = bool(row.get("in_specs") or row.get("in_traceability") or row.get("in_func_req"))
    has_code = bool(row.get("in_code"))
    has_test = bool(row.get("in_tests"))

    if has_spec and has_code and has_test:
        return "COVERED"
    if has_spec and has_code:
        return "IMPL-NO-TEST"
    if has_spec:
        return "SPEC-ONLY"
    return "CODE-ONLY-no-spec"


def main() -> int:
    if not INVENTORY.exists():
        print(f"Inventory not found: {INVENTORY}", file=sys.stderr)
        return 1

    data = json.loads(INVENTORY.read_text(encoding="utf-8"))
    ids = data.get("ids", [])

    rows = []
    totals = defaultdict(int)
    by_epic = defaultdict(int)
    by_epic_status = defaultdict(lambda: defaultdict(int))

    for item in ids:
        eid = item["id"]
        status = classify(item)
        epic = epic_from_id(eid)

        spec_refs = []
        if item.get("in_func_req"):
            spec_refs.append(item["in_func_req"])
        spec_refs.extend(item.get("in_specs", []))
        spec_refs.extend(item.get("in_traceability", []))
        # De-duplicate while preserving order.
        seen = set()
        spec_refs = [r for r in spec_refs if not (r in seen or seen.add(r))]

        rows.append({
            "id": eid,
            "status": status,
            "epic": epic,
            "spec_refs": spec_refs,
            "code_refs": item.get("in_code", []),
            "test_refs": item.get("in_tests", []),
        })

        totals[status] += 1
        by_epic[epic] += 1
        by_epic_status[epic][status] += 1

    total_ids = len(rows)

    out = {
        "schema_version": 2,
        "generated_at": date.today().isoformat(),
        "source_inventory": "docs/audits/_id_inventory_v3.json",
        "status_legend": STATUS_LEGEND,
        "totals": {
            "rows": total_ids,
            "by_status": {s: totals.get(s, 0) for s in STATUS_ORDER},
            "by_epic": dict(sorted(by_epic.items())),
            "by_epic_status": {
                epic: {s: by_epic_status[epic].get(s, 0) for s in STATUS_ORDER}
                for epic in sorted(by_epic_status.keys())
            },
        },
        "rows": rows,
    }

    OUT_JSON.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {OUT_JSON} ({total_ids} rows)", file=sys.stderr)

    # Markdown report
    md = []
    md.append("# FR Coverage Audit")
    md.append("")
    md.append(f"**Generated:** {date.today().isoformat()}  ")
    md.append(f"**Source inventory:** `{out['source_inventory']}`  ")
    md.append(f"**Total IDs scanned:** {total_ids}")
    md.append("")
    md.append("## Status legend")
    md.append("")
    md.append("| Status | Meaning |")
    md.append("|--------|---------|")
    for s in STATUS_ORDER:
        md.append(f"| `{s}` | {STATUS_LEGEND[s]} |")
    md.append("")
    md.append("## Summary")
    md.append("")
    md.append("| Status | Count | % |")
    md.append("|--------|------:|--:|")
    for s in STATUS_ORDER:
        c = totals.get(s, 0)
        pct = (c / total_ids * 100) if total_ids else 0.0
        md.append(f"| `{s}` | {c} | {pct:.1f} |")
    md.append(f"| **Total** | **{total_ids}** | **100.0** |")
    md.append("")
    md.append("## Coverage by epic")
    md.append("")
    md.append("| Epic | Total | COVERED | IMPL-NO-TEST | SPEC-ONLY | CODE-ONLY-no-spec |")
    md.append("|------|------:|--------:|-------------:|----------:|------------------:|")
    for epic in sorted(by_epic_status.keys()):
        counts = by_epic_status[epic]
        md.append(
            f"| {epic} | {by_epic[epic]} | "
            f"{counts.get('COVERED', 0)} | "
            f"{counts.get('IMPL-NO-TEST', 0)} | "
            f"{counts.get('SPEC-ONLY', 0)} | "
            f"{counts.get('CODE-ONLY-no-spec', 0)} |"
        )
    md.append("")

    def list_ids(status: str, title: str) -> None:
        items = [r for r in rows if r["status"] == status]
        md.append(f"## {title} ({len(items)})")
        md.append("")
        if not items:
            md.append("_None._")
            md.append("")
            return
        for r in items:
            md.append(f"- `{r['id']}`")
            if r["spec_refs"]:
                md.append(f"  - spec: {', '.join(r['spec_refs'][:3])}")
            if r["code_refs"]:
                md.append(f"  - code: {', '.join(r['code_refs'][:3])}")
            if r["test_refs"]:
                md.append(f"  - tests: {', '.join(r['test_refs'][:3])}")
        md.append("")

    list_ids("SPEC-ONLY", "Spec-only IDs (need implementation)")
    list_ids("IMPL-NO-TEST", "Implemented but untested IDs")
    list_ids("CODE-ONLY-no-spec", "Code-only IDs (missing spec/traceability)")

    OUT_MD.write_text("\n".join(md) + "\n", encoding="utf-8")
    print(f"Wrote {OUT_MD}", file=sys.stderr)

    return 0


if __name__ == "__main__":
    sys.exit(main())
