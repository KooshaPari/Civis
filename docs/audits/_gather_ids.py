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
import os
import re
import sys
from datetime import date
from pathlib import Path
from collections import defaultdict

WORK = Path(os.environ.get("CIVIS_AUDIT_WORK", ".")).resolve()
OUT_JSON = WORK / "docs/audits/_id_inventory_v3.json"
GENERATED_AT = os.environ.get("CIVIS_AUDIT_DATE", date.today().isoformat())

# The audit's own removal tooling replaces a requirement tag with a
# `// [unbound] <id>: <reason>` rationale explaining why the tag was wrong:
# see docs/audits/_apply_verdicts.py, whose is_tag_line() already skips these
# lines so it never mistakes one for a live tag. The gatherer did not, so every
# one of those rationales was being counted as `in_code` evidence for the very
# requirement the comment says is unbound. Measured, not estimated: 139 IDs
# carry at least one `[unbound]` rationale (183 references). 96 had no other
# code reference, and the matrix moved 97 of 1430 rows out of COVERED. An
# earlier draft here claimed "133 IDs ... reported COVERED"; that was a
# pre-regeneration heuristic, not a matrix diff, and it was wrong.
UNBOUND_TOKEN = "[unbound]"

# The same tooling has a second output shape. `_apply_verdicts.py:175`,
# `_unbind_false_tags.py:261` and `_unbind_protocol_modhost.py:404` all open a
# plain `//` comment block with the literal line
#
#     // Removed, with the reason each cannot be discharged here:
#
# and then list one `// <id>: <reason>` line per requirement it unbound. Those
# per-id lines carry NO `[unbound]` token, so the rule above does not see them,
# and the gatherer credited every one of them as `in_code`. Each line is a
# verbatim restatement of why the tag was removed from the declaration below,
# so crediting it asserts precisely the coverage the comment denies.
#
# Measured: 113 such blocks exist, and 83 references across 77 IDs sit inside
# one. Every one is a false `in_code` credit. Detection is structural rather
# than keyword-based on purpose -- three tools emit the header, and matching
# prose like "does not exist" would also catch legitimate discussion.
REMOVAL_MARKER = "Removed, with the reason each cannot be discharged here"


def removal_block_ranges(lines):
    """Yield (first, last) 1-based line spans of removal-rationale blocks.

    A block starts at a line containing REMOVAL_MARKER and extends forward over
    contiguous plain `//` line comments. A `///` or `//!` doc comment ENDS the
    block: in `crates/engine/src/fixed_math.rs` the removal block at 18-20 is
    immediately followed by the `Fixed` type's real doc comment at 21-27, and
    that doc comment legitimately carries requirement tags. Treating `///` as
    in-block would silently strip ~200 real tags and was the reason an earlier
    pass of this analysis over-reported the blast radius.
    """
    for i, line in enumerate(lines, 1):
        if REMOVAL_MARKER not in line:
            continue
        last = i
        k = i
        while k < len(lines):
            k += 1
            t = lines[k - 1].strip()
            if t.startswith("///") or t.startswith("//!"):
                break
            if not t.startswith("//"):
                break
            last = k
        yield (i, last)


def in_removal_block(ranges, line_no):
    """True if `line_no` falls inside any removal-rationale block span."""
    return any(a <= line_no <= b for a, b in ranges)

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
    # The audit tooling itself. It is Python/shell that *reports on* FR IDs, so
    # a literal `FR-CIV-ACCESS-010` inside a test fixture or a docstring is the
    # audit citing itself, not evidence that anything implements the
    # requirement. Measured: 24 IDs carried a `scripts/traceability/**`
    # reference in `in_code` or `in_tests`, and for the crates/hud
    # accessibility family that tooling reference was the *only* reference
    # outside the module's own definition lines.
    #
    # This directory holds no product code (Python and shell only), so nothing
    # real is lost by excluding it. See
    # docs/audits/spec-only-triage-2026-09-29.md, "Detector fix applied
    # 2026-10-01".
    "scripts/traceability",
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
    # Root roadmap documents state intended work; they are not code.
    "PLAN.md",
    "MASTER_PLAN.md",
}

