#!/usr/bin/env python3
"""Gather ALL FR-* and NFR-* IDs with VERIFIED file:line references.

Classification rules:
  SPEC sources:
    - agileplus-specs/**/spec.md
    - agileplus-specs/**/plan.md
    - agileplus-specs/**/meta.json
    - FUNCTIONAL_REQUIREMENTS.md
    - PRD.md
    - docs/FR.md
    - docs/FR_DETAILED.md
    - docs/reference/non-functional-requirements.md  (NFR index)
  TRACEABILITY sources: docs/traceability/**.md
    - Authoritative FR spec matrices:
        docs/traceability/TRACEABILITY_MATRIX.md
        docs/traceability/fr-3d-matrix.md
        docs/traceability/full-traceability-matrix.md
      (rows here ARE the spec for FRs only listed there)
  CODE sources: anything else (crates/, clients/, web/, scripts/, scenarios/,
                schemas/, mods/, and other docs not classified above)
  TEST sources: files under */tests/*, files with .test.* / .spec.* suffixes,
                files starting with test_, files in __tests__/.

Spec mirror matrices are treated as SPEC sources (the row IS the spec).

Self-referential files (our own audit intermediates, fragmented docs dumps,
PR body artifacts, upstream-governance docs from other projects) are EXCLUDED
so they don't pollute CODE-ONLY results.
"""
import json
import re
import sys
from pathlib import Path
from collections import defaultdict

WORK = Path("G:/civis-wt-matrix")
OUT_JSON = WORK / "docs/audits/_id_inventory_v3.json"

SCAN_DIRS = [
    "crates", "clients", "docs", "agileplus-specs", "web", "scripts", "mods",
    "scenarios", "schemas",
]
SCAN_FILES = [
    "FUNCTIONAL_REQUIREMENTS.md", "PRD.md", "PLAN.md", "README.md",
    "STATUS.md", "SPEC.md", "ADR.md", "AGENTS.md", "CLAUDE.md",
    "CHANGELOG.md", "justfile", "Taskfile.yml",
    "Cargo.toml", "package.json", "hashmap.json",
]

# Top-level dirs we never scan
SKIP_TOP_DIRS = {
    ".git", "node_modules", "target", "dist", "build", "out", "vendor",
    "target-check-build", "target-check-build2", "target-check-clippy",
    "target-check-clippy2", "target-check-clippy3", "target-check-test",
    "target-check-test2", "bun.lock",
}
SKIP_EXT = {
    ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".ico", ".svg", ".webp",
    ".wav", ".mp3", ".ogg", ".flac", ".mp4", ".mov", ".webm",
    ".zip", ".tar", ".gz", ".tgz", ".7z", ".rar",
    ".exe", ".dll", ".so", ".dylib", ".a", ".lib", ".o", ".obj",
    ".pdb", ".exp", ".ilk", ".rmeta", ".rlib", ".d", ".timestamp", ".bin",
    ".uasset", ".umap", ".usf", ".ush", ".uplugin",
    ".pfx", ".pem", ".key", ".crt",
}
TEXT_EXT = {
    ".rs", ".toml", ".md", ".txt", ".json", ".yaml", ".yml", ".csv",
    ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".html", ".css", ".scss",
    ".py", ".gd", ".cs", ".cpp", ".h", ".hpp", ".c", ".sh", ".bash", ".ps1",
    ".ron", ".ronx",
    ".godot", ".tscn", ".tres", ".cfg", ".ini", ".env", ".example",
    ".hlsl", ".glsl", ".wgsl", ".frag", ".vert", ".shader", ".material",
    ".kt", ".swift", ".m", ".mm",
}
TEXT_NAMES = {n for n in SCAN_FILES}

# Self-referential / dump dirs we EXCLUDE entirely so we don't cite ourselves
# or pull in third-party project docs as if they were ours.
SELF_REF_DIRS = {
    "docs/audits",                  # our own intermediate files
    "docs/fragemented",             # root fragmented dump
    "docs/architecture/fragemented",
    "docs/models/civ-sim/fragemented",
    "docs/reference/fragemented",
    "docs/research/fragemented",
    "docs/upstream-governance",     # upstream docs from thegent/crun/task2/trace/zen-mcp-server
    "docs/upstream-governance/thegent/fragemented",
    "docs/upstream-governance/trace/fragemented",
    "docs/upstream-governance/crun/fragemented",
    "docs/upstream-governance/task2/fragemented",
    "docs/upstream-governance/zen-mcp-server/fragemented",
}
# PR body / diff files are excluded from scanning (they are PR review artifacts)
for n in (
    "pr-354.body.raw", "pr-354.diff",
    "pr-355.body.raw", "pr-355.diff",
    "pr-356.body.json", "pr-356.body.raw", "pr-356.diff",
    "pr-357.body.json", "pr-357.diff",
    "pr-358.body.json", "pr-358.diff",
    "pr-359.body.json", "pr-359.body.raw", "pr-359.body.txt", "pr-359.diff",
    "pr-360.body.json", "pr-360.body.json.full", "pr-360.body.raw",
    "pr-360.body.txt", "pr-360.diff",
    "pr-361.body.json", "pr-361.body.raw", "pr-361.diff",
):
    TEXT_NAMES.discard(n)
    SCAN_FILES = [f for f in SCAN_FILES if f != n]

# Spec files (any references found here count as a "spec" source)
SPEC_FILES_EXACT = {
    "FUNCTIONAL_REQUIREMENTS.md",
    "PRD.md",
    "docs/FR.md",
    "docs/FR_DETAILED.md",
    "docs/reference/non-functional-requirements.md",
    # Authoritative FR spec matrices
    "docs/traceability/TRACEABILITY_MATRIX.md",
    "docs/traceability/fr-3d-matrix.md",
    "docs/traceability/full-traceability-matrix.md",
}

