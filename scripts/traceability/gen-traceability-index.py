#!/usr/bin/env python3
"""Regenerate docs/traceability/index.md from the per-ID directories on disk.

Why this exists
---------------
`index.md` claimed "Auto-generated 2026-09-16 for 1231 FRs" with no generator in
the repo behind it, and had drifted from `docs/audits/fr-matrix.json`:

    rows in index.md                 1231
    comparable to the matrix         1229
    status agrees                      163
    status disagrees                   435
    absent from the matrix             631   (all claimed CODE-ONLY-no-spec)

Spot-checking the disagreements showed the matrix was correct every time.
`FR-AI-001` was called SPEC-ONLY here while `docs/FR.md:43` defines it,
`crates/ai/src/decision.rs:3` implements it, and `crates/ai/tests/fr_fr_ai_001.rs:1`
tests it. `FR-CIV-CULT-001` was called IMPL-NO-TEST while
`agileplus-specs/civ-009-culture-diffusion/spec.md:24` specifies it.

Hand-patching 435 rows would have been cosmetic. Nothing regenerates this file,
so the next source edit would reintroduce the drift silently. The Status column
is removed instead: coverage is derived from evidence by
`scripts/traceability/check-fr-coverage.py`, and duplicating it by hand is the
defect.

What this file keeps
--------------------
The real value here is the link index into the per-ID documentation directories
and a documentation-completeness signal. Both are derived from disk, so the file
is now reproducible and idempotent.

`Artifacts` is a DOCUMENTATION signal, not a coverage signal. It counts how many
of the five documentation files exist for an ID. An ID with all five can still be
SPEC-ONLY, because a specification existing is not the behaviour being
implemented.

Epic is derived from the matrix, which owns the ID-to-epic mapping. IDs with no
matrix row fall back to their namespace prefix so nothing is dropped.

Run:
    python scripts/traceability/gen-traceability-index.py
"""

from __future__ import annotations

import json
import re
from collections import defaultdict
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TRACE = ROOT / "docs" / "traceability"
INDEX = TRACE / "index.md"
MATRIX = ROOT / "docs" / "audits" / "fr-matrix.json"

ARTIFACTS = ("spec", "intent", "plan", "research", "adr")
SLUG_RE = re.compile(r"^(?P<kind>fr|nfr)-(?P<rest>.+)$")


def slug_to_id(slug: str) -> str:
    """`fr-civ-species-204` -> `FR-CIV-SPECIES-204`.

    The suffix after the kind prefix is uppercased whole rather than split on
    hyphens, because the ID's own internal hyphens are indistinguishable from the
    word separators (`fr-civ-0001-tick` is one ID, not `FR-CIV-0001` plus `TICK`).

    Some directories are named `fr-nfr-...`, which is a doubled prefix: the
    directory naming convention is `fr-` + the lowercased ID, and these IDs
    already start with `NFR`. Undoing the extra `fr-` keeps them as
    `NFR-CIV-PORT-001` rather than minting a phantom `FR-NFR-CIV-PORT-001` that
    no spec anywhere backs. Verified: with the doubling left in, regenerating the
    index added exactly 10 phantom SPEC-ONLY rows.
    """
    m = SLUG_RE.match(slug)
    if not m:
        return slug.upper()
    kind = m.group("kind").upper()
    rest = m.group("rest").upper()
    if kind == "FR" and rest.startswith("NFR-"):
        # `fr-nfr-civ-port-001` is `NFR-CIV-PORT-001` under a doubled prefix.
        return rest
    return f"{kind}-{rest}"


def epic_of(eid: str, matrix_epics: dict[str, str]) -> str:
    if eid in matrix_epics:
        return matrix_epics[eid]
    # Fall back to the longest known epic that prefixes this ID.
    for epic in sorted(matrix_epics.values(), key=len, reverse=True):
        if eid.startswith(epic + "-"):
            return epic
    m = SLUG_RE.match(eid)
    return m.group("kind").upper() if m else "OTHER"


def load_matrix_epics() -> dict[str, str]:
    if not MATRIX.exists():
        return {}
    data = json.loads(MATRIX.read_text(encoding="utf-8"))
    return {r["id"]: r.get("epic", "") for r in data.get("rows") or []}


