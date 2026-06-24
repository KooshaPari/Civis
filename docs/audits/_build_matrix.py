#!/usr/bin/env python3
"""Build docs/audits/fr-matrix-2026-06-10.md and docs/audits/fr-matrix.json
from docs/audits/_id_inventory_v3.json.

Matrix columns:
  id | spec | code refs | test refs | status

Status (per task):
  COVERED          : spec + code + test
  IMPL-NO-TEST     : spec + code, no test
  SPEC-ONLY        : spec, no code
  CODE-ONLY-no-spec: code, no spec
"""
import json
import os
import re
from collections import Counter, defaultdict
from datetime import date
from pathlib import Path

WORK = Path(os.environ.get("CIVIS_AUDIT_WORK", ".")).resolve()
GENERATED_AT = os.environ.get("CIVIS_AUDIT_DATE", date.today().isoformat())
INVENTORY = WORK / "docs/audits/_id_inventory_v3.json"
MD_OUT = WORK / f"docs/audits/fr-matrix-{GENERATED_AT}.md"
JSON_OUT = WORK / "docs/audits/fr-matrix.json"

data = json.loads(INVENTORY.read_text(encoding="utf-8"))
ids = data["ids"]


def has_spec(e):
    return bool(e["in_specs"]) or bool(e["in_meta"]) or bool(e["in_func_req"])


def has_code(e):
    return bool(e["in_code"])


def has_test(e):
    return bool(e["in_tests"])


def derive_status(e):
    hspec, hc, ht = has_spec(e), has_code(e), has_test(e)
    if hspec and hc and ht:
        return "COVERED"
    if hspec and hc and not ht:
        return "IMPL-NO-TEST"
    if hspec and not hc:
        return "SPEC-ONLY"
    if hc and not hspec:
        return "CODE-ONLY-no-spec"
    # Traceability-only (no spec source, no code, no test) -> treat as SPEC-ONLY
    return "SPEC-ONLY"


def primary_spec_file(e):
    if e["in_specs"]:
        specs = e["in_specs"]
        ap = [s for s in specs if s.startswith("agileplus-specs/")]
        if ap:
            return ap[0].rsplit(":", 1)[0]
        return specs[0].rsplit(":", 1)[0]
    if e["in_meta"]:
        return e["in_meta"][0].rsplit(":", 1)[0]
    if e["in_func_req"]:
        return e["in_func_req"]
    spec_matrices = {
        "docs/traceability/TRACEABILITY_MATRIX.md",
        "docs/traceability/fr-3d-matrix.md",
        "docs/traceability/full-traceability-matrix.md",
    }
    for t in e["in_traceability"]:
        if t.split(":", 1)[0] in spec_matrices:
            return t.rsplit(":", 1)[0]
    if e["in_traceability"]:
        return e["in_traceability"][0].rsplit(":", 1)[0]
    return "NONE"


def epic_prefix(eid):
    """Return the FR-... epic prefix (e.g. FR-CIV-LAWS, FR-CORE)."""
    parts = eid.split("-")
    if not parts:
        return "OTHER"
    head = parts[:-1]
    # Drop trailing numeric token(s) (the FR number segment)
    while head and re.search(r"\d", head[-1]):
        head = head[:-1]
    if not head:
        return parts[0]
    return "-".join(head)


def fmt_refs(refs, limit=3):
    if not refs:
        return "NONE"
    if len(refs) <= limit:
        return ", ".join(refs)
    return ", ".join(refs[:limit]) + f", … (+{len(refs) - limit} more)"


# Build rows
rows = []
for e in ids:
    eid = e["id"]
    rows.append({
        "id": eid,
        "epic": epic_prefix(eid),
        "spec": primary_spec_file(e),
        "code_refs": e["in_code"],
        "test_refs": e["in_tests"],
        "status": derive_status(e),
    })

# Summary
status_counts = Counter(r["status"] for r in rows)
epic_counts = Counter(r["epic"] for r in rows)
epic_status = defaultdict(Counter)
for r in rows:
    epic_status[r["epic"]][r["status"]] += 1

STATUS_ORDER = ["COVERED", "IMPL-NO-TEST", "SPEC-ONLY", "CODE-ONLY-no-spec"]


def esc(x):
    return x.replace("|", "\\|")


