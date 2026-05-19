#!/usr/bin/env python3
"""Tautological-test detector — Pattern #91 CI gate.

Detects three signatures of test-shaped code that asserts nothing:
  1. ``true.Should().BeTrue(...)``       — Fluent assertion against literal true
  2. ``Assert.True(true[, ...])`` /
     ``Assert.Equal(N, N)``              — xUnit assertion against literal constants
  3. ``[Fact]``/``[Theory]`` methods     — body has no Assert.*, .Should(), Mock.Verify
                                            or Verify( call (after stripping
                                            ``.WithInnerException`` chains).

Scans `src/Tests/**/*.cs`. Allowlist of grandfathered FQNs is read from
``docs/qa/known-tautological-tests.txt`` (one FQN per line, ``#`` for comments).
Methods named ``*_DoesNotThrow``, ``*_NoOp``, ``*_Smoke_*`` or ``*_Compiles``
are auto-skipped because the name itself documents the intent. Tests carrying
``[Fact(Skip="…")]`` or ``[Theory(Skip="…")]`` are also skipped.

CLI:
    python scripts/ci/tautological_test_check.py [--root <path>]
                                                  [--allowlist <path>]
                                                  [--output <json>]
                                                  [--quiet|--verbose]
                                                  [--self-test]

Exit 0 = no NEW instances (allowlist matches all); 1 = new instances detected
(CI fails); 2 = scan/usage error.

Modeled on ``scripts/analysis/enumerate_orphan_classes.py`` (#229/#237) and
``scripts/analysis/check_trait_fraud.py`` (#190/#239).

Background: see Pattern #91 in TRUTH_TABLE — tests that assert literal-true /
have no real assertion at all. Iter-54 audit identified 13 confirmed instances;
allowlist seeds with that grandfathered set so existing debt does not break
CI while #246 migrates them to ``[Fact(Skip)]`` and #248 hardens fuzz smoke
tests.

This is task #247.
"""
from __future__ import annotations

import argparse
import fnmatch
import json
import re
import sys
from datetime import datetime
from pathlib import Path


# ----------------------------------------------------------------------------
# Regex catalogue
# ----------------------------------------------------------------------------

# Signature 1: literal true followed by Fluent assertion BeTrue.
TRUE_SHOULD_BE_TRUE_RE = re.compile(
    r"\btrue\s*\.\s*Should\s*\(\s*\)\s*\.\s*BeTrue\s*\("
)

# Signature 2a: xUnit Assert.True against literal true (with or without message).
ASSERT_TRUE_OF_TRUE_RE = re.compile(r"\bAssert\s*\.\s*True\s*\(\s*true\b")

# Signature 2b: Assert.Equal(N, N) where the same literal/identifier appears
# twice. Captures simple int / string / identifier on both sides. Comparing a
# variable to itself is also tautological.
ASSERT_EQUAL_TAUTOLOGY_RE = re.compile(
    r"\bAssert\s*\.\s*Equal\s*\(\s*"
    r"(?P<lhs>"
    r"\d+"                     # integer literal
    r"|\"(?:[^\"\\]|\\.)*\""   # string literal
    r"|[A-Za-z_]\w*"           # identifier
    r")"
    r"\s*,\s*"
    r"(?P=lhs)"                # same token on the right
    r"\s*\)"
)

# Skip markers: [Fact(Skip=...)] / [Theory(Skip=...)].
SKIP_ATTR_RE = re.compile(
    r"\[\s*(?:Fact|Theory)\s*\(\s*Skip\s*=", re.IGNORECASE
)

# Real-assertion markers — presence of any of these inside a method body marks
# it as having a genuine assertion. Strip ``.WithInnerException`` chains first
# so they cannot count as ``.Should()``.
REAL_ASSERTION_TOKENS = (
    "Assert.",        # xUnit / NUnit / MSTest static asserts
    ".Should(",       # FluentAssertions
    "Mock.Verify",    # Moq strict expectations
    "Verify(",        # Moq instance verify, Verify.That, NSubstitute Received
    "Received(",      # NSubstitute
    "Expect(",        # Shouldly / various
    ".Throws<",       # Assert.Throws<…>
    ".ThrowsAsync<",
    "Assert(",        # xUnit Assertion.Assert
    "FluentActions",  # FluentAssertions FluentActions.Invoking(...)
)

