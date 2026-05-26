#!/usr/bin/env python3
"""Test-isolation linter — Pattern #93 CI gate.

Pattern #93 ("Order-Dependent / Process-Global State") is the failure mode
where xUnit's default class-level parallelism races on shared process state
that test classes touch without opting into a serial ``[Collection(...)]``.
The classic offenders are:

  * ``Environment.SetEnvironmentVariable(...)`` without a snapshot/restore
    pattern AND without ``[Collection]`` — sister tests reading the same
    var see arbitrary values.
  * Class-level ``static (readonly)? string`` paths under ``Path.Combine`` /
    ``Path.GetTempPath`` that lack a ``Guid`` discriminator — two test
    classes (or two instances of the same class) end up sharing one folder.
  * ``Directory.SetCurrentDirectory(...)`` — process-wide. ANY use is a
    timebomb because xUnit interleaves classes by default.
  * Hardcoded temp paths reused across tests (cross-class collision risk).
  * Tests setting *production* env vars (i.e. ``DINOFORGE_*`` not prefixed
    ``DINOFORGE_TEST_``) — leaks into prod code paths during the same test
    process and corrupts later tests / parallel CI shards.

This gate scans ``src/Tests/`` for those signatures and groups violations by
severity (HIGH=raw prod env, MED=missing [Collection], LOW=potential
temp-path race).

CLI:
    python scripts/ci/detect_global_state_tests.py [--root <path>]
                                                    [--allowlist <path>]
                                                    [--output <json>]
                                                    [--quiet|--verbose]
                                                    [--self-test]

Exit 0 = no NEW (un-allowlisted) violations; 1 = new violations detected
(CI fails); 2 = scan/usage error.

Modeled on ``scripts/ci/tautological_test_check.py`` (#247) and
``scripts/ci/changelog_lint.py`` (#251).

This is task #257.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime
from pathlib import Path


# ----------------------------------------------------------------------------
# Regex catalogue
# ----------------------------------------------------------------------------

# Class declaration — capture name + start offset (used to determine which
# class an offense lives in for FQN reporting + [Collection] enclosing scope).
CLASS_DECL_RE = re.compile(
    r"^[ \t]*"
    r"(?:public|internal|private|protected)"
    r"(?:[ \t]+(?:sealed|abstract|static|partial|new|unsafe))*"
    r"[ \t]+class[ \t]+(?P<name>[A-Za-z_]\w*)",
    re.MULTILINE,
)

# Namespace (block- or file-scoped).
NAMESPACE_RE = re.compile(
    r"^\s*namespace\s+(?P<ns>[A-Za-z_][\w\.]*)", re.MULTILINE
)

# [Collection("Name")] or [Collection(Foo.Name)] attribute on a class.
# We don't try to resolve the name expression — its mere presence is enough
# to opt the test into a non-default collection (which xUnit treats as
# serially-scheduled if the corresponding [CollectionDefinition] passes
# DisableParallelization=true). False-negatives are acceptable here since
# we still flag at the class level when missing entirely.
COLLECTION_ATTR_RE = re.compile(r"\[\s*Collection\s*\(")

# Environment.SetEnvironmentVariable(...) — fundamental env mutation site.
ENV_SET_RE = re.compile(
    r"\bEnvironment\s*\.\s*SetEnvironmentVariable\s*\(\s*"
    r"(?P<name>"
    r"\"[^\"]+\""           # string literal
    r"|@\"[^\"]+\""         # verbatim string literal
    r"|[A-Za-z_]\w*"        # identifier (e.g. const)
    r")"
)

# Directory.SetCurrentDirectory(...) — process-wide working-dir mutation.
SETCWD_RE = re.compile(
    r"\bDirectory\s*\.\s*SetCurrentDirectory\s*\("
)

# class-level `static (readonly)? string Foo = Path.(Combine|GetTempPath)(...)`
# We grab the right-hand-side expression up to the terminating `;` so we can
# inspect it for a Guid discriminator.
STATIC_PATH_FIELD_RE = re.compile(
    r"^[ \t]*"
    r"(?:public|internal|private|protected)?"
    r"(?:[ \t]+(?:static|readonly|const|new))+"
    r"[ \t]+(?:string|var)"
    r"[ \t]+(?P<field>[A-Za-z_]\w*)"
    r"\s*=\s*"
    r"(?P<rhs>Path\s*\.\s*(?:Combine|GetTempPath)\s*\([^;]*?\))"
    r"\s*;",
    re.MULTILINE,
)

# Path.Combine(Path.GetTempPath(), "literal") — hardcoded temp path with no
# unique component (no Guid, no Path.GetRandomFileName, no Pid). Captures the
# literal so we can suppress when it does include a discriminator.
HARDCODED_TEMP_RE = re.compile(
    r"\bPath\s*\.\s*Combine\s*\(\s*"
    r"Path\s*\.\s*GetTempPath\s*\(\s*\)\s*,\s*"
    r"(?P<lit>\"[^\"]+\")"
    r"\s*\)"
)

# DINOFORGE_ env-var literal — production env-vars must use DINOFORGE_TEST_
# prefix when set from tests; raw DINOFORGE_FOO is a HIGH-severity violation.
DINOFORGE_ENV_LITERAL_RE = re.compile(
    r"\"(DINOFORGE_(?!TEST_)[A-Z0-9_]+)\""
)

# Markers that indicate a string contains a unique discriminator and is
# therefore safe to share across tests.
DISCRIMINATOR_TOKENS = (
    "Guid.NewGuid",
    "Guid.New",
    "Path.GetRandomFileName",
    "Path.GetTempFileName",
    "Process.GetCurrentProcess",
    "Environment.ProcessId",
    "Environment.TickCount",
    "DateTime.Now.Ticks",
    "DateTime.UtcNow.Ticks",
    ".Ticks.ToString",
)

EXCLUDED_DIR_PARTS = {"bin", "obj"}

# Severity tiers (used in JSON grouping).
SEV_HIGH = "HIGH"   # Raw prod env-var name set by a test.
SEV_MED = "MED"     # Env mutation / SetCurrentDirectory without [Collection].
SEV_LOW = "LOW"     # Potential temp-path race (no Guid / static class field).


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


def find_balanced_block(text: str, open_idx: int) -> int:
    """Return offset just past the matching ``}`` for the ``{`` at *open_idx*.
    Skips strings, char literals, line/block comments, and verbatim strings.
    Mirrors the helper in :mod:`tautological_test_check`."""
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
                    i += 1
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


# ----------------------------------------------------------------------------
# Class-scope index
# ----------------------------------------------------------------------------


def _build_class_scope_index(text: str) -> list[tuple[int, int, int, str, str]]:
    """Return (decl_start, body_start, body_end, name, header_text) tuples
    for every class declaration. ``header_text`` is the slice from
    ``decl_start`` to the opening brace (where ``[Collection(...)]`` would
    live if applied to the class declaration directly above it; we widen
    later by walking attribute lines upward)."""
    spans: list[tuple[int, int, int, str, str]] = []
    for m in CLASS_DECL_RE.finditer(text):
        decl_start = m.start()
        brace_idx = text.find("{", m.end())
        if brace_idx == -1:
            continue
        body_end = find_balanced_block(text, brace_idx)
        spans.append(
            (decl_start, brace_idx + 1, body_end, m.group("name"),
             text[decl_start:brace_idx])
        )
    return spans


def _attribute_block_for_class(text: str, decl_start: int) -> str:
    """Walk upward from ``decl_start`` to gather contiguous attribute lines
    (``[...]``) that belong to the class. Stops at the first non-attribute,
    non-blank, non-comment line. Returns the joined block (may be empty)."""
    # Find start of the line at decl_start.
    line_start = text.rfind("\n", 0, decl_start) + 1  # 0 if not found
    cursor = line_start
    parts: list[str] = []
    while cursor > 0:
        # previous line's start
        prev_end = cursor - 1   # the \n at end of previous line
        prev_start = text.rfind("\n", 0, prev_end) + 1
        prev = text[prev_start:prev_end]
        stripped = prev.strip()
        if not stripped:
            cursor = prev_start
            continue
        if stripped.startswith("//") or stripped.startswith("/*") or \
                stripped.endswith("*/"):
            cursor = prev_start
            continue
        if stripped.startswith("[") and stripped.endswith("]"):
            parts.append(prev)
            cursor = prev_start
            continue
        break
    return "\n".join(reversed(parts))


def find_namespace_for(text: str, offset: int) -> str:
    ns = ""
    for m in NAMESPACE_RE.finditer(text):
        if m.start() <= offset:
            ns = m.group("ns")
        else:
            break
    return ns


def innermost_class(
    scopes: list[tuple[int, int, int, str, str]],
    offset: int,
) -> tuple[int, int, int, str, str] | None:
    out: tuple[int, int, int, str, str] | None = None
    for span in scopes:
        decl_start, _body_start, body_end, _name, _header = span
        if decl_start <= offset < body_end:
            out = span  # later (deeper) wins
    return out


def class_has_collection(text: str, span: tuple[int, int, int, str, str]) -> bool:
    decl_start, _body_start, _body_end, _name, _header = span
    attrs = _attribute_block_for_class(text, decl_start)
    if COLLECTION_ATTR_RE.search(attrs):
        return True
    # Sometimes attributes are inlined on the same line as the declaration.
    line_start = text.rfind("\n", 0, decl_start) + 1
    line = text[line_start:decl_start]
    return bool(COLLECTION_ATTR_RE.search(line))


# ----------------------------------------------------------------------------
# Allowlist
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
# Detection rules
# ----------------------------------------------------------------------------


def _has_discriminator(rhs: str) -> bool:
    return any(tok in rhs for tok in DISCRIMINATOR_TOKENS)


def scan_file(path: Path, root: Path) -> list[dict]:
    """Return a list of violation records for *path*."""
    text = read_text_safe(path)
    if not text:
        return []
    rel = path.relative_to(root).as_posix()
    scopes = _build_class_scope_index(text)
    if not scopes:
        return []

    # Pre-compute, for each class span, whether it has [Collection(...)].
    has_coll: dict[int, bool] = {
        span[0]: class_has_collection(text, span) for span in scopes
    }

    records: list[dict] = []

    # Helper to attach class/namespace context to a hit.
    def _attach(offset: int) -> tuple[str, str, str, bool]:
        cls_span = innermost_class(scopes, offset)
        cls_name = cls_span[3] if cls_span else ""
        cls_has_coll = has_coll[cls_span[0]] if cls_span else False
        ns = find_namespace_for(text, offset)
        fqn = ".".join(p for p in (ns, cls_name) if p)
        return cls_name, ns, fqn, cls_has_coll

    # ---- Rule 1: Environment.SetEnvironmentVariable(...) ----
    for m in ENV_SET_RE.finditer(text):
        offset = m.start()
        cls_name, ns, fqn, cls_has_coll = _attach(offset)
        line = line_of(text, offset)
        name_token = m.group("name")
        # HIGH: name is a string literal that starts with DINOFORGE_ but NOT
        # DINOFORGE_TEST_. We only check string-literal forms — identifier
        # constants get checked separately by DINOFORGE_ENV_LITERAL_RE below.
        is_high = False
        var_label: str | None = None
        if name_token.startswith('"') or name_token.startswith('@"'):
            # strip leading @ and quotes
            inner = name_token.lstrip("@").strip('"')
            var_label = inner
            if inner.startswith("DINOFORGE_") and not inner.startswith(
                "DINOFORGE_TEST_"
            ):
                is_high = True

        if is_high:
            records.append(
                {
                    "rule": "raw_prod_env_var",
                    "severity": SEV_HIGH,
                    "file": rel,
                    "line": line,
                    "fqn": fqn,
                    "class": cls_name,
                    "detail": (
                        f'Environment.SetEnvironmentVariable("{var_label}") '
                        f"— production env-var name set from a test. Use "
                        f"DINOFORGE_TEST_ prefix or move to InternalsVisibleTo."
                    ),
                    "in_collection": cls_has_coll,
                }
            )
            # Even if HIGH, also track the missing-[Collection] aspect below.

        if not cls_has_coll:
            records.append(
                {
                    "rule": "env_mutation_without_collection",
                    "severity": SEV_MED,
                    "file": rel,
                    "line": line,
                    "fqn": fqn,
                    "class": cls_name,
                    "detail": (
                        "Environment.SetEnvironmentVariable in a class "
                        "without [Collection(EnvVarMutation)] — xUnit "
                        "default class-parallelism races on env state."
                    ),
                    "var": var_label,
                    "in_collection": False,
                }
            )

    # Catch raw prod env-var literals even when the surrounding call doesn't
    # match ENV_SET_RE (e.g. ``Foo.Resolve("DINOFORGE_RESOLVER_PATH")``).
    # We only flag inside test files — root scoping handles that.
    for m in DINOFORGE_ENV_LITERAL_RE.finditer(text):
        var_label = m.group(1)
        offset = m.start()
        # de-dup against rule 1: skip if an ENV_SET hit at the same line
        # already produced a HIGH for the same var.
        cls_name, ns, fqn, _cls_has_coll = _attach(offset)
        line = line_of(text, offset)
        if any(
            r["rule"] == "raw_prod_env_var"
            and r["file"] == rel
            and r["line"] == line
            for r in records
        ):
            continue
        # Suppress on lines that are only string concatenation of test names
        # — heuristic: only flag if call context is SetEnvironmentVariable
        # OR if the literal is followed shortly by a comma + value (typical
        # of a Set call). Use a small lookbehind window for the call name.
        prefix = text[max(0, offset - 80):offset]
        if "SetEnvironmentVariable" not in prefix:
            # Not in a SetEnvironmentVariable call — likely a Get-style usage
            # which is read-only and harmless. Skip.
            continue
        records.append(
            {
                "rule": "raw_prod_env_var",
                "severity": SEV_HIGH,
                "file": rel,
                "line": line,
                "fqn": fqn,
                "class": cls_name,
                "detail": (
                    f'SetEnvironmentVariable on production env-var '
                    f'"{var_label}" — use DINOFORGE_TEST_ prefix.'
                ),
            }
        )

    # ---- Rule 2: Directory.SetCurrentDirectory(...) ----
    for m in SETCWD_RE.finditer(text):
        offset = m.start()
        cls_name, _ns, fqn, cls_has_coll = _attach(offset)
        line = line_of(text, offset)
        records.append(
            {
                "rule": "set_current_directory",
                "severity": SEV_MED,
                "file": rel,
                "line": line,
                "fqn": fqn,
                "class": cls_name,
                "detail": (
                    "Directory.SetCurrentDirectory mutates process-wide "
                    "working directory. Always opt into "
                    "[Collection(WorkingDirectory)]; better yet, refactor "
                    "to pass cwd explicitly."
                ),
                "in_collection": cls_has_coll,
            }
        )

    # ---- Rule 3: class-level static path field without Guid/random ----
    for m in STATIC_PATH_FIELD_RE.finditer(text):
        offset = m.start()
        cls_span = innermost_class(scopes, offset)
        if not cls_span:
            continue
        # confirm we're at class scope, not inside a method body. The match
        # itself is already constrained by the regex (must be a field-style
        # declaration at indent depth, terminated by ;), but to be safe we
        # check that the offset is between body_start and body_end and not
        # inside a deeper `{ ... }` block.
        decl_start, body_start, body_end, _name, _header = cls_span
        if not (body_start <= offset < body_end):
            continue
        rhs = m.group("rhs")
        if _has_discriminator(rhs):
            continue
        cls_name, _ns, fqn, cls_has_coll = _attach(offset)
        records.append(
            {
                "rule": "class_static_path_no_guid",
                "severity": SEV_LOW,
                "file": rel,
                "line": line_of(text, offset),
                "fqn": fqn,
                "class": cls_name,
                "field": m.group("field"),
                "rhs": rhs.strip(),
                "detail": (
                    f"Class-static path field `{m.group('field')}` lacks a "
                    "Guid/random discriminator — sister test classes in the "
                    "same process share the same directory and race on it."
                ),
                "in_collection": cls_has_coll,
            }
        )

    # ---- Rule 4: hardcoded Path.Combine(GetTempPath(), \"literal\") ----
    for m in HARDCODED_TEMP_RE.finditer(text):
        offset = m.start()
        # Suppress when the surrounding statement also emits a Guid (e.g. on
        # the next argument as part of a longer Combine chain). Cheap check:
        # look 80 chars ahead for a Guid token.
        window = text[offset:offset + 200]
        if _has_discriminator(window):
            continue
        cls_name, _ns, fqn, cls_has_coll = _attach(offset)
        records.append(
            {
                "rule": "hardcoded_temp_path",
                "severity": SEV_LOW,
                "file": rel,
                "line": line_of(text, offset),
                "fqn": fqn,
                "class": cls_name,
                "literal": m.group("lit"),
                "detail": (
                    "Hardcoded temp path with no Guid/random discriminator "
                    "— two parallel test runs (or two classes) collide."
                ),
                "in_collection": cls_has_coll,
            }
        )

    return records


def scan_root(root: Path) -> tuple[list[dict], int]:
    files = sorted(p for p in root.rglob("*.cs") if not is_excluded_path(p))
    out: list[dict] = []
    for f in files:
        out.extend(scan_file(f, root))
    return out, len(files)


# ----------------------------------------------------------------------------
# Report
# ----------------------------------------------------------------------------


def _violation_key(r: dict) -> str:
    """Stable allowlist key: ``rule|fqn|line|file``. Keeps allowlist entries
    line-locked so movement of code requires a fresh allowlist refresh."""
    return f"{r['rule']}|{r.get('fqn', '')}|{r['line']}|{r['file']}"


def build_report(
    records: list[dict],
    allowlist: set[str],
    strict: bool = False,
) -> dict:
    new_records: list[dict] = []
    for r in records:
        key = _violation_key(r)
        in_allow = key in allowlist or r.get("fqn", "") in allowlist
        r["allowlist_key"] = key
        r["in_allowlist"] = in_allow
        if not in_allow:
            new_records.append(r)
    records.sort(key=lambda r: (r["file"], r["line"], r["rule"]))
    new_records.sort(key=lambda r: (r["severity"], r["file"], r["line"]))

    by_severity = {SEV_HIGH: [], SEV_MED: [], SEV_LOW: []}
    for r in new_records:
        by_severity.setdefault(r["severity"], []).append(r)

    # Spec-aligned rule-keyed lists (Pattern #93 task #257 schema). Mirrors
    # the SeverityCategory mapping documented in test-isolation-policy.md.
    by_rule: dict[str, list[dict]] = {
        "raw_prod_env_var": [],          # HIGH
        "static_path_no_guid": [],       # HIGH (alias of class_static_path_no_guid)
        "working_dir_mutation": [],      # HIGH (alias of set_current_directory)
        "env_var_no_collection": [],     # MED  (alias of env_mutation_without_collection)
        "temp_path_no_guid": [],         # LOW  (alias of hardcoded_temp_path)
    }
    rule_aliases = {
        "raw_prod_env_var": "raw_prod_env_var",
        "class_static_path_no_guid": "static_path_no_guid",
        "set_current_directory": "working_dir_mutation",
        "env_mutation_without_collection": "env_var_no_collection",
        "hardcoded_temp_path": "temp_path_no_guid",
    }
    for r in new_records:
        bucket = rule_aliases.get(r["rule"])
        if bucket:
            by_rule[bucket].append(r)

    high_count = len(by_severity.get(SEV_HIGH, []))
    med_count = len(by_severity.get(SEV_MED, []))
    low_count = len(by_severity.get(SEV_LOW, []))
    fail = high_count > 0 or med_count > 0 or (strict and low_count > 0)
    exit_code = 1 if fail else 0

    return {
        "scan_utc": datetime.utcnow().isoformat() + "Z",
        "scan_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "total_violations": len(records),
        "allowlist_size": len(allowlist),
        "new_violations": len(new_records),
        "strict": strict,
        "exit_code": exit_code,
        "by_severity": {
            "HIGH": by_severity.get(SEV_HIGH, []),
            "MED": by_severity.get(SEV_MED, []),
            "LOW": by_severity.get(SEV_LOW, []),
        },
        # Spec-aligned rule-keyed groupings (#257).
        "raw_prod_env_var": by_rule["raw_prod_env_var"],
        "static_path_no_guid": by_rule["static_path_no_guid"],
        "working_dir_mutation": by_rule["working_dir_mutation"],
        "env_var_no_collection": by_rule["env_var_no_collection"],
        "temp_path_no_guid": by_rule["temp_path_no_guid"],
        "all_violations": records,
    }


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect tests that touch process-global state without "
            "[Collection(...)] (Pattern #93). Severities: HIGH=raw prod "
            "env-var, MED=missing [Collection], LOW=temp-path race."
        )
    )
    p.add_argument(
        "--root",
        default="src/Tests",
        help="Test source root to scan (default: src/Tests)",
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/global-state-allowlist.txt",
        help=(
            "Allowlist file; one ``rule|fqn|line|file`` key per line "
            "(or bare FQN); ``#`` for comments "
            "(default: docs/qa/global-state-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/global-state-report.json",
        help="JSON report output path",
    )
    p.add_argument("--quiet", action="store_true", help="One-line CI summary")
    p.add_argument("--verbose", action="store_true", help="Verbose progress")
    p.add_argument(
        "--strict",
        action="store_true",
        help=(
            "Promote LOW-severity findings (class_static_path_no_guid, "
            "hardcoded_temp_path) to fail the gate. Default: only HIGH+MED "
            "fail."
        ),
    )
    return p.parse_args(argv)


def repo_root_from_script() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_paths(args: argparse.Namespace) -> tuple[Path, Path, Path]:
    repo = repo_root_from_script()

    def _abs(p: str) -> Path:
        path = Path(p)
        return path if path.is_absolute() else (repo / path).resolve()

    return _abs(args.root), _abs(args.allowlist), _abs(args.output)


def print_summary(report: dict, output_path: Path) -> None:
    print("global-state-test scan (Pattern #93)")
    print(f"  total violations  : {report['total_violations']}")
    print(f"  allowlist size    : {report['allowlist_size']}")
    print(f"  NEW violations    : {report['new_violations']}")
    for sev in ("HIGH", "MED", "LOW"):
        items = report["by_severity"][sev]
        print(f"    {sev:<4} : {len(items)}")
    if report["new_violations"]:
        print()
        print("NEW global-state violations (will fail CI):")
        for sev in ("HIGH", "MED", "LOW"):
            items = report["by_severity"][sev]
            if not items:
                continue
            print(f"  -- {sev} --")
            for r in items:
                print(
                    f"    {r['rule']:<32} {r['fqn']}  "
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
    using System;
    using System.IO;
    using Xunit;

    // 1) Class WITHOUT [Collection] that mutates env — should be MED.
    //    Also sets a raw prod env-var DINOFORGE_RESOLVER_PATH — HIGH.
    public class BadEnvNoCollection
    {
        public void Test_RawProdEnv()
        {
            Environment.SetEnvironmentVariable("DINOFORGE_RESOLVER_PATH", "/foo");
        }
    }

    // 2) Class WITH [Collection] using DINOFORGE_TEST_ prefix — clean.
    [Collection("EnvVarMutation")]
    public class GoodEnvWithCollection
    {
        public void Test_TestPrefixed()
        {
            Environment.SetEnvironmentVariable("DINOFORGE_TEST_FOO", "/foo");
        }
    }

    // 3) Class with class-static path WITHOUT Guid — LOW.
    public class BadStaticPath
    {
        private static readonly string Shared =
            Path.Combine(Path.GetTempPath(), "dinoforge-shared-fixtures");

        public void Test_UsesShared()
        {
            File.WriteAllText(Shared, "hello");
        }
    }

    // 4) Class with class-static path WITH Guid — clean.
    public class GoodStaticPath
    {
        private static readonly string Unique =
            Path.Combine(Path.GetTempPath(), "df-" + Guid.NewGuid().ToString());
    }

    // 5) Class that calls SetCurrentDirectory — MED, always.
    public class BadCwd
    {
        public void Test_BadCwd()
        {
            Directory.SetCurrentDirectory("/tmp");
        }
    }

    // 6) Hardcoded temp path inside a method body — LOW.
    public class BadHardcodedTemp
    {
        public void Test_HardcodedTemp()
        {
            var p = Path.Combine(Path.GetTempPath(), "dino-tests-hardcoded");
            File.WriteAllText(p, "x");
        }
    }
}
'''
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        cs_file = td_path / "SelfTest.cs"
        cs_file.write_text(fixture, encoding="utf-8")
        records = scan_file(cs_file, td_path)

    # Build a quick (rule, class) lookup.
    pairs = {(r["rule"], r["class"]) for r in records}

    # Rule 1a: HIGH on BadEnvNoCollection.
    assert ("raw_prod_env_var", "BadEnvNoCollection") in pairs, (
        f"raw_prod_env_var on BadEnvNoCollection not detected; got {pairs}"
    )
    # Rule 1b: MED on BadEnvNoCollection.
    assert ("env_mutation_without_collection", "BadEnvNoCollection") in pairs, (
        f"env_mutation_without_collection on BadEnvNoCollection not detected; "
        f"got {pairs}"
    )
    # Rule 1c: GoodEnvWithCollection should NOT trip MED (has [Collection])
    # and NOT trip HIGH (uses DINOFORGE_TEST_ prefix).
    assert ("env_mutation_without_collection", "GoodEnvWithCollection") \
        not in pairs, (
            f"GoodEnvWithCollection should be clean; got {pairs}"
        )
    assert ("raw_prod_env_var", "GoodEnvWithCollection") not in pairs, (
        f"GoodEnvWithCollection used DINOFORGE_TEST_; got {pairs}"
    )

    # Rule 3: class-static path without Guid trips LOW.
    assert ("class_static_path_no_guid", "BadStaticPath") in pairs, (
        f"class_static_path_no_guid not detected; got {pairs}"
    )
    # Rule 3b: GoodStaticPath uses Guid.NewGuid — clean.
    assert ("class_static_path_no_guid", "GoodStaticPath") not in pairs, (
        f"GoodStaticPath should be clean; got {pairs}"
    )

    # Rule 2: SetCurrentDirectory — always MED.
    assert ("set_current_directory", "BadCwd") in pairs, (
        f"set_current_directory not detected; got {pairs}"
    )

    # Rule 4: hardcoded temp path — LOW. The match is inside a method body
    # but our regex catches all sites.
    assert ("hardcoded_temp_path", "BadHardcodedTemp") in pairs, (
        f"hardcoded_temp_path not detected; got {pairs}"
    )

    # Allowlist suppression — build report once to compute keys, pick the
    # SetCurrentDirectory entry, then re-run with that key allowlisted and
    # confirm it drops out of new_violations.
    pre_report = build_report(list(records), set())
    target = next(
        r for r in pre_report["by_severity"]["MED"]
        if r["rule"] == "set_current_directory" and r["class"] == "BadCwd"
    )
    target_key = target["allowlist_key"]
    # Re-scan to reset allowlist_key state.
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        cs_file = td_path / "SelfTest.cs"
        cs_file.write_text(fixture, encoding="utf-8")
        records2 = scan_file(cs_file, td_path)
    report = build_report(records2, {target_key})
    new_keys = {r["allowlist_key"] for r in report["by_severity"]["MED"]}
    assert target_key not in new_keys, (
        f"allowlist did not suppress entry; target_key={target_key} "
        f"new MED keys={new_keys}"
    )

    # Severity grouping sanity — at least one HIGH, at least one MED, at
    # least one LOW from the fixture (without allowlist).
    report_full = build_report(records, set())
    assert len(report_full["by_severity"]["HIGH"]) >= 1, report_full
    assert len(report_full["by_severity"]["MED"]) >= 1, report_full
    assert len(report_full["by_severity"]["LOW"]) >= 1, report_full

    # Discriminator helper — direct unit checks.
    assert _has_discriminator("Path.Combine(x, Guid.NewGuid().ToString())")
    assert _has_discriminator("Path.GetRandomFileName()")
    assert not _has_discriminator('Path.Combine(x, "literal-only")')

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

    root, allow_path, output = resolve_paths(args)
    if not root.exists():
        print(f"ERROR: scan root not found: {root}", file=sys.stderr)
        return 2

    if args.verbose:
        print(f"scanning {root} ...", file=sys.stderr)

    records, n_files = scan_root(root)
    allowlist = load_allowlist(allow_path)
    report = build_report(records, allowlist, strict=args.strict)
    report["scan_root"] = root.as_posix()
    report["allowlist_path"] = allow_path.as_posix()
    report["files_scanned"] = n_files

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2), encoding="utf-8")

    new_count = report["new_violations"]

    if args.quiet:
        print(
            "global-state-test: "
            f"total={report['total_violations']} "
            f"allowlisted={report['allowlist_size']} "
            f"new={new_count} "
            f"HIGH={len(report['by_severity']['HIGH'])} "
            f"MED={len(report['by_severity']['MED'])} "
            f"LOW={len(report['by_severity']['LOW'])} "
            f"strict={'on' if args.strict else 'off'} "
            f"-> {output}"
        )
    else:
        print_summary(report, output)

    return report["exit_code"]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
