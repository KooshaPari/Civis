#!/usr/bin/env python3
"""Stringly-Typed Enum Discriminator detector — Pattern #101 CI gate.

Pattern #101 ("stringly-typed enum discriminator on a model class") is
the failure mode where a C# data model exposes a field whose *name*
strongly implies a small closed set of values (``Mode``, ``Status``,
``Phase``, ``Kind``, ``Category``, ``Type``, ``State``) but whose
*type* is ``string``:

    public class WinConditionDefinition
    {
        // BAD: silent typo. "elimnate_faction" parses fine, evaluates
        // to "never", and the scenario silently never wins.
        public string Type { get; set; } = "";

        // BAD: "agressive" / "balanced " (trailing space) etc.
        public string LODStrategy { get; set; } = "aggressive";
    }

This is a problem for several reasons:

  * **Silent typos**. ``"elimnate_faction"`` looks like
    ``"eliminate_faction"`` to a human reviewer; the dispatcher just
    falls through to the default branch and the scenario never ends.
  * **No IDE / compiler help**. Refactoring a value (rename
    ``"reach_pop"`` → ``"reach_population"``) needs a full repo-wide
    string sweep; missing a call site is silent until runtime.
  * **No exhaustiveness**. A ``switch`` on the string can't be checked
    for exhaustiveness; new values added later are silently ignored
    until someone hits the default branch.
  * **Schema drift**. The same field needs to enumerate the closed set
    in JSON Schema (``enum: [...]``) AND in code; they drift.

The healthy pattern is one of:

  1. ``enum`` type + JSON converter (``[JsonConverter(typeof(StringEnumConverter))]``).
  2. Closed ``KnownXxx`` HashSet + ``IValidatable.Validate()`` that
     rejects unknown values at deserialize-time.

This gate scans MODEL files for ``public string`` fields whose name
suggests a closed set, classifies each by whether the surrounding
class constrains the value, and flags the gap:

  * **HIGH** — neither a ``KnownXxx`` HashSet nor an
    ``IValidatable.Validate()`` exists in the surrounding class. The
    field is unconstrained at deserialize-time; a typo is a silent
    runtime fault.
  * **MED**  — a ``KnownXxx`` HashSet exists OR ``IValidatable``
    Validate() exists. The value is rejected at runtime, but the type
    system still accepts any string. Migration to a real enum is
    desirable for IDE / compiler support but not gating.

Auto-skipped sites (zero recorded as hits):

  * Test files (``src/Tests/``) — fixture builders intentionally
    poke at string discriminators.
  * Avalonia GUI (``src/Tools/Installer/GUI/``) — XAML view-models
    bind to free-text strings.
  * Free-text-by-name allowlist: ``Author``, ``Description``,
    ``Title``, ``Notes``, ``Comment``, ``Message``, ``Reason``,
    ``Summary``, ``Detail``, ``Url`` — these contain "Type"-ish or
    "State"-ish substrings only by coincidence.
  * Trailing comment ``// pattern-101-allowed: <reason>`` blesses a
    site with rationale.
  * File path in ``docs/qa/stringly-enums-allowlist.txt``.

Allowlisting (one entry per line, ``#`` for comments):
``docs/qa/stringly-enums-allowlist.txt``. Two entry forms:

  1. ``severity|file|line`` — line-locked allowlist key.
  2. Bare ``relative/path.cs`` — suppress every site in that file.

CLI:
    python scripts/ci/detect_stringly_enums.py
        [--root <src/>]
        [--allowlist <path>]
        [--output <json>]
        [--strict]
        [--quiet|--verbose]
        [--self-test]

Exit 0 = no NEW (un-allowlisted) HIGH hits; 1 = new violations
detected (CI fails); 2 = scan/usage error. ``--strict`` promotes MED
findings to fail the gate as well.

Modeled on ``scripts/ci/detect_direct_datetime.py`` (#286) and
``scripts/ci/detect_missing_configureawait.py`` (#275). Pairs with the
enum-migration sweep being driven by #289 / #290.

This is task #291.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path


# ----------------------------------------------------------------------------
# Regex catalogue
# ----------------------------------------------------------------------------

# Match a ``public string`` field/property whose name ends in one of the
# closed-set suffixes. Tolerates ``required`` modifier. The terminator
# discriminates between auto-property (``{``), assignment (``=``), and
# field declaration (``;``).
ENUMISH_SUFFIXES = (
    "Type", "Mode", "Status", "Phase", "Kind", "Category", "State",
)
STRINGLY_RE = re.compile(
    r"public\s+(?:required\s+)?string\s+"
    r"(?P<name>\w*(?:Type|Mode|Status|Phase|Kind|Category|State))"
    r"\s*[=;{]"
)

# Trailing-comment opt-out token. Anywhere on the same line as the
# match, suppresses the hit.
PATTERN_101_ALLOWED_RE = re.compile(r"//\s*pattern-101-allowed\b")

# Free-text-by-name auto-skip. These property names contain a
# closed-set suffix only by accident (e.g. "State" inside "RealEstate"
# would be regex-rejected by word boundary, but "Status" inside
# "AuthorStatus" wouldn't — we don't worry about those because the
# property name needs to *end* in the suffix and our regex enforces
# that). The list below is the small set of legitimate free-text fields
# whose name ends in an enum-ish suffix purely by domain accident.
FREE_TEXT_NAME_ALLOWLIST = frozenset({
    # No common free-text field ends in Type/Mode/Status/etc.; this is
    # primarily a future-proofing slot. Add domain free-text keys here
    # if a false positive shows up. Note that ``Author`` /
    # ``Description`` / ``Title`` etc. don't end in any enum-ish
    # suffix so they don't even match the regex.
})

# KnownXxx HashSet declarations within the surrounding class. We accept
# any HashSet<string> / IReadOnlySet<string> / ImmutableHashSet<string>
# / string[] field whose name starts with "Known" and ends in a plural
# (or singular) form of the enum-ish suffix. This is intentionally
# permissive — a class with ANY ``Known`` constraint set scoped to
# ``string`` is treated as constraining the value.
KNOWN_VALUES_RE = re.compile(
    r"\b(?:public|private|internal|protected)\s+"
    r"(?:static\s+)?(?:readonly\s+)?(?:static\s+)?"
    r"(?:HashSet<string>|IReadOnlySet<string>|ISet<string>|"
    r"ImmutableHashSet<string>|FrozenSet<string>|"
    r"string\s*\[\s*\]|IReadOnlyCollection<string>|"
    r"IReadOnlyList<string>|IList<string>)\s+"
    r"(?P<known_name>Known\w+|Valid\w+)\b"
)

# IValidatable.Validate() method on the surrounding class. Approximate
# by ":-clause inheriting IValidatable" or a method body like
# ``ValidationResult Validate()``. We don't require strict type
# inheritance because the marker interface itself is enough signal.
IVALIDATABLE_INHERIT_RE = re.compile(r":\s*[^{}\n]*\bIValidatable\b")
IVALIDATABLE_METHOD_RE = re.compile(
    r"\b(?:public|internal)\s+(?:override\s+|virtual\s+|sealed\s+)?"
    r"ValidationResult\s+Validate\s*\("
)

# Default scan root.
DEFAULT_SCAN_ROOT = "src"

# Path fragments (POSIX-relative, lowercased) that scope the scan.
# Order matters: a file outside these scopes is auto-skipped.
SCAN_BOUNDARY_PARTS = (
    "src/sdk/models/",
    "src/domains/",  # any src/Domains/<X>/Models/ matches
    "src/bridge/protocol/",
    "src/tools/packcompiler/models/",
)
# Fine-grained Domain Models filter — only Domains/*/Models/.
DOMAIN_MODEL_RE = re.compile(r"src/domains/[^/]+/models/", re.IGNORECASE)

# Path fragments that mean "do not scan this file at all".
SKIP_BOUNDARY_PARTS = (
    "src/tests/",
    "src/tools/installer/gui/",
)

EXCLUDED_DIR_PARTS = {"bin", "obj", "node_modules", ".git"}

SEV_HIGH = "HIGH"
SEV_MED = "MED"


# ----------------------------------------------------------------------------
# IO helpers
# ----------------------------------------------------------------------------


def is_excluded_path(path: Path) -> bool:
    return bool(set(path.parts) & EXCLUDED_DIR_PARTS)


def read_text_safe(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


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
# Detection
# ----------------------------------------------------------------------------


def _path_skipped(rel_posix: str) -> bool:
    lo = rel_posix.lower()
    return any(p in lo for p in SKIP_BOUNDARY_PARTS)


def _path_in_scope(rel_posix: str) -> bool:
    """Is *rel_posix* under one of the model-bearing scan roots? Tests
    and GUI are filtered earlier; this is the positive scope filter."""
    lo = rel_posix.lower()
    if any(p in lo for p in SCAN_BOUNDARY_PARTS if not p.startswith("src/domains/")):
        return True
    # Domains/<X>/Models/ requires the regex (boundary parts is a
    # plain substring match and ``src/domains/`` would over-match).
    if DOMAIN_MODEL_RE.search(lo):
        return True
    return False


def _line_text(text: str, offset: int) -> str:
    """Return the text of the source line containing *offset*."""
    line_start = text.rfind("\n", 0, offset) + 1
    line_end = text.find("\n", offset)
    if line_end == -1:
        line_end = len(text)
    return text[line_start:line_end]


def _line_carries_optout(text: str, offset: int) -> bool:
    """Return True if the source line containing *offset* carries the
    pattern-101-allowed trailing comment."""
    line = _line_text(text, offset)
    return bool(PATTERN_101_ALLOWED_RE.search(line))


def _is_free_text_name(name: str) -> bool:
    """Free-text name allowlist (currently empty by design — kept as a
    future-proofing slot for domain false positives)."""
    return name in FREE_TEXT_NAME_ALLOWLIST


def _enclosing_class_span(text: str, offset: int) -> tuple[int, int] | None:
    """Walk outward from *offset* to find the ``{ ... }`` span of the
    enclosing ``class``/``struct``/``record`` declaration. Returns
    ``(start, end)`` indices into *text* or ``None`` if no enclosing
    type is found.

    Heuristic: locate the nearest preceding ``class|struct|record Name``
    declaration, then find its opening ``{`` and the matching closing
    ``}`` via brace-depth counting (string/char literals are skipped so
    braces inside them don't break the walker).
    """
    head = text[:offset]
    type_decl_re = re.compile(
        r"\b(class|struct|record)\s+[A-Za-z_][A-Za-z0-9_]*",
    )
    last = None
    for m in type_decl_re.finditer(head):
        last = m
    if last is None:
        return None

    # Find the opening ``{`` after the type-decl.
    i = last.end()
    end = len(text)
    while i < end and text[i] != "{":
        i += 1
    if i >= end:
        return None
    open_brace = i
    depth = 0
    in_string = False
    in_verbatim = False
    in_interp_string = False
    in_char = False
    in_line_comment = False
    in_block_comment = False
    while i < end:
        c = text[i]
        n = text[i + 1] if i + 1 < end else ""
        if in_line_comment:
            if c == "\n":
                in_line_comment = False
        elif in_block_comment:
            if c == "*" and n == "/":
                in_block_comment = False
                i += 2
                continue
        elif in_verbatim:
            if c == '"':
                if n == '"':
                    i += 2
                    continue
                in_verbatim = False
        elif in_string or in_interp_string:
            if c == "\\" and n:
                i += 2
                continue
            if c == '"':
                in_string = False
                in_interp_string = False
        elif in_char:
            if c == "\\" and n:
                i += 2
                continue
            if c == "'":
                in_char = False
        else:
            if c == "/" and n == "/":
                in_line_comment = True
                i += 2
                continue
            if c == "/" and n == "*":
                in_block_comment = True
                i += 2
                continue
            if c == "@" and n == '"':
                in_verbatim = True
                i += 2
                continue
            if c == "$" and n == '"':
                in_interp_string = True
                i += 2
                continue
            if c == '"':
                in_string = True
            elif c == "'":
                in_char = True
            elif c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    return open_brace, i
        i += 1
    return None


def _class_has_known_values(text: str, span: tuple[int, int]) -> bool:
    """Does the enclosing class body contain a ``KnownXxx`` /
    ``ValidXxx`` constraint set?"""
    body = text[span[0]:span[1] + 1]
    return bool(KNOWN_VALUES_RE.search(body))


def _class_has_ivalidatable(text: str, span: tuple[int, int]) -> bool:
    """Does the enclosing class implement ``IValidatable`` or expose a
    ``ValidationResult Validate()`` method?

    Note: the inheritance clause sits BEFORE the opening brace — we
    need a wider window than just the body. We pull the ~300 chars
    preceding ``span[0]`` to capture the ``: IValidatable`` clause."""
    body = text[span[0]:span[1] + 1]
    # Inheritance clause window: from a reasonable point before the
    # opening brace.
    pre_start = max(0, span[0] - 400)
    pre = text[pre_start:span[0]]
    if IVALIDATABLE_INHERIT_RE.search(pre):
        return True
    if IVALIDATABLE_METHOD_RE.search(body):
        return True
    return False


@dataclass
class Hit:
    file: str            # POSIX-relative to repo root
    line: int
    name: str            # property name (e.g. "Mode", "ResourceType")
    severity: str
    detail: str
    has_known_values: bool
    has_ivalidatable: bool
    snippet: str         # short slice of the source line
    allowlist_key: str = field(default="")
    in_allowlist: bool = field(default=False)


def _classify_match(
    text: str,
    offset: int,
) -> tuple[str, bool, bool]:
    """Return (severity, has_known_values, has_ivalidatable) for the
    match at *offset*."""
    span = _enclosing_class_span(text, offset)
    if span is None:
        # No enclosing class — probably a top-level record or a parse
        # quirk; treat as HIGH (no constraint visible).
        return SEV_HIGH, False, False
    has_known = _class_has_known_values(text, span)
    has_valid = _class_has_ivalidatable(text, span)
    if has_known or has_valid:
        return SEV_MED, has_known, has_valid
    return SEV_HIGH, has_known, has_valid


def _scan_one_match(
    text: str,
    rel: str,
    m: re.Match,
) -> Hit | None:
    offset = m.start()
    name = m.group("name")

    # Free-text name allowlist (auto-skip).
    if _is_free_text_name(name):
        return None

    # Trailing-comment opt-out.
    if _line_carries_optout(text, offset):
        return None

    severity, has_known, has_valid = _classify_match(text, offset)

    line = line_of(text, offset)
    line_body = _line_text(text, offset).strip()
    snippet = line_body if len(line_body) <= 120 else line_body[:117] + "..."

    if severity == SEV_HIGH:
        detail = (
            f"public string {name} in {rel} has an enum-ish name but "
            f"no KnownXxx HashSet and no IValidatable.Validate() — "
            f"silent typo risk. Migrate to an enum + JsonConverter or "
            f"add a KnownXxx constraint set + IValidatable Validate() "
            f"that rejects unknown values at deserialize-time."
        )
    else:  # MED
        why = []
        if has_known:
            why.append("KnownXxx constraint set present")
        if has_valid:
            why.append("IValidatable.Validate() present")
        detail = (
            f"public string {name} in {rel} is constrained at runtime "
            f"({'; '.join(why)}) but the type system still accepts any "
            f"string. Migration to a real enum + JsonConverter is "
            f"recommended for IDE/compiler support; not gating."
        )

    return Hit(
        file=rel,
        line=line,
        name=name,
        severity=severity,
        detail=detail,
        has_known_values=has_known,
        has_ivalidatable=has_valid,
        snippet=snippet,
    )


def scan_file(path: Path, repo_root: Path) -> list[Hit]:
    text = read_text_safe(path)
    if not text:
        return []
    rel = path.relative_to(repo_root).as_posix()
    if _path_skipped(rel):
        return []
    if not _path_in_scope(rel):
        return []
    hits: list[Hit] = []
    for m in STRINGLY_RE.finditer(text):
        h = _scan_one_match(text, rel, m)
        if h is not None:
            hits.append(h)
    return hits


def enumerate_target_files(repo_root: Path, root: str) -> list[Path]:
    rp = (repo_root / root).resolve()
    if not rp.exists():
        return []
    out: list[Path] = []
    for cs in rp.rglob("*.cs"):
        if is_excluded_path(cs):
            continue
        out.append(cs)
    out.sort()
    return out


def scan_root(repo_root: Path, root: str) -> tuple[list[Hit], int]:
    files = enumerate_target_files(repo_root, root)
    hits: list[Hit] = []
    scanned = 0
    for f in files:
        rel = f.relative_to(repo_root).as_posix()
        if _path_skipped(rel):
            continue
        if not _path_in_scope(rel):
            continue
        scanned += 1
        hits.extend(scan_file(f, repo_root))
    return hits, scanned


# ----------------------------------------------------------------------------
# Report
# ----------------------------------------------------------------------------


def _hit_key(h: Hit) -> str:
    """Stable allowlist key: ``severity|file|line``."""
    return f"{h.severity}|{h.file}|{h.line}"


def apply_allowlist(hits: list[Hit], allowlist: set[str]) -> list[Hit]:
    new_hits: list[Hit] = []
    for h in hits:
        key = _hit_key(h)
        in_allow = (
            key in allowlist
            or h.file in allowlist
        )
        h.allowlist_key = key
        h.in_allowlist = in_allow
        if not in_allow:
            new_hits.append(h)
    return new_hits


def _h2d(h: Hit) -> dict:
    return {
        "file": h.file,
        "line": h.line,
        "name": h.name,
        "severity": h.severity,
        "has_known_values": h.has_known_values,
        "has_ivalidatable": h.has_ivalidatable,
        "snippet": h.snippet,
        "detail": h.detail,
        "allowlist_key": h.allowlist_key,
        "in_allowlist": h.in_allowlist,
    }


def build_report(
    hits: list[Hit],
    allowlist: set[str],
    files_scanned: int,
    strict: bool = False,
) -> dict:
    new_hits = apply_allowlist(hits, allowlist)

    high_violations = sorted(
        [h for h in new_hits if h.severity == SEV_HIGH],
        key=lambda h: (h.file, h.line),
    )
    med_violations = sorted(
        [h for h in new_hits if h.severity == SEV_MED],
        key=lambda h: (h.file, h.line),
    )

    high_count = len(high_violations)
    med_count = len(med_violations)
    fail = high_count > 0 or (strict and med_count > 0)
    exit_code = 1 if fail else 0

    return {
        "scan_utc": datetime.utcnow().isoformat() + "Z",
        "scan_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "scanned_files": files_scanned,
        "files_scanned": files_scanned,  # alias for cross-gate consistency
        "total_hits": len(hits),
        "new_hits": len(new_hits),
        "allowlist_size": len(allowlist),
        "strict": strict,
        "high_count": high_count,
        "med_count": med_count,
        "high_violations": [_h2d(h) for h in high_violations],
        "med_violations": [_h2d(h) for h in med_violations],
        "all_hits": [_h2d(h) for h in hits],
        "exit_code": exit_code,
    }


def print_summary(report: dict, output_path: Path) -> None:
    print("stringly-enums gate (Pattern #101)")
    print(f"  files scanned          : {report['scanned_files']}")
    print(f"  total hits             : {report['total_hits']}")
    print(f"  allowlist size         : {report['allowlist_size']}")
    print(f"  NEW hits               : {report['new_hits']}")
    print(f"    HIGH (no constraint)       : {report['high_count']}")
    print(f"    MED  (Known/IValidatable)  : {report['med_count']}")
    if report["new_hits"]:
        print()
        print("NEW stringly-enum sites:")
        for sev, items in (
            ("HIGH", report["high_violations"]),
            ("MED", report["med_violations"]),
        ):
            if not items:
                continue
            print(f"  -- {sev} --")
            for h in items:
                print(
                    f"    [{h['severity']}] {h['file']}:{h['line']}  "
                    f"public string {h['name']}"
                )
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect stringly-typed enum discriminators on model classes "
            "(Pattern #101). HIGH = ``public string <Name>(Type|Mode|"
            "Status|Phase|Kind|Category|State)`` with no KnownXxx "
            "HashSet and no IValidatable.Validate() in the surrounding "
            "class. MED = constrained at runtime but still string-typed. "
            "Tests and Avalonia GUI are auto-skipped."
        )
    )
    p.add_argument(
        "--root",
        default=DEFAULT_SCAN_ROOT,
        help=f"Source root to scan (default: {DEFAULT_SCAN_ROOT})",
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/stringly-enums-allowlist.txt",
        help=(
            "Allowlist file; one ``severity|file|line`` key or "
            "bare relative path per line; ``#`` for comments "
            "(default: docs/qa/stringly-enums-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/stringly-enums-report.json",
        help="JSON report output path",
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help=(
            "Promote MED (Known/IValidatable-constrained) findings to "
            "fail the gate. Default: only HIGH fails."
        ),
    )
    p.add_argument("--quiet", action="store_true", help="One-line CI summary")
    p.add_argument("--verbose", action="store_true", help="Verbose progress")
    return p.parse_args(argv)


def repo_root_from_script() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_paths(args: argparse.Namespace) -> tuple[Path, Path, Path]:
    repo = repo_root_from_script()

    def _abs(p: str) -> Path:
        path = Path(p)
        return path if path.is_absolute() else (repo / path).resolve()

    return repo, _abs(args.allowlist), _abs(args.output)


# ----------------------------------------------------------------------------
# Self-tests
# ----------------------------------------------------------------------------


_FIXTURE_HIGH_BARE = """
namespace DINOForge.Fixture.Models
{
    public class WinCondition
    {
        // No KnownXxx, no IValidatable -> HIGH.
        public string Mode { get; set; } = "";
        public string Type { get; set; } = "";
    }
}
"""

_FIXTURE_MED_KNOWN_VALUES = """
namespace DINOForge.Fixture.Models
{
    using System.Collections.Generic;

    public class ResourceRate
    {
        // KnownXxx HashSet present -> MED.
        public string ResourceType { get; set; } = "";

        public static readonly HashSet<string> KnownResourceTypes =
            new HashSet<string> { "food", "wood", "stone", "iron", "gold" };
    }
}
"""

_FIXTURE_MED_IVALIDATABLE = """
namespace DINOForge.Fixture.Models
{
    using DINOForge.SDK.Validation;

    public class GuardedSkill : IValidatable
    {
        // IValidatable inheritance present -> MED.
        public string TargetType { get; set; } = "";

        public ValidationResult Validate() => ValidationResult.Success;
    }
}
"""

_FIXTURE_MED_VALIDATE_METHOD = """
namespace DINOForge.Fixture.Models
{
    public class ManualValidate
    {
        // Validate() method body present -> MED (even without inherit).
        public string PolicyKind { get; set; } = "";

        public ValidationResult Validate()
        {
            // body
            return new ValidationResult();
        }
    }
}
"""

_FIXTURE_FREE_TEXT_BY_NAME = """
namespace DINOForge.Fixture.Models
{
    public class PackManifest
    {
        // Author / Description / Title — these don't END in any
        // enum-ish suffix so they don't even match the regex; this
        // fixture exists to confirm the regex doesn't pick them up.
        public string Author { get; set; } = "";
        public string Description { get; set; } = "";
        public string Title { get; set; } = "";
    }
}
"""

_FIXTURE_PATTERN_ALLOWED_COMMENT = """
namespace DINOForge.Fixture.Models
{
    public class CommentBlessed
    {
        // Trailing pattern-101-allowed comment opts out.
        public string LegacyMode { get; set; } = ""; // pattern-101-allowed: schema-frozen
    }
}
"""

_FIXTURE_SKIP_TESTS = """
namespace DINOForge.Fixture.Tests
{
    public class TestFixture
    {
        // Tests scope is auto-skipped.
        public string Mode { get; set; } = "";
    }
}
"""

_FIXTURE_SKIP_GUI = """
namespace DINOForge.Fixture.Gui
{
    public class WelcomeViewModel
    {
        // GUI scope is auto-skipped.
        public string Mode { get; set; } = "";
    }
}
"""

_FIXTURE_OUT_OF_SCOPE = """
namespace DINOForge.Fixture.Random
{
    public class OutOfScopeClass
    {
        // Out-of-scope path (not a Models/ directory) -> not scanned.
        public string Mode { get; set; } = "";
    }
}
"""


def _build_fixture_repo(td: Path) -> Path:
    """Lay out a synthetic repo under *td* matching the gate's expected
    structure: SDK/Models, Domains/<X>/Models, Bridge/Protocol, plus
    auto-skipped (Tests, GUI) and out-of-scope buckets."""
    repo = td / "repo"
    sdk_models = repo / "src" / "SDK" / "Models"
    domains_models = repo / "src" / "Domains" / "Economy" / "Models"
    bridge_proto = repo / "src" / "Bridge" / "Protocol"
    pc_models = repo / "src" / "Tools" / "PackCompiler" / "Models"
    sdk_iv = repo / "src" / "SDK" / "Models"  # reused
    sdk_manual = repo / "src" / "SDK" / "Models"  # reused
    sdk_freetext = repo / "src" / "SDK" / "Models"  # reused
    sdk_blessed = repo / "src" / "SDK" / "Models"  # reused
    tests = repo / "src" / "Tests"
    gui = repo / "src" / "Tools" / "Installer" / "GUI"
    oos = repo / "src" / "Tools" / "Cli"  # out-of-scope (no /Models/)
    for d in (sdk_models, domains_models, bridge_proto, pc_models,
              tests, gui, oos):
        d.mkdir(parents=True, exist_ok=True)

    (sdk_models / "WinCondition.cs").write_text(
        _FIXTURE_HIGH_BARE, encoding="utf-8"
    )
    (domains_models / "ResourceRate.cs").write_text(
        _FIXTURE_MED_KNOWN_VALUES, encoding="utf-8"
    )
    (sdk_iv / "GuardedSkill.cs").write_text(
        _FIXTURE_MED_IVALIDATABLE, encoding="utf-8"
    )
    (sdk_manual / "ManualValidate.cs").write_text(
        _FIXTURE_MED_VALIDATE_METHOD, encoding="utf-8"
    )
    (sdk_freetext / "PackManifest.cs").write_text(
        _FIXTURE_FREE_TEXT_BY_NAME, encoding="utf-8"
    )
    (sdk_blessed / "CommentBlessed.cs").write_text(
        _FIXTURE_PATTERN_ALLOWED_COMMENT, encoding="utf-8"
    )
    (tests / "TestFixture.cs").write_text(
        _FIXTURE_SKIP_TESTS, encoding="utf-8"
    )
    (gui / "WelcomeViewModel.cs").write_text(
        _FIXTURE_SKIP_GUI, encoding="utf-8"
    )
    (oos / "OutOfScopeClass.cs").write_text(
        _FIXTURE_OUT_OF_SCOPE, encoding="utf-8"
    )

    return repo


def _self_test() -> int:  # noqa: C901
    import tempfile

    # 1) Regex sanity — direct string checks.
    assert STRINGLY_RE.search('public string Mode { get; set; } = "";')
    assert STRINGLY_RE.search('public string ResourceType { get; set; } = "";')
    assert STRINGLY_RE.search('public required string ConditionType { get; init; }')
    assert STRINGLY_RE.search('public string Mode;')
    assert STRINGLY_RE.search('public string Mode = "x";')
    # ``public string Description`` should NOT match (doesn't end in any
    # enum-ish suffix).
    assert not STRINGLY_RE.search('public string Description { get; set; } = "";')
    assert not STRINGLY_RE.search('public string Author { get; set; } = "";')
    assert not STRINGLY_RE.search('public string Title { get; set; } = "";')
    # ``public string Estate`` would match because it ends in "State"...
    # but only via the literal suffix; word boundary is enforced by the
    # regex's ``\w+`` capture, which is fine — "Estate" is a real
    # English word with "State" embedded but our regex IS happy to
    # match it. Document that and add to FREE_TEXT_NAME_ALLOWLIST if
    # it ever shows up. For now we accept the false-positive risk
    # because no such field exists in the repo (audited 2026-04-26).
    # ``public int Mode`` should NOT match (different type).
    assert not STRINGLY_RE.search('public int Mode { get; set; }')
    # Identifier with the suffix in the middle — should NOT match.
    assert not STRINGLY_RE.search('public string Modes { get; set; }')

    # KnownValues regex.
    assert KNOWN_VALUES_RE.search(
        "public static readonly HashSet<string> KnownResourceTypes ="
    )
    assert KNOWN_VALUES_RE.search(
        "private static readonly string[] ValidResourceTypes = ..."
    )
    assert KNOWN_VALUES_RE.search(
        "public static readonly FrozenSet<string> KnownTargets = ..."
    )

    # IValidatable inheritance.
    assert IVALIDATABLE_INHERIT_RE.search(
        "public class GuardedSkill : IValidatable"
    )
    assert IVALIDATABLE_INHERIT_RE.search(
        "public class GuardedSkill : SomeBase, IValidatable"
    )
    # IValidatable method.
    assert IVALIDATABLE_METHOD_RE.search(
        "public ValidationResult Validate("
    )
    assert IVALIDATABLE_METHOD_RE.search(
        "public override ValidationResult Validate("
    )

    # Trailing-comment opt-out.
    assert PATTERN_101_ALLOWED_RE.search(
        'public string Mode { get; set; } = ""; // pattern-101-allowed: schema-frozen'
    )

    # 2) End-to-end scan against the synthetic repo.
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        repo = _build_fixture_repo(td_path)

        hits, n_files = scan_root(repo, "src")
        files_seen = {h.file for h in hits}

        # Auto-skipped: tests, GUI, out-of-scope, free-text, blessed.
        assert not any(
            "TestFixture.cs" in f for f in files_seen
        ), f"Tests scope leaked: {files_seen}"
        assert not any(
            "WelcomeViewModel.cs" in f for f in files_seen
        ), f"GUI scope leaked: {files_seen}"
        assert not any(
            "OutOfScopeClass.cs" in f for f in files_seen
        ), f"Out-of-scope path leaked: {files_seen}"
        assert not any(
            "CommentBlessed.cs" in f for f in files_seen
        ), f"// pattern-101-allowed not honored: {files_seen}"
        assert not any(
            "PackManifest.cs" in f for f in files_seen
        ), (
            f"free-text-by-name fields incorrectly flagged: "
            f"{files_seen}"
        )

        # HIGH: WinCondition.cs (Mode + Type, no constraint).
        high = [h for h in hits if h.severity == SEV_HIGH]
        high_files = {h.file for h in high}
        assert any(
            "WinCondition.cs" in f for f in high_files
        ), f"missing HIGH for WinCondition.cs: {high_files}"
        # Both Mode and Type matches under WinCondition.cs.
        wc_hits = [h for h in high if "WinCondition.cs" in h.file]
        wc_names = {h.name for h in wc_hits}
        assert "Mode" in wc_names, wc_names
        assert "Type" in wc_names, wc_names

        # MED: ResourceRate.cs (KnownResourceTypes), GuardedSkill.cs
        # (IValidatable inherit), ManualValidate.cs (Validate method).
        med = [h for h in hits if h.severity == SEV_MED]
        med_files = {h.file for h in med}
        assert any(
            "ResourceRate.cs" in f for f in med_files
        ), f"missing MED for ResourceRate.cs: {med_files}"
        assert any(
            "GuardedSkill.cs" in f for f in med_files
        ), f"missing MED for GuardedSkill.cs: {med_files}"
        assert any(
            "ManualValidate.cs" in f for f in med_files
        ), f"missing MED for ManualValidate.cs: {med_files}"

        # MED hit on ResourceRate.cs records has_known_values=True.
        rr_hit = next(
            h for h in med if "ResourceRate.cs" in h.file
        )
        assert rr_hit.has_known_values is True, rr_hit
        # MED hit on GuardedSkill.cs records has_ivalidatable=True.
        gs_hit = next(
            h for h in med if "GuardedSkill.cs" in h.file
        )
        assert gs_hit.has_ivalidatable is True, gs_hit

        # 3) Allowlist suppression — line-locked key.
        report_pre = build_report(list(hits), set(), n_files)
        target = next(
            h for h in hits
            if h.severity == SEV_HIGH and "WinCondition.cs" in h.file
        )
        target_key = target.allowlist_key
        hits2, _ = scan_root(repo, "src")
        report_post = build_report(list(hits2), {target_key}, n_files)
        post_keys = {h["allowlist_key"] for h in report_post["high_violations"]}
        assert target_key not in post_keys, (
            f"allowlist did not suppress {target_key}; remaining: {post_keys}"
        )
        assert report_post["new_hits"] < report_pre["new_hits"], (
            f"allowlist did not reduce new_hits: "
            f"pre={report_pre['new_hits']} post={report_post['new_hits']}"
        )

        # 4) Bare-path allowlist — listing a relative file path drops
        #    every hit in that file.
        hits3, _ = scan_root(repo, "src")
        bare_path = "src/Domains/Economy/Models/ResourceRate.cs"
        report_bare = build_report(list(hits3), {bare_path}, n_files)
        bare_files = {h["file"] for h in report_bare["med_violations"]}
        assert bare_path not in bare_files, (
            f"bare-path allowlist did not suppress {bare_path}"
        )

        # 5) Strict mode promotes MED to fail.
        hits4, _ = scan_root(repo, "src")
        non_strict = build_report(list(hits4), set(), n_files, strict=False)
        # HIGH already fails in non-strict.
        assert non_strict["exit_code"] == 1, non_strict

        suppress = {
            h.allowlist_key for h in hits4 if h.severity == SEV_HIGH
        }
        hits5, _ = scan_root(repo, "src")
        rpt_strict_med_only = build_report(
            list(hits5), suppress, n_files, strict=True,
        )
        assert rpt_strict_med_only["high_count"] == 0
        assert rpt_strict_med_only["med_count"] >= 1
        assert rpt_strict_med_only["exit_code"] == 1, rpt_strict_med_only

        hits6, _ = scan_root(repo, "src")
        rpt_lax_med_only = build_report(
            list(hits6), suppress, n_files, strict=False,
        )
        assert rpt_lax_med_only["exit_code"] == 0, rpt_lax_med_only

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

    repo, allow_path, output = resolve_paths(args)
    scan_root_path = repo / args.root
    if not scan_root_path.exists():
        print(
            f"ERROR: scan root not found: {scan_root_path}",
            file=sys.stderr,
        )
        return 2

    if args.verbose:
        print(f"scanning {scan_root_path} ...", file=sys.stderr)

    hits, n_files = scan_root(repo, args.root)
    allowlist = load_allowlist(allow_path)
    report = build_report(hits, allowlist, n_files, strict=args.strict)
    report["scan_root"] = scan_root_path.as_posix()
    report["allowlist_path"] = allow_path.as_posix()

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2), encoding="utf-8")

    if args.quiet:
        print(
            "stringly-enums: "
            f"files={report['scanned_files']} "
            f"total={report['total_hits']} "
            f"allowlisted={report['allowlist_size']} "
            f"new={report['new_hits']} "
            f"HIGH={report['high_count']} "
            f"MED={report['med_count']} "
            f"strict={'on' if args.strict else 'off'} "
            f"-> {output}"
        )
    else:
        print_summary(report, output)

    return report["exit_code"]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
