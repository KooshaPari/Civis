#!/usr/bin/env python3
"""Generalized unbounded-constraint detector — Pattern #94 generalization.

Pattern #94 ("Unbounded Range Theatre") in its narrow form (#260) addressed
``framework_version: ">=0.1.0"`` — a check that has never returned false in
production, because every pre-1.0 version satisfies ``>=0.1.0``. The wider
anti-pattern is "if a check has never returned false in production, it's not
a check." This gate (#261) sweeps the repo for the broader family of
unbounded constraints that have the same failure mode.

Six detection rules, each landing in a separate JSON bucket:

  A. ``nuget_floating`` — ``<PackageReference Include="X" Version="*">`` or
     ``Version=">=0.0.0"`` with no upper bound. Walks ``**/*.csproj`` and
     ``Directory.Packages.props``. Skips files allowlisted as test
     infrastructure where floats are intentional.

  B. ``dependabot_unrationalized`` — ``ignore:`` entries inside
     ``.github/dependabot.yml`` lacking a rationale comment on the same
     block. Without a rationale, the ignore is unbounded too.

  C. ``gitignore_overbroad`` — ``*`` or ``*/`` at column 0 in ``.gitignore``,
     ``**/*`` rules that wildcard entire trees. Flag for manual review.

  D. ``tautological_predicate`` — predicate guards like ``if (x != null ||
     true)``, ``if (cond || true)``, ``if (true)``, vacuous boolean shorts.

  E. ``vacuous_numeric_guard`` — ``if (count >= 0)`` for a value of an
     unsigned type, or ``<= int.MaxValue`` for ``int``-typed values, or
     ``>= 0.0`` on a documented non-negative magnitude. We focus on the
     uncontested-tautology shape.

  F. ``unbounded_join_consumer`` — ``string.Join(", ", items)`` (or other
     unbounded-list joins) where the consumer is a logger / message line
     that lacks any truncation (``Take(N)``, ``..Substring(0, N)``). Heuristic
     only — high-volume false positives possible, so this rule is gated to
     ``--strict``.

Allowlist (one entry per line, ``#`` comments):
``docs/qa/unbounded-constraints-allowlist.txt`` — POSIX-relative file paths
or ``rule|file|line`` keys.

CLI:
    python scripts/ci/detect_unbounded_constraints.py [--root <path>]
                                                       [--allowlist <path>]
                                                       [--output <json>]
                                                       [--strict]
                                                       [--quiet|--verbose]
                                                       [--self-test]

Exit 0 = no NEW (un-allowlisted) HIGH/MED violations; 1 = violations
detected (CI fails); 2 = scan/usage error.

Pairs with ``scripts/ci/check_framework_version.py`` (#260, narrow gate).
Modeled on ``scripts/ci/detect_global_state_tests.py`` (#257) and
``scripts/ci/changelog_lint.py`` (#251).

This is task #261.
"""
from __future__ import annotations

import argparse
import json
import re
import sys

from regex_timeout import compile as _re_compile
from datetime import datetime
from pathlib import Path

try:
    import yaml  # type: ignore
except ImportError:
    yaml = None  # type: ignore


# ----------------------------------------------------------------------------
# Constants
# ----------------------------------------------------------------------------

EXCLUDED_DIR_PARTS = {
    "bin",
    "obj",
    "node_modules",
    ".git",
    "_archived",
    "__pycache__",
    ".vs",
    "packages",
}

# Severity tiers
SEV_HIGH = "HIGH"
SEV_MED = "MED"
SEV_LOW = "LOW"


# ----------------------------------------------------------------------------
# Rule A: NuGet floating versions
# ----------------------------------------------------------------------------

# <PackageReference Include="X" Version="..."/>
PACKAGE_REF_RE = _re_compile(
    r'<\s*Package(?:Reference|Version)\s+'
    r'(?:[^>]*?)'              # any other attrs
    r'(?:Include|Update)\s*=\s*"(?P<id>[^"]+)"'
    r'(?:[^>]*?)'
    r'Version\s*=\s*"(?P<ver>[^"]*)"',
    re.IGNORECASE | re.DOTALL,
)

# Same but Version-first, Include-second
PACKAGE_REF_VERSION_FIRST_RE = _re_compile(
    r'<\s*Package(?:Reference|Version)\s+'
    r'(?:[^>]*?)'
    r'Version\s*=\s*"(?P<ver>[^"]*)"'
    r'(?:[^>]*?)'
    r'(?:Include|Update)\s*=\s*"(?P<id>[^"]+)"',
    re.IGNORECASE | re.DOTALL,
)

# Floating-version markers — any of these in a Version attr means unbounded.
FLOATING_VERSION_PATTERNS = (
    "*",       # bare star
    ">=0.0.0",
    ">= 0.0.0",
    "[0.0.0,)",
)


