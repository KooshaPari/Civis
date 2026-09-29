"""Verify FR/NFR traceability bindings honestly.

Defects detected:

1. EPIC-PAIRING-FALSE - source claims an epic for an id via an
   `(FR-XXX-NNN, CIV-NNNN)` pairing, but no spec belonging to CIV-NNNN
   defines that id. This is the exact shape of the FR-UX-001..005 render
   tags, which bound to USER_SPEC.md requirements they do not implement.

2. UNDEFINED-TAG       - a source file tags an id that no spec file under
   docs/specs or docs/models mentions at all (a fabricated id).

3. MATRIX-MISBINDING   - a matrix row cites `path:line` but that line does
   not contain the row's id.

Run:  python docs/audits/_verify_bindings.py [--json OUT]
"""
from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

ID_RE = re.compile(r"\b((?:FR|NFR)-[A-Z0-9]+(?:-[A-Z0-9]+)*-\d{3})\b")
# `(FR-UX-001, CIV-0300)` or `FR-UX-001 (CIV-0300)` - an id claiming an epic.
PAIR_RE = re.compile(
    r"\(?\b((?:FR|NFR)-[A-Z0-9]+(?:-[A-Z0-9]+)*-\d{3})\b\s*,?\s*\(?\b(CIV-\d{4})\b"
)
EPIC_RE = re.compile(r"CIV-\d{4}")

# Authoritative spec roots, matching what the matrix generator itself scans.
#
# Deliberately EXCLUDED because they are outputs or scaffolding rather than
# requirements:
#   docs/audits/**        - audit output
#   docs/traceability/**  - 1,221 unfilled SPEC-TEMPLATE files; an id appearing
#                           only here has no requirement text behind it
#   docs/fragemented/**   - duplicated copies of real specs
#   docs/reference/**, docs/research/** - analysis, not normative requirements
SPEC_ROOTS = ("docs/specs", "docs/models", "docs/design", "agileplus-specs")


def spec_files() -> list[Path]:
    out: set[Path] = set()
    for d in SPEC_ROOTS:
        root = REPO / d
        if root.is_dir():
            out |= {
                f
                for f in root.rglob("*.md")
                if "fragemented" not in f.parts and "audits" not in f.parts
            }
    return sorted(out)


def build_definitions() -> tuple[dict[str, set[str]], dict[str, set[str]]]:
    """(id -> defining spec paths, epic -> ids that epic's specs define)."""
    by_id: dict[str, set[str]] = defaultdict(set)
    by_epic: dict[str, set[str]] = defaultdict(set)
    for f in spec_files():
        rel = f.relative_to(REPO).as_posix()
        epics = set(EPIC_RE.findall(f.name))
        ids = set(ID_RE.findall(f.read_text(encoding="utf-8", errors="replace")))
        for ident in ids:
            by_id[ident].add(rel)
            for epic in epics:
                by_epic[epic].add(ident)
    return by_id, by_epic


def check_matrix(by_id: dict[str, set[str]]) -> list[dict]:
    matrix = REPO / "docs/audits/fr-matrix.json"
    if not matrix.exists():
        return []
    rows = json.loads(matrix.read_text(encoding="utf-8")).get("rows", [])
    bad: list[dict] = []
    cache: dict[str, list[str]] = {}

    def resolve(spec: str) -> Path | None:
        """Spec refs are sometimes repo-relative, sometimes bare filenames."""
        direct = REPO / spec
        if direct.exists():
            return direct
        name = spec.rsplit("/", 1)[-1]
        hits = [
            p
            for p in REPO.rglob(name)
            if ".git" not in p.parts and "fragemented" not in p.parts
        ]
        return hits[0] if len(hits) == 1 else None

    for row in rows:
        ident = row.get("id")
        for ref in row.get("spec_refs") or []:
            path, _, lineno = ref.rpartition(":")
            if not path or not lineno.isdigit():
                # File-level ref: record once, and only if the file is missing.
                f = resolve(ref)
                if f is None:
                    bad.append({"id": ident, "ref": ref, "why": "spec file not found"})
                continue
            f = REPO / path
            if not f.exists():
                bad.append({"id": ident, "ref": ref, "why": "spec file missing"})
                continue
            key = path
            if key not in cache:
                cache[key] = f.read_text(encoding="utf-8", errors="replace").splitlines()
            lines = cache[key]
            n = int(lineno)
            if not 1 <= n <= len(lines):
                bad.append({"id": ident, "ref": ref, "why": "line out of range"})
            elif ident not in lines[n - 1]:
                bad.append({"id": ident, "ref": ref, "why": "line does not mention id"})
    return bad


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", type=Path)
    args = ap.parse_args()

    by_id, by_epic = build_definitions()
    false_pairing: dict[str, list[str]] = defaultdict(list)
    undefined: dict[str, list[str]] = defaultdict(list)

    for f in sorted((REPO / "crates").rglob("*.rs")):
        rel = f.relative_to(REPO).as_posix()
        text = f.read_text(encoding="utf-8", errors="replace")
        for ident, epic in set(PAIR_RE.findall(text)):
            if ident not in by_epic.get(epic, set()):
                false_pairing[f"{ident} -> {epic}"].append(rel)
        for ident in set(ID_RE.findall(text)):
            if ident not in by_id:
                undefined[ident].append(rel)

    misbindings = check_matrix(by_id)
    report = {
        "defined_ids": len(by_id),
        "epic_pairing_false": {k: sorted(v) for k, v in sorted(false_pairing.items())},
        "undefined_source_ids": {k: sorted(v) for k, v in sorted(undefined.items())},
        "matrix_misbindings": misbindings,
    }

    print(f"defined ids: {len(by_id)}")
    print(f"FALSE EPIC PAIRINGS: {len(false_pairing)}")
    for key, files in sorted(false_pairing.items()):
        print(f"  {key} <- {files}")
    print(f"UNDEFINED source ids: {len(undefined)}")
    for ident, files in sorted(undefined.items()):
        print(f"  {ident} <- {files[:4]}")
    print(f"MATRIX MISBINDINGS: {len(misbindings)}")
    for row in misbindings[:15]:
        print(f"  {row}")

    if args.json:
        args.json.write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(f"wrote {args.json}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
