#!/usr/bin/env python3
"""Public mutable collection detector — Pattern #123 CI gate.

Pattern #123 ("Public mutable collection properties in DTOs") is the failure
mode where data-transfer objects expose mutable collection properties with
public setters, allowing external code to modify collection state directly.

    public List<T> Items { get; set; }  // BAD: externally mutable

This pattern breaks encapsulation and makes it impossible to track/validate
mutations. The healthy pattern is to expose read-only interfaces:

    public IReadOnlyList<T> Items { get; init; } = new();  // GOOD: immutable contract

…or use the backing-field pattern when deserializers don't support init-setters:

    [YamlMember(Alias = "items")]
    public List<T> ItemsInternal { get; set; } = new();

    [YamlIgnore]
    public IReadOnlyList<T> Items => ItemsInternal;

Classification:

  * **HIGH** — NuGet-published surface (SDK/, Bridge/Protocol/). These are part
    of the public API contract consumed by external mods. Mutable collections
    break the API guarantee and enable arbitrary mutation by mod code.
  * **MED** — Internal production code (Runtime/, Domains/). Mutable collections
    are a code smell within internal systems but don't expose API contracts.

Allowlisting (one entry per line, # for comments):
``docs/qa/public-mutable-collections-allowlist.txt``. Three suppression mechanisms:

  1. ``severity|file|line`` — line-locked allowlist key. Pull from the
     ``allowlist_key`` field of the JSON report. Moving the line forces
     a fresh review (intentional — prevents silent drift).
  2. Bare ``relative/path.cs`` — suppress every site in that file.
  3. Trailing ``// public-mutable-ok: <reason>`` comment on the same
     source line as the mutable collection property. Inline self-
     documenting suppression for cases where the audit trail belongs
     in the code.

Excluded:
  * `IReadOnlyList<T>`, `IReadOnlyCollection<T>`, `IEnumerable<T>`,
    `IReadOnlyDictionary<T,U>` (already immutable from caller perspective)
  * Test fixtures (Tests/ get MED severity instead of HIGH)
  * Files in bin/, obj/, .git/, node_modules/, packages/

Pattern matching:
  * Generic form: `public List<T> Items { get; set; }`
  * Other forms: `IList<T>`, `Collection<T>`, `ICollection<T>`
  * Match only when BOTH getter and setter are public (not `{ get; private set; }`)

CLI:
    python scripts/ci/detect_public_mutable_collections.py
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

Pairs with Pattern #110 (open-ended assertions) and Pattern #111 (silent catch)
as part of the code-quality audit suite. Baseline task #123.
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

# Match public mutable collection properties with both get; and set;
# Captures: public <Type> <Name> { get; set; }
# where Type is List<T>, IList<T>, Collection<T>, or ICollection<T>
# but NOT IReadOnlyList<T>, IReadOnlyCollection<T>, IEnumerable<T>, etc.
PUBLIC_MUTABLE_COLLECTION_RE = re.compile(
    r"public\s+(?:List|IList|Collection|ICollection)<[^>]+>\s+\w+\s*\{\s*get;\s*set;\s*\}",
    re.DOTALL | re.IGNORECASE
)

# Excluded immutable types (these are safe and should not be flagged).
IMMUTABLE_TYPE_PATTERNS = [
    r"IReadOnlyList<",
    r"IReadOnlyCollection<",
    r"IEnumerable<",
    r"IReadOnlyDictionary<",
]

# Trailing per-line allowance comment.
PUBLIC_MUTABLE_OK_RE = re.compile(
    r"//\s*public-mutable-ok\b[^\n]*"
)

# Test-fixture detection.
TEST_DIR_PARTS = {"Tests", "tests"}

# NuGet-published surface.
NUGET_SURFACE_PREFIXES = (
    "src/SDK/",
    "src/Bridge/Protocol/",
)

# Skip build artifacts, vendored code, etc.
EXCLUDED_DIR_PARTS = {"bin", "obj", "node_modules", ".git", "packages"}

# Default scan root.
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
# Context analysis
# ----------------------------------------------------------------------------


def is_nuget_surface(rel_posix: str) -> bool:
    """Return True if file is in NuGet-published surface (SDK/, Bridge/Protocol/)."""
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


def is_immutable_type(type_str: str) -> bool:
    """Return True if type_str uses an immutable collection interface."""
    for pattern in IMMUTABLE_TYPE_PATTERNS:
        if re.search(pattern, type_str, re.IGNORECASE):
            return True
    return False


# ----------------------------------------------------------------------------
# Scan
# ----------------------------------------------------------------------------


@dataclass
class Hit:
    file: str            # POSIX-relative to repo root
    line: int
    severity: str        # HIGH or MED
    rule: str            # public_mutable_nuget or public_mutable_internal
    detail: str
    is_test: bool
    has_inline_allow: bool
    line_excerpt: str
    type_name: str = field(default="")
    prop_name: str = field(default="")
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

    # Find all public mutable collection properties.
    for m in PUBLIC_MUTABLE_COLLECTION_RE.finditer(text):
        match_text = m.group(0)
        offset = m.start()
        ln = line_of(text, offset)
        excerpt = line_text(text, offset).strip()

        # Check for inline allowance.
        has_inline_allow = PUBLIC_MUTABLE_OK_RE.search(line_text(text, offset)) is not None
        if has_inline_allow:
            continue

        # Double-check that the match doesn't use an immutable type
        # (defensive; regex should have already excluded them).
        if is_immutable_type(match_text):
            continue

        # Extract type and property name from match for detail.
        # match_text e.g. "public List<Unit> Units { get; set; }"
        parts = match_text.split()
        type_name = parts[1] if len(parts) > 1 else "?"
        prop_name = parts[2] if len(parts) > 2 else "?"

        # Determine severity.
        if is_published and not is_test:
            severity = SEV_HIGH
            rule = "public_mutable_nuget"
            detail = (
                f"NuGet-published API surface exposes mutable collection property "
                f"'{prop_name}' ({type_name}) with public setter. This breaks "
                f"encapsulation and allows external mod code to mutate state. "
                f"Convert to: public IReadOnlyList<T> {prop_name} {{ get; init; }} = new(); "
                f"OR use backing-field pattern with [YamlIgnore] property. "
                f"Suppress with `// public-mutable-ok: <reason>` if intentional."
            )
        elif is_test and not is_published:
            severity = SEV_MED
            rule = "public_mutable_test"
            detail = (
                f"Test fixture exposes mutable collection property '{prop_name}' "
                f"({type_name}) with public setter. For consistency and to avoid "
                f"accidental mutations in fixture setup, prefer immutable interfaces. "
                f"Suppress with `// public-mutable-ok: <reason>` if intentional."
            )
        else:
            severity = SEV_MED
            rule = "public_mutable_internal"
            detail = (
                f"Production code exposes mutable collection property '{prop_name}' "
                f"({type_name}) with public setter. Prefer immutable interfaces "
                f"(IReadOnlyList<T>) to prevent accidental mutations. Convert to: "
                f"public IReadOnlyList<T> {prop_name} {{ get; init; }} = new(); "
                f"Suppress with `// public-mutable-ok: <reason>` if intentional."
            )

        hits.append(Hit(
            file=rel,
            line=ln,
            severity=severity,
            rule=rule,
            detail=detail,
            is_test=is_test,
            has_inline_allow=has_inline_allow,
            line_excerpt=excerpt[:200],
            type_name=type_name,
            prop_name=prop_name,
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
        _sev_bucket("public_mutable_nuget"),
        key=lambda h: (h.file, h.line),
    )
    internal_sites = sorted(
        _sev_bucket("public_mutable_internal"),
        key=lambda h: (h.file, h.line),
    )
    test_sites = sorted(
        _sev_bucket("public_mutable_test"),
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
            "type_name": h.type_name,
            "prop_name": h.prop_name,
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
        "public_mutable_nuget": [_h2d(h) for h in nuget_sites],
        "public_mutable_internal": [_h2d(h) for h in internal_sites],
        "public_mutable_test": [_h2d(h) for h in test_sites],
        "all_hits": [_h2d(h) for h in hits],
        "exit_code": exit_code,
    }


def print_summary(report: dict, output_path: Path) -> None:
    print("public-mutable-collections gate (Pattern #123)")
    print(f"  files scanned          : {report['files_scanned']}")
    print(f"  total hits             : {report['total_hits']}")
    print(f"  allowlist size         : {report['allowlist_size']}")
    print(f"  NEW hits               : {report['new_hits']}")
    print(f"    HIGH (NuGet-published)    : {report['high_count']}")
    print(f"    MED  (internal/test)      : {report['med_count']}")
    print(f"  threshold              : {report['threshold']}")
    print(f"  strict mode            : {report['strict']}")
    if report["new_hits"]:
        print()
        print("NEW public mutable collection sites:")
        if report["public_mutable_nuget"]:
            print("  -- public_mutable_nuget (MUST FIX) --")
            for h in report["public_mutable_nuget"]:
                print(
                    f"    [{h['severity']}] {h['file']}:{h['line']} "
                    f"{h['prop_name']} ({h['type_name']})"
                )
        if report["public_mutable_internal"]:
            print("  -- public_mutable_internal (should fix) --")
            for h in report["public_mutable_internal"]:
                print(
                    f"    [{h['severity']}] {h['file']}:{h['line']} "
                    f"{h['prop_name']} ({h['type_name']})"
                )
        if report["public_mutable_test"]:
            print("  -- public_mutable_test (info; --strict to fail) --")
            for h in report["public_mutable_test"]:
                print(
                    f"    [{h['severity']}] {h['file']}:{h['line']} "
                    f"{h['prop_name']} ({h['type_name']})"
                )
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect public mutable collection properties in DTOs (Pattern #123). "
            "HIGH = NuGet-published surface (SDK/, Bridge/Protocol/); "
            "MED = internal/test (info by default; --strict to fail)."
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
        default="docs/qa/public-mutable-collections-allowlist.txt",
        help=(
            "Allowlist file; one ``severity|file|line`` key or bare "
            "relative path per line; ``#`` for comments "
            "(default: docs/qa/public-mutable-collections-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/public-mutable-collections-report.json",
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


_FIXTURE_MUTABLE_LIST = """\
namespace DINOForge.SDK.Models
{
    using System.Collections.Generic;