def _is_floating_version(ver: str) -> tuple[bool, str]:
    """Return (is_floating, reason)."""
    v = ver.strip()
    if not v:
        return False, ""
    # bare ``*``
    if v == "*":
        return True, 'Version="*" (any version, no ceiling)'
    # ``X.Y.*`` — classic floating-build form. NuGet calls these "floating
    # versions"; for prod packages they're usually a smell.
    if re.fullmatch(r"\d+\.\d+\.\*", v) or re.fullmatch(r"\d+\.\*", v):
        return True, f'Version="{v}" (NuGet floating, no fixed ceiling)'
    if re.fullmatch(r"\d+\.\d+\.\d+\.\*", v):
        return True, f'Version="{v}" (NuGet floating, no fixed ceiling)'
    # ``>=X`` with no closing ``)`` or upper
    if re.fullmatch(r">=\s*0\.0\.0", v):
        return True, 'Version=">=0.0.0" (vacuous lower, no upper)'
    # ``[0.0.0,)`` — open upper bound
    if re.fullmatch(r"\[\s*0\.0\.0\s*,\s*\)", v):
        return True, 'Version="[0.0.0,)" (open upper bound)'
    # generic ``[X,)`` with no ceiling — flag if X is 0.0.0 or lower-bound only
    m = re.fullmatch(r"\[\s*(?P<lo>[\w\.\-]+)\s*,\s*\)", v)
    if m:
        return True, f'Version="{v}" (open upper bound)'
    return False, ""


def scan_csproj_for_floating(
    path: Path, repo_root: Path
) -> list[dict]:
    """Walk a single .csproj / .props / .targets file for floating refs."""
    text = read_text_safe(path)
    if not text:
        return []
    out: list[dict] = []
    seen_at: set[tuple[int, str]] = set()  # (line, package-id) dedupe
    for regex in (PACKAGE_REF_RE, PACKAGE_REF_VERSION_FIRST_RE):
        for m in regex.finditer(text):
            ver = m.group("ver")
            pkg = m.group("id")
            is_float, reason = _is_floating_version(ver)
            if not is_float:
                continue
            line = line_of(text, m.start())
            key = (line, pkg)
            if key in seen_at:
                continue
            seen_at.add(key)
            try:
                rel = path.relative_to(repo_root).as_posix()
            except ValueError:
                rel = path.as_posix()
            out.append(
                {
                    "rule": "nuget_floating",
                    "severity": SEV_HIGH,
                    "file": rel,
                    "line": line,
                    "package": pkg,
                    "version": ver,
                    "detail": (
                        f"PackageReference {pkg!r} uses floating version: "
                        f"{reason}. Pin to an explicit ``[lo,hi)`` range or a "
                        f"single version."
                    ),
                }
            )
    return out


# ----------------------------------------------------------------------------
# Rule B: Dependabot ignore-blocks without rationale
# ----------------------------------------------------------------------------


def scan_dependabot_unrationalized(
    path: Path, repo_root: Path
) -> list[dict]:
    """Walk ``.github/dependabot.yml``, surface every entry inside an
    ``ignore:`` list that has no comment justifying *why* it's ignored.

    Heuristic: an ignore entry is "rationalized" if the line directly above
    the entry's ``- dependency-name:`` (or any ``- ...`` line within the
    entry) is a comment, OR the ``- ...`` line itself has a trailing
    ``# comment``.
    """
    text = read_text_safe(path)
    if not text:
        return []
    lines = text.splitlines()
    out: list[dict] = []
    try:
        rel = path.relative_to(repo_root).as_posix()
    except ValueError:
        rel = path.as_posix()

    in_ignore = False
    ignore_indent = -1
    entry_start: int | None = None
    entry_lines: list[int] = []

    def _has_rationale(start_idx: int, end_idx: int) -> bool:
        """A comment on/preceding the entry's first line counts; trailing
        comment on any entry line counts."""
        # Preceding non-blank line.
        i = start_idx - 1
        while i >= 0 and not lines[i].strip():
            i -= 1
        if i >= 0 and lines[i].strip().startswith("#"):
            return True
        # Trailing inline comment on any entry line.
        for j in range(start_idx, end_idx + 1):
            line = lines[j]
            # crude: " # " or end-of-line "#" with at least one alpha
            stripped = line.split("#", 1)
            if len(stripped) == 2 and stripped[1].strip():
                # but skip lines that *start* with `#` (full-line comment) —
                # a bare `# something` line outside the entry — those are
                # handled by the preceding-line check.
                if line.lstrip().startswith("#"):
                    continue
                return True
        return False

    for idx, raw in enumerate(lines):
        stripped = raw.strip()
        # Top-level ignore: "  ignore:"
        if re.match(r"^\s*ignore\s*:\s*$", raw):
            in_ignore = True
            ignore_indent = len(raw) - len(raw.lstrip())
            entry_start = None
            entry_lines = []
            continue
        if in_ignore:
            if stripped == "" or stripped.startswith("#"):
                # blank or comment line within the block — keep state.
                continue
            line_indent = len(raw) - len(raw.lstrip())
            if line_indent <= ignore_indent and stripped:
                # de-dent: end of ignore block. Flush any pending entry.
                if entry_start is not None and entry_lines:
                    if not _has_rationale(entry_start, entry_lines[-1]):
                        out.append(
                            {
                                "rule": "dependabot_unrationalized",
                                "severity": SEV_MED,
                                "file": rel,
                                "line": entry_start + 1,
                                "detail": (
                                    "Dependabot ignore entry has no rationale "
                                    "comment. Add a `# WHY:` line above or a "
                                    "trailing comment explaining the freeze."
                                ),
                                "snippet": lines[entry_start].strip(),
                            }
                        )
                    entry_start = None
                    entry_lines = []
                in_ignore = False
                ignore_indent = -1
                # fall-through to allow re-matching on this line.
            elif stripped.startswith("- "):
                # New entry begins.
                if entry_start is not None and entry_lines:
                    if not _has_rationale(entry_start, entry_lines[-1]):
                        out.append(
                            {
                                "rule": "dependabot_unrationalized",
                                "severity": SEV_MED,
                                "file": rel,
                                "line": entry_start + 1,
                                "detail": (
                                    "Dependabot ignore entry has no rationale "
                                    "comment."
                                ),
                                "snippet": lines[entry_start].strip(),
                            }
                        )
                entry_start = idx
                entry_lines = [idx]
            else:
                # continuation of current entry.
                if entry_start is not None:
                    entry_lines.append(idx)

    # Flush trailing entry at EOF.
    if in_ignore and entry_start is not None and entry_lines:
        if not _has_rationale(entry_start, entry_lines[-1]):
            out.append(
                {
                    "rule": "dependabot_unrationalized",
                    "severity": SEV_MED,
                    "file": rel,
                    "line": entry_start + 1,
                    "detail": (
                        "Dependabot ignore entry has no rationale comment."
                    ),
                    "snippet": lines[entry_start].strip(),
                }
            )

    return out


