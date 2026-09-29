"""Detect "container-only" FR bindings.

A traceability tag like `// FR-CIV-ASSET-003` sitting directly above a bare
`struct` / `enum` / `const` asserts that the requirement is implemented there.
For most requirements that is false: a struct that merely holds fields named
after a requirement is a container, not an implementation. The spec text for
`FR-SOC-INTG-001` asks that "social module outputs couple correctly to
insurgency"; a `pub struct WorldState { tick, population, .. }` does not do
that, yet the tag claimed it.

This is not proof of non-coverage on its own: a struct can legitimately be the
named artifact of a data-shape requirement, and a tagged struct may be
implemented transitively. So this tool only *flags* candidates and prints the
requirement sentence, leaving judgement to a human. It is a triage aid, not a
gate.

Usage:  python docs/audits/_detect_container_bindings.py [--json OUT] [--id ID]
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SOURCE_DIRS = ("crates", "clients")

ID_RE = re.compile(r"\b((?:FR|NFR)-[A-Z0-9]+(?:-[A-Z0-9]+)*-\d{3}[A-Z0-9-]*)\b")

# A whole-line comment that is nothing but a list of requirement ids.
TAG_LINE_RE = re.compile(r"^\s*///?\s*((?:(?:FR|NFR)-[A-Z0-9][A-Z0-9-]*)(?:[,\s]+(?:(?:FR|NFR)-[A-Z0-9][A-Z0-9-]*))*)\s*$")
DECL_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?(?:unsafe\s+)?"
    r"(fn|struct|enum|trait|type|const|static|impl|mod|macro_rules!)\b"
)
NON_CODE_PREFIX = ("#[", "///", "//!", "//")

# A tag block is contiguous comment lines immediately above one declaration.
DATA_ITEM_KINDS = {"struct", "enum", "type", "const", "static"}


def tracked_rs_files() -> list[Path]:
    out = subprocess.run(
        ["git", "ls-files", *SOURCE_DIRS],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split()
    return [REPO / f for f in out if f.endswith(".rs")]


def find_tag_blocks() -> list[dict]:
    """Yield {file, line, ids, kind, name} for every tag block above an item."""
    blocks: list[dict] = []
    for path in tracked_rs_files():
        try:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError:
            continue
        pending: list[tuple[int, list[str]]] = []  # (line_no, ids)
        for i, line in enumerate(lines):
            m = TAG_LINE_RE.match(line)
            if m:
                pending.append((i + 1, ID_RE.findall(m.group(1))))
                continue
            if not line.strip():
                continue  # blank lines do not break a tag block
            if line.lstrip().startswith(NON_CODE_PREFIX):
                continue  # attributes, doc prose, other comments
            d = DECL_RE.match(line)
            if d:
                kind = d.group(1)
                if pending:
                    name = ""
                    nm = re.search(r"\b(?:struct|enum|trait|type|mod|const|static|fn)\s+([A-Za-z_][A-Za-z0-9_]*)", line)
                    if nm:
                        name = nm.group(1)
                    for ln, ids in pending:
                        blocks.append(
                            {
                                "file": path.relative_to(REPO).as_posix(),
                                "line": ln,
                                "ids": ids,
                                "kind": kind,
                                "name": name,
                            }
                        )
                    pending = []
                else:
                    pending = []
                continue
            pending = []
    return blocks


def spec_files() -> list[Path]:
    out: set[Path] = set()
    for d in ("docs/specs", "docs/models", "docs/design", "agileplus-specs"):
        root = REPO / d
        if root.is_dir():
            out |= {f for f in root.rglob("*.md") if "fragemented" not in f.parts}
    return sorted(out)


def load_spec_texts() -> dict[str, str]:
    """Map requirement id -> the sentence that defines it.

    Prefers a bolded/heading-style definition, else the first line that looks
    like prose rather than a table row.
    """
    texts: dict[str, str] = {}
    for f in spec_files():
        try:
            body = f.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for line in body.splitlines():
            for ident in ID_RE.findall(line):
                if ident in texts:
                    continue
                s = line.strip()
                s = re.sub(r"^[#>*\-|\s]+", "", s)
                s = re.sub(r"^\|?\s*" + re.escape(ident) + r"\s*\|?", "", s).strip(" |")
                if len(s) < 12 or s.startswith("|") or s.isdigit():
                    continue
                texts[ident] = s[:220]
    return texts


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", type=Path)
    ap.add_argument("--id", action="append", dest="only")
    args = ap.parse_args()

    blocks = find_tag_blocks()
    texts = load_spec_texts()

    flagged = []
    for b in blocks:
        if b["kind"] not in DATA_ITEM_KINDS:
            continue
        for ident in b["ids"]:
            if args.only and ident not in args.only:
                continue
            flagged.append(
                {
                    **b,
                    "id": ident,
                    "defined_by_spec": ident in texts,
                    "requirement": texts.get(ident, ""),
                }
            )

    by_file: dict[str, list[dict]] = {}
    for f in flagged:
        by_file.setdefault(f["file"], []).append(f)

    print(f"tag blocks above a data container: {len(flagged)} bindings in {len(by_file)} files")
    print(f"  of which the id has an authoritative definition: {sum(1 for f in flagged if f['defined_by_spec'])}")
    print(f"  ids with no authoritative definition at all:   {sum(1 for f in flagged if not f['defined_by_spec'])}")

    if args.only:
        for f in flagged:
            print(f"\n{f['file']}:{f['line']}  {f['id']}  -> {f['kind']} {f['name']}")
            print(f"    requirement: {f['requirement'] or '<no spec text found>'}")
        if args.json:
            args.json.write_text(json.dumps(flagged, indent=2), encoding="utf-8")
        return 0

    for path in sorted(by_file):
        rows = by_file[path]
        print(f"\n{path}  ({len(rows)})")
        for r in sorted(rows, key=lambda r: (r["line"], r["id"])):
            mark = "" if r["defined_by_spec"] else "  [NO SPEC TEXT]"
            print(f"    {r['line']:>5}  {r['id']:<30} {r['kind']} {r['name']}{mark}")

    if args.json:
        args.json.write_text(json.dumps(flagged, indent=2), encoding="utf-8")
        print(f"\nwrote {args.json}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