WITH_INNER_EXCEPTION_CHAIN_RE = re.compile(r"\.WithInnerException(?:Exactly)?\b")

# Default allowlist patterns (method-name globs). Names that document the
# intent of "this test exists to prove a code-path does not throw / compiles".
DEFAULT_NAME_ALLOWLIST = (
    "*_DoesNotThrow",
    "*_NoOp",
    "*_Smoke_*",
    "*_Compiles",
    "*_Compiles_*",
    "*Compiles*",
)

EXCLUDED_DIR_PARTS = {"bin", "obj"}


# ----------------------------------------------------------------------------
# C# parsing helpers
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


def find_balanced_block(text: str, open_idx: int) -> int:
    """Return index just after the matching ``}`` for the ``{`` at *open_idx*.

    Skips strings, char literals, line/block comments, and verbatim strings.
    Returns ``len(text)`` if no balance is found (graceful degradation).
    """
    depth = 0
    i = open_idx
    end = len(text)
    in_string = False
    in_verbatim = False
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
                i += 1
        elif in_verbatim:
            if c == '"':
                if n == '"':
                    i += 1  # escaped quote
                else:
                    in_verbatim = False
        elif in_string:
            if c == "\\" and n:
                i += 1
            elif c == '"':
                in_string = False
        elif in_char:
            if c == "\\" and n:
                i += 1
            elif c == "'":
                in_char = False
        else:
            if c == "/" and n == "/":
                in_line_comment = True
                i += 1
            elif c == "/" and n == "*":
                in_block_comment = True
                i += 1
            elif c == "@" and n == '"':
                in_verbatim = True
                i += 1
            elif c == '"':
                in_string = True
            elif c == "'":
                in_char = True
            elif c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    return i + 1
        i += 1
    return end


# Capture namespace declarations, both block and file-scoped.
NAMESPACE_RE = re.compile(
    r"^\s*namespace\s+(?P<ns>[A-Za-z_][\w\.]*)", re.MULTILINE
)

# Capture class declarations (any access modifier; partial/sealed/abstract OK).
# Used to associate methods with their containing class for FQN.
CLASS_DECL_RE = re.compile(
    r"^[ \t]*"
    r"(?:public|internal|private|protected)"
    r"(?:[ \t]+(?:sealed|abstract|static|partial|new|unsafe))*"
    r"[ \t]+class[ \t]+(?P<name>[A-Za-z_]\w*)",
    re.MULTILINE,
)

# Method signature with ``[Fact]`` or ``[Theory]`` somewhere in the leading
# attribute block. We deliberately match a permissive method head — any
# access modifier, any return type, name + ``(...)`` — and grab the line
# number of the method itself.
FACT_THEORY_METHOD_RE = re.compile(
    r"(?P<attrs>(?:^[ \t]*\[[^\]]*\][ \t]*\r?\n)+)"
    r"(?P<head>[ \t]*(?:public|internal|private|protected)?"
    r"(?:[ \t]+(?:static|async|virtual|override|new|sealed))*"
    r"[ \t]+[A-Za-z_][\w<>?,\[\]\s\.]*?"      # return type (greedy-ish)
    r"[ \t]+(?P<name>[A-Za-z_]\w*)\s*\([^)]*\)"
    r"[^;{]*\{)",  # opening brace of body
    re.MULTILINE,
)


def find_namespace_for(text: str, offset: int) -> str:
    ns = ""
    for m in NAMESPACE_RE.finditer(text):
        if m.start() <= offset:
            ns = m.group("ns")
        else:
            break
    return ns


def _build_class_scope_index(text: str) -> list[tuple[int, int, str]]:
    """Return a list of (start, end, name) tuples for every class
    declaration. ``start`` = byte offset of first char of declaration line,
    ``end`` = offset just past the closing brace of the class body. Inner
    classes are listed after their parent so a later-matching scope wins,
    which is exactly what we want when picking the innermost enclosing class
    for a given offset.
    """
    spans: list[tuple[int, int, str]] = []
    for m in CLASS_DECL_RE.finditer(text):
        start = m.start()
        # find the next `{` after the declaration head — accept any whitespace
        # and a possible base-clause first.
        brace_idx = text.find("{", m.end())
        if brace_idx == -1:
            continue
        end = find_balanced_block(text, brace_idx)
        spans.append((start, end, m.group("name")))
    return spans


