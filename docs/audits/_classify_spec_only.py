#!/usr/bin/env python3
"""Classify every SPEC-ONLY FR/NFR ID into one explicit deferral category.

Run order (the first script is required; the second supplies the status):

    python docs/audits/_gather_ids.py
    python scripts/traceability/gen-fr-audit.py
    python docs/audits/_classify_spec_only.py

Inputs:
  docs/audits/_id_inventory_v3.json  per-ID verified file:line provenance
  docs/audits/fr-matrix.json          status assigned by gen-fr-audit.py

Output:
  docs/audits/spec-only-deferrals.json  the ledger
  docs/audits/spec-only-deferrals.md    the human-readable ledger

Every SPEC-ONLY ID lands in exactly one category. Categories are a pure
function of the provenance that _gather_ids.py recorded, so re-running this
on unchanged inputs reproduces the ledger byte-for-byte.

  reporting-only        The ID is only ever named by a matrix, audit report,
                        status report, or tracker. Those documents restate
                        other requirements; they do not define new ones.
  stub-template         The ID's only "spec" is an auto-generated
                        docs/traceability/<id>/<id>-spec.md that was never
                        filled in. All 1221 of those files still say
                        "Status: SPEC-TEMPLATE" and keep the placeholder
                        text under every heading, so they scaffold a slot for
                        an ID rather than specifying it.
  synthetic-expansion   The ID's area has genuinely spec-backed siblings, but
                        this particular ID has no spec section of its own.
                        The numbering was expanded past what the spec defines.
  design-document       The ID's only genuine home is a design / direction
                        document. Those carry intent, not acceptance criteria.
  traceable-requirement The ID is defined in a real spec and that spec names a
                        concrete test file or check. The work is unimplemented
                        but fully specified.
  actionable            A real requirement in a real spec with no named test.
                        This is the hand-review list.

NOTE on honesty: two earlier revisions of this script were wrong and the
mistakes are recorded here so they are not reintroduced.

1. The design category used a loose regex that also matched
   docs/development-guide/ and any path containing "guide". That silently
   absorbed 12 IDs whose only spec home was a development guide. The rule is
   now a plain docs/design/ prefix match.

2. Every docs/traceability/<id>/<id>-spec.md was treated as a genuine spec.
   All 1221 are unfilled auto-generated templates marked
   "Status: SPEC-TEMPLATE" with placeholder text under every heading. That
   made 50 IDs look spec-backed when nothing had been written. is_stub_spec()
   now detects and excludes them.
"""

from __future__ import annotations

import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INVENTORY = ROOT / "docs" / "audits" / "_id_inventory_v3.json"
MATRIX = ROOT / "docs" / "audits" / "fr-matrix.json"
LEDGER_JSON = ROOT / "docs" / "audits" / "spec-only-deferrals.json"
LEDGER_MD = ROOT / "docs" / "audits" / "spec-only-deferrals.md"

# Paths that restate or summarize IDs rather than defining them.
REPORTING_PREFIXES = (
    "docs/audits/",
    "docs/traceability/",
    "docs/reports/",
    "docs/reference/FR_TRACKER",
    "docs/IMPLEMENTATION_STATUS",
)
REPORTING_FILES = {
    "docs/IMPLEMENTATION_STATUS.md",
    "docs/traceability/TRACEABILITY_MATRIX.md",
}

# Design / direction documents. Deliberately a narrow prefix match.
DESIGN_PREFIXES = ("docs/design/",)
DESIGN_EXACT = {"docs/development-guide/p-w1-kickoff.md"}

# A spec line that names a concrete test file.
TEST_REF_RE = re.compile(
    r"(tests?/[A-Za-z0-9_./-]+\.(?:rs|py|ts|tsx|js)|test_[A-Za-z0-9_]+\.py|"
    r"`[A-Za-z0-9_]*test[A-Za-z0-9_]*\.(?:rs|py)`)",
    re.I,
)

# ID grammar: FR/NFR - <AREA segments> - <digits> [ - suffix ]
ID_RE = re.compile(r"^(?:FR|NFR)-([A-Z0-9]+(?:-[A-Z0-9]+)*?)-(\d+)(?:-[A-Z0-9]+)*$")

