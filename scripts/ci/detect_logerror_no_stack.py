#!/usr/bin/env python3
"""LogError-no-stack detector — Pattern #96 CI gate.

Pattern #96 ("Log calls that drop the exception stack trace") is the
failure mode where C# code catches an exception and logs only its
``.Message`` property. The string-interpolation form is the most common:

    catch (Exception ex)
    {
        _log.LogError($"Failed to do thing: {ex.Message}");
    }

The concatenation form is rarer but equally bad:

    _log.LogError("Failed to do thing: " + ex.Message);

Both throw away ``StackTrace``, ``InnerException``, and the exception
type name. When the same code path fails in production we get a single
flat string and have no way to find which call site / which framework
exception triggered the catch. The healthy pattern passes the exception
as a SECOND argument so the logger renders the full type + message +
stack chain (or interpolates ``{ex}`` directly, which is equivalent
because the default ``Exception.ToString()`` already includes type +
message + stack):

    _log.LogError(ex, "Failed to do thing");          // proper logger
    _log.LogError($"Failed to do thing: {ex}");        // also OK

This gate scans ``src/`` for log calls that match the lossy pattern and
classifies each site:

  * **HIGH** — call site lives under ``src/Runtime/``, ``src/Bridge/``,
    ``src/SDK/``, or ``src/Domains/``. These are core paths; missing
    stack traces hide failures that cross runtime boundaries.
  * **MED**  — call site lives under ``src/Tools/`` or ``src/Tests/``.
    Cleanup-quality. Tools frequently rerun manually so the operator
    can re-trigger; Tests inspect the message string directly.
  * **LOW**  — any file but ONLY a ``LogWarning`` hit (not LogError /
    LogCritical / LogFatal / LogException). Warnings are sometimes
    legitimately message-only (expected fallbacks). Reported for
    visibility; ``--strict`` promotes to fail the gate.

Auto-skipped sites (zero recorded as hits):

  * Calls passing the exception as a SECOND positional argument
    (``LogError("msg", ex)`` or ``LogError(ex, "msg")``) — proper
    logger pattern.
  * Calls using the full exception interpolation (``{ex}``, ``{e}``,
    or ``{exception}`` without the ``.Message`` suffix) — renders
    ``ToString()`` = type+msg+stack.

Allowlisting (one entry per line, ``#`` for comments):
``docs/qa/logerror-no-stack-allowlist.txt``. Two entry forms:

  1. ``severity|file|line|method`` — line-locked allowlist key.
  2. Bare ``relative/path.cs`` — suppress every site in that file.

CLI:
    python scripts/ci/detect_logerror_no_stack.py
        [--root <src/>]
        [--allowlist <path>]
        [--output <json>]
        [--strict]
        [--quiet|--verbose]
        [--self-test]

Exit 0 = no NEW (un-allowlisted) HIGH/MED hits; 1 = new violations
detected (CI fails); 2 = scan/usage error. ``--strict`` promotes LOW
findings to fail the gate as well.

Modeled on ``scripts/ci/detect_unguarded_deserialize.py`` (#265) and
``scripts/ci/detect_global_state_tests.py`` (#257). Pairs with the
LogError mop-up sweep being driven by #267 (Plugin.cs + ModPlatform.cs
27 residuals).

This is task #268.
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

# Identifiers used as the exception variable name inside the interpolation.
# Anything else (e.g. ``response.Error``) is excluded from the lossy form
# because the gate cannot reason about whether an arbitrary expression carries
# stack info — only the canonical caught-exception variables do.
_EX_NAMES = ("ex", "e", "exception")

# Names that count as "log methods that ought to carry stack" for the gate.
LOG_METHODS = (
    "LogError",
    "LogWarning",
    "LogCritical",
    "LogFatal",
    "LogException",
)

_LOG_METHOD_RE = "|".join(re.escape(m) for m in LOG_METHODS)
_EX_NAME_RE = "|".join(_EX_NAMES)

# Form 1: $"...{ex.Message}..." inside a LogXxx(...) call.
# Captures the log method and the exception variable name. We DO NOT
# anchor on the closing paren — the call may span lines.
INTERPOLATION_RE = re.compile(
    r"\b(?P<method>" + _LOG_METHOD_RE + r")\s*\("
    r"\s*\$\"[^\"]*"                       # opening of $"..."
    r"\{(?P<exvar>" + _EX_NAME_RE + r")\.Message\}",
    re.DOTALL,
)

# Form 2: "..." + ex.Message  (concatenation form). We allow optional
# whitespace and span up to the ``+ ex.Message`` token. The "..." literal
# may be a non-interpolated regular string.
CONCAT_RE = re.compile(
    r"\b(?P<method>" + _LOG_METHOD_RE + r")\s*\("
    r"\s*\"[^\"]*\"\s*\+\s*"
    r"(?P<exvar>" + _EX_NAME_RE + r")\.Message\b",
    re.DOTALL,
)

# Healthy: full exception interpolation ``{ex}`` (without ``.Message``).
# When present, we DO NOT flag the site; ``ex.ToString()`` already
# includes type+msg+stack.
FULL_EX_INTERP_RE = re.compile(
    r"\{(?P<exvar>" + _EX_NAME_RE + r")(?:[:,][^}]*)?\}"
)

# Default scan root.
DEFAULT_SCAN_ROOT = "src"

# Path fragments (POSIX-relative, lowercased) that promote a hit to HIGH
# severity when not LogWarning.
HIGH_BOUNDARY_PARTS = (
    "src/runtime/",
    "src/bridge/",
    "src/sdk/",
    "src/domains/",
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


def _is_high_boundary(rel_posix: str) -> bool:
    lo = rel_posix.lower()
    return any(p in lo for p in HIGH_BOUNDARY_PARTS)


def _find_call_close(text: str, open_paren_idx: int) -> int:
    """Return offset just past the matching ``)`` for the ``(`` at
    *open_paren_idx*. Skips strings and char literals so commas/parens
    inside them do not throw the bracket counter off. Returns ``-1`` on
    EOF without a match."""
    depth = 0
    i = open_paren_idx
    end = len(text)
    in_string = False
    in_verbatim = False
    in_interp_string = False
    in_char = False
    while i < end:
        c = text[i]
        n = text[i + 1] if i + 1 < end else ""
        if in_verbatim:
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
            elif c == "(":
                depth += 1
            elif c == ")":
                depth -= 1
                if depth == 0:
                    return i + 1
        i += 1
    return -1


def _has_exception_second_arg(call_body: str, exvar: str) -> bool:
    """Heuristic: does the LogXxx(...) call body include the exception
    as a positional argument outside the format string? We flag
    ``LogError("...", ex)`` and ``LogError(ex, "...")`` as healthy. The
    check is intentionally lenient: we look for ``,\s*<exvar>\b`` or
    ``\b<exvar>\s*,`` outside of any string literal — any such
    occurrence implies the exception itself is being passed."""
    # Strip out string literals so we don't false-positive on a `, ex`
    # sequence appearing inside the formatted message itself.
    stripped = _strip_strings(call_body)
    pat_after_comma = re.compile(r",\s*" + re.escape(exvar) + r"\b")
    pat_before_comma = re.compile(r"\b" + re.escape(exvar) + r"\s*,")
    return bool(pat_after_comma.search(stripped) or
                pat_before_comma.search(stripped))


def _has_full_ex_interp(call_body: str, exvar: str) -> bool:
    """Return True when *call_body* contains a ``{<exvar>}`` interpolation
    (without ``.Message``). The interpolation may carry a format spec
    (``{ex:G}``) — that's fine, ToString() is still rendered."""
    for m in FULL_EX_INTERP_RE.finditer(call_body):
        if m.group("exvar") == exvar:
            return True
    return False


def _strip_strings(text: str) -> str:
    """Replace string-literal contents with spaces so structural commas
    and parens are preserved but format-string content cannot trigger
    accidental matches. Handles regular, verbatim, and interpolated
    strings."""
    out: list[str] = []
    i = 0
    end = len(text)
    in_string = False
    in_verbatim = False
    in_interp = False
    while i < end:
        c = text[i]
        n = text[i + 1] if i + 1 < end else ""
        if in_verbatim:
            if c == '"':
                if n == '"':
                    out.append("  ")
                    i += 2
                    continue
                in_verbatim = False
                out.append('"')
            else:
                out.append(" ")
        elif in_string or in_interp:
            if c == "\\" and n:
                out.append("  ")
                i += 2
                continue
            if c == '"':
                in_string = False
                in_interp = False
                out.append('"')
            else:
                out.append(" ")
        else:
            if c == "@" and n == '"':
                in_verbatim = True
                out.append('@"')
                i += 2
                continue
            if c == "$" and n == '"':
                in_interp = True
                out.append('$"')
                i += 2
                continue
            if c == '"':
                in_string = True
                out.append('"')
            else:
                out.append(c)
        i += 1
    return "".join(out)


@dataclass
class Hit:
    file: str            # POSIX-relative to repo root
    line: int
    method: str          # LogError / LogWarning / etc.
    form: str            # interpolation | concat
    exvar: str
    severity: str
    detail: str
    boundary: str        # high|med — coarse path-bucket tag
    allowlist_key: str = field(default="")
    in_allowlist: bool = field(default=False)


def _classify(method: str, rel: str) -> str:
    """Return SEV_HIGH / SEV_MED / SEV_LOW for a (method, file) pair.
    LogWarning is always LOW (informational). LogError + friends in core
    paths are HIGH; elsewhere MED."""
    if method == "LogWarning":
        return SEV_LOW
    if _is_high_boundary(rel):
        return SEV_HIGH
    return SEV_MED


def _scan_one_match(
    text: str,
    rel: str,
    method_match: re.Match,
    form: str,
) -> Hit | None:
    method = method_match.group("method")
    exvar = method_match.group("exvar")
    # Locate the opening ``(`` of the LogXxx call. The regex anchored on
    # the method name — find the first ``(`` after that token.
    after_method = method_match.start() + len(method)
    paren_idx = text.find("(", after_method)
    if paren_idx == -1:
        return None
    close_idx = _find_call_close(text, paren_idx)
    if close_idx == -1:
        return None
    call_body = text[paren_idx + 1:close_idx - 1]

    # Healthy: exception as second positional argument.
    if _has_exception_second_arg(call_body, exvar):
        return None
    # Healthy: full ``{ex}`` interpolation already in the format string.
    # (Interpolation form caught us via ``{ex.Message}`` so we know
    # ``.Message`` is present; but the user could also include ``{ex}``
    # alongside — if BOTH are present, ToString() covers the stack so
    # the site is healthy.)
    if _has_full_ex_interp(call_body, exvar):
        return None

    line = line_of(text, method_match.start())
    sev = _classify(method, rel)
    high_boundary = _is_high_boundary(rel)

    if form == "interpolation":
        detail = (
            f"{method}($\"...{{{exvar}}}.Message...\") drops the stack "
            f"trace. Pass {exvar} as a positional argument: "
            f"{method}({exvar}, \"msg\") or interpolate "
            f"{{{exvar}}} (without .Message) to render ToString()."
        )
    else:  # concat
        detail = (
            f"{method}(\"...\" + {exvar}.Message) drops the stack trace. "
            f"Pass {exvar} as a positional argument: "
            f"{method}({exvar}, \"msg\")."
        )

    return Hit(
        file=rel,
        line=line,
        method=method,
        form=form,
        exvar=exvar,
        severity=sev,
        detail=detail,
        boundary="high" if high_boundary else "med",
    )


def scan_file(path: Path, repo_root: Path) -> list[Hit]:
    text = read_text_safe(path)
    if not text:
        return []
    rel = path.relative_to(repo_root).as_posix()
    hits: list[Hit] = []

    for m in INTERPOLATION_RE.finditer(text):
        h = _scan_one_match(text, rel, m, "interpolation")
        if h is not None:
            hits.append(h)

    for m in CONCAT_RE.finditer(text):
        h = _scan_one_match(text, rel, m, "concat")
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
    for f in files:
        hits.extend(scan_file(f, repo_root))
    return hits, len(files)


# ----------------------------------------------------------------------------
# Report
# ----------------------------------------------------------------------------


def _hit_key(h: Hit) -> str:
    """Stable allowlist key: ``severity|file|line|method``."""
    return f"{h.severity}|{h.file}|{h.line}|{h.method}"


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
        "method": h.method,
        "form": h.form,
        "exvar": h.exvar,
        "severity": h.severity,
        "boundary": h.boundary,
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

    # In strict mode, LogWarning hits in core paths could promote to HIGH
    # — but we keep classification simple: strict only affects the
    # exit_code, not the bucket assignment. This mirrors #265.
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
        "files_scanned": files_scanned,
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
    print("logerror-no-stack gate (Pattern #96)")
    print(f"  files scanned          : {report['files_scanned']}")
    print(f"  total hits             : {report['total_hits']}")
    print(f"  allowlist size         : {report['allowlist_size']}")
    print(f"  NEW hits               : {report['new_hits']}")
    print(f"    HIGH (core paths)         : {report['high_count']}")
    print(f"    MED  (Tools / Tests)      : {report['med_count']}")
    print(f"    LOW  (LogWarning)         : {report['low_count']}")
    if report["new_hits"]:
        print()
        print("NEW logerror-no-stack sites:")
        for sev, items in (
            ("HIGH", report["high_violations"]),
            ("MED", report["med_violations"]),
            ("LOW", report["low_violations"]),
        ):
            if not items:
                continue
            print(f"  -- {sev} --")
            for h in items:
                print(
                    f"    [{h['severity']}] {h['method']:<14} "
                    f"{h['file']}:{h['line']}  ({h['form']})"
                )
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect log calls that drop the exception stack trace "
            "(Pattern #96). HIGH = LogError/Critical/Fatal/Exception in "
            "core paths (Runtime/Bridge/SDK/Domains); MED = same in "
            "Tools/Tests; LOW = LogWarning anywhere (info)."
        )
    )
    p.add_argument(
        "--root",
        default=DEFAULT_SCAN_ROOT,
        help=f"Source root to scan (default: {DEFAULT_SCAN_ROOT})",
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/logerror-no-stack-allowlist.txt",
        help=(
            "Allowlist file; one ``severity|file|line|method`` key or "
            "bare relative path per line; ``#`` for comments "
            "(default: docs/qa/logerror-no-stack-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/logerror-no-stack-report.json",
        help="JSON report output path",
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help=(
            "Promote LOW (LogWarning) findings to fail the gate. "
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


_FIXTURE_INTERP = """
namespace DINOForge.Fixture.Interp
{
    using Microsoft.Extensions.Logging;
    using System;

    public class BadInterp
    {
        private readonly ILogger _log;
        public BadInterp(ILogger log) { _log = log; }

        public void Boom()
        {
            try { } catch (Exception ex)
            {
                // Lossy interpolation form -> HIGH (core path under SDK).
                _log.LogError($"Failed to do thing: {ex.Message}");
            }
        }
    }
}
"""

_FIXTURE_CONCAT = """
namespace DINOForge.Fixture.Concat
{
    using Microsoft.Extensions.Logging;
    using System;

    public class BadConcat
    {
        private readonly ILogger _log;
        public BadConcat(ILogger log) { _log = log; }

        public void Boom()
        {
            try { } catch (Exception ex)
            {
                // Lossy concat form -> HIGH (core path under Runtime).
                _log.LogError("Failed to do thing: " + ex.Message);
            }
        }
    }
}
"""

_FIXTURE_HEALTHY_SECOND_ARG = """
namespace DINOForge.Fixture.Healthy
{
    using Microsoft.Extensions.Logging;
    using System;

    public class GoodSecondArg
    {
        private readonly ILogger _log;
        public GoodSecondArg(ILogger log) { _log = log; }

        public void Boom()
        {
            try { } catch (Exception ex)
            {
                // Proper logger pattern: ex as positional arg.
                _log.LogError(ex, "Failed to do thing: {Message}", ex.Message);
            }
        }
    }
}
"""

_FIXTURE_HEALTHY_FULL_EX = """
namespace DINOForge.Fixture.HealthyFull
{
    using Microsoft.Extensions.Logging;
    using System;

    public class GoodFullEx
    {
        private readonly ILogger _log;
        public GoodFullEx(ILogger log) { _log = log; }

        public void Boom()
        {
            try { } catch (Exception ex)
            {
                // {ex} renders ToString() = type+msg+stack -> HEALTHY.
                _log.LogError($"Failed: {ex}");
            }
        }
    }
}
"""

_FIXTURE_LOGWARNING = """
namespace DINOForge.Fixture.Warn
{
    using Microsoft.Extensions.Logging;
    using System;

    public class WarnSite
    {
        private readonly ILogger _log;
        public WarnSite(ILogger log) { _log = log; }

        public void Boom()
        {
            try { } catch (Exception ex)
            {
                // LogWarning -> LOW by default; --strict promotes to fail.
                _log.LogWarning($"Recoverable: {ex.Message}");
            }
        }
    }
}
"""

_FIXTURE_TOOLS_MED = """
namespace DINOForge.Fixture.ToolsMed
{
    using Microsoft.Extensions.Logging;
    using System;

    public class ToolsSite
    {
        private readonly ILogger _log;
        public ToolsSite(ILogger log) { _log = log; }

        public void Boom()
        {
            try { } catch (Exception ex)
            {
                // Tools path -> MED.
                _log.LogError($"CLI broke: {ex.Message}");
            }
        }
    }
}
"""


def _build_fixture_repo(td: Path) -> Path:
    """Lay out a synthetic repo under *td* matching the gate's expected
    structure: HIGH boundaries (Runtime/Bridge/SDK/Domains), MED
    boundaries (Tools/Tests), and a few healthy variants."""
    repo = td / "repo"
    sdk = repo / "src" / "SDK" / "Validation"
    runtime = repo / "src" / "Runtime"
    bridge = repo / "src" / "Bridge" / "Protocol"
    domains = repo / "src" / "Domains" / "Warfare"
    tools = repo / "src" / "Tools" / "Cli"
    tests = repo / "src" / "Tests"
    for d in (sdk, runtime, bridge, domains, tools, tests):
        d.mkdir(parents=True, exist_ok=True)

    # HIGH boundary: SDK + Runtime, two lossy forms.
    (sdk / "BadInterp.cs").write_text(_FIXTURE_INTERP, encoding="utf-8")
    (runtime / "BadConcat.cs").write_text(_FIXTURE_CONCAT, encoding="utf-8")

    # Healthy sites in HIGH boundary — should NOT be flagged.
    (bridge / "GoodSecondArg.cs").write_text(
        _FIXTURE_HEALTHY_SECOND_ARG, encoding="utf-8"
    )
    (domains / "GoodFullEx.cs").write_text(
        _FIXTURE_HEALTHY_FULL_EX, encoding="utf-8"
    )

    # LogWarning in HIGH boundary — LOW by default.
    (sdk / "WarnSite.cs").write_text(_FIXTURE_LOGWARNING, encoding="utf-8")

    # MED boundary: Tools.
    (tools / "ToolsSite.cs").write_text(
        _FIXTURE_TOOLS_MED, encoding="utf-8"
    )

    return repo


def _self_test() -> int:  # noqa: C901
    import tempfile

    # 1) Regex sanity — direct string checks.
    assert INTERPOLATION_RE.search('LogError($"x: {ex.Message}")')
    assert INTERPOLATION_RE.search('_log.LogError($"x: {e.Message}")')
    assert INTERPOLATION_RE.search(
        '_log.LogError($"x: {exception.Message}")'
    )
    # Concat form.
    assert CONCAT_RE.search('LogError("x: " + ex.Message)')
    assert CONCAT_RE.search('_log.LogWarning("x: " + e.Message)')
    # Non-matching: ``{ex}`` without ``.Message``.
    assert not INTERPOLATION_RE.search('LogError($"x: {ex}")')

    # _strip_strings smoke check — preserves structural tokens.
    sample = 'LogError($"oops: {ex.Message}", ex)'
    stripped = _strip_strings(sample)
    assert "(" in stripped and ")" in stripped, stripped
    assert ", ex" in stripped, stripped

    # 2) End-to-end scan against the synthetic repo.
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        repo = _build_fixture_repo(td_path)

        hits, n_files = scan_root(repo, "src")
        files_seen = {h.file for h in hits}

        # Healthy sites must NOT appear.
        assert not any(
            "GoodSecondArg.cs" in f for f in files_seen
        ), f"healthy second-arg flagged: {files_seen}"
        assert not any(
            "GoodFullEx.cs" in f for f in files_seen
        ), f"healthy full-ex flagged: {files_seen}"

        # HIGH: BadInterp under SDK + BadConcat under Runtime.
        high = [h for h in hits if h.severity == SEV_HIGH]
        high_files = {h.file for h in high}
        assert any(
            "BadInterp.cs" in f for f in high_files
        ), f"missing HIGH for BadInterp.cs: {high_files}"
        assert any(
            "BadConcat.cs" in f for f in high_files
        ), f"missing HIGH for BadConcat.cs: {high_files}"

        # Form discrimination — interpolation vs concat captured.
        forms = {(h.file.split("/")[-1], h.form) for h in high}
        assert ("BadInterp.cs", "interpolation") in forms, forms
        assert ("BadConcat.cs", "concat") in forms, forms

        # MED: Tools site.
        med = [h for h in hits if h.severity == SEV_MED]
        med_files = {h.file for h in med}
        assert any(
            "ToolsSite.cs" in f for f in med_files
        ), f"missing MED for ToolsSite.cs: {med_files}"

        # LOW: LogWarning site (in HIGH boundary, but LogWarning -> LOW).
        low = [h for h in hits if h.severity == SEV_LOW]
        low_files = {h.file for h in low}
        assert any(
            "WarnSite.cs" in f for f in low_files
        ), f"missing LOW for WarnSite.cs: {low_files}"

        # 3) Allowlist suppression — pick the BadInterp hit, allowlist it.
        report_pre = build_report(list(hits), set(), n_files)
        target = next(
            h for h in hits
            if h.severity == SEV_HIGH and "BadInterp.cs" in h.file
        )
        target_key = target.allowlist_key
        # Re-scan to reset state.
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

        # 4) Bare-path allowlist form — listing a relative file path drops
        #    every hit in that file.
        hits3, _ = scan_root(repo, "src")
        bare_path = "src/Tools/Cli/ToolsSite.cs"
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

        # If we suppress all HIGH+MED, strict still fails because of LOW;
        # non-strict passes.
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
            "logerror-no-stack: "
            f"files={report['files_scanned']} "
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