def find_class_for(text: str, offset: int, scopes: list[tuple[int, int, str]] | None = None) -> str:
    """Return the *innermost* class that encloses ``offset``. ``scopes`` is
    an optional pre-built index from :func:`_build_class_scope_index`."""
    if scopes is None:
        scopes = _build_class_scope_index(text)
    cls = ""
    for start, end, name in scopes:
        if start <= offset < end:
            cls = name  # later (deeper) match overrides
    return cls


def has_fact_or_theory(attrs: str) -> bool:
    # Be tolerant — match [Fact], [Theory], [Fact(Skip=...)] etc. We separately
    # check for Skip below.
    return bool(re.search(r"\[\s*(?:Fact|Theory)\b", attrs))


def name_is_allowlisted(name: str, patterns: tuple[str, ...]) -> bool:
    return any(fnmatch.fnmatchcase(name, p) for p in patterns)


def body_has_real_assertion(body: str) -> bool:
    stripped = WITH_INNER_EXCEPTION_CHAIN_RE.sub("", body)
    return any(token in stripped for token in REAL_ASSERTION_TOKENS)


# ----------------------------------------------------------------------------
# Allowlist file
# ----------------------------------------------------------------------------


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
# Main scan
# ----------------------------------------------------------------------------


def line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def scan_file(path: Path, root: Path) -> tuple[list[dict], int]:
    """Return (records, total_test_methods_seen)."""
    text = read_text_safe(path)
    if not text:
        return [], 0
    rel = path.relative_to(root).as_posix()

    records: list[dict] = []
    total_test_methods = 0

    for m in FACT_THEORY_METHOD_RE.finditer(text):
        attrs = m.group("attrs") or ""
        if not has_fact_or_theory(attrs):
            continue
        total_test_methods += 1

        if SKIP_ATTR_RE.search(attrs):
            continue  # already gated, not tautological by definition

        method_name = m.group("name")
        if name_is_allowlisted(method_name, DEFAULT_NAME_ALLOWLIST):
            continue

        head_end = m.end()
        body_end = find_balanced_block(text, head_end - 1)
        body = text[head_end:body_end]

        head_offset = m.start("head")
        head_line = line_of(text, head_offset)

        ns = find_namespace_for(text, m.start())
        cls = find_class_for(text, m.start())
        fqn = ".".join(p for p in (ns, cls, method_name) if p)

        # Signature 1: true.Should().BeTrue(
        sig1 = TRUE_SHOULD_BE_TRUE_RE.search(body)
        # Signature 2a: Assert.True(true...)
        sig2a = ASSERT_TRUE_OF_TRUE_RE.search(body)
        # Signature 2b: Assert.Equal(X, X)
        sig2b = ASSERT_EQUAL_TAUTOLOGY_RE.search(body)
        # Signature 3: no real assertions in body
        has_real = body_has_real_assertion(body)

        signature = None
        offender = None
        if sig1:
            signature = "true.Should().BeTrue"
            offender = sig1
        elif sig2a:
            signature = "Assert.True(true)"
            offender = sig2a
        elif sig2b:
            signature = "Assert.Equal(N,N)"
            offender = sig2b
        elif not has_real:
            signature = "no_real_assertion"

        if signature is None:
            continue

        if offender is not None:
            offender_line = head_line + text[
                head_offset:head_offset + offender.start()
            ].count("\n")
        else:
            offender_line = head_line

        records.append(
            {
                "fqn": fqn,
                "file": rel,
                "line": offender_line,
                "method_line": head_line,
                "signature": signature,
                "method": method_name,
            }
        )

    return records, total_test_methods


def scan_root(root: Path) -> tuple[list[dict], int, int]:
    files = sorted(p for p in root.rglob("*.cs") if not is_excluded_path(p))
    records: list[dict] = []
    total_methods = 0
    for f in files:
        recs, n = scan_file(f, root)
        records.extend(recs)
        total_methods += n
    return records, total_methods, len(files)


# ----------------------------------------------------------------------------
# Report + CLI
# ----------------------------------------------------------------------------