# Documents under `docs/` state requirements or record project state; none of
# them implement anything. Counting a document as "code" made an ID look
# implemented when only a note existed, which inflated coverage.
#
# Evidence: `docs/design/psyche-social.md` declares itself "specs / AC /
# pseudocode only, no implementation code"; `docs/models/civ-sim/USER_SPEC.md`
# is a set of bold requirement statements; `docs/reference/FR_TRACKER.md` and
# `PLAN.md` are trackers and roadmaps. 707 IDs had their only "code" reference
# in a document, and a further 92 were reported COVERED for that reason alone.
#
# This does not drop real code references: an ID that cites both a document here
# and a `crates/**` path keeps the `crates/**` reference and stays classified on
# its implementation. `docs/traceability/` is handled earlier as `trace`.
DOC_DIR_PREFIXES = (
    "docs/",
    "agileplus-specs/",
)

# IDs must end with digits, with at least one FR-/NFR- <EPIC> <NUMBER> shape.
#
# The trailing suffix group is `(?:-?[A-Z]+\d*)*`. Two constraints shape it:
#
#  * The hyphen is optional so IDs whose segment ends in digit-then-letters
#    still match: `FR-CIV-PROTO3D` and `FR-CIV-3D`. Requiring `-[A-Z]` broke
#    those, silently dropping two real IDs that are referenced by their own
#    tests and `/// Covers FR-CIV-PROTO3D` markers.
#  * `[A-Z]+` (not `[-A-Z]+`) forbids a bare trailing hyphen. The earlier
#    `[-A-Z]+` form consumed the `-` in prose like `FR-CIV-TACTICS-025-int`
#    or `FR-CIV-0100-int1..int4`, minting phantom rows such as
#    `FR-CIV-TACTICS-025-` that duplicated the real `FR-CIV-TACTICS-025`.
#  * `[A-Za-z]+` in the *leading* segments admits lowercase namespaces. The
#    physics substrate ships a real, separately-numbered namespace written
#    `FR-PHYS-substrate-000..007`, documented item-by-item in
#    `crates/physics-substrate/src/lib.rs:780-1023`. An `[A-Z]+`-only pattern
#    rejected every one of them, so an entire namespace of implemented,
#    self-tagged requirements was invisible to the audit.
#
# Lowercase is allowed only BEFORE the final numeric group. The trailing
# `-int` / `-9xx` / `-live` forms stay rejected, because those are prose
# fragments of an already-numbered ID rather than IDs of their own.
#
# A match therefore always ends in a letter or digit, never a hyphen.
#
# The trailing `\b` is load-bearing and must be kept. Dropping it lets the
# pattern match the leading digits of a placeholder wildcard, so
# `FR-CIV-INSPECT-9xx` harvested `FR-CIV-INSPECT-9` and `FR-CIV-LIFE-014a`
# harvested `FR-CIV-LIFE-014`. Both phantom-row protections depend on it.
#
# A quoted shell glob is NOT an ID. A comment reading
# `git grep -n "FR-CIV-MOD-00" -- docs/specs/CIV-0700.md` quotes a search
# pattern, and harvesting the truncated prefix minted the phantom row
# FR-CIV-MOD-00. Quoting alone cannot distinguish that from a *complete* ID
# that happens to be quoted ("`NFR-CIV-001` needs a budget"), which is why the
# previous attempts failed: a `\d*` tail deleted 12 real NFR rows outright, and
# a `\d+` tail let the phantom straight back in.
#
# The signal that actually separates them is a longer sibling on the same line.
# crates/mod-host/src/lib.rs:82 carries FR-CIV-MOD-000, FR-CIV-MOD-00 and
# FR-CIV-MOD-001 together: the short one is a grep prefix for the other two.
# A quoted/glob numeric tail is only rejected when the same line also yields a
# strictly longer match of the same ID family. See `is_truncated_glob`.
ID_RE = re.compile(
    r"\b(FR|NFR)-(?:[A-Za-z]+-)?[A-Za-z]+[-A-Z0-9]*\d+(?:-?[A-Z]+\d*)*\b"
)

# A numeric tail immediately followed by a quote/glob marker is a candidate
# truncation. A bare ID with no marker (`FR-CIV-LIFE-004`) is still valid.
GLOB_TAIL_RE = re.compile(r"\d(?=[`'\"])")


def is_truncated_glob(text: str, start: int, end: int, matched: str) -> bool:
    """True when `matched` is a grep-style prefix of a longer ID on the line.

    Requires all three signals so a genuinely quoted complete ID is untouched:
      1. the character right after the match is a quote/glob marker,
      2. the match ends in a digit, and
      3. some other match on the same line shares this ID's family
         (same `FR-`/`NFR-` + alphabetic prefix) and is strictly longer.
    """
    if end >= len(text) or text[end] not in "`'\"":
        return False
    if not matched[-1].isdigit():
        return False
    head = _family(matched)
    for m in ID_RE.finditer(text):
        if m.start() == start and m.end() == end:
            continue
        other = m.group(0)
        if _family(other) == head and len(other) > len(matched):
            return True
    return False


