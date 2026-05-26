#!/usr/bin/env python3
"""Unguarded JsonSerializer.Deserialize detector — Pattern #120 CI gate.

Pattern #120 ("Unguarded JsonSerializer.Deserialize without options") is the failure
mode where JsonSerializer.Deserialize is called WITHOUT passing a JsonSerializerOptions
argument. This allows the default .NET runtime options to be used, which differ from
DINOForge's canonical options (PropertyNameCaseInsensitive=true, Converters with
JsonStringEnumConverter, etc.). When deserialization crosses FFI boundaries (Rust,
Go, Sketchfab APIs, or external processes), this asymmetry leads to:

  * Case sensitivity mismatches (field "FooBar" in JSON doesn't match property "foobar")
  * Missing enum conversions (raw integer instead of typed Enum)
  * Silent truncation of unknown fields (no error on extra JSON properties)
  * Inconsistent behavior vs. serialization paths

The healthy pattern (already established in Pattern #109):

    using DINOForge.SDK.Json;
    var dto = JsonSerializer.Deserialize<MyDto>(payload, JsonOptions.Default);

…or for FFI-specific options:

    using DINOForge.Tools.PackCompiler.Json;
    var dto = JsonSerializer.Deserialize<MyDto>(json, GoFfiJsonOptions.Default);

This gate scans C# source under src/ for every bare JsonSerializer.Deserialize call
lacking a JsonSerializerOptions argument. Classification:

  * **HIGH** — FFI-adjacent code (class/method names contain "FFI", "External",
    "Native", "Sketchfab", "Remote", "Api", "RustAsset", "GoAsset", "Subprocess").
    These MUST use explicit options because they cross process boundaries.
  * **MED** — All other production code. Should use explicit options for consistency;
    permitted with audit trail but flagged for manual review.

Allowlisting (one entry per line, # for comments):
``docs/qa/unguarded-json-deserialize-allowlist.txt``. Three suppression mechanisms:

  1. ``severity|file|line`` — line-locked allowlist key. Pull from the
     ``allowlist_key`` field of the JSON report. Moving the line forces
     a fresh review (intentional — prevents silent drift).
  2. Bare ``relative/path.cs`` — suppress every site in that file.
  3. Trailing ``// json-deserialize-ok: <reason>`` comment on the same
     source line as the JsonSerializer.Deserialize call. Inline self-
     documenting suppression for cases where the audit trail belongs
     in the code.

Patterns (two main forms):

  * Generic form: ``JsonSerializer.Deserialize<T>(input)`` — no 2nd arg
  * Non-generic form: ``JsonSerializer.Deserialize(input, typeof(T))`` — 2 args but 2nd is typeof(), not options

Excluded: Test fixtures (under Tests/) get MED severity instead of HIGH.

CLI:
    python scripts/ci/detect_unguarded_json_deserialize.py
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

Pairs with Pattern #109 (inline JsonOptions detection) as part of the
JSON serialization audit suite. Baseline task #120.
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

# Generic form: JsonSerializer.Deserialize<T>(...) with only ONE argument (no comma in parens).
# Simple but effective: match Deserialize<...>(...) where there's no comma inside the parens.
# Use DOTALL flag to allow . to match newlines within the parens.
GENERIC_DESERIALIZE_RE = re.compile(
    r"\bJsonSerializer\s*\.\s*Deserialize\s*<[^>]+>\s*\([^,)]*\)",
    re.DOTALL
)

# Non-generic form: JsonSerializer.Deserialize(input, typeof(T)) — 2nd arg is typeof(), not JsonSerializerOptions
# Match Deserialize(..., typeof(...)) — allow newlines/spaces in between.
NOGENERIC_TYPEOF_DESERIALIZE_RE = re.compile(
    r"\bJsonSerializer\s*\.\s*Deserialize\s*\([^)]*?,\s*typeof\s*\(",
    re.DOTALL
)

# Trailing per-line allowance comment.
JSON_DESERIALIZE_OK_RE = re.compile(
    r"//\s*json-deserialize-ok\b[^\n]*"
)

# Test-fixture detection.
TEST_DIR_PARTS = {"Tests", "tests"}

# FFI-adjacent keywords that elevate severity to HIGH.
FFI_KEYWORDS = (
    "ffi",
    "external",
    "native",
    "sketchfab",
    "remote",
    "api",
    "rustasset",
    "goasset",
    "subprocess",
    "bridge",
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


def context_around(text: str, offset: int, radius: int = 200) -> str:
    """Return ~radius chars around offset for FFI keyword detection."""
    start = max(0, offset - radius)
    end = min(len(text), offset + radius)
    return text[start:end].lower()


def is_ffi_adjacent(text: str, offset: int) -> bool:
    """Return True if FFI keywords appear in context around the call."""
    ctx = context_around(text, offset, radius=400)
    for kw in FFI_KEYWORDS:
        if kw in ctx:
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


# ----------------------------------------------------------------------------
# Scan
# ----------------------------------------------------------------------------


@dataclass
class Hit:
    file: str            # POSIX-relative to repo root
    line: int
    severity: str        # HIGH or MED
    rule: str            # unguarded_deserialize_ffi or unguarded_deserialize_prod
    detail: str
    is_test: bool
    has_inline_allow: bool
    line_excerpt: str
    allowlist_key: str = field(default="")
    in_allowlist: bool = field(default=False)


def scan_file(path: Path, repo_root: Path) -> list[Hit]:
    text = read_text_safe(path)
    if not text:
        return []
    rel = path.relative_to(repo_root).as_posix()

    is_test = is_test_path(rel)
    hits: list[Hit] = []

    # Normalize whitespace for matching (replace newlines with spaces within a reasonable radius).
    # This allows multi-line patterns like Deserialize(\n    json,\n    typeof(...))
    # We'll search the original text but normalize context when matching.

    # Find all generic Deserialize<T>(...) calls without 2nd arg.
    for m in GENERIC_DESERIALIZE_RE.finditer(text):
        offset = m.start()
        ln = line_of(text, offset)
        excerpt = line_text(text, offset).strip()

        # Check for inline allowance.
        has_inline_allow = JSON_DESERIALIZE_OK_RE.search(excerpt) is not None
        if has_inline_allow:
            continue

        # Detect FFI adjacency.
        is_ffi = is_ffi_adjacent(text, offset)

        if is_test and not is_ffi:
            severity = SEV_MED
            rule = "unguarded_deserialize_test"
            detail = (
                "Test fixture calls JsonSerializer.Deserialize<T>(...) without "
                "explicit JsonSerializerOptions. For consistency with production "
                "code and to ensure enum/naming-policy handling, pass JsonOptions.Default "
                "or a test-specific options object. Suppress with "
                "`// json-deserialize-ok:<reason>` if intentional."
            )
        elif is_ffi:
            severity = SEV_HIGH
            rule = "unguarded_deserialize_ffi"
            detail = (
                "FFI-adjacent code (Rust, Go, Sketchfab, external API, or subprocess) "
                "calls JsonSerializer.Deserialize without explicit JsonSerializerOptions. "
                "FFI payloads require consistent deserialize options (case-insensitive, "
                "enum converters). Failure to specify options can cause silent case-mismatch "
                "bugs and missing enum conversions. MUST use an options object (e.g., "
                "JsonOptions.Default, GoFfiJsonOptions.Default, RustAssetJsonOptions.Default)."
            )
        else:
            severity = SEV_MED
            rule = "unguarded_deserialize_prod"
            detail = (
                "Production code calls JsonSerializer.Deserialize<T>(...) without "
                "explicit JsonSerializerOptions. Use JsonOptions.Default or a "
                "project-static holder (src/<Project>/Json/*JsonOptions.cs) for "
                "consistency. Suppress with `// json-deserialize-ok:<reason>` if intentional."
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
        ))

    # Find all non-generic Deserialize(input, typeof(T)) forms — also unguarded.
    for m in NOGENERIC_TYPEOF_DESERIALIZE_RE.finditer(text):
        offset = m.start()
        ln = line_of(text, offset)
        excerpt = line_text(text, offset).strip()

        has_inline_allow = JSON_DESERIALIZE_OK_RE.search(excerpt) is not None
        if has_inline_allow:
            continue

        is_ffi = is_ffi_adjacent(text, offset)

        if is_test and not is_ffi:
            severity = SEV_MED
            rule = "unguarded_deserialize_test"
        elif is_ffi:
            severity = SEV_HIGH
            rule = "unguarded_deserialize_ffi"
        else:
            severity = SEV_MED
            rule = "unguarded_deserialize_prod"

        detail = (
            "Non-generic JsonSerializer.Deserialize(input, typeof(T)) call without "
            "explicit JsonSerializerOptions. Pass a JsonSerializerOptions object "
            "as the third argument for consistent handling of naming policies and "
            "enum converters."
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

    ffi_sites = sorted(
        _sev_bucket("unguarded_deserialize_ffi"),
        key=lambda h: (h.file, h.line),
    )
    prod_sites = sorted(
        _sev_bucket("unguarded_deserialize_prod"),
        key=lambda h: (h.file, h.line),
    )
    test_sites = sorted(
        _sev_bucket("unguarded_deserialize_test"),
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
        "unguarded_deserialize_ffi": [_h2d(h) for h in ffi_sites],
        "unguarded_deserialize_prod": [_h2d(h) for h in prod_sites],
        "unguarded_deserialize_test": [_h2d(h) for h in test_sites],
        "all_hits": [_h2d(h) for h in hits],
        "exit_code": exit_code,
    }


def print_summary(report: dict, output_path: Path) -> None:
    print("unguarded-json-deserialize gate (Pattern #120)")
    print(f"  files scanned          : {report['files_scanned']}")
    print(f"  total hits             : {report['total_hits']}")
    print(f"  allowlist size         : {report['allowlist_size']}")
    print(f"  NEW hits               : {report['new_hits']}")
    print(f"    HIGH (FFI-adjacent)       : {report['high_count']}")
    print(f"    MED  (production/test)    : {report['med_count']}")
    print(f"  threshold              : {report['threshold']}")
    print(f"  strict mode            : {report['strict']}")
    if report["new_hits"]:
        print()
        print("NEW unguarded JsonSerializer.Deserialize sites:")
        if report["unguarded_deserialize_ffi"]:
            print("  -- unguarded_deserialize_ffi (MUST FIX) --")
            for h in report["unguarded_deserialize_ffi"]:
                print(
                    f"    [{h['severity']}] {h['file']}:{h['line']}"
                )
        if report["unguarded_deserialize_prod"]:
            print("  -- unguarded_deserialize_prod (should fix) --")
            for h in report["unguarded_deserialize_prod"]:
                print(
                    f"    [{h['severity']}] {h['file']}:{h['line']}"
                )
        if report["unguarded_deserialize_test"]:
            print("  -- unguarded_deserialize_test (info; --strict to fail) --")
            for h in report["unguarded_deserialize_test"]:
                print(
                    f"    [{h['severity']}] {h['file']}:{h['line']}"
                )
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect bare JsonSerializer.Deserialize<T>(...) calls without "
            "JsonSerializerOptions argument (Pattern #120). HIGH = FFI-adjacent; "
            "MED = production/test (info by default; --strict to fail)."
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
        default="docs/qa/unguarded-json-deserialize-allowlist.txt",
        help=(
            "Allowlist file; one ``severity|file|line`` key or bare "
            "relative path per line; ``#`` for comments "
            "(default: docs/qa/unguarded-json-deserialize-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/unguarded-json-deserialize-report.json",
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


_FIXTURE_BARE_GENERIC = """\
namespace DINOForge.Tools.Cli.Assetctl
{
    using System.Text.Json;