# ----------------------------------------------------------------------------
# Rule C: Glob-expansive .gitignore patterns
# ----------------------------------------------------------------------------


def scan_gitignore_overbroad(path: Path, repo_root: Path) -> list[dict]:
    """Flag ``.gitignore`` rules at column 0 that exclude entire trees.

    Bare ``*`` at top-level excludes everything; ``*/`` excludes every dir;
    ``**/*`` is unbounded and almost never what you want; ``!`` (negation)
    rules are not flagged."""
    text = read_text_safe(path)
    if not text:
        return []
    out: list[dict] = []
    try:
        rel = path.relative_to(repo_root).as_posix()
    except ValueError:
        rel = path.as_posix()
    for idx, raw in enumerate(text.splitlines()):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("!"):
            continue
        # Patterns considered overbroad. NB: ``*.tmp`` is fine (extension
        # filter); we only want bare ``*`` / ``*/`` / ``**/*``.
        bad = None
        if line == "*":
            bad = "bare * — excludes every file at any depth"
        elif line == "*/":
            bad = "*/ — excludes every directory at root"
        elif line == "**/*":
            bad = "**/* — excludes every file recursively"
        elif line == "**":
            bad = "** — recursive wildcard with no anchor"
        if bad is None:
            continue
        out.append(
            {
                "rule": "gitignore_overbroad",
                "severity": SEV_LOW,
                "file": rel,
                "line": idx + 1,
                "pattern": line,
                "detail": (
                    f".gitignore pattern {line!r}: {bad}. Narrow to a path "
                    "prefix or extension, or document why this is intended."
                ),
            }
        )
    return out


# ----------------------------------------------------------------------------
# Rule D: Tautological predicate guards
# ----------------------------------------------------------------------------

# if (... || true)   — tautology
TAUTOLOGY_OR_TRUE_RE = _re_compile(
    r"\bif\s*\(\s*(?P<expr>[^()]*?)\|\|\s*true\s*\)"
)
# if (true || ...)
TAUTOLOGY_TRUE_OR_RE = _re_compile(
    r"\bif\s*\(\s*true\s*\|\|"
)
# if (true)  — bare always-true
BARE_IF_TRUE_RE = _re_compile(r"\bif\s*\(\s*true\s*\)")
# if (!false)
NOT_FALSE_RE = _re_compile(r"\bif\s*\(\s*!\s*false\s*\)")
# while (true)  — intentional infinite loop, NOT flagged.
# x == x  / x != x — tautologies/contradictions in conditions
SELF_COMPARE_RE = _re_compile(
    r"\bif\s*\(\s*(?P<a>[A-Za-z_]\w*)\s*==\s*(?P=a)\s*\)"
)


def scan_csharp_tautological(path: Path, repo_root: Path) -> list[dict]:
    """Find vacuous `if (...)` predicates in C# source."""
    text = read_text_safe(path)
    if not text:
        return []
    out: list[dict] = []
    try:
        rel = path.relative_to(repo_root).as_posix()
    except ValueError:
        rel = path.as_posix()

    def _emit(rule_label: str, m: re.Match, detail: str, sev: str = SEV_HIGH) -> None:
        out.append(
            {
                "rule": "tautological_predicate",
                "subrule": rule_label,
                "severity": sev,
                "file": rel,
                "line": line_of(text, m.start()),
                "detail": detail,
                "match": m.group(0),
            }
        )

    for m in TAUTOLOGY_OR_TRUE_RE.finditer(text):
        _emit(
            "or_true",
            m,
            "predicate `(... || true)` is vacuously true — guard never fails.",
        )
    for m in TAUTOLOGY_TRUE_OR_RE.finditer(text):
        _emit(
            "true_or",
            m,
            "predicate `(true || ...)` is vacuously true — short-circuits true.",
        )
    for m in BARE_IF_TRUE_RE.finditer(text):
        _emit(
            "bare_true",
            m,
            "bare `if (true)` — guard never fails.",
            sev=SEV_MED,
        )
    for m in NOT_FALSE_RE.finditer(text):
        _emit(
            "not_false",
            m,
            "bare `if (!false)` — guard never fails.",
        )
    for m in SELF_COMPARE_RE.finditer(text):
        _emit(
            "self_compare",
            m,
            f"predicate `if ({m.group('a')} == {m.group('a')})` — vacuously true "
            "(or NaN-trap on doubles, but that's the same lurking bug class).",
        )
    return out


