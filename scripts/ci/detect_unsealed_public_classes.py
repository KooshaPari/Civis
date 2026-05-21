#!/usr/bin/env python3
"""Unsealed public classes detector — Pattern #124 CI gate.

Pattern #124 ("Unsealed public concrete classes in NuGet-published assemblies")
is the failure mode where public concrete classes lack the `sealed` modifier,
allowing external code to subclass them even when no explicit subclassing
contract exists.

    public class UnitDefinition { }  // BAD: implicitly extensible

This pattern breaks the API contract boundary and makes it impossible to
guarantee behavior invariants. The healthy pattern is to seal classes unless
an explicit subclassing contract (documented virtual methods) exists:

    public sealed class UnitDefinition { }  // GOOD: sealed by default

…or document the contract explicitly:

    /// <summary>
    /// Extensible base for custom unit types. Override <see cref="Validate()"/>
    /// to implement validation logic.
    /// </summary>
    public class CustomUnitBase
    {
        public virtual void Validate() { }  // GOOD: explicit extensibility
    }

Classification:

  * **HIGH** — NuGet-published surface (SDK/, Bridge/Client/, Bridge/Protocol/,
    Domains/). These are part of the public API contract consumed by external
    mods. Unsealed classes allow arbitrary subclassing that may break across
    versions.
  * **MED** — Internal production code (Runtime/). Unsealed classes are a code
    smell within internal systems but don't expose API contracts to external
    mods.

Exclusions:

  * Classes with `virtual` members (explicit extensibility contract exists).
  * Classes with `abstract` members (base class intended for subclassing).
  * Classes inheriting from another class (may have subclassing semantics).
  * Classes inside Tests/, bin/, obj/.
  * Interfaces (already extensible by design).
  * Records (C# records are implicitly extensible; separate pattern).

Allowlisting (one entry per line, # for comments):
``docs/qa/unsealed-public-classes-allowlist.txt``. Three suppression mechanisms:

  1. ``severity|file|line`` — line-locked allowlist key. Pull from the
     ``allowlist_key`` field of the JSON report. Moving the line forces
     a fresh review (intentional — prevents silent drift).
  2. Bare ``relative/path.cs`` — suppress every site in that file.
  3. Trailing ``// unsealed-by-design: <reason>`` comment on the same
     line as the class declaration. Inline self-documenting suppression.

Pattern matching:
  * Class declaration: `public class X` (without `sealed`)
  * Exclude: lines containing `virtual` or `abstract` keyword
  * Exclude: lines with `: BaseClass` (inheritance — already has subclassing)

CLI:
    python scripts/ci/detect_unsealed_public_classes.py
        [--root <repo>]
        [--allowlist <path>]
        [--output <json>]
        [--threshold N]
        [--strict]
        [--quiet|--verbose]
        [--self-test]

Exit 0 = no new HIGH hits or (HIGH count <= threshold); 1 = too many NEW
unallowlisted hits (CI fails); 2 = usage error. ``--strict`` fails on any
unallowlisted HIGH, regardless of threshold.

Pairs with Pattern #123 (public mutable collections) and Pattern #111 (silent
catch) as part of the code-quality audit suite. Baseline task #124.
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

# Match public class declaration without sealed modifier
# Captures: public class <Name>
# Excludes: classes with ':' (inheritance), 'virtual', or 'abstract' nearby
# Also excludes 'sealed' keyword
PUBLIC_CLASS_DECL_RE = re.compile(
    r"public\s+(?!sealed\s)class\s+\w+\s*(?:\s*:\s*[^\{]+)?",
    re.IGNORECASE
)

# Check if line has virtual or abstract (explicit extensibility contract)
HAS_VIRTUAL_OR_ABSTRACT_RE = re.compile(
    r"\b(?:virtual|abstract)\b",
    re.IGNORECASE
)

# Trailing per-line allowance comment
UNSEALED_BY_DESIGN_RE = re.compile(
    r"//\s*unsealed-by-design\b[^\n]*"
)

# Test-fixture detection
TEST_DIR_PARTS = {"Tests", "tests"}

# NuGet-published surface (PUBLIC API)
NUGET_SURFACE_PREFIXES = (
    "src/SDK/",
    "src/Bridge/Client/",
    "src/Bridge/Protocol/",
    "src/Domains/",
)

# Skip build artifacts, vendored code, etc.
EXCLUDED_DIR_PARTS = {"bin", "obj", "node_modules", ".git", "packages"}

# Default scan root
DEFAULT_SCAN_ROOTS = ("src",)

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


def line_text(text: str, offset: int) -> str:
    """Return the full text of the line containing *offset*."""
    line_start = text.rfind("\n", 0, offset) + 1
    line_end = text.find("\n", offset)
    if line_end == -1:
        line_end = len(text)
    return text[line_start:line_end]


def load_allowlist(path: Path) -> set[str]:
    """Load allowlist entries. Tolerates inline ``#`` comments, trailing
    whitespace, and Windows-style ``\\`` separators in bare-path entries
    (normalized to POSIX ``/`` so they match ``h.file``)."""
    if not path.exists():
        return set()
    out: set[str] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        # Strip BOM/whitespace and skip blank/comment lines
        line = raw.lstrip("﻿").strip()
        if not line or line.startswith("#"):
            continue
        # Strip trailing inline comments (e.g. ``src/Foo.cs  # legacy``)
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        # Normalize Windows separators to POSIX for bare-path entries;
        # severity|file|line keys also benefit since the middle field is
        # a posix-relative path.
        line = line.replace("\\", "/")
        out.add(line)
    return out


# ----------------------------------------------------------------------------
# Context analysis
# ----------------------------------------------------------------------------


def is_nuget_surface(rel_posix: str) -> bool:
    """Return True if file is in NuGet-published surface."""
    for prefix in NUGET_SURFACE_PREFIXES:
        if rel_posix.startswith(prefix):
            return True
    return False


def is_test_path(rel_posix: str) -> bool:
    """Return True if file is in Tests/ or filename ends with Tests.cs."""
    parts = rel_posix.split("/")
    if any(p in TEST_DIR_PARTS for p in parts):
        return True
    fname = parts[-1].lower()
    if fname.endswith("tests.cs") or fname.endswith("test.cs"):
        return True
    return False


def has_extensibility_contract(class_decl: str) -> bool:
    """Return True if class declaration or nearby context has virtual/abstract."""
    # Check in the class declaration itself and next few tokens
    return bool(HAS_VIRTUAL_OR_ABSTRACT_RE.search(class_decl))


def has_inheritance(class_decl: str) -> bool:
    """Return True if class declaration includes inheritance (: BaseClass)."""
    return ":" in class_decl


# ----------------------------------------------------------------------------
# Scan
# ----------------------------------------------------------------------------


@dataclass
class Hit:
    file: str            # POSIX-relative to repo root
    line: int
    severity: str        # HIGH or MED
    rule: str            # unsealed_nuget or unsealed_internal
    detail: str
    is_test: bool
    has_inline_allow: bool
    line_excerpt: str
    class_name: str = field(default="")
    allowlist_key: str = field(default="")
    in_allowlist: bool = field(default=False)


def scan_file(path: Path, repo_root: Path) -> list[Hit]:
    text = read_text_safe(path)
    if not text:
        return []
    rel = path.relative_to(repo_root).as_posix()

    is_test = is_test_path(rel)
    is_published = is_nuget_surface(rel)
    hits: list[Hit] = []

    # Find all public class declarations (without sealed)
    for m in PUBLIC_CLASS_DECL_RE.finditer(text):
        match_text = m.group(0)
        offset = m.start()
        ln = line_of(text, offset)
        excerpt = line_text(text, offset).strip()

        # Check for inline allowance
        if UNSEALED_BY_DESIGN_RE.search(excerpt):
            continue

        # Skip if class inherits from another class (may have subclassing semantics)
        if has_inheritance(match_text):
            continue

        # Check for explicit extensibility contract in the class body (next ~500 chars)
        # This allows detection of virtual/abstract members in the class
        context_end = min(offset + 500, len(text))
        context = text[offset:context_end]
        if has_extensibility_contract(context):
            continue

        # Extract class name from match
        # match_text e.g. "public class UnitDefinition"
        parts = match_text.split()
        class_name = parts[2] if len(parts) > 2 else "?"

        # Determine severity
        if is_published and not is_test:
            severity = SEV_HIGH
            rule = "unsealed_nuget"
            detail = (
                f"NuGet-published API surface declares unsealed public concrete class "
                f"'{class_name}'. This allows external mod code to subclass it, "
                f"breaking the API contract boundary and preventing version-safe changes. "
                f"Add `sealed` modifier: `public sealed class {class_name}` "
                f"UNLESS an explicit subclassing contract exists (document virtual methods). "
                f"Suppress with `// unsealed-by-design: <reason>` if intentional."
            )
        else:
            severity = SEV_MED
            rule = "unsealed_internal"
            detail = (
                f"Production code declares unsealed public concrete class '{class_name}'. "
                f"Prefer sealed classes by default unless an explicit subclassing contract "
                f"exists (document virtual methods). Add `sealed` modifier for clarity. "
                f"Suppress with `// unsealed-by-design: <reason>` if intentional."
            )

        hits.append(Hit(
            file=rel,
            line=ln,
            severity=severity,
            rule=rule,
            detail=detail,
            is_test=is_test,
            has_inline_allow=False,
            line_excerpt=excerpt[:200],
            class_name=class_name,
        ))

    return hits


def enumerate_target_files(repo_root: Path, roots: list[str]) -> list[Path]:
    out: list[Path] = []
    seen: set[Path] = set()
    for r in roots:
        rp = (repo_root / r).resolve()
        if not rp.exists():
            continue
        for cs in rp.rglob("*.cs"):
            if is_excluded_path(cs):
                continue
            if cs in seen:
                continue
            seen.add(cs)
            out.append(cs)
    out.sort()
    return out


def scan_roots(repo_root: Path, roots: list[str]) -> tuple[list[Hit], int]:
    files = enumerate_target_files(repo_root, roots)
    hits: list[Hit] = []
    for f in files:
        hits.extend(scan_file(f, repo_root))
    return hits, len(files)


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


def build_report(
    hits: list[Hit],
    allowlist: set[str],
    files_scanned: int,
    strict: bool = False,
    threshold: int = 5,
) -> dict:
    new_hits = apply_allowlist(hits, allowlist)

    def _sev_bucket(rule: str) -> list[Hit]:
        return [h for h in new_hits if h.rule == rule]

    nuget_sites = sorted(
        _sev_bucket("unsealed_nuget"),
        key=lambda h: (h.file, h.line),
    )
    internal_sites = sorted(
        _sev_bucket("unsealed_internal"),
        key=lambda h: (h.file, h.line),
    )

    high_count = sum(1 for h in new_hits if h.severity == SEV_HIGH)
    med_count = sum(1 for h in new_hits if h.severity == SEV_MED)

    # Fail if: (strict and any HIGH), or (not strict and HIGH > threshold)
    fail = (strict and high_count > 0) or (not strict and high_count > threshold)
    exit_code = 1 if fail else 0

    def _h2d(h: Hit) -> dict:
        return {
            "file": h.file,
            "line": h.line,
            "severity": h.severity,
            "rule": h.rule,
            "is_test": h.is_test,
            "has_inline_allow": h.has_inline_allow,
            "line_excerpt": h.line_excerpt,
            "class_name": h.class_name,
            "detail": h.detail,
            "allowlist_key": h.allowlist_key,
            "in_allowlist": h.in_allowlist,
        }

    return {
        "scan_utc": datetime.utcnow().isoformat() + "Z",
        "scan_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "files_scanned": files_scanned,
        "total_hits": len(hits),
        "new_hits": len(new_hits),
        "allowlist_size": len(allowlist),
        "strict": strict,
        "threshold": threshold,
        "high_count": high_count,
        "med_count": med_count,
        "unsealed_nuget": [_h2d(h) for h in nuget_sites],
        "unsealed_internal": [_h2d(h) for h in internal_sites],
        "all_hits": [_h2d(h) for h in hits],
        "exit_code": exit_code,
    }


def print_summary(report: dict, output_path: Path) -> None:
    print("unsealed-public-classes gate (Pattern #124)")
    print(f"  files scanned          : {report['files_scanned']}")
    print(f"  total hits             : {report['total_hits']}")
    print(f"  allowlist size         : {report['allowlist_size']}")
    print(f"  NEW hits               : {report['new_hits']}")
    print(f"    HIGH (NuGet-published)    : {report['high_count']}")
    print(f"    MED  (internal)           : {report['med_count']}")
    print(f"  threshold              : {report['threshold']}")
    print(f"  strict mode            : {report['strict']}")
    if report["new_hits"]:
        print()
        print("NEW unsealed public class sites:")
        if report["unsealed_nuget"]:
            print("  -- unsealed_nuget (MUST FIX) --")
            for h in report["unsealed_nuget"]:
                print(
                    f"    [{h['severity']}] {h['file']}:{h['line']} "
                    f"class {h['class_name']}"
                )
        if report["unsealed_internal"]:
            print("  -- unsealed_internal (should fix) --")
            for h in report["unsealed_internal"]:
                print(
                    f"    [{h['severity']}] {h['file']}:{h['line']} "
                    f"class {h['class_name']}"
                )
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect unsealed public concrete classes in NuGet-published surface "
            "(Pattern #124). "
            "HIGH = published surface (SDK/, Bridge/, Domains/); "
            "MED = internal code (info by default; --strict to fail)."
        )
    )
    p.add_argument(
        "--root",
        default=None,
        help=(
            "Repo root (default: auto-detected from script location). "
            "Scan root is src/ relative to this root."
        ),
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/unsealed-public-classes-allowlist.txt",
        help=(
            "Allowlist file; one ``severity|file|line`` key or bare "
            "relative path per line; ``#`` for comments "
            "(default: docs/qa/unsealed-public-classes-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/unsealed-public-classes-report.json",
        help="JSON report output path",
    )
    p.add_argument(
        "--threshold",
        type=int,
        default=5,
        help=(
            "Max unallowlisted HIGH findings before CI fails (default: 5). "
            "Use --strict to fail on any HIGH."
        ),
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help=(
            "Fail on ANY unallowlisted HIGH finding, regardless of threshold. "
            "Default: fail if HIGH > threshold."
        ),
    )
    p.add_argument("--quiet", action="store_true", help="One-line CI summary")
    p.add_argument("--verbose", action="store_true", help="Verbose progress")
    return p.parse_args(argv)


def repo_root_from_script() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_paths(args: argparse.Namespace) -> tuple[Path, Path, Path]:
    repo = Path(args.root).resolve() if args.root else repo_root_from_script()

    def _abs(p: str) -> Path:
        path = Path(p)
        return path if path.is_absolute() else (repo / path).resolve()

    return repo, _abs(args.allowlist), _abs(args.output)


# ----------------------------------------------------------------------------
# Self-tests
# ----------------------------------------------------------------------------


_FIXTURE_UNSEALED = """\
namespace DINOForge.SDK.Models
{
    public class UnitDefinition
    {
        public string Id { get; set; }
    }
}
"""

_FIXTURE_SEALED = """\
namespace DINOForge.SDK.Models
{
    public sealed class BuildingDefinition
    {
        public string Id { get; set; }
    }
}
"""

_FIXTURE_VIRTUAL = """\
namespace DINOForge.SDK.Models
{
    public class CustomUnitBase
    {
        public virtual void Validate() { }
    }
}
"""

_FIXTURE_INHERITANCE = """\
namespace DINOForge.SDK.Models
{
    public class ExtendedUnit : UnitDefinition
    {
    }
}
"""

_FIXTURE_ANNOTATED_OK = """\
namespace DINOForge.SDK.Models
{
    public class LegacyApi // unsealed-by-design: v0.1 extensibility guarantee
    {
        public string Data { get; set; }
    }
}
"""


def _build_fixture_repo(td: Path) -> Path:
    repo = td / "repo"
    sdk = repo / "src" / "SDK" / "Models"
    sdk.mkdir(parents=True, exist_ok=True)

    (sdk / "UnitDefinition.cs").write_text(_FIXTURE_UNSEALED, encoding="utf-8")
    (sdk / "BuildingDefinition.cs").write_text(_FIXTURE_SEALED, encoding="utf-8")
    (sdk / "CustomUnitBase.cs").write_text(_FIXTURE_VIRTUAL, encoding="utf-8")
    (sdk / "ExtendedUnit.cs").write_text(_FIXTURE_INHERITANCE, encoding="utf-8")
    (sdk / "LegacyApi.cs").write_text(_FIXTURE_ANNOTATED_OK, encoding="utf-8")
    return repo


def _self_test() -> int:
    import tempfile

    # 1) Regex matches unsealed
    assert PUBLIC_CLASS_DECL_RE.search("public class UnitDefinition")
    assert PUBLIC_CLASS_DECL_RE.search("public class BuildingDefinition : Base")

    # 2) Regex excludes sealed
    assert not PUBLIC_CLASS_DECL_RE.search("public sealed class Foo")

    # 3) Virtual/abstract detection
    assert has_extensibility_contract("public virtual void Validate() { }")
    assert has_extensibility_contract("public abstract void Foo();")
    assert not has_extensibility_contract("public void Bar() { }")

    # 4) Inheritance detection
    assert has_inheritance("public class ExtendedUnit : UnitDefinition")
    assert not has_inheritance("public class UnitDefinition")

    # 5) NuGet surface detection
    assert is_nuget_surface("src/SDK/Models/Unit.cs")
    assert is_nuget_surface("src/Bridge/Protocol/Message.cs")
    assert is_nuget_surface("src/Domains/Warfare/Models/Unit.cs")
    assert not is_nuget_surface("src/Runtime/Bridge/System.cs")

    # 6) End-to-end scan
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        repo = _build_fixture_repo(td_path)

        hits, n_files = scan_roots(repo, ["src"])

        # Unsealed NuGet-surface MUST be HIGH
        high_hits = [h for h in hits if h.severity == SEV_HIGH]
        high_files = {h.file for h in high_hits}
        assert any(
            "UnitDefinition.cs" in f for f in high_files
        ), f"UnitDefinition (NuGet HIGH) not flagged: {high_files}"

        # Sealed MUST NOT be flagged
        files_with_hits = {h.file for h in hits}
        assert not any(
            "BuildingDefinition.cs" in f for f in files_with_hits
        ), f"BuildingDefinition (sealed) flagged: {files_with_hits}"

        # Virtual/abstract MUST NOT be flagged
        assert not any(
            "CustomUnitBase.cs" in f for f in files_with_hits
        ), f"CustomUnitBase (virtual) flagged: {files_with_hits}"

        # Inheritance MUST NOT be flagged
        assert not any(
            "ExtendedUnit.cs" in f for f in files_with_hits
        ), f"ExtendedUnit (inheritance) flagged: {files_with_hits}"

        # Annotated-ok MUST NOT be flagged
        assert not any(
            "LegacyApi.cs" in f for f in files_with_hits
        ), f"// unsealed-by-design annotation not honored: {files_with_hits}"

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

    missing = [r for r in DEFAULT_SCAN_ROOTS if not (repo / r).exists()]
    if len(missing) == len(DEFAULT_SCAN_ROOTS):
        print(
            f"ERROR: no scan roots found under {repo}: {missing}",
            file=sys.stderr,
        )
        return 2

    if args.verbose:
        print(
            f"scanning {repo} roots={list(DEFAULT_SCAN_ROOTS)} "
            f"(missing={missing})",
            file=sys.stderr,
        )

    hits, n_files = scan_roots(repo, list(DEFAULT_SCAN_ROOTS))
    allowlist = load_allowlist(allow_path)
    report = build_report(hits, allowlist, n_files, strict=args.strict, threshold=args.threshold)
    report["scan_root"] = repo.as_posix()
    report["scan_paths"] = list(DEFAULT_SCAN_ROOTS)
    report["allowlist_path"] = allow_path.as_posix()

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2), encoding="utf-8")

    if args.quiet:
        print(
            "unsealed-public-classes: "
            f"files={report['files_scanned']} "
            f"total={report['total_hits']} "
            f"allowlisted={report['allowlist_size']} "
            f"new={report['new_hits']} "
            f"HIGH={report['high_count']} "
            f"MED={report['med_count']} "
            f"threshold={report['threshold']} "
            f"strict={'on' if args.strict else 'off'} "
            f"-> {output}"
        )
    else:
        print_summary(report, output)

    return report["exit_code"]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
