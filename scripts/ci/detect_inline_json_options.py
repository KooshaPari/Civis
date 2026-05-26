#!/usr/bin/env python3
"""Inline-JsonSerializerOptions detector — Pattern #109 CI gate.

Pattern #109 ("JSON Serialization Options Drift") is the failure mode
where multiple call sites construct ad-hoc ``new JsonSerializerOptions
{ ... }`` literals at the read/write boundary. Each site picks its own
naming policy (CamelCase vs SnakeCaseLower vs none), comment handling,
trailing-comma toleration, and converter list. Read+write asymmetry —
where one side writes with CamelCase and the other reads with
PropertyNameCaseInsensitive=true — masks the drift behind happy-path
tests until a downstream consumer (a different language, a strict
parser, a schema validator) trips over it.

The healthy pattern (already wired in :class:`DINOForge.SDK.Json.JsonOptions`):

    using DINOForge.SDK.Json;
    var dto = JsonSerializer.Deserialize<MyDto>(payload, JsonOptions.Default);

…or for project-specific policies, a project-static holder (see #325):

    using DINOForge.Tools.PackCompiler.Json;
    var dto = JsonSerializer.Deserialize<MyDto>(payload, GoFfiJsonOptions.Default);

This gate scans the C# source tree for every ``new JsonSerializerOptions``
literal and classifies each site:

  * **HIGH** (``inline_json_options_prod``) — production path outside
    designated holder files. Forces drift between read/write paths.
  * **MED**  (``inline_json_options_test``) — test fixtures sometimes
    need custom options (e.g. a writer test that proves indentation
    formatting). Permitted with explicit per-line audit; flagged so
    the corpus is visible in CI.

Allowlisting (one entry per line, ``#`` for comments):
``docs/qa/inline-json-options-allowlist.txt``. Three suppression
mechanisms:

  1. ``severity|file|line`` — line-locked allowlist key. Pull these
     directly from the ``allowlist_key`` field of the JSON report.
     Moving the line forces a fresh entry (intentional — re-review on
     movement).
  2. Bare ``relative/path.cs`` — suppress every site in that file.
  3. Trailing ``// pattern-109-allowed:<reason>`` comment on the
     same source line as the ``new JsonSerializerOptions`` token.
     Inline self-documenting suppression for cases where the audit
     trail belongs in the code, not a sidecar list.

Designated holder paths (always exempt — these files DEFINE the
canonical options):

  * ``src/SDK/Json/JsonOptions.cs``
  * any ``src/**/Json/*JsonOptions.cs`` (project-specific holders)
  * any ``src/**/Json/JsonOptions.cs``

CLI:
    python scripts/ci/detect_inline_json_options.py
        [--root <repo>]
        [--allowlist <path>]
        [--output <json>]
        [--strict]
        [--quiet|--verbose]
        [--self-test]

Exit 0 = no NEW (un-allowlisted) HIGH/MED hits; 1 = new violations
detected (CI fails); 2 = scan/usage error. ``--strict`` promotes MED
findings (test fixtures) to fail the gate as well.

Mirrors the CLI shape of #275 (detect_missing_configureawait.py),
#265 (detect_unguarded_deserialize.py), and #297
(detect_orphan_process_start.py). Pairs with Pattern #109
consolidation work in #325 (JsonOptions.Default + project-static
holders).

This is task #326.
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

# ``new JsonSerializerOptions { ... }`` and ``new JsonSerializerOptions(...)``.
# Allows optional whitespace and a fully-qualified ``System.Text.Json.``
# prefix. Matches both object-initializer ``{`` and constructor ``(`` forms.
INLINE_OPTIONS_RE = re.compile(
    r"\bnew\s+(?:System\s*\.\s*Text\s*\.\s*Json\s*\.\s*)?"
    r"JsonSerializerOptions\s*[{(]"
)

# Trailing per-line allowance comment. Anchored after the matched
# ``new JsonSerializerOptions ...`` so we only honor it on the same line.
PATTERN_109_ALLOWED_RE = re.compile(
    r"//\s*pattern-109-allowed\b[^\n]*"
)

# Test-fixture detection: a file is a "test" file if it lives under any
# directory called ``Tests`` or has a filename ending in ``Tests.cs``.
TEST_DIR_PARTS = {"Tests", "tests"}

# Designated holder paths (POSIX-relative, lowercased substring match).
# These files DEFINE the canonical options and are always exempt.
HOLDER_PATH_HINTS = (
    "src/sdk/json/jsonoptions.cs",
)

# Designated holder filename patterns (case-insensitive).
HOLDER_FILENAME_PATTERNS = (
    "jsonoptions.cs",          # JsonOptions.cs
    "_jsonoptions.cs",         # NameJsonOptions.cs (suffix variant)
)


# Skip these directory parts (build artifacts, vendored, etc.).
EXCLUDED_DIR_PARTS = {"bin", "obj", "node_modules", ".git", "packages"}

# Default scan roots (POSIX-relative to repo root).
DEFAULT_SCAN_ROOTS = (
    "src",
)

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
# Path classification
# ----------------------------------------------------------------------------


def is_holder_path(rel_posix: str) -> bool:
    """Return True if *rel_posix* is a designated JsonOptions holder file.

    A holder file is one of:
      * ``src/SDK/Json/JsonOptions.cs`` (the canonical default)
      * any ``src/**/Json/JsonOptions.cs``
      * any ``src/**/Json/<Name>JsonOptions.cs`` (project-specific holders
        like ``GoFfiJsonOptions.cs`` or ``OutputJsonOptions.cs``)
    """
    lo = rel_posix.lower()
    # Exact canonical path.
    for hint in HOLDER_PATH_HINTS:
        if hint in lo:
            return True
    # Glob-style: ``src/**/Json/*JsonOptions.cs``.
    parts = lo.split("/")
    if len(parts) >= 3 and parts[0] == "src" and parts[-2] == "json":
        fname = parts[-1]
        for pat in HOLDER_FILENAME_PATTERNS:
            if pat == "jsonoptions.cs" and fname == "jsonoptions.cs":
                return True
            if pat == "_jsonoptions.cs" and fname.endswith("jsonoptions.cs"):
                return True
    return False


def is_test_path(rel_posix: str) -> bool:
    """Return True if *rel_posix* is a test file (lives under a Tests
    directory OR filename ends with ``Tests.cs`` / ``Test.cs``)."""
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
    severity: str
    rule: str            # inline_json_options_prod / inline_json_options_test
    detail: str
    is_test: bool
    is_holder: bool
    has_inline_allow: bool
    line_excerpt: str
    allowlist_key: str = field(default="")
    in_allowlist: bool = field(default=False)


def scan_file(path: Path, repo_root: Path) -> list[Hit]:
    text = read_text_safe(path)
    if not text:
        return []
    rel = path.relative_to(repo_root).as_posix()

    # Holder files are exempt by construction.
    if is_holder_path(rel):
        return []

    is_test = is_test_path(rel)
    hits: list[Hit] = []

    for m in INLINE_OPTIONS_RE.finditer(text):
        offset = m.start()
        ln = line_of(text, offset)
        excerpt = line_text(text, offset).strip()

        # Per-line ``// pattern-109-allowed:<reason>`` suppression.
        has_inline_allow = PATTERN_109_ALLOWED_RE.search(excerpt) is not None
        if has_inline_allow:
            # Don't even emit a hit; the inline annotation IS the audit
            # trail. Quiet by design.
            continue

        if is_test:
            severity = SEV_MED
            rule = "inline_json_options_test"
            detail = (
                "Test fixture constructs ad-hoc JsonSerializerOptions. "
                "If the options are intentionally divergent, append "
                "`// pattern-109-allowed:<reason>` to this line; otherwise "
                "switch to JsonOptions.Default or a project-static holder."
            )
        else:
            severity = SEV_HIGH
            rule = "inline_json_options_prod"
            detail = (
                "Production code constructs ad-hoc JsonSerializerOptions. "
                "Use DINOForge.SDK.Json.JsonOptions.Default or a project-"
                "specific static holder under src/<Project>/Json/. Inline "
                "options drift between read/write paths and mask "
                "asymmetric naming-policy bugs."
            )

        hits.append(Hit(
            file=rel,
            line=ln,
            severity=severity,
            rule=rule,
            detail=detail,
            is_test=is_test,
            is_holder=False,
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
) -> dict:
    new_hits = apply_allowlist(hits, allowlist)

    def _sev_bucket(rule: str) -> list[Hit]:
        return [h for h in new_hits if h.rule == rule]

    prod_sites = sorted(
        _sev_bucket("inline_json_options_prod"),
        key=lambda h: (h.file, h.line),
    )
    test_sites = sorted(
        _sev_bucket("inline_json_options_test"),
        key=lambda h: (h.file, h.line),
    )

    high_count = sum(1 for h in new_hits if h.severity == SEV_HIGH)
    med_count = sum(1 for h in new_hits if h.severity == SEV_MED)
    fail = high_count > 0 or (strict and med_count > 0)
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
        "high_count": high_count,
        "med_count": med_count,
        "inline_json_options_prod": [_h2d(h) for h in prod_sites],
        "inline_json_options_test": [_h2d(h) for h in test_sites],
        "all_hits": [_h2d(h) for h in hits],
        "exit_code": exit_code,
    }


def print_summary(report: dict, output_path: Path) -> None:
    print("inline-json-options gate (Pattern #109)")
    print(f"  files scanned          : {report['files_scanned']}")
    print(f"  total hits             : {report['total_hits']}")
    print(f"  allowlist size         : {report['allowlist_size']}")
    print(f"  NEW hits               : {report['new_hits']}")
    print(f"    HIGH (production)         : {report['high_count']}")
    print(f"    MED  (test fixtures)      : {report['med_count']}")
    if report["new_hits"]:
        print()
        print("NEW inline-JsonSerializerOptions sites:")
        if report["inline_json_options_prod"]:
            print("  -- inline_json_options_prod (production code) --")
            for h in report["inline_json_options_prod"]:
                print(
                    f"    [{h['severity']}] {h['file']}:{h['line']}"
                )
        if report["inline_json_options_test"]:
            print("  -- inline_json_options_test (test fixtures; --strict to fail) --")
            for h in report["inline_json_options_test"]:
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
            "Detect ad-hoc ``new JsonSerializerOptions`` literals outside "
            "designated holder paths (Pattern #109). HIGH = production "
            "code; MED = test fixtures (info; --strict to fail)."
        )
    )
    p.add_argument(
        "--root",
        default=None,
        help=(
            "Repo root (default: auto-detected from the script location). "
            "Scan root is src/ relative to this root."
        ),
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/inline-json-options-allowlist.txt",
        help=(
            "Allowlist file; one ``severity|file|line`` key or bare "
            "relative path per line; ``#`` for comments "
            "(default: docs/qa/inline-json-options-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/inline-json-options-report.json",
        help="JSON report output path",
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help=(
            "Promote MED (test fixtures) findings to fail the gate. "
            "Default: only HIGH (production) fails."
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


_FIXTURE_HOLDER = """\
namespace DINOForge.SDK.Json
{
    using System.Text.Json;
    using System.Text.Json.Serialization;