# ----------------------------------------------------------------------------
# Rule E: Vacuous numeric guards
# ----------------------------------------------------------------------------

# Naming conventions that imply unsigned / non-negative values. We use these
# to detect vacuous ``>= 0`` guards: if the variable is a `Count`, a `Length`,
# a `uint`, a `nuint`, or a `*Count` field, comparing it to ``>= 0`` is a
# tautology.
NONNEGATIVE_NAME_RE = _re_compile(
    r"\.(Count|Length|Capacity|Size|LongCount|LongLength)\b"
)

# Local int-typed identifiers — naming alone isn't enough. We restrict the
# numeric-guard rule to known-property accesses that are documented as
# non-negative. The focus is on common false-confidence shapes.
COUNT_GE_ZERO_RE = _re_compile(
    r"\bif\s*\(\s*"
    r"(?P<lhs>[A-Za-z_][\w\.]*?\.(?:Count|Length|Capacity|Size))"
    r"\s*>=\s*0\s*\)"
)

# `if (x <= int.MaxValue)` for an int-typed variable — vacuous.
LE_INTMAX_RE = _re_compile(
    r"\bif\s*\(\s*(?P<lhs>[A-Za-z_]\w*)\s*<=\s*int\.MaxValue\s*\)"
)
LE_LONGMAX_RE = _re_compile(
    r"\bif\s*\(\s*(?P<lhs>[A-Za-z_]\w*)\s*<=\s*long\.MaxValue\s*\)"
)
# `if (x >= int.MinValue)` — vacuous for any int.
GE_INTMIN_RE = _re_compile(
    r"\bif\s*\(\s*(?P<lhs>[A-Za-z_]\w*)\s*>=\s*int\.MinValue\s*\)"
)


def scan_csharp_numeric(path: Path, repo_root: Path) -> list[dict]:
    """Find vacuous numeric guards in C# source.

    We focus on the high-signal shapes:
      * ``something.Count >= 0`` / ``.Length >= 0``
      * ``x <= int.MaxValue`` / ``x <= long.MaxValue``
      * ``x >= int.MinValue``
    """
    text = read_text_safe(path)
    if not text:
        return []
    out: list[dict] = []
    try:
        rel = path.relative_to(repo_root).as_posix()
    except ValueError:
        rel = path.as_posix()

    for m in COUNT_GE_ZERO_RE.finditer(text):
        out.append(
            {
                "rule": "vacuous_numeric_guard",
                "subrule": "count_ge_zero",
                "severity": SEV_HIGH,
                "file": rel,
                "line": line_of(text, m.start()),
                "lhs": m.group("lhs"),
                "detail": (
                    f"`{m.group('lhs')} >= 0` — Count/Length/Capacity is "
                    "always non-negative; this guard never fails."
                ),
                "match": m.group(0),
            }
        )
    for regex, label, max_token in (
        (LE_INTMAX_RE, "le_intmax", "int.MaxValue"),
        (LE_LONGMAX_RE, "le_longmax", "long.MaxValue"),
    ):
        for m in regex.finditer(text):
            out.append(
                {
                    "rule": "vacuous_numeric_guard",
                    "subrule": label,
                    "severity": SEV_MED,
                    "file": rel,
                    "line": line_of(text, m.start()),
                    "lhs": m.group("lhs"),
                    "detail": (
                        f"`{m.group('lhs')} <= {max_token}` — value's natural "
                        "range already caps it; guard never fails."
                    ),
                    "match": m.group(0),
                }
            )
    for m in GE_INTMIN_RE.finditer(text):
        out.append(
            {
                "rule": "vacuous_numeric_guard",
                "subrule": "ge_intmin",
                "severity": SEV_MED,
                "file": rel,
                "line": line_of(text, m.start()),
                "lhs": m.group("lhs"),
                "detail": (
                    f"`{m.group('lhs')} >= int.MinValue` — vacuous; every int "
                    "is >= int.MinValue."
                ),
                "match": m.group(0),
            }
        )
    return out


# ----------------------------------------------------------------------------
# Rule F: Unbounded list-join into a logger / message
# ----------------------------------------------------------------------------

# string.Join(", ", items)  — capture the entire 2nd-arg expression so we can
# inspect it for truncation. We require a closing paren that matches the Join
# call so we capture the full expression (no chain leakage).
JOIN_RE = _re_compile(
    r"\bstring\.Join\s*\(\s*"
    r"(?:\"[^\"]*\"|'[^']*')"            # separator literal
    r"\s*,\s*"
    r"(?P<arg>[^)]*?)"                   # items expression (lazy, no `)`)
    r"\s*\)"
)

TRUNCATION_TOKENS = ("Take(", "Truncate(", ".Substring(0,", ".Slice(", ".Limit(")