# IDs must end with digits, with at least one FR-/NFR- <EPIC> <NUMBER> shape
ID_RE = re.compile(
    r"\b(FR|NFR)-(?:[A-Z]+-)?[A-Z]+[-A-Z0-9]*\d+(?:[-A-Z]+\d*)*\b"
)


def is_self_ref(rel: str) -> bool:
    rel_p = rel.replace("\\", "/")
    for d in SELF_REF_DIRS:
        if rel_p == d or rel_p.startswith(d + "/"):
            return True
    base = rel_p.rsplit("/", 1)[-1]
    if base.startswith("pr-") and (base.endswith(".body.json") or base.endswith(".body.raw")
                                    or base.endswith(".body.txt") or base.endswith(".diff")):
        return True
    return False


def should_skip(p: Path) -> bool:
    parts = set(p.parts)
    if parts & SKIP_TOP_DIRS:
        return True
    if p.name in SKIP_TOP_DIRS:
        return True
    if p.suffix.lower() in SKIP_EXT:
        return True
    if p.suffix.lower() in TEXT_EXT:
        return False
    if p.name in TEXT_NAMES:
        return False
    try:
        if p.stat().st_size > 5_000_000:
            return True
    except OSError:
        return True
    if not p.suffix:
        return True
    return True


def is_test_path(rel: str) -> bool:
    rel_p = rel.replace("\\", "/")
    if "/tests/" in rel_p:
        return True
    base = rel_p.rsplit("/", 1)[-1]
    if base.endswith((".test.mjs", ".test.ts", ".test.tsx", ".test.js", ".test.jsx",
                       ".spec.ts", ".spec.tsx", ".spec.mjs")):
        return True
    if base.startswith("test_") and base.endswith(".py"):
        return True
    if "/__tests__/" in rel_p:
        return True
    return False


def classify(rel: str) -> str:
    """Return one of: 'spec' | 'meta' | 'trace' | 'test' | 'code'."""
    rel_p = rel.replace("\\", "/")
    if is_test_path(rel_p):
        return "test"
    if rel_p in SPEC_FILES_EXACT:
        return "spec"
    if rel_p.startswith("agileplus-specs/") and rel_p.endswith("meta.json"):
        return "meta"
    if rel_p.startswith("agileplus-specs/") and (rel_p.endswith("spec.md") or rel_p.endswith("plan.md")):
        return "spec"
    if rel_p.startswith("docs/traceability/") and rel_p.endswith(".md"):
        return "trace"
    return "code"


def main():
    candidates = []
    for d in SCAN_DIRS:
        base = WORK / d
        if not base.exists():
            continue
        for p in base.rglob("*"):
            if not p.is_file():
                continue
            if should_skip(p):
                continue
            candidates.append(p)
    for fname in SCAN_FILES:
        p = WORK / fname
        if p.exists() and p.is_file() and not should_skip(p):
            candidates.append(p)
    candidates = sorted(set(candidates))
    pre = len(candidates)
    candidates = [p for p in candidates if not is_self_ref(str(p.relative_to(WORK)).replace("\\", "/"))]
    print(f"Scanning {len(candidates)} files (dropped {pre - len(candidates)} self-ref)", file=sys.stderr)

    by_id = defaultdict(lambda: {
        "in_specs": [],
        "in_meta": [],
        "in_func_req": None,
        "in_traceability": [],
        "in_code": [],
        "in_tests": [],
    })

    max_refs = 8  # cap per category

    for p in candidates:
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except Exception:
            continue
        rel = str(p.relative_to(WORK)).replace("\\", "/")
        kind = classify(rel)
        in_func = rel == "FUNCTIONAL_REQUIREMENTS.md"

        for m in ID_RE.finditer(text):
            line_no = text.count("\n", 0, m.start()) + 1
            eid = m.group(0)
            # Strip range suffix "...001..005" -> keep the start "FR-...-001"
            if ".." in eid:
                cleaned = eid.split("..", 1)[0]
                if not re.search(r"\d", cleaned):
                    continue
                eid = cleaned
            rec = by_id[eid]
            ref = f"{rel}:{line_no}"
            if kind == "test":
                if ref not in rec["in_tests"] and len(rec["in_tests"]) < max_refs:
                    rec["in_tests"].append(ref)
            elif kind == "meta":
                if ref not in rec["in_meta"] and len(rec["in_meta"]) < max_refs:
                    rec["in_meta"].append(ref)
            elif kind == "spec":
                if in_func:
                    if rec["in_func_req"] is None:
                        rec["in_func_req"] = "FUNCTIONAL_REQUIREMENTS.md"
                else:
                    if ref not in rec["in_specs"] and len(rec["in_specs"]) < max_refs:
                        rec["in_specs"].append(ref)
            elif kind == "trace":
                if ref not in rec["in_traceability"] and len(rec["in_traceability"]) < max_refs:
                    rec["in_traceability"].append(ref)
            else:  # code
                if ref not in rec["in_code"] and len(rec["in_code"]) < max_refs:
                    rec["in_code"].append(ref)

    ids_out = []
    for eid in sorted(by_id.keys()):
        ids_out.append({"id": eid, **by_id[eid]})

    OUT_JSON.write_text(
        json.dumps({"schema_version": 3, "generated_at": "2026-06-10", "ids": ids_out}, indent=2) + "\n",
        encoding="utf-8",
    )
    print(f"Wrote {len(ids_out)} unique IDs to {OUT_JSON}", file=sys.stderr)


if __name__ == "__main__":
    main()
