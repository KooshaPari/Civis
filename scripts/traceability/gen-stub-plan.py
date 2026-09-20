"""Produce a per-stub work plan from the FR matrix.

Each row gets a markdown checklist of:
  * the FR ID and its current bucket
  * the implementing crate (from the spec/ADR/intent traceability docs)
  * the placeholder test file to convert
  * suggested entry points for the agent (spec, ADR, intent, plan)

Output: docs/audits/stub-fill-plan.md
"""
from __future__ import annotations
import json
import re
from collections import defaultdict
from pathlib import Path

ROOT = Path(r"C:\Users\koosh\Civis-clone")
MATRIX = ROOT / "docs" / "audits" / "fr-matrix.json"
INVENTORY = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
OUT = ROOT / "docs" / "audits" / "stub-fill-plan.md"


def first_crate(spec_refs):
    for r in spec_refs:
        if "crates/" in r:
            return r.split(":")[0]
    return None


def spec_path_for(eid: str) -> str | None:
    p = ROOT / "docs" / "traceability" / eid.lower()
    if p.is_dir():
        return f"docs/traceability/{eid.lower()}/"
    # Try without NFR- prefix
    bare = eid.split("-", 1)[1] if "-" in eid else eid
    p2 = ROOT / "docs" / "traceability" / bare.lower()
    if p2.is_dir():
        return f"docs/traceability/{bare.lower()}/"
    return None


def main():
    matrix = json.loads(MATRIX.read_text(encoding="utf-8"))
    inv_data = json.loads(INVENTORY.read_text(encoding="utf-8"))
    inv = {r["id"]: r for r in inv_data["ids"]}

    stubs = [r for r in matrix["rows"] if r["status"] == "STUB-TEST-ONLY"]
    stubs.sort(key=lambda r: (r["epic"], r["id"]))

    # group by epic
    by_epic = defaultdict(list)
    for r in stubs:
        by_epic[r["epic"]].append(r)

    lines = []
    lines.append("# Stub-test fill work plan")
    lines.append("")
    lines.append(f"Total stub IDs to convert: **{len(stubs)}** across "
                 f"**{len(by_epic)}** epics.")
    lines.append("")
    lines.append("Each section is one epic. Within an epic, each stub gets a "
                 "checklist the agent follows:")
    lines.append("")
    lines.append("  1. Read the FR spec + intent + ADR")
    lines.append("  2. Locate the implementing crate (or write the impl)")
    lines.append("  3. Replace the placeholder test body with real FR assertions")
    lines.append("  4. Remove the `Stub: TDD-red` marker line")
    lines.append("  5. Run `cargo test -p <crate>` and confirm green")
    lines.append("  6. Commit `test(<crate>): real assertions for FR-XYZ-NNN`")
    lines.append("")
    lines.append("## Epic summary")
    lines.append("")
    lines.append("| Epic | Stubs |")
    lines.append("|------|------:|")
    for epic in sorted(by_epic.keys()):
        lines.append(f"| {epic} | {len(by_epic[epic])} |")
    lines.append("")

    for epic in sorted(by_epic.keys()):
        lines.append(f"## {epic} ({len(by_epic[epic])})")
        lines.append("")
        for r in by_epic[epic]:
            eid = r["id"]
            crate = first_crate(r.get("code_refs", []) + r.get("spec_refs", []))
            spec_dir = spec_path_for(eid)
            stub_files = [t.split(":")[0] for t in inv[eid].get("in_stub_tests", [])]
            stub_files = sorted(set(stub_files))
            lines.append(f"### {eid}")
            lines.append("")
            lines.append(f"- crate: `{crate or 'UNKNOWN — locate from spec'}`")
            lines.append(f"- spec dir: `{spec_dir or 'UNKNOWN'}`")
            lines.append("- stub file(s):")
            for s in stub_files:
                lines.append(f"  - `{s}`")
            lines.append("")

    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Wrote {OUT}")
    print(f"Total stubs: {len(stubs)}")
    print(f"Total epics: {len(by_epic)}")


if __name__ == "__main__":
    main()
