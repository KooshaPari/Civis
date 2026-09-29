"""Detect FR/NFR id-namespace collisions: an id defined by >1 spec file, or a
source tag whose id is never defined by the spec that file's header cites.

Run:  python docs/audits/_detect_id_collisions.py [--json OUT]
"""
from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

# A requirement id as used in source and matrixes.
ID_RE = re.compile(r"\b((?:FR|NFR)-[A-Z0-9]+(?:-[A-Z0-9]+)*-\d{3})\b")
# A line that looks like it *defines* the id (headline row or bold heading).
DEF_RE = re.compile(r"(?m)^\s*(?:\*\*)?((?:FR|NFR)-[A-Z0-9-]+-\d{3})(?:\*\*)?\s*(?:[:\-|]|\.\s|$)")

SPEC_DIRS = ("docs/specs", "docs/models", "docs")


def spec_files() -> list[Path]:
    out: set[Path] = set()
    for d in SPEC_DIRS:
        p = REPO / d
        if p.is_dir():
            out |= {f for f in p.rglob("*.md") if "audits" not in f.parts}
    return sorted(out)


def definitions() -> dict[str, set[str]]:
    """id -> {relative spec paths that define it}"""
    defs: dict[str, set[str]] = defaultdict(set)
    for f in spec_files():
        text = f.read_text(encoding="utf-8", errors="replace")
        for ident in set(DEF_RE.findall(text)):
            defs[ident].add(f.relative_to(REPO).as_posix())
    return defs


def source_tags() -> dict[str, list[str]]:
    """id -> [relative source files tagged with it]"""
    tags: dict[str, list[str]] = defaultdict(list)
    for f in sorted((REPO / "crates").rglob("*.rs")):
        text = f.read_text(encoding="utf-8", errors="replace")
        for ident in set(ID_RE.findall(text)):
            tags[ident].append(f.relative_to(REPO).as_posix())
    return tags


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", type=Path)
    args = ap.parse_args()

    defs = definitions()
    tags = source_tags()

    # 1. Ids defined by more than one spec file.
    collisions = {
        ident: sorted(paths) for ident, paths in defs.items() if len(paths) > 1
    }
    # 2. Source tags that resolve to no spec definition at all (phantom ids).
    phantom = {i: p for i, p in tags.items() if i not in defs}
    # 3. Source tags that resolve to more than one spec (ambiguous binding).
    ambiguous = {i: sorted(p) for i, p in tags.items() if len(defs.get(i, ())) > 1}

    report = {
        "defined_id_count": len(defs),
        "tagged_id_count": len(tags),
        "cross_spec_collisions": collisions,
        "phantom_source_ids": {k: sorted(v) for k, v in sorted(phantom.items())},
        "ambiguous_tag_bindings": {k: v for k, v in sorted(ambiguous.items())},
    }

    print(f"defined ids: {len(defs)}   tagged ids: {len(tags)}")
    print(f"cross-spec collisions: {len(collisions)}")
    for ident, paths in sorted(collisions.items()):
        print(f"  {ident} <- {paths}")
    print(f"phantom source ids: {len(phantom)}")
    for ident, files in sorted(phantom.items()):
        print(f"  {ident} <- {files[:3]}")
    print(f"ambiguous tag bindings: {len(ambiguous)}")
    for ident, paths in sorted(ambiguous.items()):
        print(f"  {ident} <- {paths}")

    if args.json:
        args.json.write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(f"wrote {args.json}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