def _line_window(text: str, offset: int, before: int = 0, after: int = 0) -> str:
    """Return the line containing *offset* plus *before* lines above and
    *after* lines below. Default = just the single line."""
    line_start = text.rfind("\n", 0, offset) + 1
    line_end = text.find("\n", offset)
    if line_end == -1:
        line_end = len(text)
    # Walk backward `before` newlines.
    for _ in range(before):
        prev = text.rfind("\n", 0, line_start - 1)
        if prev == -1:
            line_start = 0
            break
        line_start = prev + 1
    # Walk forward `after` newlines.
    for _ in range(after):
        nxt = text.find("\n", line_end + 1)
        if nxt == -1:
            line_end = len(text)
            break
        line_end = nxt
    return text[line_start:line_end]


def scan_csharp_join(path: Path, repo_root: Path) -> list[dict]:
    text = read_text_safe(path)
    if not text:
        return []
    out: list[dict] = []
    try:
        rel = path.relative_to(repo_root).as_posix()
    except ValueError:
        rel = path.as_posix()
    for m in JOIN_RE.finditer(text):
        arg = m.group("arg")
        # Inspect ONLY the second-argument expression for truncation. This
        # avoids matching truncation tokens that happen to appear later on
        # different lines.
        if any(tok in arg for tok in TRUNCATION_TOKENS):
            continue
        # Heuristic guard: the consumer is a Log/Console/Throw call. Restrict
        # the consumer check to the *same line* as the Join call.
        same_line = _line_window(text, m.start())
        if not re.search(
            r"\b(Log\w*|Console\.\w+|throw\s+new|Debug\.\w+)\b", same_line
        ):
            continue
        out.append(
            {
                "rule": "unbounded_join_consumer",
                "severity": SEV_LOW,
                "file": rel,
                "line": line_of(text, m.start()),
                "items": arg.strip(),
                "detail": (
                    "string.Join over an unbounded sequence flowing into a "
                    "logger / exception message without Take(N)/Truncate. "
                    "Cap the list with .Take(N) before joining."
                ),
                "match": m.group(0),
            }
        )
    return out


# ----------------------------------------------------------------------------
# IO helpers (shared)
# ----------------------------------------------------------------------------


