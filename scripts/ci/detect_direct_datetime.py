#!/usr/bin/env python3
"""Direct DateTime.UtcNow / DateTime.Now usage detector — Pattern #100 CI gate.

Pattern #100 ("direct DateTime.{UtcNow,Now} access in production code")
is the failure mode where C# code reaches for the wall-clock singleton
instead of an injected ``TimeProvider`` (or equivalent ``IClock``
abstraction):

    public bool IsExpired(DateTime issuedAt, TimeSpan ttl)
    {
        // BAD: untestable, can't fast-forward, can't freeze.
        return DateTime.UtcNow - issuedAt > ttl;
    }

This is a problem for several reasons:

  * **Untestable**. Any unit test that wants to assert behavior at a
    specific instant has to either ``Thread.Sleep`` (slow, flaky) or
    accept arbitrary skew. With ``TimeProvider`` the test injects a
    ``FakeTimeProvider`` and time is deterministic.
  * **Deadline-loop signal**. In runtime hot paths the pattern often
    appears in deadline / timeout loops:

        while (DateTime.UtcNow < deadline) { ... DoStep(); ... }

    These are subtle bugs: the wall clock can jump backward (NTP step,
    DST off-net, VM resume) and the loop never terminates. A monotonic
    source (``Stopwatch.GetTimestamp()``) or a ``TimeProvider`` is the
    correct primitive.
  * **NuGet API surface**. Code in ``src/SDK/`` and ``src/Bridge/Client/``
    is consumed by external integrators; baking the wall clock into
    public methods denies those consumers the same testability.

The healthy pattern is dependency-injected time:

    public sealed class MyService(TimeProvider time)
    {
        public bool IsExpired(DateTime issuedAt, TimeSpan ttl)
            => time.GetUtcNow() - issuedAt > ttl;
    }

This gate scans ``src/`` for ``DateTime.UtcNow`` and ``DateTime.Now``
references and classifies each site:

  * **HIGH** — call site lives under ``src/SDK/``,
    ``src/Bridge/Client/``, or ``src/Bridge/Protocol/``. NuGet API
    surface; consumers need ``TimeProvider`` testability.
  * **HIGH (deadline-loop signal)** — call site lives under
    ``src/Runtime/Bridge/`` AND the surrounding context shows a
    ``while`` / ``for`` loop with a comparison to the captured timestamp
    (deadline form). These are the subtle wall-clock-jump bugs.
  * **MED**  — call site lives under ``src/Tools/`` or ``src/Domains/``.
    Library code; testability matters but lower urgency.
  * **LOW**  — call site lives in a debug-log / diagnostic file
    (``*Logger.cs``, ``*Diagnostics.cs``, ``*.Debug.cs``). Cosmetic
    timestamps for human-readable output.

Auto-skipped sites (zero recorded as hits):

  * Test files (``src/Tests/``) — tests assert real-clock behavior.
  * Trailing comment ``// TimeProvider-deferred (netstandard2.0)``
    blesses sites that can't migrate yet because the consuming TFM is
    netstandard2.0 (TimeProvider was added in .NET 8).
  * Trailing comment ``// debug-log timestamp`` blesses sites where the
    DateTime is purely cosmetic in a log line.
  * Types decorated with ``[ExcludeFromCodeCoverage]`` — by convention
    these are diagnostic / glue types.
  * File path in ``docs/qa/direct-datetime-allowlist.txt``.

Allowlisting (one entry per line, ``#`` for comments):
``docs/qa/direct-datetime-allowlist.txt``. Two entry forms:

  1. ``severity|file|line`` — line-locked allowlist key.
  2. Bare ``relative/path.cs`` — suppress every site in that file.

CLI:
    python scripts/ci/detect_direct_datetime.py
        [--root <src/>]
        [--allowlist <path>]
        [--output <json>]
        [--strict]
        [--quiet|--verbose]
        [--self-test]

Exit 0 = no NEW (un-allowlisted) HIGH/MED hits; 1 = new violations
detected (CI fails); 2 = scan/usage error. ``--strict`` promotes LOW
findings to fail the gate as well.

Modeled on ``scripts/ci/detect_missing_configureawait.py`` (#275) and
``scripts/ci/detect_logerror_no_stack.py`` (#268). Pairs with the
TimeProvider migration sweep being driven by #285.

This is task #286.
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

# Match ``DateTime.UtcNow`` or ``DateTime.Now`` with a leading word boundary
# so we don't pick up identifiers like ``MyDateTime.UtcNow`` or string
# fragments. We accept either form; ``which`` discriminates.
DATETIME_RE = re.compile(r"(?<![A-Za-z0-9_.])DateTime\.(?P<which>UtcNow|Now)\b")

# Trailing-comment opt-out tokens. Anywhere on the same line as the match,
# suppresses the hit.
TIMEPROVIDER_DEFERRED_RE = re.compile(
    r"//\s*TimeProvider-deferred\b"
)
DEBUG_LOG_TIMESTAMP_RE = re.compile(
    r"//\s*debug-log\s+timestamp\b"
)

# Type-level [ExcludeFromCodeCoverage] decorator. We approximate by checking
# whether the file contains a class / struct / record decorated this way
# AND the match falls inside that type. The simplest sound heuristic is:
# if the file has the attribute at all and the file is otherwise classified
# LOW (cosmetic), suppress. We keep the check coarse — see _is_excluded_type.
EXCLUDE_COVERAGE_RE = re.compile(r"\[ExcludeFromCodeCoverage\]")

# Default scan root.
DEFAULT_SCAN_ROOT = "src"

# Path fragments (POSIX-relative, lowercased) that participate in the
# severity classification. Order matters: HIGH bucket checked first,
# then MED, then LOW. Anything not matching falls through to the
# default tier (MED for src/, otherwise auto-skipped).
HIGH_BOUNDARY_PARTS = (
    "src/sdk/",
    "src/bridge/client/",
    "src/bridge/protocol/",
    # #300: extend HIGH scope to all of src/Runtime/ (BepInEx plugin —
    # wall-clock bugs here can desync the in-game ECS world). Runtime/Bridge
    # remains specially handled below for deadline-loop signal escalation.
    "src/runtime/",
)

# Special HIGH bucket — Runtime/Bridge deadline-loop signal. Files in
# this scope get HIGH (always, since src/runtime/ is in HIGH_BOUNDARY_PARTS),
# but with the ``deadline_loop`` flag set when surrounded by a while/for
# + comparison context. The flag drives the deadline-loop detail message.
RUNTIME_BRIDGE_PART = "src/runtime/bridge/"

MED_BOUNDARY_PARTS = (
    # #300: src/Tools/ remains MED by default (CLI tools, dev utilities).
    # User-visible TUI / WinUI display sites are individually allowlisted
    # in docs/qa/direct-datetime-allowlist.txt (pattern-103-allowed).
    "src/tools/",
    "src/domains/",
)

# Path fragments that mean "do not scan this file at all".
SKIP_BOUNDARY_PARTS = (
    "src/tests/",
)

# Filename patterns (basename, lowercased) that classify a file as
# LOW-tier (debug/diagnostic). The match still fires but ranks LOW.
LOW_FILENAME_PATTERNS = (
    "logger.cs",
    "diagnostics.cs",
    ".debug.cs",
)

EXCLUDED_DIR_PARTS = {"bin", "obj", "node_modules", ".git"}

SEV_HIGH = "HIGH"
SEV_MED = "MED"
SEV_LOW = "LOW"


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


def _is_low_filename(rel_posix: str) -> bool:
    name = rel_posix.rsplit("/", 1)[-1].lower()
    return any(pat in name for pat in LOW_FILENAME_PATTERNS)


def _path_skipped(rel_posix: str) -> bool:
    lo = rel_posix.lower()
    return any(p in lo for p in SKIP_BOUNDARY_PARTS)


def _is_high_path(rel_posix: str) -> bool:
    lo = rel_posix.lower()
    return any(p in lo for p in HIGH_BOUNDARY_PARTS)


def _is_runtime_bridge_path(rel_posix: str) -> bool:
    return RUNTIME_BRIDGE_PART in rel_posix.lower()


def _is_med_path(rel_posix: str) -> bool:
    lo = rel_posix.lower()
    return any(p in lo for p in MED_BOUNDARY_PARTS)


def _line_text(text: str, offset: int) -> str:
    """Return the text of the source line containing *offset*."""
    line_start = text.rfind("\n", 0, offset) + 1
    line_end = text.find("\n", offset)
    if line_end == -1:
        line_end = len(text)
    return text[line_start:line_end]


def _line_carries_optout(text: str, offset: int) -> str | None:
    """Return the opt-out token type (``timeprovider`` / ``debuglog``) if
    the source line containing *offset* has a recognized trailing
    comment, else ``None``."""
    line = _line_text(text, offset)
    if TIMEPROVIDER_DEFERRED_RE.search(line):
        return "timeprovider"
    if DEBUG_LOG_TIMESTAMP_RE.search(line):
        return "debuglog"
    return None


def _is_in_excluded_type(text: str, offset: int) -> bool:
    """Heuristic: is *offset* inside a type decorated with
    ``[ExcludeFromCodeCoverage]``? We walk backward from *offset* to find
    the nearest enclosing ``class``/``struct``/``record`` declaration,
    then check whether the preceding non-blank lines contain the
    attribute. This is intentionally loose — false negatives are
    preferred to false positives because the gate is reporting code
    smells, not correctness violations."""
    # Find the nearest enclosing type-declaration keyword.
    head = text[:offset]
    type_decl_re = re.compile(
        r"\b(class|struct|record)\s+[A-Za-z_][A-Za-z0-9_]*",
    )
    last = None
    for m in type_decl_re.finditer(head):
        last = m
    if last is None:
        return False
    # Walk backward from the type-decl start, collecting attributes.
    pre = head[:last.start()]
    # Look at the last ~10 lines before the decl for an attribute.
    tail_lines = pre.splitlines()[-10:]
    return any(EXCLUDE_COVERAGE_RE.search(ln) for ln in tail_lines)


def _is_in_deadline_loop(text: str, offset: int) -> bool:
    """Heuristic: is *offset* inside a ``while`` or ``for`` loop body
    that uses DateTime arithmetic + a comparison? We take a window of
    +/- 5 lines around the match and check for two signals on different
    lines:

      1. A ``while (`` or ``for (`` token.
      2. A relational comparison operator (``<``, ``>``, ``<=``, ``>=``).
         The match line itself counts.

    Both signals must be present. This is deliberately a heuristic; the
    sound version would require an AST. False positives here are
    acceptable because they only escalate severity, not gate failure
    boundary. False negatives are also acceptable — a missed deadline
    loop just stays MED."""
    lines = text.splitlines()
    # Find the line index of the match.
    line_idx = text.count("\n", 0, offset)
    lo = max(0, line_idx - 5)
    hi = min(len(lines), line_idx + 6)
    window = lines[lo:hi]
    has_loop = any(
        re.search(r"\b(while|for)\s*\(", ln) for ln in window
    )
    if not has_loop:
        return False
    has_cmp = any(
        re.search(r"(<=?|>=?)", ln) for ln in window
    )
    return has_cmp


@dataclass
class Hit:
    file: str            # POSIX-relative to repo root
    line: int
    which: str           # "UtcNow" | "Now"
    severity: str
    detail: str
    boundary: str        # high|med|low — coarse path-bucket tag
    deadline_loop: bool  # True iff classified HIGH via runtime-bridge loop heuristic
    snippet: str         # short slice of the source line
    allowlist_key: str = field(default="")
    in_allowlist: bool = field(default=False)


def _classify(rel: str, text: str, offset: int) -> tuple[str, str, bool]:
    """Return (severity, boundary_tag, deadline_loop_flag) for a hit at
    *offset* in *rel*."""
    # 1) HIGH (Runtime/Bridge deadline loop) — checked BEFORE the broader
    #    src/Runtime/ HIGH bucket so the deadline_loop flag can fire.
    #    Both branches return HIGH; the flag distinguishes the detail
    #    message (deadline-loop wording vs. plain Runtime wording).
    if _is_runtime_bridge_path(rel):
        if _is_in_deadline_loop(text, offset):
            return SEV_HIGH, "high", True
        # fall through to general HIGH below (broader src/Runtime/ scope).

    # 2) HIGH (NuGet API surface + broader src/Runtime/ scope per #300).
    if _is_high_path(rel):
        return SEV_HIGH, "high", False

    # 3) LOW (debug-log / diagnostic filename). Checked before MED so the
    #    filename signal can demote a Tools/Domains file when it's
    #    clearly a diagnostic surface.
    if _is_low_filename(rel):
        return SEV_LOW, "low", False

    # 4) MED (Tools / Domains library code).
    if _is_med_path(rel):
        return SEV_MED, "med", False

    # 5) Default: under src/ but outside known buckets — treat as LOW.
    return SEV_LOW, "low", False


def _scan_one_match(
    text: str,
    rel: str,
    m: re.Match,
) -> Hit | None:
    offset = m.start()

    # Trailing-comment opt-outs.
    if _line_carries_optout(text, offset) is not None:
        return None

    # [ExcludeFromCodeCoverage] type containment.
    if _is_in_excluded_type(text, offset):
        return None

    severity, boundary, deadline_loop = _classify(rel, text, offset)

    line = line_of(text, offset)
    line_body = _line_text(text, offset).strip()
    snippet = line_body if len(line_body) <= 120 else line_body[:117] + "..."
    which = m.group("which")

    if deadline_loop:
        detail = (
            f"DateTime.{which} inside a while/for deadline loop in "
            f"{rel}. Wall-clock comparisons can spin forever on NTP "
            f"step / DST drift / VM resume. Use Stopwatch.GetTimestamp() "
            f"or TimeProvider.GetUtcNow() for monotonic deadlines."
        )
    elif severity == SEV_HIGH:
        detail = (
            f"DateTime.{which} in NuGet API surface ({rel}). Inject "
            f"TimeProvider so consumers can substitute FakeTimeProvider "
            f"in tests. Append // TimeProvider-deferred (netstandard2.0) "
            f"if the consuming TFM blocks migration."
        )
    elif severity == SEV_MED:
        detail = (
            f"DateTime.{which} in library code ({rel}). Prefer an "
            f"injected TimeProvider for testability. Append "
            f"// TimeProvider-deferred (netstandard2.0) on this line "
            f"if migration is blocked, or // debug-log timestamp if "
            f"this is a cosmetic log timestamp."
        )
    else:  # LOW
        detail = (
            f"DateTime.{which} in diagnostic file ({rel}). Cosmetic "
            f"timestamp; safe to leave but tag with "
            f"// debug-log timestamp to bless explicitly."
        )

    return Hit(
        file=rel,
        line=line,
        which=which,
        severity=severity,
        detail=detail,
        boundary=boundary,
        deadline_loop=deadline_loop,
        snippet=snippet,
    )


def scan_file(path: Path, repo_root: Path) -> list[Hit]:
    text = read_text_safe(path)
    if not text:
        return []
    rel = path.relative_to(repo_root).as_posix()
    if _path_skipped(rel):
        return []
    hits: list[Hit] = []
    for m in DATETIME_RE.finditer(text):
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
        "which": h.which,
        "severity": h.severity,
        "boundary": h.boundary,
        "deadline_loop": h.deadline_loop,
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
    low_violations = sorted(
        [h for h in new_hits if h.severity == SEV_LOW],
        key=lambda h: (h.file, h.line),
    )

    high_count = len(high_violations)
    med_count = len(med_violations)
    low_count = len(low_violations)
    fail = high_count > 0 or med_count > 0 or (strict and low_count > 0)
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
        "low_count": low_count,
        "high_violations": [_h2d(h) for h in high_violations],
        "med_violations": [_h2d(h) for h in med_violations],
        "low_violations": [_h2d(h) for h in low_violations],
        "all_hits": [_h2d(h) for h in hits],
        "exit_code": exit_code,
    }


def print_summary(report: dict, output_path: Path) -> None:
    print("direct-datetime gate (Pattern #100)")
    print(f"  files scanned          : {report['scanned_files']}")
    print(f"  total hits             : {report['total_hits']}")
    print(f"  allowlist size         : {report['allowlist_size']}")
    print(f"  NEW hits               : {report['new_hits']}")
    print(f"    HIGH (SDK / Bridge / loops): {report['high_count']}")
    print(f"    MED  (Tools / Domains)     : {report['med_count']}")
    print(f"    LOW  (debug / cosmetic)    : {report['low_count']}")
    if report["new_hits"]:
        print()
        print("NEW direct-DateTime sites:")
        for sev, items in (
            ("HIGH", report["high_violations"]),
            ("MED", report["med_violations"]),
            ("LOW", report["low_violations"]),
        ):
            if not items:
                continue
            print(f"  -- {sev} --")
            for h in items:
                tag = " [deadline-loop]" if h.get("deadline_loop") else ""
                print(
                    f"    [{h['severity']}] {h['file']}:{h['line']}{tag}  "
                    f"DateTime.{h['which']}"
                )
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect direct DateTime.UtcNow / DateTime.Now usage in "
            "production code (Pattern #100). HIGH = SDK + "
            "Bridge.Client + Bridge.Protocol + Runtime/Bridge "
            "deadline loops; MED = Tools + Domains; LOW = debug / "
            "diagnostic files. Tests and TimeProvider-deferred sites "
            "are auto-skipped."
        )
    )
    p.add_argument(
        "--root",
        default=DEFAULT_SCAN_ROOT,
        help=f"Source root to scan (default: {DEFAULT_SCAN_ROOT})",
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/direct-datetime-allowlist.txt",
        help=(
            "Allowlist file; one ``severity|file|line`` key or "
            "bare relative path per line; ``#`` for comments "
            "(default: docs/qa/direct-datetime-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/direct-datetime-report.json",
        help="JSON report output path",
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help=(
            "Promote LOW (debug / cosmetic) findings to fail the gate. "
            "Default: only HIGH+MED fail."
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


_FIXTURE_HIGH_SDK = """
namespace DINOForge.Fixture.Sdk
{
    using System;

    public class BadSdk
    {
        public DateTime Stamp()
        {
            // SDK direct DateTime.UtcNow -> HIGH.
            return DateTime.UtcNow;
        }
    }
}
"""

_FIXTURE_HIGH_NOW = """
namespace DINOForge.Fixture.SdkNow
{
    using System;

    public class BadSdkNow
    {
        public DateTime Stamp()
        {
            // SDK direct DateTime.Now -> HIGH (UtcNow + Now both detected).
            return DateTime.Now;
        }
    }
}
"""

_FIXTURE_DEADLINE_LOOP = """
namespace DINOForge.Fixture.RuntimeBridge
{
    using System;

    public class DeadlineLoop
    {
        public void Pump(DateTime deadline)
        {
            // Runtime/Bridge deadline loop -> HIGH (deadline-loop signal).
            while (DateTime.UtcNow < deadline)
            {
                Step();
            }
        }
        private void Step() { }
    }
}
"""

_FIXTURE_RUNTIME_BRIDGE_PLAIN = """
namespace DINOForge.Fixture.RuntimeBridgePlain
{
    using System;

    public class JustATimestamp
    {
        public DateTime Stamp()
        {
            // Runtime/Bridge but no while/for + cmp -> MED.
            return DateTime.UtcNow;
        }
    }
}
"""

_FIXTURE_TOOLS_MED = """
namespace DINOForge.Fixture.Tools
{
    using System;

    public class ToolSite
    {
        public DateTime Stamp()
        {
            // Tools/Cli direct DateTime.UtcNow -> MED.
            return DateTime.UtcNow;
        }
    }
}
"""

_FIXTURE_LOW_LOGGER_FILENAME = """
namespace DINOForge.Fixture.Logger
{
    using System;

    public class SomeLogger
    {
        public string Format()
        {
            // *Logger.cs filename -> LOW (cosmetic).
            return DateTime.UtcNow.ToString("o");
        }
    }
}
"""

_FIXTURE_TIMEPROVIDER_DEFERRED = """
namespace DINOForge.Fixture.SdkDeferred
{
    using System;

    public class DeferredSdk
    {
        public DateTime Stamp()
        {
            // Trailing comment opt-out — netstandard2.0 blocker.
            return DateTime.UtcNow; // TimeProvider-deferred (netstandard2.0)
        }
    }
}
"""

_FIXTURE_DEBUG_LOG_COMMENT = """
namespace DINOForge.Fixture.SdkDebugLog
{
    using System;

    public class DebugLogSdk
    {
        public string LogLine()
        {
            // Trailing comment opt-out — purely cosmetic.
            return DateTime.UtcNow.ToString("o"); // debug-log timestamp
        }
    }
}
"""

_FIXTURE_EXCLUDE_FROM_COVERAGE = """
namespace DINOForge.Fixture.SdkExcl
{
    using System;
    using System.Diagnostics.CodeAnalysis;

    [ExcludeFromCodeCoverage]
    public class ExcludedFromCoverageSdk
    {
        public DateTime Stamp()
        {
            // [ExcludeFromCodeCoverage] suppresses the hit.
            return DateTime.UtcNow;
        }
    }
}
"""

_FIXTURE_SKIP_TESTS = """
namespace DINOForge.Fixture.Tests
{
    using System;

    public class TestSite
    {
        public DateTime Stamp()
        {
            // Tests scope is auto-skipped.
            return DateTime.UtcNow;
        }
    }
}
"""

_FIXTURE_HEALTHY_TIMEPROVIDER = """
namespace DINOForge.Fixture.SdkHealthy
{
    using System;

    public class GoodSdk
    {
        private readonly TimeProvider _time;
        public GoodSdk(TimeProvider time) { _time = time; }
        public DateTimeOffset Stamp()
        {
            // Healthy: routes through injected TimeProvider, no direct
            // wall-clock singleton in this method.
            return _time.GetUtcNow();
        }
    }
}
"""


def _build_fixture_repo(td: Path) -> Path:
    """Lay out a synthetic repo under *td* matching the gate's expected
    structure."""
    repo = td / "repo"
    sdk = repo / "src" / "SDK" / "ApiSurface"
    sdk_now = repo / "src" / "SDK" / "NowFlavor"
    sdk_deferred = repo / "src" / "SDK" / "Deferred"
    sdk_debug = repo / "src" / "SDK" / "DebugLog"
    sdk_excl = repo / "src" / "SDK" / "Excl"
    sdk_healthy = repo / "src" / "SDK" / "Healthy"
    runtime_bridge = repo / "src" / "Runtime" / "Bridge" / "Loop"
    runtime_bridge_plain = repo / "src" / "Runtime" / "Bridge" / "Plain"
    tools = repo / "src" / "Tools" / "Cli"
    logger = repo / "src" / "Tools" / "Cli"  # *Logger.cs lives here
    tests = repo / "src" / "Tests"
    for d in (
        sdk, sdk_now, sdk_deferred, sdk_debug, sdk_excl, sdk_healthy,
        runtime_bridge, runtime_bridge_plain, tools, logger, tests,
    ):
        d.mkdir(parents=True, exist_ok=True)

    (sdk / "BadSdk.cs").write_text(_FIXTURE_HIGH_SDK, encoding="utf-8")
    (sdk_now / "BadSdkNow.cs").write_text(_FIXTURE_HIGH_NOW, encoding="utf-8")
    (sdk_deferred / "DeferredSdk.cs").write_text(
        _FIXTURE_TIMEPROVIDER_DEFERRED, encoding="utf-8"
    )
    (sdk_debug / "DebugLogSdk.cs").write_text(
        _FIXTURE_DEBUG_LOG_COMMENT, encoding="utf-8"
    )
    (sdk_excl / "ExcludedFromCoverageSdk.cs").write_text(
        _FIXTURE_EXCLUDE_FROM_COVERAGE, encoding="utf-8"
    )
    (sdk_healthy / "GoodSdk.cs").write_text(
        _FIXTURE_HEALTHY_TIMEPROVIDER, encoding="utf-8"
    )
    (runtime_bridge / "DeadlineLoop.cs").write_text(
        _FIXTURE_DEADLINE_LOOP, encoding="utf-8"
    )
    (runtime_bridge_plain / "JustATimestamp.cs").write_text(
        _FIXTURE_RUNTIME_BRIDGE_PLAIN, encoding="utf-8"
    )
    (tools / "ToolSite.cs").write_text(_FIXTURE_TOOLS_MED, encoding="utf-8")
    (logger / "SomeLogger.cs").write_text(
        _FIXTURE_LOW_LOGGER_FILENAME, encoding="utf-8"
    )
    (tests / "TestSite.cs").write_text(_FIXTURE_SKIP_TESTS, encoding="utf-8")

    return repo


def _self_test() -> int:  # noqa: C901
    import tempfile

    # 1) Regex sanity — direct string checks.
    assert DATETIME_RE.search("var t = DateTime.UtcNow;")
    assert DATETIME_RE.search("return DateTime.Now.ToString();")
    # ``MyDateTime.UtcNow`` — leading character is ``y`` (alnum), so the
    # negative lookbehind blocks the match.
    assert not DATETIME_RE.search("return MyDateTime.UtcNow;")
    # ``DateTimeOffset.UtcNow`` — the regex anchors on ``DateTime.``
    # exactly; ``DateTimeOffset.UtcNow`` has ``DateTimeO`` before the
    # dot, so the literal ``DateTime.`` token doesn't appear.
    assert not DATETIME_RE.search("return DateTimeOffset.UtcNow;")

    # Opt-out token recognition.
    assert TIMEPROVIDER_DEFERRED_RE.search(
        "return DateTime.UtcNow; // TimeProvider-deferred (netstandard2.0)"
    )
    assert DEBUG_LOG_TIMESTAMP_RE.search(
        'return DateTime.UtcNow.ToString(); // debug-log timestamp'
    )

    # Severity classification on canonical paths.
    sev, bnd, dl = _classify(
        "src/SDK/Foo.cs",
        "var t = DateTime.UtcNow;\n",
        len("var t = "),
    )
    assert sev == SEV_HIGH and bnd == "high" and dl is False, (sev, bnd, dl)

    sev, bnd, dl = _classify(
        "src/Tools/Cli/Foo.cs",
        "var t = DateTime.UtcNow;\n",
        len("var t = "),
    )
    assert sev == SEV_MED and bnd == "med" and dl is False, (sev, bnd, dl)

    sev, bnd, dl = _classify(
        "src/Tools/Cli/SomeLogger.cs",
        "var t = DateTime.UtcNow;\n",
        len("var t = "),
    )
    assert sev == SEV_LOW and bnd == "low" and dl is False, (sev, bnd, dl)

    # Runtime/Bridge with surrounding while loop -> HIGH (deadline-loop).
    rt_loop_text = (
        "while (DateTime.UtcNow < deadline)\n"
        "{\n"
        "    var t = DateTime.UtcNow;\n"
        "}\n"
    )
    inner = rt_loop_text.find("var t = ") + len("var t = ")
    sev, bnd, dl = _classify(
        "src/Runtime/Bridge/Foo.cs",
        rt_loop_text,
        inner,
    )
    assert sev == SEV_HIGH and dl is True, (sev, bnd, dl)

    # #300: Runtime/Plugin.cs (NOT under Bridge/) -> HIGH via broadened scope.
    sev, bnd, dl = _classify(
        "src/Runtime/Plugin.cs",
        "var t = DateTime.Now;\n",
        len("var t = "),
    )
    assert sev == SEV_HIGH and bnd == "high" and dl is False, (sev, bnd, dl)

    # #300: Runtime/UI/Foo.cs -> HIGH (broader Runtime scope, not Bridge).
    sev, bnd, dl = _classify(
        "src/Runtime/UI/Foo.cs",
        "var t = DateTime.UtcNow;\n",
        len("var t = "),
    )
    assert sev == SEV_HIGH and bnd == "high" and dl is False, (sev, bnd, dl)

    # #300: src/Tools/Cli/Commands/WatchCommand.cs site is bare-path
    # allowlisted (pattern-103-allowed: user-visible TUI display) — the
    # raw classification is still MED (Tools/), but the allowlist drops it.
    sev, bnd, dl = _classify(
        "src/Tools/Cli/Commands/WatchCommand.cs",
        "var t = DateTime.Now;\n",
        len("var t = "),
    )
    assert sev == SEV_MED and bnd == "med" and dl is False, (sev, bnd, dl)

    # 2) End-to-end scan against the synthetic repo.
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        repo = _build_fixture_repo(td_path)

        hits, n_files = scan_root(repo, "src")
        files_seen = {h.file for h in hits}

        # Auto-skipped: tests, TimeProvider-deferred line, debug-log
        # timestamp line, [ExcludeFromCodeCoverage] type, healthy
        # TimeProvider site.
        assert not any(
            "TestSite.cs" in f for f in files_seen
        ), f"Tests scope leaked: {files_seen}"
        assert not any(
            "DeferredSdk.cs" in f for f in files_seen
        ), f"// TimeProvider-deferred not honored: {files_seen}"
        assert not any(
            "DebugLogSdk.cs" in f for f in files_seen
        ), f"// debug-log timestamp not honored: {files_seen}"
        assert not any(
            "ExcludedFromCoverageSdk.cs" in f for f in files_seen
        ), f"[ExcludeFromCodeCoverage] not honored: {files_seen}"
        assert not any(
            "GoodSdk.cs" in f for f in files_seen
        ), f"healthy TimeProvider flagged: {files_seen}"

        # HIGH: BadSdk (UtcNow), BadSdkNow (Now), DeadlineLoop.
        high = [h for h in hits if h.severity == SEV_HIGH]
        high_files = {h.file for h in high}
        assert any(
            "BadSdk.cs" in f for f in high_files
        ), f"missing HIGH for BadSdk.cs (UtcNow): {high_files}"
        assert any(
            "BadSdkNow.cs" in f for f in high_files
        ), f"missing HIGH for BadSdkNow.cs (Now): {high_files}"
        assert any(
            "DeadlineLoop.cs" in f for f in high_files
        ), f"missing HIGH for DeadlineLoop.cs: {high_files}"

        # Deadline-loop flag set on the runtime/bridge loop hit but not
        # on plain SDK hits.
        deadline_hits = [h for h in high if h.deadline_loop]
        assert any(
            "DeadlineLoop.cs" in h.file for h in deadline_hits
        ), f"deadline_loop flag missing: {deadline_hits}"
        non_deadline_high = [h for h in high if not h.deadline_loop]
        assert any(
            "BadSdk.cs" in h.file for h in non_deadline_high
        ), f"BadSdk.cs incorrectly flagged as deadline-loop"

        # MED: ToolSite (Tools/ default classification).
        med = [h for h in hits if h.severity == SEV_MED]
        med_files = {h.file for h in med}
        assert any(
            "ToolSite.cs" in f for f in med_files
        ), f"missing MED for ToolSite.cs: {med_files}"

        # #300: JustATimestamp (Runtime/Bridge no loop) is now HIGH —
        # broader src/Runtime/ scope catches it after the deadline-loop
        # check fails. The deadline_loop flag stays False (no loop), so
        # the detail wording is the plain "NuGet API surface" form.
        assert any(
            "JustATimestamp.cs" in h.file
            and h.severity == SEV_HIGH
            and h.deadline_loop is False
            for h in hits
        ), (
            f"#300: JustATimestamp.cs (Runtime/Bridge plain) should "
            f"now be HIGH (non-deadline) under broadened scope: "
            f"{[(h.file, h.severity, h.deadline_loop) for h in hits]}"
        )

        # LOW: SomeLogger.cs (filename signal).
        low = [h for h in hits if h.severity == SEV_LOW]
        low_files = {h.file for h in low}
        assert any(
            "SomeLogger.cs" in f for f in low_files
        ), f"missing LOW for SomeLogger.cs: {low_files}"

        # Both UtcNow and Now flavors detected.
        whichs = {h.which for h in hits}
        assert "UtcNow" in whichs, whichs
        assert "Now" in whichs, whichs

        # 3) Allowlist suppression — line-locked key.
        report_pre = build_report(list(hits), set(), n_files)
        target = next(
            h for h in hits
            if h.severity == SEV_HIGH and "BadSdk.cs" in h.file
        )
        target_key = target.allowlist_key
        hits2, _ = scan_root(repo, "src")
        report_post = build_report(list(hits2), {target_key}, n_files)
        post_keys = {h["allowlist_key"] for h in report_post["high_violations"]}
        assert target_key not in post_keys, (
            f"allowlist did not suppress {target_key}; remaining: {post_keys}"
        )
        assert report_post["new_hits"] < report_pre["new_hits"], (
            f"allowlist did not reduce new_hits: pre={report_pre['new_hits']} "
            f"post={report_post['new_hits']}"
        )

        # 4) Bare-path allowlist — listing a relative file path drops
        #    every hit in that file.
        hits3, _ = scan_root(repo, "src")
        bare_path = "src/Tools/Cli/ToolSite.cs"
        report_bare = build_report(list(hits3), {bare_path}, n_files)
        bare_files = {h["file"] for h in report_bare["med_violations"]}
        assert bare_path not in bare_files, (
            f"bare-path allowlist did not suppress {bare_path}"
        )

        # 5) Strict mode promotes LOW to fail.
        hits4, _ = scan_root(repo, "src")
        non_strict = build_report(list(hits4), set(), n_files, strict=False)
        # HIGH+MED already fail in non-strict.
        assert non_strict["exit_code"] == 1, non_strict

        suppress = {
            h.allowlist_key for h in hits4
            if h.severity in (SEV_HIGH, SEV_MED)
        }
        hits5, _ = scan_root(repo, "src")
        rpt_strict_low_only = build_report(
            list(hits5), suppress, n_files, strict=True,
        )
        assert rpt_strict_low_only["high_count"] == 0
        assert rpt_strict_low_only["med_count"] == 0
        assert rpt_strict_low_only["low_count"] >= 1
        assert rpt_strict_low_only["exit_code"] == 1, rpt_strict_low_only

        hits6, _ = scan_root(repo, "src")
        rpt_lax_low_only = build_report(
            list(hits6), suppress, n_files, strict=False,
        )
        assert rpt_lax_low_only["exit_code"] == 0, rpt_lax_low_only

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
            "direct-datetime: "
            f"files={report['scanned_files']} "
            f"total={report['total_hits']} "
            f"allowlisted={report['allowlist_size']} "
            f"new={report['new_hits']} "
            f"HIGH={report['high_count']} "
            f"MED={report['med_count']} "
            f"LOW={report['low_count']} "
            f"strict={'on' if args.strict else 'off'} "
            f"-> {output}"
        )
    else:
        print_summary(report, output)

    return report["exit_code"]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