    public class AssetDownloader
    {
        public AssetManifest ParseManifest(string json)
        {
            // BAD: no options, and this is FFI (Sketchfab API)
            return JsonSerializer.Deserialize<AssetManifest>(json)!;
        }
    }
}
"""

_FIXTURE_BARE_TYPEOF = """\
namespace DINOForge.SDK.Validation
{
    using System.Text.Json;

    public class SchemaValidator
    {
        public object Validate(string json)
        {
            // BAD: non-generic form with typeof() instead of options
            return JsonSerializer.Deserialize(
                json,
                typeof(MySchema)
            )!;
        }
    }
}
"""

_FIXTURE_WITH_OPTIONS = """\
namespace DINOForge.SDK.Json
{
    using System.Text.Json;
    using DINOForge.SDK.Json;

    public class ContentLoader
    {
        public MyDto Load(string text)
        {
            // GOOD: uses explicit options
            return JsonSerializer.Deserialize<MyDto>(text, JsonOptions.Default)!;
        }
    }
}
"""

_FIXTURE_ANNOTATED_OK = """\
namespace DINOForge.Tests
{
    using System.Text.Json;

    public class WriterTests
    {
        [Fact]
        public void Writer_RoundTrip()
        {
            var json = "{}";
            var obj = JsonSerializer.Deserialize<MyDto>(json);  // json-deserialize-ok: test fixture
        }
    }
}
"""


def _build_fixture_repo(td: Path) -> Path:
    repo = td / "repo"
    cli = repo / "src" / "Tools" / "Cli" / "Assetctl"
    sdk = repo / "src" / "SDK" / "Json"
    tests = repo / "src" / "Tests"
    for d in (cli, sdk, tests):
        d.mkdir(parents=True, exist_ok=True)

    (cli / "AssetDownloader.cs").write_text(_FIXTURE_BARE_GENERIC, encoding="utf-8")
    (sdk / "SchemaValidator.cs").write_text(_FIXTURE_BARE_TYPEOF, encoding="utf-8")
    (sdk / "ContentLoader.cs").write_text(_FIXTURE_WITH_OPTIONS, encoding="utf-8")
    (tests / "WriterTests.cs").write_text(_FIXTURE_ANNOTATED_OK, encoding="utf-8")
    return repo


def _self_test() -> int:
    import tempfile

    # 1) Regex matches.
    assert GENERIC_DESERIALIZE_RE.search(
        "JsonSerializer.Deserialize<MyDto>(json)"
    )
    assert GENERIC_DESERIALIZE_RE.search(
        "return JsonSerializer.Deserialize<AssetManifest>(payload);"
    )
    assert NOGENERIC_TYPEOF_DESERIALIZE_RE.search(
        "JsonSerializer.Deserialize(json, typeof(MyType))"
    )
    # Negative: with options present (2nd arg is NOT typeof()).
    assert not GENERIC_DESERIALIZE_RE.search(
        "JsonSerializer.Deserialize<MyDto>(json, options)"
    )

    # 2) FFI keyword detection.
    text_ffi = "class SketchfabAssetDownloader { }"
    text_normal = "class NormalLoader { }"
    assert is_ffi_adjacent(text_ffi, 5)
    assert not is_ffi_adjacent(text_normal, 5)

    # 3) Test path detection.
    assert is_test_path("src/Tests/MyTests.cs")
    assert is_test_path("src/Tools/Cli/Tests/QuuxTests.cs")
    assert not is_test_path("src/Tools/Cli/Program.cs")

    # 4) End-to-end scan.
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        repo = _build_fixture_repo(td_path)

        hits, n_files = scan_roots(repo, ["src"])

        # FFI-adjacent bare call MUST be HIGH.
        ffi_hits = [h for h in hits if h.severity == SEV_HIGH]
        ffi_files = {h.file for h in ffi_hits}
        assert any(
            "AssetDownloader.cs" in f for f in ffi_files
        ), f"AssetDownloader (FFI) not flagged HIGH: {ffi_files}"

        # Good code with options must NOT be flagged.
        files_with_hits = {h.file for h in hits}
        assert not any(
            "ContentLoader.cs" in f for f in files_with_hits
        ), f"ContentLoader (with options) flagged: {files_with_hits}"

        # Annotated-ok must NOT be flagged.
        assert not any(
            "WriterTests.cs" in f for f in files_with_hits
        ), f"// json-deserialize-ok annotation not honored: {files_with_hits}"

        # typeof() form must be flagged.
        assert any(
            "SchemaValidator.cs" in f for f in files_with_hits
        ), f"SchemaValidator (typeof form) not flagged: {files_with_hits}"

        # 5) Allowlist suppression.
        bad_hit = next(
            h for h in hits
            if h.rule == "unguarded_deserialize_ffi" and "AssetDownloader.cs" in h.file
        )
        report_pre = build_report(list(hits), set(), n_files, threshold=5)
        target_key = bad_hit.allowlist_key
        hits2, _ = scan_roots(repo, ["src"])
        report_post = build_report(list(hits2), {target_key}, n_files, threshold=5)
        post_keys = {h["allowlist_key"] for h in report_post["unguarded_deserialize_ffi"]}
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
            "unguarded-json-deserialize: "
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