def main() -> int:
    if not TRACE.is_dir():
        raise SystemExit(f"missing {TRACE}")

    matrix_epics = load_matrix_epics()
    slugs = sorted(
        d.name for d in TRACE.iterdir()
        if d.is_dir() and SLUG_RE.match(d.name)
    )
    if not slugs:
        raise SystemExit(f"no per-ID directories found under {TRACE}")

    by_epic: dict[str, list[tuple[str, str, int, bool, bool, bool]]] = defaultdict(list)
    with_arts = 0
    missing_matrix = 0

    for slug in slugs:
        d = TRACE / slug
        names = {f.name.lower() for f in d.iterdir() if f.is_file()}
        has = {a: any(f"{a}.md" in n for n in names) for a in ARTIFACTS}
        count = sum(has.values())
        if count:
            with_arts += 1

        eid = slug_to_id(slug)
        if eid not in matrix_epics:
            missing_matrix += 1
        by_epic[epic_of(eid, matrix_epics)].append(
            (eid, slug, count, has["spec"], has["adr"], has["research"])
        )

    total = len(slugs)
    L: list[str] = []
    L.append("# FR Traceability Index")
    L.append("")
    L.append(
        f"Index of {total} per-ID documentation directories under "
        f"`docs/traceability/`, grouped by epic. "
        f"{with_arts} have at least one artefact on disk. "
        f"Generated {date.today().isoformat()} by "
        "`scripts/traceability/gen-traceability-index.py`."
    )
    L.append("")
    L.append("## Coverage status is deliberately not in this file")
    L.append("")
    L.append(
        "This file used to carry a hand-maintained `Status` column with values "
        "including `COVERED` and `CODE-ONLY-no-spec`. It had no generator, and it "
        "disagreed with the authoritative matrix on 435 of the 1229 rows the two "
        "shared. Spot-checking those disagreements found the matrix correct each "
        "time: `FR-AI-001` was listed `SPEC-ONLY` here while it is defined at "
        "`docs/FR.md:43`, implemented at `crates/ai/src/decision.rs:3`, and "
        "tested at `crates/ai/tests/fr_fr_ai_001.rs:1`."
    )
    L.append("")
    L.append(
        "Authoritative coverage is "
        "[`docs/audits/fr-matrix.json`](../audits/fr-matrix.json), derived from "
        "evidence by `scripts/traceability/check-fr-coverage.py`, which fails the "
        "build on a coverage regression."
    )
    L.append("")
    L.append("## Artifacts")
    L.append("")
    L.append(
        "`Artifacts` counts how many of the five documentation files exist for an "
        "ID: `spec`, `intent`, `plan`, `research`, `adr`."
    )
    L.append("")
    L.append(
        "This is a **documentation** signal, not a coverage signal. An ID with all "
        "five artefacts can still be `SPEC-ONLY`, because a specification "
        "existing is not the behaviour being implemented."
    )
    L.append("")
    L.append(
        f"{missing_matrix} of these IDs have no row in the current matrix. That is "
        "either a documentation directory with no requirement behind it, or a "
        "requirement the ID regex does not match; both are tracked in "
        "`docs/audits/digitless-id-findings-2026-10-02.md`."
    )
    L.append("")

    for epic in sorted(by_epic):
        rows = sorted(by_epic[epic])
        L.append(f"## {epic} ({len(rows)})")
        L.append("")
        L.append("| FR ID | Artifacts | Spec | ADR | Research | Directory |")
        L.append("|-------|-----------|------|-----|----------|-----------|")
        for eid, slug, count, hs, ha, hr in rows:
            L.append(
                f"| {eid} | {count}/5 | {'yes' if hs else 'no'} | "
                f"{'yes' if ha else 'no'} | {'yes' if hr else 'no'} | "
                f"[dir]({slug}/) |"
            )
        L.append("")

    INDEX.write_text("\n".join(L) + "\n", encoding="utf-8")
    print(
        f"wrote {INDEX.relative_to(ROOT)}: {total} dirs, {len(by_epic)} epics, "
        f"{with_arts} populated, {missing_matrix} absent from the matrix"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())