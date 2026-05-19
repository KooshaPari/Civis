#!/usr/bin/env python3
"""Changelog ledger-discipline linter — Pattern #92 CI gate.

Pattern #92 ("Audit Ledger Decay") is the failure mode where ``CHANGELOG.md``
silently grows duplicate version blocks, multiple ``[Unreleased]`` sections,
non-spec ``### Section`` headings, repeated ``### Added`` blocks within a
single version, missing rows for shipped tags, and stale ``TODO(#NNN)`` markers
pointing at long-closed task IDs. The fix landed under #250 (structural
rewrite); this gate (#251) is the defense that prevents the decay from
re-accumulating.

Five hard checks (any HIT → exit 1):

  1. Version-header uniqueness — every ``## [N.N.N]`` line is unique, and
     ``## [Unreleased]`` appears at most once.
  2. Tag-coverage parity — every non-prerelease ``v\\d+\\.\\d+\\.\\d+`` git tag
     has a corresponding ``## [N.N.N]`` row. Pre-release tags are skipped.
  3. Keep-a-Changelog section whitelist — within each version block, only
     ``### Added``, ``### Changed``, ``### Deprecated``, ``### Removed``,
     ``### Fixed``, ``### Security`` are permitted by default. Project-specific
     extensions are added via the allowlist file.
  4. Within-block section uniqueness — each version block has at most one
     ``### Added``, one ``### Fixed``, etc. Duplicates indicate ledger decay.
  5. Stale TODO sweep (soft warning, exit-0 by default; --strict promotes
     to hard) — scans ``src/**/*.cs`` for ``TODO(#NNN)`` markers and flags
     any whose task ID appears in ``docs/qa/closed-tasks.txt``.

CLI:
    python scripts/ci/changelog_lint.py [--root <path>]
                                         [--changelog <path>]
                                         [--allowlist <path>]
                                         [--src-root <path>]
                                         [--closed-tasks <path>]
                                         [--output <json>]
                                         [--strict]
                                         [--quiet|--verbose]
                                         [--self-test]

Exit 0 = all hard checks passed; 1 = any hard violation; 2 = scan/usage error.

Modeled on ``scripts/ci/tautological_test_check.py`` (#247) and
``scripts/analysis/enumerate_orphan_classes.py`` (#229/#237).

This is task #251.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from datetime import datetime
from pathlib import Path


# ----------------------------------------------------------------------------
# Regex catalogue
# ----------------------------------------------------------------------------

# Version header — captures the bracketed identifier (e.g. ``0.11.0``,
# ``0.24.0-dev``, ``Unreleased``). Tolerates either ``-`` or em-dash separator
# after the bracket. We do NOT validate semver shape here; that's done after
# capture so we can isolate the unreleased / canonical pre-release variants.
VERSION_HEADER_RE = re.compile(r"^##\s+\[(?P<id>[^\]]+)\]", re.MULTILINE)

# Section header inside a version block (### Added, ### Fixed, ...).
SECTION_HEADER_RE = re.compile(r"^###\s+(?P<name>.+?)\s*$", re.MULTILINE)

# TODO(#NNN) inside a C# source file. Accepts both `// TODO(#123): note` and
# bare `TODO(#123)`.
TODO_TASK_RE = re.compile(r"TODO\(#(?P<id>\d+)\)")

# Strict semver tag — v\d+.\d+.\d+ with NO suffix (no -rc, -beta, etc.).
STRICT_SEMVER_TAG_RE = re.compile(r"^v(?P<ver>\d+\.\d+\.\d+)$")

# Loose semver inside a header bracket — ``0.11.0``, ``0.24.0-dev``. Used to
# decide whether a header is a versioned row (vs ``Unreleased``).
HEADER_VERSION_RE = re.compile(r"^\d+\.\d+\.\d+(?:[\-+][\w\.\-]+)?$")

# Keep-a-Changelog canonical sections.
KEEP_A_CHANGELOG_SECTIONS = frozenset(
    {"Added", "Changed", "Deprecated", "Removed", "Fixed", "Security"}
)

# Excluded directory parts when sweeping for TODOs.
EXCLUDED_DIR_PARTS = {"bin", "obj"}


# ----------------------------------------------------------------------------
# IO helpers
# ----------------------------------------------------------------------------


def read_text_safe(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def is_excluded_path(path: Path) -> bool:
    return bool(set(path.parts) & EXCLUDED_DIR_PARTS)


def line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def load_allowlist(path: Path) -> set[str]:
    """Each line is one allowed extra ``### Section`` label, ``#`` for
    comments and blanks ignored. Missing file → empty set."""
    if not path.exists():
        return set()
    out: set[str] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        out.add(line)
    return out


def load_closed_tasks(path: Path) -> set[int]:
    """Each line is a closed task ID (digits only, may be prefixed with ``#``).
    ``#`` at column 0 followed by space is a comment; ``#123`` (no space) is
    a task ID. Missing file → empty set (TODO sweep degrades gracefully)."""
    if not path.exists():
        return set()
    out: set[int] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line:
            continue
        # comment line: starts with `#` followed by space, or starts with `//`.
        if line.startswith("# ") or line.startswith("//"):
            continue
        # accept either `123` or `#123`
        cand = line[1:] if line.startswith("#") else line
        # split off trailing whitespace / inline comment
        cand = cand.split()[0] if cand.split() else ""
        if cand.isdigit():
            out.add(int(cand))
    return out


# ----------------------------------------------------------------------------
# Git tag enumeration
# ----------------------------------------------------------------------------


def get_git_tags(repo_root: Path) -> list[str]:
    """Return all tags matching ``v*`` from the repo. Empty list on error."""
    try:
        result = subprocess.run(
            ["git", "tag", "-l", "v*"],
            cwd=str(repo_root),
            capture_output=True,
            text=True,
            check=False,
            timeout=30,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return []
    if result.returncode != 0:
        return []
    return [
        line.strip()
        for line in result.stdout.splitlines()
        if line.strip()
    ]


def filter_strict_semver_tags(tags: list[str]) -> list[str]:
    """Return only canonical ``vN.N.N`` tags — pre-release and suffixed
    variants are dropped."""
    out: list[str] = []
    for t in tags:
        if STRICT_SEMVER_TAG_RE.match(t):
            out.append(t)
    return out


# ----------------------------------------------------------------------------
# Changelog parsing
# ----------------------------------------------------------------------------


def parse_changelog_blocks(text: str) -> list[dict]:
    """Return a list of version-block records.

    Each record:
        {
            "id": str,            # bracketed identifier
            "header_line": int,
            "header_offset": int,
            "body_start": int,    # offset after the header newline
            "body_end": int,      # offset of the next header (or EOF)
            "body": str,
            "is_unreleased": bool,
            "is_versioned": bool, # matches HEADER_VERSION_RE
            "version": str | None,  # canonical N.N.N if is_versioned
        }
    """
    blocks: list[dict] = []
    matches = list(VERSION_HEADER_RE.finditer(text))
    for i, m in enumerate(matches):
        ident = m.group("id").strip()
        body_start = m.end()
        # advance past trailing newline so body starts at next line
        if body_start < len(text) and text[body_start] == "\n":
            body_start += 1
        body_end = matches[i + 1].start() if i + 1 < len(matches) else len(text)
        body = text[body_start:body_end]
        is_unreleased = ident.lower() == "unreleased"
        is_versioned = bool(HEADER_VERSION_RE.match(ident))
        # canonical version stripping any suffix (e.g. ``0.24.0-dev`` → ``0.24.0``)
        version: str | None = None
        if is_versioned:
            version = ident.split("-", 1)[0].split("+", 1)[0]
        blocks.append(
            {
                "id": ident,
                "header_line": line_of(text, m.start()),
                "header_offset": m.start(),
                "body_start": body_start,
                "body_end": body_end,
                "body": body,
                "is_unreleased": is_unreleased,
                "is_versioned": is_versioned,
                "version": version,
            }
        )
    return blocks


def find_block_sections(block_text: str, block_start_line: int) -> list[dict]:
    """Return a list of ``### Foo`` headings inside a block body.

    Each record: ``{"name": str, "line": int}``. Line is 1-based against the
    *original* changelog (we add ``block_start_line - 1``)."""
    out: list[dict] = []
    for m in SECTION_HEADER_RE.finditer(block_text):
        name = m.group("name").strip()
        # strip trailing markdown-style emphasis or punctuation we don't care
        # about — we only check the leading word.
        line_in_block = block_text.count("\n", 0, m.start()) + 1
        out.append(
            {
                "name": name,
                "line": block_start_line + line_in_block - 1,
            }
        )
    return out


def first_word(name: str) -> str:
    """Extract the leading bare word of a heading (Keep-a-Changelog labels are
    single-word: Added/Changed/etc.). ``"Added — extras"`` → ``"Added"``."""
    # split on whitespace, em-dash, dash, parens
    for sep in (" ", "—", "–", "-", "("):
        if sep in name:
            return name.split(sep, 1)[0].strip()
    return name.strip()


# ----------------------------------------------------------------------------
# Hard-check implementations
# ----------------------------------------------------------------------------


def check_duplicate_headers(blocks: list[dict]) -> list[dict]:
    """Return offenders: any ``[id]`` appearing >1 time. ``Unreleased`` is
    enforced as ``at most 1 occurrence``."""
    seen: dict[str, list[dict]] = {}
    for b in blocks:
        seen.setdefault(b["id"], []).append(b)
    offenders: list[dict] = []
    for ident, occurrences in seen.items():
        if len(occurrences) > 1:
            offenders.append(
                {
                    "id": ident,
                    "count": len(occurrences),
                    "lines": [o["header_line"] for o in occurrences],
                }
            )
    offenders.sort(key=lambda r: (r["id"], r["lines"][0]))
    return offenders


def check_missing_tags(
    blocks: list[dict], strict_tags: list[str]
) -> list[dict]:
    """Return offenders: each strict-semver tag with no matching versioned
    ``[N.N.N]`` row in the changelog."""
    versions_in_changelog: set[str] = set()
    for b in blocks:
        if b["is_versioned"] and b["version"]:
            versions_in_changelog.add(b["version"])
    offenders: list[dict] = []
    for tag in strict_tags:
        m = STRICT_SEMVER_TAG_RE.match(tag)
        if not m:
            continue
        ver = m.group("ver")
        if ver not in versions_in_changelog:
            offenders.append({"tag": tag, "version": ver})
    offenders.sort(key=lambda r: tuple(int(p) for p in r["version"].split(".")))
    return offenders


def check_non_keep_changelog_sections(
    blocks: list[dict],
    allowlist: set[str],
) -> list[dict]:
    """Return offenders: any ``### X`` in a block where X-first-word is NOT
    a Keep-a-Changelog label and NOT in the allowlist."""
    offenders: list[dict] = []
    for b in blocks:
        sections = find_block_sections(b["body"], b["header_line"])
        for sec in sections:
            label = first_word(sec["name"])
            if label in KEEP_A_CHANGELOG_SECTIONS:
                continue
            # whole heading allowlisted? OR first-word allowlisted?
            if sec["name"] in allowlist or label in allowlist:
                continue
            offenders.append(
                {
                    "block": b["id"],
                    "section": sec["name"],
                    "line": sec["line"],
                }
            )
    offenders.sort(key=lambda r: (r["line"], r["section"]))
    return offenders


def check_duplicate_within_block_sections(
    blocks: list[dict],
) -> list[dict]:
    """Return offenders: each version block where the same Keep-a-Changelog
    label (Added/Changed/etc.) appears >1 time."""
    offenders: list[dict] = []
    for b in blocks:
        sections = find_block_sections(b["body"], b["header_line"])
        seen: dict[str, list[int]] = {}
        for sec in sections:
            label = first_word(sec["name"])
            if label not in KEEP_A_CHANGELOG_SECTIONS:
                continue
            seen.setdefault(label, []).append(sec["line"])
        for label, lines in seen.items():
            if len(lines) > 1:
                offenders.append(
                    {
                        "block": b["id"],
                        "section": label,
                        "count": len(lines),
                        "lines": lines,
                    }
                )
    offenders.sort(key=lambda r: (r["block"], r["section"]))
    return offenders


def check_stale_todos(
    src_root: Path,
    closed_task_ids: set[int],
) -> list[dict]:
    """Sweep ``src_root/**/*.cs`` for ``TODO(#NNN)`` markers; return any whose
    NNN is in ``closed_task_ids``."""
    offenders: list[dict] = []
    if not src_root.exists():
        return offenders
    for cs_file in sorted(src_root.rglob("*.cs")):
        if is_excluded_path(cs_file):
            continue
        text = read_text_safe(cs_file)
        if not text or "TODO(#" not in text:
            continue
        for m in TODO_TASK_RE.finditer(text):
            try:
                tid = int(m.group("id"))
            except ValueError:
                continue
            if tid not in closed_task_ids:
                continue
            line_num = line_of(text, m.start())
            try:
                rel = cs_file.relative_to(src_root).as_posix()
            except ValueError:
                rel = cs_file.as_posix()
            offenders.append(
                {
                    "task_id": tid,
                    "file": rel,
                    "line": line_num,
                }
            )
    offenders.sort(key=lambda r: (r["file"], r["line"]))
    return offenders


# ----------------------------------------------------------------------------
# Report + CLI
# ----------------------------------------------------------------------------


def build_report(
    duplicate_headers: list[dict],
    missing_tags: list[dict],
    non_keep_sections: list[dict],
    duplicate_within_block: list[dict],
    stale_todos: list[dict],
    strict: bool,
) -> dict:
    hard_violation_count = (
        len(duplicate_headers)
        + len(missing_tags)
        + len(non_keep_sections)
        + len(duplicate_within_block)
    )
    if strict:
        hard_violation_count += len(stale_todos)
    exit_code = 1 if hard_violation_count > 0 else 0
    return {
        "scan_utc": datetime.utcnow().isoformat() + "Z",
        "scan_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "duplicate_headers": duplicate_headers,
        "missing_tags": missing_tags,
        "non_keep_changelog_sections": non_keep_sections,
        "duplicate_within_block_sections": duplicate_within_block,
        "stale_todos": stale_todos,
        "strict": strict,
        "exit_code": exit_code,
    }


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Lint CHANGELOG.md for Pattern #92 (Audit Ledger Decay) — "
            "duplicate headers, missing tag rows, non-Keep-a-Changelog "
            "sections, duplicate within-block sections, stale TODO(#NNN)."
        )
    )
    p.add_argument(
        "--root",
        default=None,
        help="Repo root (default: derived from script location)",
    )
    p.add_argument(
        "--changelog",
        default="CHANGELOG.md",
        help="Path to changelog file (default: CHANGELOG.md)",
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/changelog-lint-allowlist.txt",
        help=(
            "Allowlist file with one extra ### Section label per line "
            "(default: docs/qa/changelog-lint-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--src-root",
        default="src",
        help="Source tree to sweep for TODO(#NNN) markers (default: src)",
    )
    p.add_argument(
        "--closed-tasks",
        default="docs/qa/closed-tasks.txt",
        help=(
            "File listing closed task IDs (one per line, optional `#` "
            "prefix; default: docs/qa/closed-tasks.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/changelog-lint-report.json",
        help="JSON report output path",
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help="Promote stale-TODO warnings to hard violations",
    )
    p.add_argument("--quiet", action="store_true", help="One-line CI summary")
    p.add_argument("--verbose", action="store_true", help="Verbose progress")
    return p.parse_args(argv)


def repo_root_from_script() -> Path:
    """scripts/ci/<this>.py → repo root is parents[2]."""
    return Path(__file__).resolve().parents[2]


def resolve_paths(args: argparse.Namespace) -> dict:
    repo = (
        Path(args.root).resolve()
        if args.root
        else repo_root_from_script()
    )

    def _abs(p: str) -> Path:
        path = Path(p)
        return path if path.is_absolute() else (repo / path).resolve()

    return {
        "repo": repo,
        "changelog": _abs(args.changelog),
        "allowlist": _abs(args.allowlist),
        "src_root": _abs(args.src_root),
        "closed_tasks": _abs(args.closed_tasks),
        "output": _abs(args.output),
    }


def print_summary(report: dict, output_path: Path) -> None:
    print("changelog-lint scan (Pattern #92)")
    print(f"  duplicate headers              : {len(report['duplicate_headers'])}")
    print(f"  missing tags                   : {len(report['missing_tags'])}")
    print(f"  non-Keep-a-Changelog sections  : {len(report['non_keep_changelog_sections'])}")
    print(f"  duplicate within-block sections: {len(report['duplicate_within_block_sections'])}")
    print(f"  stale TODO markers             : {len(report['stale_todos'])} "
          f"({'HARD' if report['strict'] else 'soft'})")

    if report["duplicate_headers"]:
        print()
        print("DUPLICATE version headers:")
        for r in report["duplicate_headers"]:
            print(f"  - [{r['id']}]  count={r['count']}  lines={r['lines']}")
    if report["missing_tags"]:
        print()
        print("MISSING changelog rows for shipped tags:")
        for r in report["missing_tags"]:
            print(f"  - {r['tag']} (no [{r['version']}] row)")
    if report["non_keep_changelog_sections"]:
        print()
        print("NON-Keep-a-Changelog ### sections (whitelist via allowlist):")
        for r in report["non_keep_changelog_sections"]:
            print(f"  - [{r['block']}]  ### {r['section']}  (line {r['line']})")
    if report["duplicate_within_block_sections"]:
        print()
        print("DUPLICATE ### Section within a single version block:")
        for r in report["duplicate_within_block_sections"]:
            print(
                f"  - [{r['block']}]  ### {r['section']}  count={r['count']}  "
                f"lines={r['lines']}"
            )
    if report["stale_todos"]:
        print()
        sev = "HARD (--strict)" if report["strict"] else "soft warning"
        print(f"STALE TODO(#NNN) markers — task closed ({sev}):")
        for r in report["stale_todos"]:
            print(f"  - TODO(#{r['task_id']})  {r['file']}:{r['line']}")
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# Self-tests
# ----------------------------------------------------------------------------


def _self_test() -> int:  # noqa: C901 — single fixture, fine to be long
    # 1) Detect duplicate ## [0.11.0]
    fixture_dup_header = """# Changelog

## [Unreleased]

### Added
- thing

## [0.11.0] - 2026-03-20

### Added
- a

## [0.11.0] - 2026-03-15

### Fixed
- b
"""
    blocks = parse_changelog_blocks(fixture_dup_header)
    dups = check_duplicate_headers(blocks)
    assert any(r["id"] == "0.11.0" and r["count"] == 2 for r in dups), (
        f"Expected duplicate [0.11.0]; got {dups}"
    )

    # 1b) Detect duplicate ## [Unreleased]
    fixture_dup_unreleased = """# Changelog

## [Unreleased]

### Added
- a

## [Unreleased]

### Fixed
- b

## [0.11.0] - 2026-03-15

### Added
- c
"""
    blocks2 = parse_changelog_blocks(fixture_dup_unreleased)
    dups2 = check_duplicate_headers(blocks2)
    assert any(r["id"] == "Unreleased" for r in dups2), (
        f"Expected duplicate [Unreleased]; got {dups2}"
    )

    # 2) Detect missing tag — fixture changelog has 0.11.0 but synthetic tag
    #    list contains v0.12.0 (no row).
    synth_tags = ["v0.11.0", "v0.12.0", "v0.1.0-rc.1"]
    strict_tags = filter_strict_semver_tags(synth_tags)
    # The pre-release tag should be filtered out.
    assert "v0.1.0-rc.1" not in strict_tags, strict_tags
    assert "v0.11.0" in strict_tags
    assert "v0.12.0" in strict_tags

    fixture_missing_tag = """# Changelog

## [0.11.0] - 2026-03-15

### Added
- a
"""
    blocks3 = parse_changelog_blocks(fixture_missing_tag)
    missing = check_missing_tags(blocks3, strict_tags)
    assert any(r["tag"] == "v0.12.0" for r in missing), (
        f"Expected v0.12.0 missing; got {missing}"
    )
    assert not any(r["tag"] == "v0.11.0" for r in missing), (
        f"v0.11.0 should be present in changelog; got {missing}"
    )

    # 3) Detect non-spec section ### Major Features
    fixture_non_keep = """# Changelog

## [0.20.0] - 2026-04-08

### Added
- a

### Major Features
- b

### Fixed
- c
"""
    blocks4 = parse_changelog_blocks(fixture_non_keep)
    non_keep = check_non_keep_changelog_sections(blocks4, allowlist=set())
    assert any(r["section"] == "Major Features" for r in non_keep), (
        f"Expected Major Features flagged; got {non_keep}"
    )
    # And allowlist should suppress it.
    non_keep_allowed = check_non_keep_changelog_sections(
        blocks4, allowlist={"Major Features"}
    )
    assert not any(
        r["section"] == "Major Features" for r in non_keep_allowed
    ), f"Allowlist should suppress Major Features; got {non_keep_allowed}"

    # 4) Detect duplicate ### Added within block
    fixture_dup_section = """# Changelog

## [0.11.0] - 2026-03-20

### Added
- a

### Fixed
- b

### Added
- c
"""
    blocks5 = parse_changelog_blocks(fixture_dup_section)
    dup_section = check_duplicate_within_block_sections(blocks5)
    assert any(
        r["block"] == "0.11.0" and r["section"] == "Added"
        and r["count"] == 2
        for r in dup_section
    ), f"Expected duplicate ### Added in [0.11.0]; got {dup_section}"

    # 5) Pass on healthy minimal changelog — all five hard checks must be empty.
    fixture_healthy = """# Changelog

## [Unreleased]

### Added
- pending feature x

## [0.11.0] - 2026-03-15

### Added
- a

### Fixed
- b
"""
    blocks_h = parse_changelog_blocks(fixture_healthy)
    assert check_duplicate_headers(blocks_h) == [], (
        check_duplicate_headers(blocks_h)
    )
    assert check_missing_tags(blocks_h, ["v0.11.0"]) == [], (
        check_missing_tags(blocks_h, ["v0.11.0"])
    )
    assert check_non_keep_changelog_sections(blocks_h, set()) == [], (
        check_non_keep_changelog_sections(blocks_h, set())
    )
    assert check_duplicate_within_block_sections(blocks_h) == [], (
        check_duplicate_within_block_sections(blocks_h)
    )

    # 6) Skip pre-release tags (v0.1.0-rc.1, v1.0.0-beta, v0.5.0-alpha.1)
    pre = [
        "v0.1.0-rc.1",
        "v1.0.0-beta",
        "v0.5.0-alpha.1",
        "v0.1.0-warfare-starwars",
    ]
    assert filter_strict_semver_tags(pre) == [], filter_strict_semver_tags(pre)
    assert filter_strict_semver_tags(["v0.1.0", "v1.0.0"]) == [
        "v0.1.0",
        "v1.0.0",
    ]

    # 7) TODO sweep — closed-task lookup
    import tempfile

    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        cs = td_path / "Foo.cs"
        cs.write_text(
            "// TODO(#100): closed\n"
            "// TODO(#999): open\n"
            "public class Foo { }\n",
            encoding="utf-8",
        )
        offenders = check_stale_todos(td_path, {100, 200})
        assert any(r["task_id"] == 100 for r in offenders), offenders
        assert not any(r["task_id"] == 999 for r in offenders), offenders

    print("self-test: OK")
    return 0


# ----------------------------------------------------------------------------
# main
# ----------------------------------------------------------------------------


def main(argv: list[str]) -> int:
    if len(argv) == 1 and argv[0] == "--self-test":
        return _self_test()

    args = parse_args(argv)
    if args.quiet and args.verbose:
        print(
            "ERROR: --quiet and --verbose are mutually exclusive",
            file=sys.stderr,
        )
        return 2

    paths = resolve_paths(args)

    if not paths["changelog"].exists():
        print(
            f"ERROR: changelog not found: {paths['changelog']}",
            file=sys.stderr,
        )
        return 2

    if args.verbose:
        print(f"reading {paths['changelog']} ...", file=sys.stderr)

    text = read_text_safe(paths["changelog"])
    blocks = parse_changelog_blocks(text)

    tags_all = get_git_tags(paths["repo"])
    strict_tags = filter_strict_semver_tags(tags_all)

    if args.verbose:
        print(
            f"  found {len(blocks)} version blocks, "
            f"{len(strict_tags)} strict-semver tags ({len(tags_all)} total)",
            file=sys.stderr,
        )

    allowlist = load_allowlist(paths["allowlist"])
    closed_tasks = load_closed_tasks(paths["closed_tasks"])

    duplicate_headers = check_duplicate_headers(blocks)
    missing_tags = check_missing_tags(blocks, strict_tags)
    non_keep_sections = check_non_keep_changelog_sections(blocks, allowlist)
    duplicate_within_block = check_duplicate_within_block_sections(blocks)
    stale_todos = check_stale_todos(paths["src_root"], closed_tasks)

    report = build_report(
        duplicate_headers,
        missing_tags,
        non_keep_sections,
        duplicate_within_block,
        stale_todos,
        strict=args.strict,
    )
    report["changelog_path"] = paths["changelog"].as_posix()
    report["allowlist_path"] = paths["allowlist"].as_posix()
    report["src_root"] = paths["src_root"].as_posix()
    report["closed_tasks_path"] = paths["closed_tasks"].as_posix()
    report["closed_task_count"] = len(closed_tasks)
    report["allowlist_size"] = len(allowlist)
    report["blocks_seen"] = len(blocks)
    report["tags_total"] = len(tags_all)
    report["tags_strict_semver"] = len(strict_tags)

    paths["output"].parent.mkdir(parents=True, exist_ok=True)
    paths["output"].write_text(json.dumps(report, indent=2), encoding="utf-8")

    if args.quiet:
        print(
            "changelog-lint: "
            f"dup_headers={len(duplicate_headers)} "
            f"missing_tags={len(missing_tags)} "
            f"non_keep={len(non_keep_sections)} "
            f"dup_section={len(duplicate_within_block)} "
            f"stale_todo={len(stale_todos)}"
            f"{' [strict]' if args.strict else ''} "
            f"-> {paths['output']}"
        )
    else:
        print_summary(report, paths["output"])

    return report["exit_code"]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