CATEGORY_ORDER = (
    "actionable",
    "traceable-requirement",
    "synthetic-expansion",
    "design-document",
    "stub-template",
    "reporting-only",
)

CATEGORY_BLURB = {
    "reporting-only": "Named only by a matrix, audit, status report, or tracker.",
    "stub-template": "Only an unfilled auto-generated traceability template.",
    "synthetic-expansion": "Numbered inside an area that has real specs, but no spec section of its own.",
    "design-document": "Defined only in a design / direction document (intent, not acceptance criteria).",
    "traceable-requirement": "Real spec, and the spec names the test to write.",
    "actionable": "Real spec requirement with no implementation reference and no named test.",
}

# An auto-generated, never-filled traceability spec template.
STUB_SPEC_MARKER = "Status: SPEC-TEMPLATE"


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def path_of(ref) -> str:
    return ref.rsplit(":", 1)[0] if isinstance(ref, str) else ""


def split_id(iid: str) -> tuple[str, int]:
    """(area, numeric tail). FR-CIV-TECH-002 -> ('CIV-TECH', 2)."""
    m = ID_RE.match(iid)
    if not m:
        return (iid, -1)
    return (m.group(1), int(m.group(2)))


def spec_homes(row: dict) -> list[str]:
    out: list[str] = []
    for key in ("in_specs", "in_meta", "in_func_req", "in_traceability"):
        for ref in row.get(key) or []:
            p = path_of(ref)
            if p:
                out.append(p)
    return sorted(set(out))


def is_reporting(p: str) -> bool:
    return p in REPORTING_FILES or any(p.startswith(x) for x in REPORTING_PREFIXES)


_STUB_SPEC_CACHE: dict[str, bool] = {}


def is_stub_spec(p: str) -> bool:
    """True for an auto-generated, never-filled traceability spec template."""
    if p in _STUB_SPEC_CACHE:
        return _STUB_SPEC_CACHE[p]
    f = ROOT / p
    verdict = False
    if f.is_file():
        try:
            head = f.read_text(encoding="utf-8", errors="replace")[:2000]
        except OSError:
            head = ""
        verdict = STUB_SPEC_MARKER in head
    _STUB_SPEC_CACHE[p] = verdict
    return verdict


def is_design(p: str) -> bool:
    return p in DESIGN_EXACT or any(p.startswith(x) for x in DESIGN_PREFIXES)


def spec_names_a_test(path: str, iid: str) -> bool:
    """True if the spec, within 8 lines of the ID, names a concrete test."""
    f = ROOT / path
    if not f.is_file():
        return False
    try:
        lines = f.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return False
    for idx, line in enumerate(lines):
        if iid in line and TEST_REF_RE.search("\n".join(lines[idx : idx + 8])):
            return True
    return False


def classify(iid: str, row: dict, area_specs: dict[str, set[str]]) -> tuple[str, str]:
    homes = spec_homes(row)
    area, _ = split_id(iid)

    # An unfilled traceability template is a distinct kind of nothing: the ID
    # has a dedicated spec *slot* that was scaffolded and never written.
    stub_only = [p for p in homes if is_stub_spec(p)]
    if stub_only:
        return "stub-template", f"unfilled traceability template: {stub_only[0]}"

    genuine = [p for p in homes if not is_reporting(p)]

    if not genuine:
        return "reporting-only", "recorded only in matrix/audit documents"

    if area in area_specs and area_specs[area] and not any(
        p.startswith("docs/specs/") or p.endswith("-spec.md") for p in genuine
    ):
        return (
            "synthetic-expansion",
            f"area {area} has spec-backed siblings, but this id has no spec section",
        )

    if all(is_design(p) for p in genuine):
        return "design-document", f"only design/direction docs: {genuine[0]}"

    for p in genuine:
        if spec_names_a_test(p, iid):
            return "traceable-requirement", f"{p} names a concrete test for this id"

    return "actionable", f"genuine spec home: {genuine[0]}"