    public class UnitDefinition
    {
        public string Id { get; set; }

        // BAD: NuGet-published surface with mutable List setter
        public List<string> DefenseTags { get; set; } = new();
    }
}
"""

_FIXTURE_IMMUTABLE = """\
namespace DINOForge.SDK.Models
{
    using System.Collections.Generic;

    public class BuildingDefinition
    {
        public string Id { get; set; }

        // GOOD: uses IReadOnlyList (immutable from caller perspective)
        public IReadOnlyList<string> SupportedFactions { get; init; } = new List<string>();
    }
}
"""

_FIXTURE_INTERNAL_MUTABLE = """\
namespace DINOForge.Runtime.Bridge
{
    using System.Collections.Generic;

    internal class EntityCache
    {
        // MED: internal code, but still a code smell
        public List<int> CachedIds { get; set; } = new();
    }
}
"""

_FIXTURE_ANNOTATED_OK = """\
namespace DINOForge.SDK.Models
{
    using System.Collections.Generic;

    public class LegacyApi
    {
        public List<string> Items { get; set; } = new(); // public-mutable-ok: backwards compat with v0.1 API
    }
}
"""


def _build_fixture_repo(td: Path) -> Path:
    repo = td / "repo"
    sdk = repo / "src" / "SDK" / "Models"
    runtime = repo / "src" / "Runtime" / "Bridge"
    for d in (sdk, runtime):
        d.mkdir(parents=True, exist_ok=True)

    (sdk / "UnitDefinition.cs").write_text(_FIXTURE_MUTABLE_LIST, encoding="utf-8")
    (sdk / "BuildingDefinition.cs").write_text(_FIXTURE_IMMUTABLE, encoding="utf-8")
    (sdk / "LegacyApi.cs").write_text(_FIXTURE_ANNOTATED_OK, encoding="utf-8")
    (runtime / "EntityCache.cs").write_text(_FIXTURE_INTERNAL_MUTABLE, encoding="utf-8")
    return repo


def _self_test() -> int:
    import tempfile

    # 1) Regex matches.
    assert PUBLIC_MUTABLE_COLLECTION_RE.search(
        "public List<string> Items { get; set; }"
    )
    assert PUBLIC_MUTABLE_COLLECTION_RE.search(
        "public ICollection<int> Ids { get; set; }"
    )
    # Negative: immutable types should not match (but defensive check in scan_file).
    # (Regex doesn't exclude them, but scan_file does via is_immutable_type.)

    # 2) Immutable type detection.
    assert is_immutable_type("IReadOnlyList<T>")
    assert is_immutable_type("IEnumerable<string>")
    assert not is_immutable_type("List<T>")
    assert not is_immutable_type("ICollection<T>")

    # 3) NuGet surface detection.
    assert is_nuget_surface("src/SDK/Models/Unit.cs")
    assert is_nuget_surface("src/Bridge/Protocol/Message.cs")
    assert not is_nuget_surface("src/Runtime/Bridge/System.cs")

    # 4) Test path detection.
    assert is_test_path("src/Tests/MyTests.cs")
    assert not is_test_path("src/SDK/Models/Unit.cs")

    # 5) End-to-end scan.
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        repo = _build_fixture_repo(td_path)

        hits, n_files = scan_roots(repo, ["src"])

        # NuGet-surface mutable call MUST be HIGH.
        high_hits = [h for h in hits if h.severity == SEV_HIGH]
        high_files = {h.file for h in high_hits}
        assert any(
            "UnitDefinition.cs" in f for f in high_files
        ), f"UnitDefinition (NuGet HIGH) not flagged: {high_files}"

        # Good immutable code must NOT be flagged.
        files_with_hits = {h.file for h in hits}
        assert not any(
            "BuildingDefinition.cs" in f for f in files_with_hits
        ), f"BuildingDefinition (IReadOnlyList) flagged: {files_with_hits}"

        # Annotated-ok must NOT be flagged.
        assert not any(
            "LegacyApi.cs" in f for f in files_with_hits
        ), f"// public-mutable-ok annotation not honored: {files_with_hits}"

        # Internal MED severity.
        med_hits = [h for h in hits if h.severity == SEV_MED]
        med_files = {h.file for h in med_hits}
        assert any(
            "EntityCache.cs" in f for f in med_files
        ), f"EntityCache (internal MED) not flagged: {med_files}"

        # 6) Allowlist suppression.
        high_hit = next(
            h for h in hits
            if h.severity == SEV_HIGH and "UnitDefinition.cs" in h.file
        )
        report_pre = build_report(list(hits), set(), n_files, threshold=5)
        target_key = high_hit.allowlist_key
        hits2, _ = scan_roots(repo, ["src"])
        report_post = build_report(list(hits2), {target_key}, n_files, threshold=5)
        post_keys = {h["allowlist_key"] for h in report_post["public_mutable_nuget"]}
        assert target_key not in post_keys, (
            f"line-locked allowlist did not suppress {target_key}; "
            f"remaining: {post_keys}"
        )

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
            "public-mutable-collections: "
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