def _family(eid: str) -> str:
    """`FR-CIV-MOD-00` and `FR-CIV-MOD-000` share the family `FR-CIV-MOD-`."""
    head = re.match(r"(?:FR|NFR)-[A-Za-z]+(?:-[A-Za-z]+)*", eid)
    return head.group(0) if head else eid

COVERS_RE = re.compile(
    r"^\s*///\s*Covers\s*:?(?:\s*(?:FR|NFR)-(?:[A-Za-z]+-)?[A-Za-z]+[-A-Z0-9]*\d+(?:-?[A-Z]+\d*)*\b)"
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
    try:
        rel_parts = p.relative_to(WORK).parts
    except ValueError:
        rel_parts = p.parts

    if rel_parts and rel_parts[0] in SKIP_TOP_DIRS:
        return True
    nested_skip_dirs = SKIP_TOP_DIRS - {"build"}
    if set(rel_parts[1:]) & nested_skip_dirs:
        return True
    if p.name in nested_skip_dirs:
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
    # Rust test-only modules: `foo_tests.rs` / `foo_test.rs` (e.g.
    # crates/watch/src/api_tests.rs, crates/engine/src/engine/engine_tests.rs).
    # These are `#[cfg(test)]`-gated modules containing only test fns; without
    # this rule their FR-ID comments counted as code refs, not test refs.
    if base.endswith(("_tests.rs", "_test.rs")):
        return True
    if base.endswith((".test.mjs", ".test.ts", ".test.tsx", ".test.js", ".test.jsx",
                       ".spec.ts", ".spec.tsx", ".spec.mjs")):
        return True
    if base.startswith("test_") and base.endswith(".py"):
        return True
    if "/__tests__/" in rel_p:
        return True
    return False


# Placeholder/stub test files carry one of these markers in their first 30
# lines. They are pre-existing shell tests that exercise a generic type (e.g.
# `WorldState::default()`) without asserting anything FR-specific, so the
# audit must not count them as real coverage. Detection is keyed off a string
# the bulk-marker script inserts, plus the legacy `Epic: auto-generated`
# header which the previous batch used on every placeholder.
_STUB_HEADER_CACHE: dict[str, bool] = {}
_STUB_MARKERS = ("Stub: TDD-red", "Epic: auto-generated")


def is_stub_test(rel: str) -> bool:
    """Return True iff the file at *rel* looks like a placeholder test."""
    if rel in _STUB_HEADER_CACHE:
        return _STUB_HEADER_CACHE[rel]
    rel_p = rel.replace("\\", "/")
    if not is_test_path(rel_p):
        _STUB_HEADER_CACHE[rel] = False
        return False
    p = WORK / rel_p
    try:
        head = "\n".join(p.read_text(encoding="utf-8", errors="replace").splitlines()[:30])
    except OSError:
        _STUB_HEADER_CACHE[rel] = False
        return False
    match = any(m in head for m in _STUB_MARKERS)
    _STUB_HEADER_CACHE[rel] = match
    return match


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
    # Documents are spec sources, never implementing code.
    if rel_p.startswith(DOC_DIR_PREFIXES) and rel_p.endswith(".md"):
        return "spec"
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
        "in_stub_tests": [],
        # Code refs that came from a `#[cfg(test)]` block inside a src/*.rs
        # file. Post-passed into the `self_test_only` flag below.
        "_selftest_code": set(),
        # `file:line` refs sitting on an `[unbound]` rationale rather than on a
        # real tag. Post-passed into `unbound_refs` below.
        "_unbound_code": set(),
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

        lines = text.splitlines()
        removal_ranges = list(removal_block_ranges(lines))
        if kind == "test":
            in_cfg_test = [True] * (len(lines) + 1)
        else:
            in_cfg_test = [False] * (len(lines) + 1)
            brace_depth = 0
            cfg_depths = []
            pending_cfg_test = False
            for i, line in enumerate(lines, start=1):
                stripped = line.strip()
                if stripped.startswith("#[cfg(test)]"):
                    pending_cfg_test = True
                    continue

                in_cfg_test[i] = bool(cfg_depths and brace_depth >= cfg_depths[-1])

                if pending_cfg_test and "{" in line:
                    cfg_depths.append(brace_depth + 1)
                    pending_cfg_test = False

                brace_depth += line.count("{") - line.count("}")
                while cfg_depths and brace_depth < cfg_depths[-1]:
                    cfg_depths.pop()

        has_cfg_test_attr = "#[cfg(test)]" in text
        for m in ID_RE.finditer(text):
            if is_truncated_glob(text, m.start(), m.end(), m.group(0)):
                # A grep-style prefix of a longer ID on the same line, e.g. the
                # `git grep -n "FR-CIV-MOD-00"` quoted inside a code comment.
                continue
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
            src = lines[line_no - 1] if 0 < line_no <= len(lines) else ""
            # A `[unbound]` rationale names the ID to record that its tag was
            # removed, so it is evidence about the requirement, not evidence
            # that the requirement is implemented. Crediting it as `in_code`
            # moved 97 of 1430 matrix rows out of COVERED; see UNBOUND_TOKEN.
            if kind == "code" and UNBOUND_TOKEN in src:
                rec["_unbound_code"].add(ref)
                continue
            # Same verdict, different shape: a per-id line inside a
            # "Removed, with the reason each cannot be discharged here:" block
            # states that the tag was unbound from the declaration below.
            # Crediting it as `in_code` asserts exactly what it denies.
            if kind == "code" and in_removal_block(removal_ranges, line_no):
                rec["_unbound_code"].add(ref)
                continue
            is_cover = bool(COVERS_RE.match(lines[line_no - 1])) if 0 < line_no <= len(lines) else False
            is_test_ref = kind == "test" or in_cfg_test[line_no] or (is_cover and has_cfg_test_attr)
            if is_test_ref:
                if is_stub_test(rel):
                    bucket = "in_stub_tests"
                else:
                    bucket = "in_tests"
                if ref not in rec[bucket] and len(rec[bucket]) < max_refs:
                    rec[bucket].append(ref)
                # A `#[cfg(test)]` module living inside a source file IS the
                # implementation for that crate. Without this, an ID whose only
                # reference is a test in the crate's own `src/lib.rs` gets a
                # test ref but no code ref, and the audit wrongly reports it as
                # SPEC-ONLY (e.g. FR-CIV-AGENTS-002 in crates/agents/src/lib.rs).
                if (
                    kind == "code"
                    and rel.endswith(".rs")
                    and not is_test_path(rel)
                ):
                    if ref not in rec["in_code"] and len(rec["in_code"]) < max_refs:
                        rec["in_code"].append(ref)
                    # Track code refs that originate inside a `#[cfg(test)]`
                    # block. When these are the ONLY code refs an ID has, the
                    # "implementation" evidence is really test evidence, and
                    # `classify()` would otherwise report COVERED on the
                    # strength of a self-assertion.
                    #
                    # The flag is advisory and deliberately narrow. It does NOT
                    # detect the broader "dead substrate" case where a symbol is
                    # defined, re-exported and self-tested but has no consumer:
                    # those IDs also carry a ref on the definition line, so they
                    # are not self_test_only. Catching that needs call-graph
                    # analysis, which this scanner does not attempt. See
                    # docs/audits/spec-only-triage-2026-09-29.md finding 2.
                    if in_cfg_test[line_no]:
                        rec["_selftest_code"].add(ref)
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
        rec = by_id[eid]
        selftests = rec.pop("_selftest_code", set())
        unbound = sorted(rec.pop("_unbound_code", set()))
        code_refs = rec["in_code"]
        # True only when EVERY code reference came from a self-test block, i.e.
        # the ID has no code reference outside test code. A single real
        # implementation line flips this back to False.
        rec["self_test_only"] = bool(code_refs) and selftests.issuperset(code_refs)
        # Recorded, not counted: where an `[unbound]` rationale says this ID's
        # tag was removed. Kept so the audit can name the reason an ID lost its
        # code evidence instead of silently dropping it.
        rec["unbound_refs"] = unbound
        ids_out.append({"id": eid, **rec})

    OUT_JSON.write_text(
        json.dumps({"schema_version": 3, "generated_at": GENERATED_AT, "ids": ids_out}, indent=2) + "\n",
        encoding="utf-8",
    )
    print(f"Wrote {len(ids_out)} unique IDs to {OUT_JSON}", file=sys.stderr)


if __name__ == "__main__":
    main()