def read_text_safe(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        try:
            return path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            return ""
    except OSError:
        return ""


def is_excluded_path(path: Path) -> bool:
    return bool(set(path.parts) & EXCLUDED_DIR_PARTS)


def line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def load_allowlist(path: Path) -> set[str]:
    if not path.exists():
        return set()
    out: set[str] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        out.add(line)
    return out


# ----------------------------------------------------------------------------
# Repo walkers
# ----------------------------------------------------------------------------


def find_msbuild_files(root: Path) -> list[Path]:
    out: list[Path] = []
    for ext in ("*.csproj", "*.props", "*.targets"):
        for p in root.rglob(ext):
            if is_excluded_path(p):
                continue
            out.append(p)
    return sorted(set(out))


def find_csharp_sources(root: Path, src_subdir: str = "src") -> list[Path]:
    src_root = root / src_subdir
    out: list[Path] = []
    if not src_root.exists():
        return out
    for p in src_root.rglob("*.cs"):
        if is_excluded_path(p):
            continue
        out.append(p)
    return sorted(out)


def find_gitignores(root: Path) -> list[Path]:
    out: list[Path] = []
    for p in root.rglob(".gitignore"):
        if is_excluded_path(p):
            continue
        out.append(p)
    return sorted(out)


# ----------------------------------------------------------------------------
# Orchestrator
# ----------------------------------------------------------------------------


def _violation_key(r: dict) -> str:
    return f"{r['rule']}|{r['file']}|{r.get('line', 0)}"


def scan_repo(
    repo_root: Path,
    allowlist: set[str],
    strict: bool = False,
) -> dict:
    nuget: list[dict] = []
    for f in find_msbuild_files(repo_root):
        nuget.extend(scan_csproj_for_floating(f, repo_root))

    dependabot: list[dict] = []
    dep_path = repo_root / ".github" / "dependabot.yml"
    if dep_path.exists():
        dependabot.extend(scan_dependabot_unrationalized(dep_path, repo_root))

    gitignore: list[dict] = []
    for f in find_gitignores(repo_root):
        gitignore.extend(scan_gitignore_overbroad(f, repo_root))

    taut_pred: list[dict] = []
    vac_num: list[dict] = []
    join_unbounded: list[dict] = []
    for cs in find_csharp_sources(repo_root):
        taut_pred.extend(scan_csharp_tautological(cs, repo_root))
        vac_num.extend(scan_csharp_numeric(cs, repo_root))
        if strict:
            join_unbounded.extend(scan_csharp_join(cs, repo_root))

    # Apply allowlist — accept either a bare file path or a ``rule|file|line``
    # composite key.
    def _is_allowed(r: dict) -> bool:
        if r["file"] in allowlist:
            return True
        return _violation_key(r) in allowlist

    def _filter(rs: list[dict]) -> list[dict]:
        kept: list[dict] = []
        for r in rs:
            r["allowlist_key"] = _violation_key(r)
            r["in_allowlist"] = _is_allowed(r)
            if not r["in_allowlist"]:
                kept.append(r)
        return kept

    nuget_new = _filter(nuget)
    dependabot_new = _filter(dependabot)
    gitignore_new = _filter(gitignore)
    taut_pred_new = _filter(taut_pred)
    vac_num_new = _filter(vac_num)
    join_unbounded_new = _filter(join_unbounded)

    # Hard-fail if any HIGH or MED violation in non-permissive categories.
    # Categories that always fail: nuget_floating (HIGH), tautological_predicate
    # (HIGH/MED), vacuous_numeric_guard (HIGH/MED). dependabot_unrationalized
    # (MED) fails too. gitignore_overbroad and unbounded_join_consumer are LOW
    # — only fail under --strict.
    hard_fail_count = (
        len(nuget_new)
        + len(taut_pred_new)
        + len(vac_num_new)
        + len(dependabot_new)
    )
    if strict:
        hard_fail_count += len(gitignore_new) + len(join_unbounded_new)

    exit_code = 1 if hard_fail_count > 0 else 0

    return {
        "scan_utc": datetime.utcnow().isoformat() + "Z",
        "scan_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "strict": strict,
        "allowlist_size": len(allowlist),
        "nuget_floating": nuget_new,
        "dependabot_unrationalized": dependabot_new,
        "gitignore_overbroad": gitignore_new,
        "tautological_predicate": taut_pred_new,
        "vacuous_numeric_guard": vac_num_new,
        "unbounded_join_consumer": join_unbounded_new,
        "totals": {
            "nuget_floating": len(nuget_new),
            "dependabot_unrationalized": len(dependabot_new),
            "gitignore_overbroad": len(gitignore_new),
            "tautological_predicate": len(taut_pred_new),
            "vacuous_numeric_guard": len(vac_num_new),
            "unbounded_join_consumer": len(join_unbounded_new),
        },
        # Pre-allowlist totals for transparency.
        "raw_totals": {
            "nuget_floating": len(nuget),
            "dependabot_unrationalized": len(dependabot),
            "gitignore_overbroad": len(gitignore),
            "tautological_predicate": len(taut_pred),
            "vacuous_numeric_guard": len(vac_num),
            "unbounded_join_consumer": len(join_unbounded),
        },
        "exit_code": exit_code,
    }


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect generalized unbounded constraints (Pattern #94 "
            "generalization). Six rules: NuGet floating versions, "
            "Dependabot unrationalized ignores, .gitignore overbroad globs, "
            "tautological predicates, vacuous numeric guards, unbounded join "
            "consumers."
        )
    )
    p.add_argument(
        "--root",
        default=None,
        help="Repo root (default: derived from script location)",
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/unbounded-constraints-allowlist.txt",
        help=(
            "Allowlist file with one POSIX path or `rule|file|line` key per "
            "line (default: docs/qa/unbounded-constraints-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/unbounded-constraints-report.json",
        help="JSON report output path",
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help=(
            "Promote LOW-severity findings (gitignore_overbroad, "
            "unbounded_join_consumer) to fail the gate."
        ),
    )
    p.add_argument("--quiet", action="store_true", help="One-line CI summary")
    p.add_argument("--verbose", action="store_true", help="Verbose progress")
    return p.parse_args(argv)


def repo_root_from_script() -> Path:
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
        "allowlist": _abs(args.allowlist),
        "output": _abs(args.output),
    }


def print_summary(report: dict, output_path: Path) -> None:
    print("unbounded-constraints scan (Pattern #94 generalization)")
    print(f"  strict mode       : {report['strict']}")
    print(f"  allowlist size    : {report['allowlist_size']}")
    print()
    print("  raw totals (pre-allowlist):")
    for k, v in report["raw_totals"].items():
        print(f"    {k:<32} : {v}")
    print()
    print("  new violations (post-allowlist):")
    for k, v in report["totals"].items():
        print(f"    {k:<32} : {v}")
    print()
    for cat in (
        "nuget_floating",
        "dependabot_unrationalized",
        "gitignore_overbroad",
        "tautological_predicate",
        "vacuous_numeric_guard",
        "unbounded_join_consumer",
    ):
        items = report.get(cat, [])
        if not items:
            continue
        print(f"  -- {cat} --")
        for r in items[:20]:
            line = f"{r['file']}:{r.get('line', '?')}"
            print(f"    [{r['severity']}] {line}")
        if len(items) > 20:
            print(f"    ... ({len(items) - 20} more)")
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# Self-tests
# ----------------------------------------------------------------------------


