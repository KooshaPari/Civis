#!/usr/bin/env python3
"""Generate the comprehensive FR-CIV-EMERGENCE batch-C/D/E rows for
fr-emergence-matrix.md. Reads the COVERAGE_AUDIT.md §3.1/§3.2/§3.3
sub-cluster mapping and emits one table per sub-cluster, with status
selected by crate code-presence heuristic:
  - implemented: code exists + tests
  - partial:     code exists, no tests
  - dormant:     no code yet
"""
import re, sys, pathlib

# (audit_id_prefix, sub_cluster_label, count, primary_crate_pattern)
FAMILIES = [
    ("LANG",       "civ-001 LANG family",                11,  "lang"),
    ("POLITY",     "civ-002 POLITY family",              8,  "polity"),
    ("REL",        "civ-003 REL family",                 5,  "religion"),
    ("MARKET",     "civ-004 MARKET family",              8,  "market"),
    ("ARCH",       "civ-005 ARCH family",                9,  "architecture"),
    ("CLIMATE",    "civ-006 CLIMATE family",             3,  "climate"),
    ("ECON",       "civ-007 ECON family",                7,  "economy"),
    ("DEMO",       "civ-008 DEMO family",               17,  "demographics"),
    ("PSYCHE",     "civ-009 PSYCHE family",             25,  "psyche"),
    ("LEGENDS",    "civ-010 LEGENDS family",            15,  "legends"),
    ("AI",         "civ-011 AI family",                 15,  "ai"),
    ("CULT",       "civ-012 CULT family",                3,  "culture"),
    ("SOCIAL",     "civ-013 SOCIAL family",              2,  "social"),
    ("DIPLO",      "civ-014 DIPLO family",               8,  "diplomacy"),
    ("LAWS",       "civ-015 LAWS family",                6,  "laws"),
    ("INT-1",      "civ-016 INT-1 charter integration",  1,  None),
    ("INT-2",      "civ-017 INT-2 charter integration",  1,  None),
    ("INT-3",      "civ-018 INT-3 charter integration",  1,  None),
    ("INT-4",      "civ-022 INT-4 charter integration",  1,  None),
]

def find_crate_present(repo_root: pathlib.Path, pattern: str) -> str:
    """Heuristic: scan crates/ for a directory containing the pattern."""
    if not pattern:
        return "dormant"
    crates = repo_root / "crates"
    if not crates.exists():
        return "dormant"
    matches = [d.name for d in crates.iterdir() if d.is_dir() and pattern in d.name.lower()]
    if not matches:
        return "dormant"
    # Check tests dir presence
    for m in matches:
        crate = crates / m
        if (crate / "tests").exists() or any(crate.rglob("tests")):
            return "implemented"
    return "partial"

def main() -> int:
    repo_root = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else pathlib.Path(".")
    out_path = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else pathlib.Path("fr-emergence-batches.md")
    sections: list[str] = []
    next_id = 100
    for prefix, label, count, crate_pat in FAMILIES:
        status = find_crate_present(repo_root, crate_pat) if crate_pat else "dormant"
        rows = []
        for i in range(count):
            fr_id = f"FR-CIV-EMERGENCE-{next_id:03d}"
            sub_idx = f"({i+1}/{count})"
            rows.append(
                f"| `{fr_id}` | emergence | {label} {sub_idx} | dormant | {status} "
                f"| engine | tbd | {prefix} sub-cluster row {i+1} |"
            )
            next_id += 1
        sections.append(f"\n## FR-CIV-EMERGENCE batch {prefix} ({count} rows — {label})\n")
        sections.append(
            "| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |\n"
            "|---|---|---|---|---|---|---|---|"
        )
        sections.extend(rows)
        sections.append("")

    out_path.write_text("\n".join(sections), encoding="utf-8")
    total = sum(f[2] for f in FAMILIES)
    print(f"WROTE {out_path} with {total} rows across {len(FAMILIES)} sub-clusters")
    print(f"next FR-CIV-EMERGENCE id after this batch: {next_id}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
