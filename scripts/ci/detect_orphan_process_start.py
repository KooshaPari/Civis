#!/usr/bin/env python3
"""Orphan Process Handle Leakage detector — Pattern #102 CI gate.

Pattern #102 ("orphan ``Process.Start()`` handle leak") is the failure
mode where C# code spawns a child process but discards the handle, or
keeps the handle in a field whose lifecycle is not protected by a
try/catch around the spawn:

    // BAD: handle is dropped on the floor; the OS handle is GC'd
    // non-deterministically, and the child process may outlive the
    // parent or accumulate as a zombie.
    Process.Start(new ProcessStartInfo { FileName = "blender", ... });

    // BAD: assigned to a field, but if `await ConnectAsync()` throws
    // we leak `_gameProcess` because there's no try/catch around the
    // spawn. The next caller sees a stale field; the spawned process
    // is still running.
    public async Task LaunchAsync()
    {
        _gameProcess = Process.Start(psi);
        await ConnectAsync();
    }

This is a problem for several reasons:

  * **Handle leak**. ``Process`` implements ``IDisposable``; the OS
    handle is closed on dispose. Discarding the result skips dispose
    and relies on finalizer-thread GC.
  * **Zombie processes**. On Windows the child process inherits the
    parent's job-object semantics only if explicit. A discarded handle
    means the child can outlive a crash of the parent — common when
    spawning helper EXEs in a CI runner.
  * **Throw-after-spawn corruption**. The deadliest variant: the
    handle IS captured into a field, but a method-local ``await`` (or
    sync call) between the spawn and a try/catch boundary throws,
    leaving the field set with a stale ``Process`` whose lifecycle is
    no longer connected to the caller's scope.

The healthy patterns are:

  1. ``using var p = Process.Start(...);`` — deterministic dispose at
     scope-end. Use when the spawn is local.
  2. Field assignment INSIDE a ``try { ... } catch { proc?.Kill(); proc?.Dispose(); }``
     wrapper, with the catch killing+disposing on failure. Use when
     the field needs to outlive the spawning method.
  3. Discard with rationale: ``using var _ = Process.Start(...);`` —
     intentional fire-and-forget that still respects ``IDisposable``.

This gate scans ``src/`` for ``Process.Start(...)`` references and
classifies each by call-site shape:

  * **HIGH** — fire-and-forget (no LHS) in core code (``src/Bridge/``,
    ``src/Runtime/``, ``src/SDK/``) — these are platform layers; we
    can't afford handle leaks. Also: field-assignment forms with an
    intervening ``await``/``throw``/method-end before any covering
    ``try { ... }`` and ``catch`` that calls ``Kill()`` or ``Dispose()``.
  * **MED**  — fire-and-forget (no LHS) in non-core code (``src/Tools/``,
    ``src/Tests/``). Cosmetic browser-open / shell-execute is a common
    legitimate case; the fix is to wrap with ``using var _`` for handle
    hygiene.
  * **LOW**  — statement-level discard outside the scopes above. We
    record but don't gate.

Auto-skipped sites (zero recorded as hits):

  * ``using var <name> = Process.Start(...);`` — IDisposable scope
    handles dispose deterministically.
  * Trailing comment ``// pattern-102-allowed: <reason>`` blesses a
    site with rationale.
  * File path in ``docs/qa/orphan-process-start-allowlist.txt``.
  * ``src/Tests/Integration/`` — process-orchestration tests legitimately
    spawn helpers without long-lived field ownership.

Allowlisting (one entry per line, ``#`` for comments):
``docs/qa/orphan-process-start-allowlist.txt``. Two entry forms:

  1. ``severity|file|line`` — line-locked allowlist key.
  2. Bare ``relative/path.cs`` — suppress every site in that file.

CLI:
    python scripts/ci/detect_orphan_process_start.py
        [--root <src/>]
        [--allowlist <path>]
        [--output <json>]
        [--strict]
        [--quiet|--verbose]
        [--self-test]

Exit 0 = no NEW (un-allowlisted) HIGH hits; 1 = new violations
detected (CI fails); 2 = scan/usage error. ``--strict`` promotes MED
findings to fail the gate as well.

Modeled on ``scripts/ci/detect_stringly_enums.py`` (#291),
``scripts/ci/detect_direct_datetime.py`` (#286), and
``scripts/ci/detect_missing_configureawait.py`` (#275). Pairs with the
top-4 HIGH/MED sweep being driven by #295 and the
``using var _`` cleanup being driven by #296.

This is task #297.
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

# Match a ``Process.Start(`` call. The leading word boundary keeps us
# from matching ``MyProcess.Start(``. We accept the
# ``System.Diagnostics.Process.Start(`` qualified form by allowing the
# ``Diagnostics.`` prefix as optional.
PROCESS_START_RE = re.compile(
    r"(?<![A-Za-z0-9_.])"
    r"(?:System\.Diagnostics\.)?Process\.Start\s*\("
)

# ``using var <name> = Process.Start(...)`` — the IDisposable scope
# form. Treated as the safe pattern; auto-skipped.
USING_DECL_RE = re.compile(
    r"\busing\s+(?:var\s+)?(?:[A-Za-z_][A-Za-z0-9_]*\s+)?"
    r"[A-Za-z_][A-Za-z0-9_]*\s*=\s*"
    r"(?:System\.Diagnostics\.)?Process\.Start\s*\("
)

# ``<lhs> = Process.Start(...)`` — an assignment form. ``lhs`` may be
# ``_field``, ``proc``, ``this.proc``, etc. We do NOT require a type
# decl prefix so that field-assignment in a method body matches.
ASSIGN_RE = re.compile(
    r"(?P<lhs>(?:[A-Za-z_][A-Za-z0-9_]*\.)*[A-Za-z_][A-Za-z0-9_]*)"
    r"\s*=\s*"
    r"(?:System\.Diagnostics\.)?Process\.Start\s*\("
)

# Trailing-comment opt-out token. Anywhere on the same line as the
# match, suppresses the hit.
PATTERN_102_ALLOWED_RE = re.compile(r"//\s*pattern-102-allowed\b")

# Cleanup patterns — a ``catch`` block that calls ``Kill()`` or
# ``Dispose()`` is treated as a covering cleanup path. We don't
# actually parse the catch; we look for the tokens within the next ~20
# lines after a ``catch`` keyword.
CLEANUP_TOKEN_RE = re.compile(r"\b(?:Kill|Dispose)\s*\(")

# Default scan root.
DEFAULT_SCAN_ROOT = "src"

# Path fragments (POSIX-relative, lowercased) that classify a hit as
# HIGH for fire-and-forget statements.
HIGH_BOUNDARY_PARTS = (
    "src/bridge/",
    "src/runtime/",
    "src/sdk/",
)

# Path fragments that classify a hit as MED for fire-and-forget
# statements (Tools + Tests root).
MED_BOUNDARY_PARTS = (
    "src/tools/",
    "src/tests/",
)

# Path fragments that mean "do not scan this file at all". Integration
# tests spawn helper processes orchestrationally; that's not the
# pattern we're looking for.
SKIP_BOUNDARY_PARTS = (
    "src/tests/integration/",
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


def _path_skipped(rel_posix: str) -> bool:
    lo = rel_posix.lower()
    return any(p in lo for p in SKIP_BOUNDARY_PARTS)


def _is_high_path(rel_posix: str) -> bool:
    lo = rel_posix.lower()
    return any(p in lo for p in HIGH_BOUNDARY_PARTS)


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


def _line_carries_optout(text: str, offset: int) -> bool:
    """Return True if the source line containing *offset* carries the
    pattern-102-allowed trailing comment."""
    line = _line_text(text, offset)
    return bool(PATTERN_102_ALLOWED_RE.search(line))


def _line_is_using_decl(text: str, offset: int) -> bool:
    """Return True if the source line containing *offset* starts with
    ``using var/<type>`` declaring a Process.Start handle. We use a
    line-scoped regex to avoid false positives from nested expressions
    that happen to contain ``using``."""
    line = _line_text(text, offset)
    return bool(USING_DECL_RE.search(line))


def _is_assignment_form(text: str, offset: int) -> tuple[bool, str | None]:
    """Return (True, lhs) if the source line containing *offset* has
    an ``<lhs> = Process.Start(`` assignment form; (False, None)
    otherwise. ``lhs`` is the captured left-hand side identifier (e.g.
    ``_proc``, ``this.proc``, ``_gameProcess``).

    NOTE: the ``using`` declaration form ALSO matches this regex
    (``using var p = Process.Start(``); call ``_line_is_using_decl``
    first and bypass this if so."""
    line = _line_text(text, offset)
    m = ASSIGN_RE.search(line)
    if m is None:
        return False, None
    return True, m.group("lhs")


def _has_covering_try_catch_with_cleanup(
    text: str, offset: int,
) -> bool:
    """Heuristic: is the call at *offset* covered by a ``try { ... }``
    whose matching ``catch`` block calls ``Kill()`` or ``Dispose()``?

    We walk backward from *offset* up to ~30 lines to find a ``try {``
    token. If found, we look forward from the END of the call line for
    the matching ``}`` followed by ``catch`` (within ~50 lines), then
    look forward from there ~20 lines for ``Kill(`` or ``Dispose(``
    tokens. Comment / string handling is not strict — false negatives
    are acceptable; they only escalate severity, not gate the FAIL
    boundary."""
    lines = text.splitlines()
    line_idx = text.count("\n", 0, offset)

    # 1) Look backward for ``try`` within ~30 lines. Two brace styles
    #    are common in the DINOForge codebase: K&R (``try {`` on the
    #    same line) and Allman (``try`` on its own line, ``{`` on the
    #    next). We accept either form.
    try_lo = max(0, line_idx - 30)
    try_window = lines[try_lo:line_idx + 1]
    has_try = any(
        re.search(r"^\s*try\s*\{?\s*$", ln)  # Allman: try alone or try {
        or re.search(r"\btry\s*\{", ln)      # K&R: try {
        for ln in try_window
    )
    if not has_try:
        return False

    # 2) Look forward for ``catch`` within ~50 lines.
    catch_hi = min(len(lines), line_idx + 50)
    catch_window = lines[line_idx:catch_hi]
    catch_idx_in_window: int | None = None
    for i, ln in enumerate(catch_window):
        if re.search(r"\bcatch\b", ln):
            catch_idx_in_window = i
            break
    if catch_idx_in_window is None:
        return False

    # 3) Look forward from the catch ~20 lines for Kill/Dispose tokens.
    catch_abs = line_idx + catch_idx_in_window
    cleanup_hi = min(len(lines), catch_abs + 20)
    cleanup_window = lines[catch_abs:cleanup_hi]
    return any(CLEANUP_TOKEN_RE.search(ln) for ln in cleanup_window)


def _has_intervening_await_or_throw(
    text: str, offset: int,
) -> bool:
    """Heuristic: in the ~30 lines AFTER *offset*, do we see ``await``,
    ``throw``, or method-end (``^}$``) before encountering a covering
    ``try { ... }`` boundary that wraps the original spawn?

    We don't try to detect nesting; the simple signal — "an await
    appears after the spawn, in the same method body" — is the failure
    case we're after."""
    lines = text.splitlines()
    line_idx = text.count("\n", 0, offset)
    fwd_hi = min(len(lines), line_idx + 30)
    window = lines[line_idx + 1:fwd_hi]
    for ln in window:
        if re.search(r"\bawait\b", ln):
            return True
        if re.search(r"\bthrow\b", ln):
            return True
        # Method-end heuristic: a line whose ONLY non-whitespace token
        # is ``}`` is plausibly the end of the enclosing method.
        if re.match(r"\s*\}\s*$", ln):
            return True
    return False


@dataclass
class Hit:
    file: str            # POSIX-relative to repo root
    line: int
    severity: str
    detail: str
    shape: str           # "fire_and_forget" | "assign_no_cleanup"
    has_await_or_throw: bool
    has_try_catch_cleanup: bool
    snippet: str         # short slice of the source line
    allowlist_key: str = field(default="")
    in_allowlist: bool = field(default=False)


def _classify_match(
    text: str, rel: str, offset: int,
) -> Hit | None:
    """Return a Hit (or None for skipped/safe sites) for the
    ``Process.Start(`` match at *offset* in *rel*."""
    # Trailing-comment opt-out (top priority).
    if _line_carries_optout(text, offset):
        return None

    # ``using var p = Process.Start(...)`` — IDisposable scope, safe.
    if _line_is_using_decl(text, offset):
        return None

    # Assignment form: ``<lhs> = Process.Start(...)``.
    is_assign, lhs = _is_assignment_form(text, offset)

    line = line_of(text, offset)
    line_body = _line_text(text, offset).strip()
    snippet = line_body if len(line_body) <= 120 else line_body[:117] + "..."

    if is_assign:
        # Pattern 2: assignment form. HIGH iff there's an intervening
        # await/throw/method-end without a covering try/catch + cleanup.
        has_cleanup = _has_covering_try_catch_with_cleanup(text, offset)
        has_aot = _has_intervening_await_or_throw(text, offset)
        if has_cleanup:
            # Wrapped in try/catch with Kill/Dispose — safe.
            return None
        # No covering try/catch with cleanup. If the surrounding
        # method has an ``await``/``throw``/method-end, that's HIGH:
        # the spawned handle can be orphaned on throw.
        if has_aot:
            severity = SEV_HIGH if _is_high_path(rel) else SEV_MED
            detail = (
                f"`{lhs} = Process.Start(...)` in {rel} is followed by "
                f"`await`/`throw`/method-end with no covering try/catch "
                f"that calls Kill()/Dispose() on failure. If the await "
                f"throws, the spawned handle is orphaned. Wrap the spawn "
                f"in `try {{ ... }} catch {{ proc?.Kill(); "
                f"proc?.Dispose(); throw; }}` or use "
                f"`using var p = Process.Start(...);` if local."
            )
            return Hit(
                file=rel,
                line=line,
                severity=severity,
                detail=detail,
                shape="assign_no_cleanup",
                has_await_or_throw=has_aot,
                has_try_catch_cleanup=has_cleanup,
                snippet=snippet,
            )
        # Assignment with no intervening await/throw/method-end and no
        # cleanup — record as LOW (we can't be sure the field has a
        # disposal path elsewhere, but the immediate hazard is small).
        severity = SEV_LOW
        detail = (
            f"`{lhs} = Process.Start(...)` in {rel} has no covering "
            f"try/catch with Kill()/Dispose(). The field's disposal "
            f"path may exist elsewhere (Dispose method); review and "
            f"either wrap the spawn or document the disposal contract."
        )
        return Hit(
            file=rel,
            line=line,
            severity=severity,
            detail=detail,
            shape="assign_no_cleanup",
            has_await_or_throw=has_aot,
            has_try_catch_cleanup=has_cleanup,
            snippet=snippet,
        )

    # Pattern 1: statement-level fire-and-forget. The line ends with
    # ``)`` followed by ``;`` (or whitespace + ``;``) — the result is
    # discarded.
    if _is_high_path(rel):
        severity = SEV_HIGH
        detail = (
            f"`Process.Start(...)` fire-and-forget statement in core "
            f"code ({rel}). The Process handle is discarded; OS handle "
            f"closure relies on finalizer GC. Wrap with "
            f"`using var _ = Process.Start(...);` for deterministic "
            f"dispose, or capture into a field and dispose explicitly."
        )
    elif _is_med_path(rel):
        severity = SEV_MED
        detail = (
            f"`Process.Start(...)` fire-and-forget statement in "
            f"non-core code ({rel}). Common for browser-open / shell-"
            f"execute calls; wrap with `using var _ = Process.Start(...);` "
            f"for handle hygiene, or append "
            f"`// pattern-102-allowed: <reason>` if intentional."
        )
    else:
        severity = SEV_LOW
        detail = (
            f"`Process.Start(...)` fire-and-forget statement in "
            f"out-of-bucket code ({rel}). Wrap with "
            f"`using var _ = Process.Start(...);` for handle hygiene."
        )

    return Hit(
        file=rel,
        line=line,
        severity=severity,
        detail=detail,
        shape="fire_and_forget",
        has_await_or_throw=False,
        has_try_catch_cleanup=False,
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
    for m in PROCESS_START_RE.finditer(text):
        h = _classify_match(text, rel, m.start())
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
        "severity": h.severity,
        "shape": h.shape,
        "has_await_or_throw": h.has_await_or_throw,
        "has_try_catch_cleanup": h.has_try_catch_cleanup,
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
        "low_count": low_count,
        "high_violations": [_h2d(h) for h in high_violations],
        "med_violations": [_h2d(h) for h in med_violations],
        "low_violations": [_h2d(h) for h in low_violations],
        "all_hits": [_h2d(h) for h in hits],
        "exit_code": exit_code,
    }


def print_summary(report: dict, output_path: Path) -> None:
    print("orphan-process-start gate (Pattern #102)")
    print(f"  files scanned          : {report['scanned_files']}")
    print(f"  total hits             : {report['total_hits']}")
    print(f"  allowlist size         : {report['allowlist_size']}")
    print(f"  NEW hits               : {report['new_hits']}")
    print(f"    HIGH (core / orphan-on-throw): {report['high_count']}")
    print(f"    MED  (Tools / Tests fire+forget): {report['med_count']}")
    print(f"    LOW  (assign / out-of-bucket): {report['low_count']}")
    if report["new_hits"]:
        print()
        print("NEW Process.Start sites:")
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
                    f"    [{h['severity']}] {h['file']}:{h['line']}  "
                    f"({h['shape']})"
                )
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect orphan Process.Start handle leakage (Pattern "
            "#102). HIGH = fire-and-forget in core code (Bridge/"
            "Runtime/SDK) or assignment-without-cleanup with "
            "intervening await/throw. MED = fire-and-forget in "
            "Tools/Tests. LOW = assignment with no immediate "
            "hazard or out-of-bucket. `using var _` is auto-skipped."
        )
    )
    p.add_argument(
        "--root",
        default=DEFAULT_SCAN_ROOT,
        help=f"Source root to scan (default: {DEFAULT_SCAN_ROOT})",
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/orphan-process-start-allowlist.txt",
        help=(
            "Allowlist file; one ``severity|file|line`` key or "
            "bare relative path per line; ``#`` for comments "
            "(default: docs/qa/orphan-process-start-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/orphan-process-start-report.json",
        help="JSON report output path",
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help=(
            "Promote MED (fire-and-forget in Tools/Tests) findings to "
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


_FIXTURE_HIGH_BRIDGE_FAF = """
namespace DINOForge.Fixture.Bridge
{
    using System.Diagnostics;

    public class BadBridgeSpawn
    {
        public void Launch()
        {
            // HIGH: fire-and-forget in src/Bridge/ (core).
            Process.Start(new ProcessStartInfo { FileName = "blender" });
        }
    }
}
"""

_FIXTURE_HIGH_ORPHAN_ON_THROW = """
namespace DINOForge.Fixture.Bridge
{
    using System.Diagnostics;
    using System.Threading.Tasks;

    public class BadOrphanOnThrow
    {
        private Process? _gameProcess;

        public async Task LaunchAsync()
        {
            // HIGH: assign-without-cleanup. `await` after spawn with
            // no covering try/catch -> orphan-on-throw.
            _gameProcess = Process.Start(new ProcessStartInfo());
            await ConnectAsync();
        }

        private Task ConnectAsync() => Task.CompletedTask;
    }
}
"""

_FIXTURE_SAFE_USING_DECL = """
namespace DINOForge.Fixture.Tools
{
    using System.Diagnostics;

    public class SafeUsing
    {
        public void Run()
        {
            // SAFE: using var p = ... -> auto-skipped.
            using var p = Process.Start(new ProcessStartInfo());
            p?.WaitForExit();
        }
    }
}
"""

_FIXTURE_SAFE_USING_DISCARD = """
namespace DINOForge.Fixture.Tools
{
    using System.Diagnostics;

    public class SafeDiscard
    {
        public void Run()
        {
            // SAFE: using var _ = ... is the documented intentional
            // discard form.
            using var _ = Process.Start(new ProcessStartInfo());
        }
    }
}
"""

_FIXTURE_SAFE_TRY_CATCH_CLEANUP = """
namespace DINOForge.Fixture.Bridge
{
    using System.Diagnostics;
    using System.Threading.Tasks;

    public class SafeTryCatch
    {
        private Process? _gameProcess;

        public async Task LaunchAsync()
        {
            try
            {
                _gameProcess = Process.Start(new ProcessStartInfo());
                await ConnectAsync();
            }
            catch
            {
                _gameProcess?.Kill();
                _gameProcess?.Dispose();
                throw;
            }
        }

        private Task ConnectAsync() => Task.CompletedTask;
    }
}
"""

_FIXTURE_MED_TOOLS_FAF = """
namespace DINOForge.Fixture.Tools
{
    using System.Diagnostics;

    public class ToolsFireAndForget
    {
        public void OpenBrowser()
        {
            // MED: fire-and-forget in src/Tools/.
            Process.Start(new ProcessStartInfo { FileName = "https://x" });
        }
    }
}
"""

_FIXTURE_LOW_TESTS_FAF = """
namespace DINOForge.Fixture.Tests
{
    using System.Diagnostics;

    public class TestsFireAndForget
    {
        public void Spawn()
        {
            // MED (Tests bucket): fire-and-forget in src/Tests/. Note
            // src/Tests/Integration/ is auto-skipped; this fixture
            // lives in src/Tests/ root.
            Process.Start(new ProcessStartInfo());
        }
    }
}
"""

_FIXTURE_PATTERN_ALLOWED_COMMENT = """
namespace DINOForge.Fixture.Bridge
{
    using System.Diagnostics;

    public class BlessedSpawn
    {
        public void Run()
        {
            // Trailing pattern-102-allowed comment opts out.
            Process.Start(new ProcessStartInfo()); // pattern-102-allowed: shell-execute-url
        }
    }
}
"""

_FIXTURE_SKIP_INTEGRATION = """
namespace DINOForge.Fixture.Tests.Integration
{
    using System.Diagnostics;

    public class IntegrationHelper
    {
        public void Spawn()
        {
            // Integration tests are auto-skipped (orchestration spawn).
            Process.Start(new ProcessStartInfo());
        }
    }
}
"""


def _build_fixture_repo(td: Path) -> Path:
    """Lay out a synthetic repo under *td* matching the gate's expected
    structure: src/Bridge/, src/Tools/, src/Tests/, plus auto-skipped
    src/Tests/Integration/."""
    repo = td / "repo"
    bridge = repo / "src" / "Bridge" / "Client"
    tools = repo / "src" / "Tools" / "Cli"
    tests = repo / "src" / "Tests"
    integration = repo / "src" / "Tests" / "Integration"
    for d in (bridge, tools, tests, integration):
        d.mkdir(parents=True, exist_ok=True)

    (bridge / "BadBridgeSpawn.cs").write_text(
        _FIXTURE_HIGH_BRIDGE_FAF, encoding="utf-8",
    )
    (bridge / "BadOrphanOnThrow.cs").write_text(
        _FIXTURE_HIGH_ORPHAN_ON_THROW, encoding="utf-8",
    )
    (bridge / "SafeTryCatch.cs").write_text(
        _FIXTURE_SAFE_TRY_CATCH_CLEANUP, encoding="utf-8",
    )
    (bridge / "BlessedSpawn.cs").write_text(
        _FIXTURE_PATTERN_ALLOWED_COMMENT, encoding="utf-8",
    )
    (tools / "SafeUsing.cs").write_text(
        _FIXTURE_SAFE_USING_DECL, encoding="utf-8",
    )
    (tools / "SafeDiscard.cs").write_text(
        _FIXTURE_SAFE_USING_DISCARD, encoding="utf-8",
    )
    (tools / "ToolsFireAndForget.cs").write_text(
        _FIXTURE_MED_TOOLS_FAF, encoding="utf-8",
    )
    (tests / "TestsFireAndForget.cs").write_text(
        _FIXTURE_LOW_TESTS_FAF, encoding="utf-8",
    )
    (integration / "IntegrationHelper.cs").write_text(
        _FIXTURE_SKIP_INTEGRATION, encoding="utf-8",
    )

    return repo


def _self_test() -> int:  # noqa: C901
    import tempfile

    # 1) Regex sanity — direct string checks.
    assert PROCESS_START_RE.search("Process.Start(psi);")
    assert PROCESS_START_RE.search("System.Diagnostics.Process.Start(psi);")
    assert PROCESS_START_RE.search("    _proc = Process.Start(psi);")
    # ``MyProcess.Start`` must NOT match (different prefix identifier).
    assert not PROCESS_START_RE.search("MyProcess.Start(psi);")
    # ``proc.Start`` (instance call on a Process instance) must NOT match.
    assert not PROCESS_START_RE.search("proc.Start(psi);")

    # using-decl regex.
    assert USING_DECL_RE.search("using var p = Process.Start(psi);")
    assert USING_DECL_RE.search("using var _ = Process.Start(psi);")
    assert USING_DECL_RE.search(
        "            using var proc = System.Diagnostics.Process.Start(psi);"
    )
    # ``var p = Process.Start`` (no using) must NOT match.
    assert not USING_DECL_RE.search("var p = Process.Start(psi);")

    # Assignment regex.
    m = ASSIGN_RE.search("_gameProcess = Process.Start(psi);")
    assert m and m.group("lhs") == "_gameProcess"
    m = ASSIGN_RE.search("this._proc = Process.Start(psi);")
    assert m and m.group("lhs") == "this._proc"
    # Bare ``Process.Start(...)`` (no LHS) must NOT match.
    assert ASSIGN_RE.search("Process.Start(psi);") is None

    # Trailing-comment opt-out.
    assert PATTERN_102_ALLOWED_RE.search(
        'Process.Start(psi); // pattern-102-allowed: shell-execute'
    )

    # Cleanup token.
    assert CLEANUP_TOKEN_RE.search("_proc?.Kill();")
    assert CLEANUP_TOKEN_RE.search("proc.Dispose();")

    # 2) End-to-end scan against the synthetic repo.
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        repo = _build_fixture_repo(td_path)

        hits, n_files = scan_root(repo, "src")
        files_seen = {h.file for h in hits}

        # Auto-skipped sites should not surface as hits.
        assert not any(
            "SafeUsing.cs" in f for f in files_seen
        ), f"`using var p = ...` leaked: {files_seen}"
        assert not any(
            "SafeDiscard.cs" in f for f in files_seen
        ), f"`using var _ = ...` leaked: {files_seen}"
        assert not any(
            "SafeTryCatch.cs" in f for f in files_seen
        ), f"try/catch+cleanup leaked: {files_seen}"
        assert not any(
            "BlessedSpawn.cs" in f for f in files_seen
        ), f"// pattern-102-allowed not honored: {files_seen}"
        assert not any(
            "IntegrationHelper.cs" in f for f in files_seen
        ), f"src/Tests/Integration/ scope leaked: {files_seen}"

        # HIGH set: BadBridgeSpawn.cs + BadOrphanOnThrow.cs.
        high = [h for h in hits if h.severity == SEV_HIGH]
        high_files = {h.file for h in high}
        assert any(
            "BadBridgeSpawn.cs" in f for f in high_files
        ), f"missing HIGH for BadBridgeSpawn.cs: {high_files}"
        assert any(
            "BadOrphanOnThrow.cs" in f for f in high_files
        ), f"missing HIGH for BadOrphanOnThrow.cs: {high_files}"

        # BadBridgeSpawn shape == fire_and_forget.
        bbs_hit = next(h for h in high if "BadBridgeSpawn.cs" in h.file)
        assert bbs_hit.shape == "fire_and_forget", bbs_hit
        # BadOrphanOnThrow shape == assign_no_cleanup, await/throw flag set.
        boot_hit = next(
            h for h in high if "BadOrphanOnThrow.cs" in h.file
        )
        assert boot_hit.shape == "assign_no_cleanup", boot_hit
        assert boot_hit.has_await_or_throw is True, boot_hit
        assert boot_hit.has_try_catch_cleanup is False, boot_hit

        # MED set: ToolsFireAndForget.cs + TestsFireAndForget.cs (both
        # in src/Tools/ and src/Tests/ root).
        med = [h for h in hits if h.severity == SEV_MED]
        med_files = {h.file for h in med}
        assert any(
            "ToolsFireAndForget.cs" in f for f in med_files
        ), f"missing MED for ToolsFireAndForget.cs: {med_files}"
        assert any(
            "TestsFireAndForget.cs" in f for f in med_files
        ), f"missing MED for TestsFireAndForget.cs: {med_files}"

        # 3) Allowlist suppression — line-locked key.
        report_pre = build_report(list(hits), set(), n_files)
        target = next(
            h for h in hits
            if h.severity == SEV_HIGH and "BadBridgeSpawn.cs" in h.file
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
        bare_path = "src/Tools/Cli/ToolsFireAndForget.cs"
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
            "orphan-process-start: "
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