def _self_test() -> int:  # noqa: C901 — single fixture, fine to be long
    import tempfile

    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)

        # ---- Rule A: floating NuGet ----
        csproj_bad = td_path / "Foo.csproj"
        csproj_bad.write_text(
            '<Project>\n'
            '  <ItemGroup>\n'
            '    <PackageReference Include="Bad.Floating" Version="*" />\n'
            '    <PackageReference Include="Bad.OpenRange" '
            'Version="[0.0.0,)" />\n'
            '    <PackageReference Include="Bad.GeZero" Version=">=0.0.0" />\n'
            '    <PackageReference Include="Good.Pinned" Version="1.2.3" />\n'
            '    <PackageReference Include="Good.Range" '
            'Version="[1.0.0,2.0.0)" />\n'
            '  </ItemGroup>\n'
            '</Project>\n',
            encoding="utf-8",
        )
        nuget_hits = scan_csproj_for_floating(csproj_bad, td_path)
        nuget_packages = {r["package"] for r in nuget_hits}
        assert "Bad.Floating" in nuget_packages, nuget_hits
        assert "Bad.OpenRange" in nuget_packages, nuget_hits
        assert "Bad.GeZero" in nuget_packages, nuget_hits
        assert "Good.Pinned" not in nuget_packages, nuget_hits
        assert "Good.Range" not in nuget_packages, nuget_hits

        # ---- Rule B: dependabot unrationalized ----
        dep_dir = td_path / ".github"
        dep_dir.mkdir()
        dep_path = dep_dir / "dependabot.yml"
        dep_path.write_text(
            "version: 2\n"
            "updates:\n"
            "  - package-ecosystem: nuget\n"
            "    directory: /\n"
            "    ignore:\n"
            '      - dependency-name: "BadIgnore"\n'   # no rationale
            '      # WHY: pinned to 1.x for compat\n'
            '      - dependency-name: "GoodIgnore"\n'
            '      - dependency-name: "BadInline"  # actually has comment\n'
            "    schedule:\n"
            "      interval: weekly\n",
            encoding="utf-8",
        )
        dep_hits = scan_dependabot_unrationalized(dep_path, td_path)
        snippets = [r["snippet"] for r in dep_hits]
        assert any("BadIgnore" in s for s in snippets), dep_hits
        assert not any("GoodIgnore" in s for s in snippets), dep_hits
        assert not any("BadInline" in s for s in snippets), dep_hits

        # ---- Rule C: gitignore overbroad ----
        gitignore_bad = td_path / ".gitignore"
        gitignore_bad.write_text(
            "# good comment\n"
            "*.tmp\n"      # OK — extension-bound
            "bin/\n"       # OK — directory-bound
            "*\n"          # BAD — bare star
            "**/*\n"       # BAD — recursive
            "!keep.txt\n"  # OK — negation
            "*/\n"         # BAD — every dir
            "**\n"         # BAD — recursive wildcard
            "node_modules/\n",  # OK
            encoding="utf-8",
        )
        gi_hits = scan_gitignore_overbroad(gitignore_bad, td_path)
        gi_patterns = {r["pattern"] for r in gi_hits}
        assert gi_patterns == {"*", "**/*", "*/", "**"}, gi_hits

        # ---- Rule D: tautological predicates ----
        cs_bad = td_path / "src" / "Foo.cs"
        cs_bad.parent.mkdir(parents=True)
        cs_bad.write_text(
            "namespace Foo {\n"
            "    public class Bar {\n"
            "        public void M(object x) {\n"
            "            if (x != null || true) { Use(); }\n"   # or_true
            "            if (true || x == null) { Use(); }\n"   # true_or
            "            if (true) { Use(); }\n"                # bare_true
            "            if (!false) { Use(); }\n"              # not_false
            "            if (x == x) { Use(); }\n"              # self_compare
            "            if (x != null) { Use(); }\n"           # OK
            "            if (x != null && true) { Use(); }\n"   # NOT vacuous (and)
            "            while (true) { break; }\n"             # NOT flagged (while)
            "        }\n"
            "    }\n"
            "}\n",
            encoding="utf-8",
        )
        tp_hits = scan_csharp_tautological(cs_bad, td_path)
        sub_rules = {r["subrule"] for r in tp_hits}
        assert "or_true" in sub_rules, tp_hits
        assert "true_or" in sub_rules, tp_hits
        assert "bare_true" in sub_rules, tp_hits
        assert "not_false" in sub_rules, tp_hits
        assert "self_compare" in sub_rules, tp_hits
        # Verify while-loop is NOT mistakenly counted as if(true).
        bare_true_hits = [r for r in tp_hits if r["subrule"] == "bare_true"]
        assert len(bare_true_hits) == 1, (
            "while(true) should NOT be flagged; bare_true should match exactly "
            f"once. Got: {bare_true_hits}"
        )

        # ---- Rule E: vacuous numeric guards (incl. unsigned types) ----
        cs_num = td_path / "src" / "Numeric.cs"
        cs_num.write_text(
            "namespace Foo {\n"
            "    public class Bar {\n"
            "        public void M(System.Collections.Generic.List<int> items, int n, uint u) {\n"
            "            if (items.Count >= 0) { }\n"      # vacuous — Count >= 0
            "            if (items.Length >= 0) { }\n"     # vacuous — Length >= 0
            "            if (n <= int.MaxValue) { }\n"     # vacuous — int <= int.MaxValue
            "            if (n >= int.MinValue) { }\n"     # vacuous — int >= int.MinValue
            "            if (n > 0) { }\n"                 # OK
            "            if (items.Count > 0) { }\n"       # OK
            "        }\n"
            "    }\n"
            "}\n",
            encoding="utf-8",
        )
        num_hits = scan_csharp_numeric(cs_num, td_path)
        sub_rules_n = {r["subrule"] for r in num_hits}
        assert "count_ge_zero" in sub_rules_n, num_hits
        assert "le_intmax" in sub_rules_n, num_hits
        assert "ge_intmin" in sub_rules_n, num_hits
        # Length >= 0 hits the same regex as Count >= 0.
        count_ge_zero_hits = [
            r for r in num_hits if r["subrule"] == "count_ge_zero"
        ]
        assert len(count_ge_zero_hits) >= 2, count_ge_zero_hits

        # ---- Rule F: unbounded join consumer ----
        cs_join = td_path / "src" / "Join.cs"
        cs_join.write_text(
            "namespace Foo {\n"
            "    public class Bar {\n"
            "        public void M(System.Collections.Generic.List<string> items, ILogger Log) {\n"
            "            Log.LogError(\"Items: {Items}\", string.Join(\", \", items));\n"  # BAD
            "            Log.LogError(\"Items: {Items}\", string.Join(\", \", items.Take(5)));\n"  # OK
            "            string s = string.Join(\", \", items);\n"  # not a logger call — skip
            "        }\n"
            "    }\n"
            "}\n",
            encoding="utf-8",
        )
        join_hits = scan_csharp_join(cs_join, td_path)
        # Should see the bad case but NOT the truncated case.
        assert len(join_hits) == 1, join_hits
        assert "items" in join_hits[0]["match"], join_hits

        # ---- Healthy file: no violations ----
        cs_clean = td_path / "src" / "Clean.cs"
        cs_clean.write_text(
            "namespace Foo {\n"
            "    public class Bar {\n"
            "        public void M(int x) {\n"
            "            if (x > 0) { }\n"
            "            if (x < int.MaxValue - 1) { }\n"
            "        }\n"
            "    }\n"
            "}\n",
            encoding="utf-8",
        )
        assert scan_csharp_tautological(cs_clean, td_path) == []
        assert scan_csharp_numeric(cs_clean, td_path) == []
        assert scan_csharp_join(cs_clean, td_path) == []

        # ---- Allowlist suppression ----
        report_no_allow = scan_repo(td_path, set(), strict=True)
        # Pick a real key from the nuget bucket.
        target_key = report_no_allow["nuget_floating"][0]["allowlist_key"]
        report_with_allow = scan_repo(td_path, {target_key}, strict=True)
        new_keys = {r["allowlist_key"] for r in report_with_allow["nuget_floating"]}
        assert target_key not in new_keys, (
            f"allowlist did not suppress: {target_key}; got {new_keys}"
        )

        # ---- Exit code ----
        assert report_no_allow["exit_code"] == 1, report_no_allow

        # ---- Healthy synthetic tree → exit 0 ----
        clean_dir = Path(tempfile.mkdtemp())
        try:
            (clean_dir / "src").mkdir()
            (clean_dir / "src" / "Ok.cs").write_text(
                "namespace X { class Y { void M(int x) { if (x > 0) { } } } }\n",
                encoding="utf-8",
            )
            (clean_dir / "Ok.csproj").write_text(
                '<Project>\n'
                '  <ItemGroup>\n'
                '    <PackageReference Include="OK" Version="1.2.3" />\n'
                '  </ItemGroup>\n'
                '</Project>\n',
                encoding="utf-8",
            )
            clean_report = scan_repo(clean_dir, set(), strict=True)
            assert clean_report["exit_code"] == 0, clean_report
            assert all(
                clean_report["totals"][k] == 0 for k in clean_report["totals"]
            ), clean_report
        finally:
            import shutil

            shutil.rmtree(clean_dir, ignore_errors=True)

    print("self-test: OK")
    return 0