def write_markdown(ledger: dict, by_area: dict[str, Counter]) -> None:
    L: list[str] = []
    L.append("# SPEC-ONLY deferral ledger")
    L.append("")
    L.append("Generated by `docs/audits/_classify_spec_only.py`. Do not hand-edit.")
    L.append("Regenerate with:")
    L.append("")
    L.append("```")
    L.append("python docs/audits/_gather_ids.py")
    L.append("python scripts/traceability/gen-fr-audit.py")
    L.append("python docs/audits/_classify_spec_only.py")
    L.append("```")
    L.append("")
    L.append(f"Total SPEC-ONLY IDs: **{ledger['spec_only_total']}**")
    L.append("")
    L.append("| Category | Count | Meaning |")
    L.append("|---|---:|---|")
    for cat in CATEGORY_ORDER:
        n = ledger["counts"].get(cat, 0)
        if n:
            L.append(f"| `{cat}` | {n} | {CATEGORY_BLURB[cat]} |")
    L.append("")
    L.append(
        "Deferred = real but not a Rust/behavioral test target: it names no code "
        "artifact, or its only home is a document that restates other "
        "requirements. Deferred does **not** mean implemented."
    )
    L.append("")
    L.append("## By area")
    L.append("")
    L.append("| Area | Total | " + " | ".join(f"`{c}`" for c in CATEGORY_ORDER) + " |")
    L.append("|---|---:|" + "|".join("---:" for _ in CATEGORY_ORDER) + "|")
    for area in sorted(by_area, key=lambda a: (-by_area[a]["actionable"], a)):
        c = by_area[area]
        cells = " | ".join(str(c.get(cat, 0)) for cat in CATEGORY_ORDER)
        L.append(f"| `{area}` | {sum(c.values())} | {cells} |")
    L.append("")
    for cat in CATEGORY_ORDER:
        items = ledger["ids"].get(cat) or []
        if not items:
            continue
        L.append(f"## `{cat}` ({len(items)})")
        L.append("")
        L.append(f"{CATEGORY_BLURB[cat]}")
        L.append("")
        L.append("| ID | Why |")
        L.append("|---|---|")
        for it in items:
            L.append(f"| `{it['id']}` | {it['reason']} |")
        L.append("")
    LEDGER_MD.write_text("\n".join(L) + "\n", encoding="utf-8")


def main() -> int:
    for p in (INVENTORY, MATRIX):
        if not p.is_file():
            print(f"missing input: {p.relative_to(ROOT)}", file=sys.stderr)
            print("run _gather_ids.py and gen-fr-audit.py first", file=sys.stderr)
            return 1

    matrix = load(MATRIX)
    inventory = load(INVENTORY)
    status = {r["id"]: r["status"] for r in matrix.get("rows", [])}

    raw = inventory.get("ids") or inventory.get("rows") or inventory
    rows = {r["id"]: r for r in raw if isinstance(r, dict) and "id" in r}

    spec_only = sorted(i for i, s in status.items() if s == "SPEC-ONLY" and i in rows)

    area_specs: dict[str, set[str]] = defaultdict(set)
    for iid, row in rows.items():
        area, _ = split_id(iid)
        for p in spec_homes(row):
            if is_reporting(p) or is_stub_spec(p):
                continue
            if p.startswith("docs/specs/") or p.endswith("-spec.md"):
                area_specs[area].add(p)

    buckets: dict[str, list[dict[str, str]]] = defaultdict(list)
    by_area: dict[str, Counter] = defaultdict(Counter)
    for iid in spec_only:
        cat, reason = classify(iid, rows[iid], area_specs)
        buckets[cat].append({"id": iid, "reason": reason})
        by_area[split_id(iid)[0]][cat] += 1

    ordered = {c: sorted(buckets.get(c, []), key=lambda x: x["id"]) for c in CATEGORY_ORDER}
    ledger = {
        "generated_by": "docs/audits/_classify_spec_only.py",
        "spec_only_total": len(spec_only),
        "category_blurb": CATEGORY_BLURB,
        "counts": {c: len(ordered[c]) for c in CATEGORY_ORDER if ordered[c]},
        "ids": ordered,
    }
    LEDGER_JSON.write_text(json.dumps(ledger, indent=2) + "\n", encoding="utf-8")
    write_markdown(ledger, by_area)

    print(f"SPEC-ONLY total: {len(spec_only)}")
    for cat in CATEGORY_ORDER:
        if ledger["counts"].get(cat):
            print(f"  {cat:24s} {ledger['counts'][cat]:4d}")
    print(f"\nWrote {LEDGER_JSON.relative_to(ROOT)}")
    print(f"Wrote {LEDGER_MD.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