def build_report(
    records: list[dict],
    allowlist: set[str],
    total_methods: int,
) -> dict:
    new_instances: list[dict] = []
    for r in records:
        in_allow = r["fqn"] in allowlist
        r["in_allowlist"] = in_allow
        if not in_allow:
            new_instances.append(r)
    records.sort(key=lambda r: (r["file"], r["line"]))
    return {
        "scan_utc": datetime.utcnow().isoformat() + "Z",
        "scan_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "total_test_methods": total_methods,
        "tautological_count": len(records),
        "allowlist_size": len(allowlist),
        "new_instances": new_instances,
        "tautological_methods": records,
    }


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect tautological tests (Pattern #91): "
            "true.Should().BeTrue, Assert.True(true), Assert.Equal(N,N), "
            "[Fact]/[Theory] with no assertion."
        )
    )
    p.add_argument(
        "--root",
        default="src/Tests",
        help="Test source root to scan (default: src/Tests)",
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/known-tautological-tests.txt",
        help=(
            "Allowlist file with one FQN per line, # comments allowed "
            "(default: docs/qa/known-tautological-tests.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/tautological-test-report.json",
        help="JSON report output path",
    )
    p.add_argument("--quiet", action="store_true", help="One-line CI summary")
    p.add_argument("--verbose", action="store_true", help="Verbose progress")
    return p.parse_args(argv)


def repo_root_from_script() -> Path:
    """scripts/ci/<this>.py → repo root is parents[2]."""
    return Path(__file__).resolve().parents[2]


def resolve_paths(args: argparse.Namespace) -> tuple[Path, Path, Path]:
    repo = repo_root_from_script()

    def _abs(p: str) -> Path:
        path = Path(p)
        return path if path.is_absolute() else (repo / path).resolve()

    return _abs(args.root), _abs(args.allowlist), _abs(args.output)


def print_summary(report: dict, output_path: Path) -> None:
    print("tautological-test scan (Pattern #91)")
    print(f"  total [Fact]/[Theory] methods : {report['total_test_methods']}")
    print(f"  tautological hits             : {report['tautological_count']}")
    print(f"  allowlist size                : {report['allowlist_size']}")
    print(f"  new (un-allowlisted) instances: {len(report['new_instances'])}")
    if report["new_instances"]:
        print()
        print("NEW tautological tests (will fail CI):")
        for r in report["new_instances"]:
            print(
                f"  - {r['signature']:<25} {r['fqn']}  "
                f"({r['file']}:{r['line']})"
            )
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# Self-tests
# ----------------------------------------------------------------------------


def _self_test() -> int:  # noqa: C901 — single fixture, fine to be long
    fixture = '''
namespace DINOForge.Tests.SelfTest
{
    using Xunit;
    using FluentAssertions;

    public class SelfTestFixtures
    {
        // 1) true.Should().BeTrue(...) — should be detected as taut. sig1.
        [Fact]
        public void Sig1_TrueShouldBeTrue_IsCaught()
        {
            true.Should().BeTrue("documented invariant");
        }

        // 2) Assert.True(true) — should be detected as taut. sig2a.
        [Fact]
        public void Sig2a_AssertTrueOfTrue_IsCaught()
        {
            Assert.True(true, "see comment");
        }

        // 3) [Fact] with no assertion — should be detected as no_real_assertion.
        [Fact]
        public void Sig3_NoAssertion_IsCaught()
        {
            var x = 1 + 1;
            // intentionally no assertion at all
        }

        // 4) Allowlisted name pattern *_DoesNotThrow — must be skipped even
        //    though body has no real assertion.
        [Fact]
        public void Public_Surface_DoesNotThrow()
        {
            var manager = new System.Object();
            // no assertion — but the *_DoesNotThrow name allowlists it
        }

        // 5) Allowlisted FQN — must be skipped even though body matches sig1.
        [Fact]
        public void Allowlisted_FQN_Sig1_NotReported()
        {
            true.Should().BeTrue("known-debt entry");
        }

        // 6) [Fact(Skip="..")] — must be excluded entirely.
        [Fact(Skip = "Documented placeholder")]
        public void Skipped_Fact_NotCounted()
        {
            Assert.True(true);
        }

        // 7) Healthy test — must NOT be reported.
        [Fact]
        public void Healthy_Test_RealAssertion()
        {
            var v = 2 + 2;
            v.Should().Be(4);
        }
    }
}
'''
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        cs_file = td_path / "SelfTest.cs"
        cs_file.write_text(fixture, encoding="utf-8")

        records, total_methods = scan_file(cs_file, td_path)
        # Quick-check: name allowlist filters reduce results to the actual hits.
        # We expect: sig1, sig2a, sig3, allowlisted-fqn-sig1 detected; skip and
        # *_DoesNotThrow auto-filtered; healthy not reported.
        names = {r["method"] for r in records}
        signatures = {r["method"]: r["signature"] for r in records}

        assert "Sig1_TrueShouldBeTrue_IsCaught" in names, (
            f"sig1 not detected; got {names}"
        )
        assert signatures["Sig1_TrueShouldBeTrue_IsCaught"] == \
            "true.Should().BeTrue", signatures
        assert "Sig2a_AssertTrueOfTrue_IsCaught" in names, (
            f"sig2a not detected; got {names}"
        )
        assert signatures["Sig2a_AssertTrueOfTrue_IsCaught"] == \
            "Assert.True(true)", signatures
        assert "Sig3_NoAssertion_IsCaught" in names, (
            f"sig3 not detected; got {names}"
        )
        assert signatures["Sig3_NoAssertion_IsCaught"] == \
            "no_real_assertion", signatures

        assert "Public_Surface_DoesNotThrow" not in names, (
            f"*_DoesNotThrow allowlist failed; got {names}"
        )
        assert "Skipped_Fact_NotCounted" not in names, (
            f"[Fact(Skip)] filter failed; got {names}"
        )
        assert "Healthy_Test_RealAssertion" not in names, (
            f"healthy test misclassified; got {names}"
        )

        # FQN-allowlist test — mark Allowlisted_FQN_Sig1_NotReported in the
        # allowlist set and verify build_report excludes it from new_instances.
        allow = {
            "DINOForge.Tests.SelfTest.SelfTestFixtures."
            "Allowlisted_FQN_Sig1_NotReported"
        }
        report = build_report(records, allow, total_methods)
        new_names = {r["method"] for r in report["new_instances"]}
        assert "Allowlisted_FQN_Sig1_NotReported" not in new_names, (
            f"FQN allowlist failed; new_instances={new_names}"
        )
        # And the other detected ones SHOULD be in new_instances.
        assert "Sig1_TrueShouldBeTrue_IsCaught" in new_names, new_names
        assert "Sig2a_AssertTrueOfTrue_IsCaught" in new_names, new_names
        assert "Sig3_NoAssertion_IsCaught" in new_names, new_names

        # Assert.Equal(N,N) tautology — separate fixture so the regex is
        # exercised even when not embedded in the [Fact] body above.
        assert ASSERT_EQUAL_TAUTOLOGY_RE.search("Assert.Equal(1, 1);")
        assert ASSERT_EQUAL_TAUTOLOGY_RE.search('Assert.Equal("x", "x")')
        assert ASSERT_EQUAL_TAUTOLOGY_RE.search("Assert.Equal(value, value);")
        # Negative — different operands must NOT match.
        assert not ASSERT_EQUAL_TAUTOLOGY_RE.search("Assert.Equal(1, 2)")
        assert not ASSERT_EQUAL_TAUTOLOGY_RE.search('Assert.Equal("a", "b")')

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
        print("ERROR: --quiet and --verbose are mutually exclusive",
              file=sys.stderr)
        return 2

    root, allow_path, output = resolve_paths(args)
    if not root.exists():
        print(f"ERROR: scan root not found: {root}", file=sys.stderr)
        return 2

    if args.verbose:
        print(f"scanning {root} ...", file=sys.stderr)

    records, total_methods, n_files = scan_root(root)
    allowlist = load_allowlist(allow_path)
    report = build_report(records, allowlist, total_methods)
    report["scan_root"] = root.as_posix()
    report["allowlist_path"] = allow_path.as_posix()
    report["files_scanned"] = n_files

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2), encoding="utf-8")

    new_count = len(report["new_instances"])

    if args.quiet:
        print(
            f"tautological-test: total={report['tautological_count']} "
            f"allowlisted={report['allowlist_size']} "
            f"new={new_count} -> {output}"
        )
    else:
        print_summary(report, output)

    return 1 if new_count > 0 else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