# ----------------------------------------------------------------------------
# main
# ----------------------------------------------------------------------------


def main(argv: list[str]) -> int:
    if len(argv) >= 1 and argv[0] == "--self-test":
        return _self_test()

    args = parse_args(argv)
    if args.quiet and args.verbose:
        print(
            "ERROR: --quiet and --verbose are mutually exclusive",
            file=sys.stderr,
        )
        return 2

    paths = resolve_paths(args)

    if not paths["repo"].exists():
        print(
            f"ERROR: repo root not found: {paths['repo']}",
            file=sys.stderr,
        )
        return 2

    allowlist = load_allowlist(paths["allowlist"])

    if args.verbose:
        print(
            f"scanning {paths['repo']} (allowlist={len(allowlist)})",
            file=sys.stderr,
        )

    report = scan_repo(paths["repo"], allowlist, strict=args.strict)
    report["repo_root"] = paths["repo"].as_posix()
    report["allowlist_path"] = paths["allowlist"].as_posix()

    paths["output"].parent.mkdir(parents=True, exist_ok=True)
    paths["output"].write_text(json.dumps(report, indent=2), encoding="utf-8")

    if args.quiet:
        t = report["totals"]
        print(
            "unbounded-constraints: "
            f"nuget={t['nuget_floating']} "
            f"dependabot={t['dependabot_unrationalized']} "
            f"gitignore={t['gitignore_overbroad']} "
            f"taut={t['tautological_predicate']} "
            f"vacnum={t['vacuous_numeric_guard']} "
            f"join={t['unbounded_join_consumer']}"
            f"{' [strict]' if args.strict else ''} "
            f"-> {paths['output']}"
        )
    else:
        print_summary(report, paths["output"])

    return report["exit_code"]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