    public static class JsonOptions
    {
        public static readonly JsonSerializerOptions Default = new JsonSerializerOptions
        {
            PropertyNameCaseInsensitive = true,
            Converters = { new JsonStringEnumConverter() },
        };
    }
}
"""

_FIXTURE_PROD_INLINE = """\
namespace DINOForge.Tools.Cli
{
    using System.Text.Json;

    public static class BadProdLoader
    {
        public static MyDto Load(string text)
        {
            var options = new JsonSerializerOptions
            {
                PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            };
            return JsonSerializer.Deserialize<MyDto>(text, options)!;
        }
    }
}
"""

_FIXTURE_PROD_INLINE_ALLOWED = """\
namespace DINOForge.Tools.Cli
{
    using System.Text.Json;

    public static class AllowedProdLoader
    {
        public static MyDto Load(string text)
        {
            // Annotated suppression: deliberate snake_case for Go FFI bridge.
            var options = new JsonSerializerOptions { WriteIndented = false };  // pattern-109-allowed: go-ffi-bridge requires raw policy
            return JsonSerializer.Deserialize<MyDto>(text, options)!;
        }
    }
}
"""

_FIXTURE_PROD_HEALTHY = """\
namespace DINOForge.Tools.Cli
{
    using System.Text.Json;
    using DINOForge.SDK.Json;