# ---- Markdown deliverable ----
md = []
md.append(f"# FR / NFR ↔ Code ↔ Test Matrix — {GENERATED_AT}")
md.append("")
md.append(f"**Generated:** {GENERATED_AT}  ")
md.append(f"**Total unique FR/NFR IDs:** {len(rows)}  ")
md.append("**Source-of-truth inventory:** `docs/audits/_id_inventory_v3.json`  ")
md.append("**Generator:** `docs/audits/_gather_ids.py`  ")
md.append("**Machine version:** `docs/audits/fr-matrix.json`")
md.append("")
md.append("This matrix is a full bidirectional audit: every `FR-*` and `NFR-*` identifier found anywhere in")
md.append("the repository (specs, `FUNCTIONAL_REQUIREMENTS.md`, traceability matrices, source code, tests)")
md.append("is mapped to its spec source, implementing code, and covering test. NONE rows are the deliverable —")
md.append("they flag what's missing, not what's implemented.")
md.append("")
md.append("## Status legend")
md.append("")
md.append("| Status | Meaning |")
md.append("|--------|---------|")
md.append("| `COVERED`          | spec + code + test all present |")
md.append("| `IMPL-NO-TEST`     | spec + code present, no test reference |")
md.append("| `SPEC-ONLY`        | spec only, no implementing code found in this scan |")
md.append("| `CODE-ONLY-no-spec`| code only, no spec / traceability reference |")
md.append("")
md.append("## Notes")
md.append("")
md.append("- `spec` column = first spec-authority file that mentions the ID. Preferred order: ")
md.append("  `agileplus-specs/*/spec.md` → `agileplus-specs/*/plan.md` → `FUNCTIONAL_REQUIREMENTS.md` → ")
md.append("  `PRD.md` → `docs/FR.md` / `docs/FR_DETAILED.md` → ")
md.append("  `docs/reference/non-functional-requirements.md` → ")
md.append("  `docs/traceability/TRACEABILITY_MATRIX.md` / `fr-3d-matrix.md` / `full-traceability-matrix.md` → ")
md.append("  any other `docs/traceability/*.md`.")
md.append("- `code refs` / `test refs` = up to 3 `file:line` entries shown; full list in JSON.")
md.append("- Self-referential files (`docs/audits/*`, all `docs/**/fragemented/*`, ")
md.append("  `docs/upstream-governance/*`, PR body / diff artifacts) are EXCLUDED from scanning so we don't")
md.append("  cite ourselves or pull in third-party project docs.")
md.append("- Range markers like `FR-CIV-LIFE-010..016` are normalized to their start (`FR-CIV-LIFE-010`).")
md.append("")
md.append("## Summary — by status")
md.append("")
md.append("| Status | Count |")
md.append("|--------|-------|")
for s in STATUS_ORDER:
    md.append(f"| `{s}` | {status_counts.get(s, 0)} |")
md.append(f"| **TOTAL** | **{len(rows)}** |")
md.append("")
md.append("## Summary — by epic")
md.append("")
md.append("| Epic prefix | COVERED | IMPL-NO-TEST | SPEC-ONLY | CODE-ONLY-no-spec | Total |")
md.append("|-------------|--------:|-------------:|----------:|------------------:|------:|")
for ep in sorted(epic_counts.keys(), key=lambda p: (-epic_counts[p], p)):
    cnts = epic_status[ep]
    md.append(
        f"| `{ep}` | {cnts.get('COVERED', 0)} | {cnts.get('IMPL-NO-TEST', 0)} | "
        f"{cnts.get('SPEC-ONLY', 0)} | {cnts.get('CODE-ONLY-no-spec', 0)} | "
        f"{epic_counts[ep]} |"
    )
md.append("")
md.append("## Full row table (alphabetical by ID)")
md.append("")
md.append("| ID | spec | code refs | test refs | status |")
md.append("|----|------|-----------|-----------|--------|")
for r in rows:
    md.append(
        f"| `{esc(r['id'])}` | `{esc(r['spec'])}` | {esc(fmt_refs(r['code_refs']))} | "
        f"{esc(fmt_refs(r['test_refs']))} | `{r['status']}` |"
    )
md.append("")
MD_OUT.write_text("\n".join(md) + "\n", encoding="utf-8")

# ---- JSON deliverable ----
json_doc = {
    "schema_version": 1,
    "generated_at": GENERATED_AT,
    "source_inventory": "docs/audits/_id_inventory_v3.json",
    "status_legend": {
        "COVERED": "spec + code + test all present",
        "IMPL-NO-TEST": "spec + code present, no test reference",
        "SPEC-ONLY": "spec only, no implementing code found",
        "CODE-ONLY-no-spec": "code only, no spec / traceability reference",
    },
    "totals": {
        "rows": len(rows),
        "by_status": {s: status_counts.get(s, 0) for s in STATUS_ORDER},
        "by_epic": dict(epic_counts),
        "by_epic_status": {
            ep: {s: epic_status[ep].get(s, 0) for s in STATUS_ORDER}
            for ep in sorted(epic_counts.keys(), key=lambda p: (-epic_counts[p], p))
        },
    },
    "rows": rows,
}
JSON_OUT.write_text(json.dumps(json_doc, indent=2) + "\n", encoding="utf-8")

print(f"Wrote {MD_OUT} ({len(rows)} rows)")
print(f"Wrote {JSON_OUT} ({len(rows)} rows)")
print("By status:", dict(status_counts))