    public static class HealthyProdLoader
    {
        public static MyDto Load(string text)
        {
            return JsonSerializer.Deserialize<MyDto>(text, JsonOptions.Default)!;
        }
    }
}
"""

_FIXTURE_TEST_INLINE = """\
namespace DINOForge.Tests
{
    using System.Text.Json;
    using Xunit;

    public class WriterTests
    {
        [Fact]
        public void Writer_RoundTrip_Indented()
        {
            var options = new JsonSerializerOptions { WriteIndented = true };
            // ... test body ...
        }
    }
}
"""

_FIXTURE_FQN_INLINE = """\
namespace DINOForge.Tools.Cli
{
    public static class FqnLoader
    {
        public static object Load()
        {
            return new System.Text.Json.JsonSerializerOptions
            {
                PropertyNameCaseInsensitive = true,
            };
        }
    }
}
"""


def _build_fixture_repo(td: Path) -> Path:
    repo = td / "repo"
    sdk_json = repo / "src" / "SDK" / "Json"
    cli = repo / "src" / "Tools" / "Cli"
    tests = repo / "src" / "Tests"
    for d in (sdk_json, cli, tests):
        d.mkdir(parents=True, exist_ok=True)

    (sdk_json / "JsonOptions.cs").write_text(_FIXTURE_HOLDER, encoding="utf-8")
    (cli / "BadProdLoader.cs").write_text(_FIXTURE_PROD_INLINE, encoding="utf-8")
    (cli / "AllowedProdLoader.cs").write_text(
        _FIXTURE_PROD_INLINE_ALLOWED, encoding="utf-8"
    )
    (cli / "HealthyProdLoader.cs").write_text(
        _FIXTURE_PROD_HEALTHY, encoding="utf-8"
    )
    (cli / "FqnLoader.cs").write_text(_FIXTURE_FQN_INLINE, encoding="utf-8")
    (tests / "WriterTests.cs").write_text(
        _FIXTURE_TEST_INLINE, encoding="utf-8"
    )
    return repo


def _self_test() -> int:  # noqa: C901
    import tempfile

    # 1) Path classification.
    assert is_holder_path("src/SDK/Json/JsonOptions.cs")
    assert is_holder_path("src/Tools/Cli/Json/OutputJsonOptions.cs")
    assert is_holder_path("src/Tools/PackCompiler/Json/GoFfiJsonOptions.cs")
    assert not is_holder_path("src/Tools/Cli/JsonOptions.cs")  # not under a Json dir
    assert not is_holder_path("src/Tools/Cli/Program.cs")
    assert not is_holder_path("src/Tests/JsonHelperTests.cs")

    assert is_test_path("src/Tests/Foo.cs")
    assert is_test_path("src/Tests/Subdir/BarTests.cs")
    assert is_test_path("src/Tools/Cli/Tests/Quux.cs")
    assert is_test_path("src/Tools/Cli/FooTests.cs")
    assert not is_test_path("src/Tools/Cli/Program.cs")

    # 2) Regex matches all expected forms.
    assert INLINE_OPTIONS_RE.search("var o = new JsonSerializerOptions {")
    assert INLINE_OPTIONS_RE.search("var o = new JsonSerializerOptions(")
    assert INLINE_OPTIONS_RE.search(
        "var o = new System.Text.Json.JsonSerializerOptions {"
    )
    assert INLINE_OPTIONS_RE.search(
        "var o = new  System . Text . Json . JsonSerializerOptions {"
    )
    # Negative: not the right type.
    assert not INLINE_OPTIONS_RE.search("var o = new JsonReaderOptions {")
    assert not INLINE_OPTIONS_RE.search("var o = new JsonWriterOptions {")

    # 3) End-to-end scan.
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        repo = _build_fixture_repo(td_path)

        hits, n_files = scan_roots(repo, ["src"])
        files_seen = {h.file for h in hits}

        # Holder must NOT be flagged.
        assert not any(
            "JsonOptions.cs" in f for f in files_seen
        ), f"holder file flagged: {files_seen}"

        # Healthy production code must NOT be flagged.
        assert not any(
            "HealthyProdLoader.cs" in f for f in files_seen
        ), f"healthy site flagged: {files_seen}"

        # Inline-allowed line must NOT be flagged.
        assert not any(
            "AllowedProdLoader.cs" in f for f in files_seen
        ), f"// pattern-109-allowed annotation not honored: {files_seen}"

        # Inline production code MUST be flagged HIGH.
        prod_hits = [h for h in hits if h.severity == SEV_HIGH]
        prod_files = {h.file for h in prod_hits}
        assert any(
            "BadProdLoader.cs" in f for f in prod_files
        ), f"BadProdLoader not flagged: {prod_files}"
        assert any(
            "FqnLoader.cs" in f for f in prod_files
        ), f"FqnLoader (FQN form) not flagged: {prod_files}"

        # Test fixture MUST be flagged MED (not HIGH).
        test_hits = [h for h in hits if h.severity == SEV_MED]
        test_files = {h.file for h in test_hits}
        assert any(
            "WriterTests.cs" in f for f in test_files
        ), f"WriterTests not flagged at MED: {test_files}"
        # And it must NOT be in HIGH.
        assert not any(
            "WriterTests.cs" in f for f in prod_files
        ), f"WriterTests promoted to HIGH: {prod_files}"

        # 4) Allowlist suppression — line-locked key.
        bad_hit = next(
            h for h in hits
            if h.rule == "inline_json_options_prod" and "BadProdLoader.cs" in h.file
        )
        report_pre = build_report(list(hits), set(), n_files)
        target_key = bad_hit.allowlist_key
        hits2, _ = scan_roots(repo, ["src"])
        report_post = build_report(list(hits2), {target_key}, n_files)
        post_keys = {h["allowlist_key"] for h in report_post["inline_json_options_prod"]}
        assert target_key not in post_keys, (
            f"line-locked allowlist did not suppress {target_key}; "
            f"remaining: {post_keys}"
        )
        assert report_post["new_hits"] < report_pre["new_hits"], (
            f"allowlist did not reduce new_hits: pre={report_pre['new_hits']} "
            f"post={report_post['new_hits']}"
        )

        # 5) Bare-path allowlist form.
        hits3, _ = scan_roots(repo, ["src"])
        bare_path = "src/Tools/Cli/BadProdLoader.cs"
        report_bare = build_report(list(hits3), {bare_path}, n_files)
        bare_files = {h["file"] for h in report_bare["inline_json_options_prod"]}
        assert bare_path not in bare_files, (
            f"bare-path allowlist did not suppress {bare_path}"
        )

        # 6) Strict mode promotes MED (test) to fail.
        hits4, _ = scan_roots(repo, ["src"])
        non_strict = build_report(list(hits4), set(), n_files, strict=False)
        strict = build_report(list(hits4), set(), n_files, strict=True)
        assert non_strict["exit_code"] == 1  # HIGH already fails
        assert strict["exit_code"] == 1
        # Suppress all HIGH; non-strict passes, strict still fails on MED.
        suppress = {
            h.allowlist_key for h in hits4 if h.severity == SEV_HIGH
        }
        hits5, _ = scan_roots(repo, ["src"])
        rpt_strict_only_med = build_report(
            list(hits5), suppress, n_files, strict=True,
        )
        assert rpt_strict_only_med["high_count"] == 0
        assert rpt_strict_only_med["med_count"] >= 1
        assert rpt_strict_only_med["exit_code"] == 1, rpt_strict_only_med
        hits6, _ = scan_roots(repo, ["src"])
        rpt_lax_only_med = build_report(
            list(hits6), suppress, n_files, strict=False,
        )
        assert rpt_lax_only_med["exit_code"] == 0, rpt_lax_only_med

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
    report = build_report(hits, allowlist, n_files, strict=args.strict)
    report["scan_root"] = repo.as_posix()
    report["scan_paths"] = list(DEFAULT_SCAN_ROOTS)
    report["allowlist_path"] = allow_path.as_posix()

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2), encoding="utf-8")

    if args.quiet:
        print(
            "inline-json-options: "
            f"files={report['files_scanned']} "
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
