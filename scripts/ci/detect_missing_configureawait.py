#!/usr/bin/env python3
"""Missing ConfigureAwait(false) detector — Pattern #98 CI gate.

Pattern #98 ("await without ConfigureAwait(false) in library code") is
the failure mode where a C# library awaits a Task without explicitly
breaking captured-context resumption:

    public async Task DoWorkAsync()
    {
        await SomeIoAsync();           // BAD: captures SynchronizationContext
        await Stream.ReadAsync(buf);   // BAD: same
    }

In a console / CLI / pure-library setting this happens to be fine
because there is no SynchronizationContext to capture. But the moment
the library is consumed from:

  * a UI host (Avalonia GUI installer, WinUI desktop companion),
  * a Unity main-thread synchronization context (BepInEx Runtime),
  * any framework that pumps a SynchronizationContext (legacy ASP.NET,
    WinForms, WPF),

every one of those awaits will marshal the continuation back onto the
captured context, which is at minimum a context-switch cost and at
worst a deadlock (sync-over-async, ``.Result`` / ``.Wait()`` on the
captured thread). The healthy library pattern is ALWAYS:

    await SomeIoAsync().ConfigureAwait(false);
    await Stream.ReadAsync(buf).ConfigureAwait(false);

This gate scans LIBRARY-FACING code for ``await`` expressions that do
not end in ``.ConfigureAwait(false)`` and classifies each site by
where it lives. UI hosts (Avalonia / WinUI), the Unity Runtime, the
Python MCP server, and tests are auto-skipped — those layers either
need the captured context (UI), have no async-aware host (Unity ECS),
or are not C# at all (MCP).

Severity ladder (first matching path-bucket wins):

  * **HIGH** — call site lives under ``src/SDK/`` or
    ``src/Bridge/Client/``. SDK is the public mod API surface and the
    bridge client is the cross-process consumer entry point;
    consumers run inside Unity or a UI host so a captured context is
    likely.
  * **MED**  — call site lives under ``src/Tools/Cli/`` or
    ``src/Tools/PackCompiler/``. CLI / build tooling is async but
    today runs without a SynchronizationContext; flagged for hygiene.
  * **LOW**  — call site lives under
    ``src/Tools/Installer/InstallerLib/``. The InstallerLib is the
    headless library that the Avalonia GUI installer composes against;
    while the GUI itself is auto-skipped, the library MUST stay
    context-free so a future GUI host doesn't deadlock. Reported for
    visibility; ``--strict`` promotes to fail the gate.

Auto-skipped scopes (zero recorded as hits):

  * ``src/Tests/`` — test code intentionally awaits without
    ConfigureAwait so xUnit's sync context (or the lack of one) shows
    up in repro.
  * ``src/Tools/Installer/GUI/`` — Avalonia app needs the captured
    UI context.
  * ``src/Runtime/`` — Unity / BepInEx; SystemBase + scene callbacks,
    no async marshalling target.
  * ``src/Tools/DesktopCompanion/`` — WinUI 3 app, needs UI context.
  * ``src/Tools/DinoforgeMcp/`` — Python, not C#.
  * ``bin/``, ``obj/``, ``node_modules/``, ``.git/``.

Auto-skipped expressions:

  * ``await Task.Yield()`` — explicit context-yield primitive.
  * ``await using`` — async-disposable; ConfigureAwait is on a
    different surface and has its own analyzer story.
  * ``await foreach`` — async-enumerable; ConfigureAwait pattern uses
    ``WithCancellation``/``ConfigureAwait`` on the source, also a
    different surface.
  * ``// CA-allowed: <reason>`` trailing comment on the await line —
    explicit per-site opt-out with rationale.

Allowlisting (one entry per line, ``#`` for comments):
``docs/qa/configureawait-allowlist.txt``. Two entry forms:

  1. ``severity|file|line`` — line-locked allowlist key.
  2. Bare ``relative/path.cs`` — suppress every site in that file.

CLI:
    python scripts/ci/detect_missing_configureawait.py
        [--root <src/>]
        [--allowlist <path>]
        [--output <json>]
        [--strict]
        [--quiet|--verbose]
        [--self-test]

Exit 0 = no NEW (un-allowlisted) HIGH/MED hits; 1 = new violations
detected (CI fails); 2 = scan/usage error. ``--strict`` promotes LOW
findings to fail the gate as well.

Modeled on ``scripts/ci/detect_tcs_sync_continuations.py`` (#272) and
``scripts/ci/detect_logerror_no_stack.py`` (#268). Pairs with the
ConfigureAwait sweep being driven by #274.

This is task #275.
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

# Opening ``await`` token. We only match keyword-form awaits (with a
# leading word boundary) so we don't pick up identifiers like
# ``waiter`` or ``awaited``. We then capture optional follow-on
# qualifiers: ``await using`` and ``await foreach`` are auto-skipped.
AWAIT_RE = re.compile(r"(?<![A-Za-z0-9_])await\b(?P<after>\s*(using|foreach)\b)?")

# The healthy suffix on an await target.
HEALTHY_SUFFIX = ".ConfigureAwait(false)"

# Auto-allowed targets — bare await of these is fine. Match by the
# textual head of the awaited expression after stripping whitespace.
AUTO_ALLOWED_HEADS = (
    "Task.Yield()",
    "Task.CompletedTask",
)

# Trailing-comment opt-out token. Anywhere on the same line as the
# closing terminator of the await, suppresses the hit.
CA_ALLOWED_COMMENT_RE = re.compile(r"//\s*CA-allowed\b")

# Default scan root.
DEFAULT_SCAN_ROOT = "src"

# Path fragments (POSIX-relative, lowercased) that participate in the
# severity classification. Order matters: HIGH bucket checked first,
# then MED, then LOW. Anything not matching is auto-skipped.
HIGH_BOUNDARY_PARTS = (
    "src/sdk/",
    "src/bridge/client/",
)
MED_BOUNDARY_PARTS = (
    "src/tools/cli/",
    "src/tools/packcompiler/",
)
LOW_BOUNDARY_PARTS = (
    "src/tools/installer/installerlib/",
)

# Path fragments that mean "do not scan this file at all".
SKIP_BOUNDARY_PARTS = (
    "src/tests/",
    "src/tools/installer/gui/",
    "src/runtime/",
    "src/tools/desktopcompanion/",
    "src/tools/dinoforgemcp/",
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


def _bucket(rel_posix: str) -> str | None:
    """Return SEV_HIGH / SEV_MED / SEV_LOW for a file path or ``None``
    when the file is outside the gate's scope (auto-skipped scope)."""
    lo = rel_posix.lower()
    if any(p in lo for p in SKIP_BOUNDARY_PARTS):
        return None
    if any(p in lo for p in HIGH_BOUNDARY_PARTS):
        return SEV_HIGH
    if any(p in lo for p in MED_BOUNDARY_PARTS):
        return SEV_MED
    if any(p in lo for p in LOW_BOUNDARY_PARTS):
        return SEV_LOW
    return None


def _find_expression_end(text: str, start: int) -> int:
    """Walk forward from *start* (just after ``await ``) and return the
    offset of the terminator that ends the await expression. Terminator
    is one of ``;``, ``,``, or ``)`` matched at depth 0 with respect to
    parens / brackets. String literals are skipped so terminators inside
    them don't break the walker. Returns ``-1`` if EOF is reached
    without a terminator (malformed source — caller skips the hit)."""
    depth_paren = 0
    depth_brace = 0
    depth_bracket = 0
    i = start
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
                depth_paren += 1
            elif c == ")":
                if depth_paren == 0:
                    return i  # closing paren of an enclosing call
                depth_paren -= 1
            elif c == "[":
                depth_bracket += 1
            elif c == "]":
                depth_bracket -= 1
            elif c == "{":
                depth_brace += 1
            elif c == "}":
                depth_brace -= 1
            elif (c == ";" or c == ",") and depth_paren == 0 \
                    and depth_brace == 0 and depth_bracket == 0:
                return i
        i += 1
    return -1


def _line_text(text: str, offset: int) -> str:
    """Return the text of the source line containing *offset*."""
    line_start = text.rfind("\n", 0, offset) + 1
    line_end = text.find("\n", offset)
    if line_end == -1:
        line_end = len(text)
    return text[line_start:line_end]


def _expression_carries_comment_optout(text: str, start: int, term: int) -> bool:
    """Check whether any line covered by ``[start, term]`` (inclusive of
    a trailing same-line comment) contains a ``// CA-allowed`` token."""
    span = text[start:term + 1]
    # Also pull in the rest of the terminator's line so a trailing
    # comment after ``;`` still counts.
    tail_start = term
    tail_end = text.find("\n", term)
    if tail_end == -1:
        tail_end = len(text)
    tail = text[tail_start:tail_end]
    return bool(
        CA_ALLOWED_COMMENT_RE.search(span)
        or CA_ALLOWED_COMMENT_RE.search(tail)
    )


@dataclass
class Hit:
    file: str            # POSIX-relative to repo root
    line: int
    severity: str
    snippet: str         # short slice of the awaited expression
    detail: str
    boundary: str        # high|med|low — coarse path-bucket tag
    allowlist_key: str = field(default="")
    in_allowlist: bool = field(default=False)


def _strip_trailing_comment(s: str) -> str:
    """Remove a trailing ``// ...`` comment from *s* (line-comment
    only; block comments are not handled because the awaited
    expression doesn't legally end mid-block-comment)."""
    idx = s.find("//")
    if idx == -1:
        return s
    return s[:idx]


def _expression_text(text: str, start: int, term: int) -> str:
    """Return the awaited expression body (between ``await `` and the
    terminator), stripped, with trailing line comments removed."""
    body = text[start:term]
    # Drop any trailing comment fragment.
    body = _strip_trailing_comment(body)
    return body.strip()


def _ends_with_configureawait(expr: str) -> bool:
    """True iff *expr* ends with ``.ConfigureAwait(false)`` (with
    arbitrary whitespace inside the parens). The expression has been
    pre-stripped by :func:`_expression_text`."""
    # Tolerate ``ConfigureAwait( false )`` whitespace variants.
    return bool(re.search(
        r"\.ConfigureAwait\s*\(\s*false\s*\)\s*\Z",
        expr,
    ))


def _is_auto_allowed(expr: str) -> bool:
    """Check whether *expr* matches any auto-allowed head. We only match
    the canonical zero-arg primitives because anything else returning a
    Task has unknown context-capture semantics."""
    e = expr.strip()
    for head in AUTO_ALLOWED_HEADS:
        if e == head or e.startswith(head + "."):
            return True
    return False


def _scan_one_match(
    text: str,
    rel: str,
    bucket: str,
    m: re.Match,
) -> Hit | None:
    # Auto-skip ``await using`` and ``await foreach`` forms.
    if m.group("after"):
        return None

    expr_start = m.end()
    # Skip whitespace after the ``await`` token.
    i = expr_start
    end = len(text)
    while i < end and text[i] in " \t\r\n":
        i += 1
    expr_start = i
    if expr_start >= end:
        return None

    term = _find_expression_end(text, expr_start)
    if term == -1:
        return None

    expr = _expression_text(text, expr_start, term)
    if not expr:
        return None

    # Healthy: already ends with .ConfigureAwait(false).
    if _ends_with_configureawait(expr):
        return None

    # Auto-allowed primitives.
    if _is_auto_allowed(expr):
        return None

    # Trailing-comment opt-out.
    if _expression_carries_comment_optout(text, expr_start, term):
        return None

    line = line_of(text, m.start())
    snippet = expr if len(expr) <= 120 else expr[:117] + "..."
    detail = (
        f"await {snippet} omits .ConfigureAwait(false). Library code "
        f"under {rel} should not capture the consumer's "
        f"SynchronizationContext or TaskScheduler — append "
        f".ConfigureAwait(false) to the awaited expression."
    )
    boundary_map = {SEV_HIGH: "high", SEV_MED: "med", SEV_LOW: "low"}
    return Hit(
        file=rel,
        line=line,
        severity=bucket,
        snippet=snippet,
        detail=detail,
        boundary=boundary_map[bucket],
    )


def scan_file(path: Path, repo_root: Path) -> list[Hit]:
    text = read_text_safe(path)
    if not text:
        return []
    rel = path.relative_to(repo_root).as_posix()
    bucket = _bucket(rel)
    if bucket is None:
        return []
    hits: list[Hit] = []
    for m in AWAIT_RE.finditer(text):
        h = _scan_one_match(text, rel, bucket, m)
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
        if _bucket(rel) is None:
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
        "boundary": h.boundary,
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
    print("missing-configureawait gate (Pattern #98)")
    print(f"  files scanned          : {report['scanned_files']}")
    print(f"  total hits             : {report['total_hits']}")
    print(f"  allowlist size         : {report['allowlist_size']}")
    print(f"  NEW hits               : {report['new_hits']}")
    print(f"    HIGH (SDK / Bridge.Client) : {report['high_count']}")
    print(f"    MED  (Cli / PackCompiler)  : {report['med_count']}")
    print(f"    LOW  (InstallerLib)        : {report['low_count']}")
    if report["new_hits"]:
        print()
        print("NEW missing-configureawait sites:")
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
                    f"{h['snippet']}"
                )
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect await expressions missing .ConfigureAwait(false) in "
            "library code (Pattern #98). HIGH = SDK + Bridge.Client; "
            "MED = Cli + PackCompiler; LOW = InstallerLib. UI hosts, "
            "Tests, Runtime, and the Python MCP are auto-skipped."
        )
    )
    p.add_argument(
        "--root",
        default=DEFAULT_SCAN_ROOT,
        help=f"Source root to scan (default: {DEFAULT_SCAN_ROOT})",
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/configureawait-allowlist.txt",
        help=(
            "Allowlist file; one ``severity|file|line`` key or "
            "bare relative path per line; ``#`` for comments "
            "(default: docs/qa/configureawait-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/configureawait-report.json",
        help="JSON report output path",
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help=(
            "Promote LOW (InstallerLib) findings to fail the gate. "
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


_FIXTURE_HIGH_BARE = """
namespace DINOForge.Fixture.Sdk
{
    using System.Threading.Tasks;

    public class BadSdk
    {
        public async Task DoAsync()
        {
            // Bare await -> HIGH (SDK path).
            await foo.BarAsync();
        }
    }
}
"""

_FIXTURE_HIGH_HEALTHY = """
namespace DINOForge.Fixture.SdkHealthy
{
    using System.Threading.Tasks;

    public class GoodSdk
    {
        public async Task DoAsync()
        {
            // Healthy: ConfigureAwait(false) present.
            await foo.BarAsync().ConfigureAwait(false);
        }
    }
}
"""

_FIXTURE_HIGH_TASK_YIELD = """
namespace DINOForge.Fixture.SdkYield
{
    using System.Threading.Tasks;

    public class YieldSdk
    {
        public async Task DoAsync()
        {
            // Auto-allowed: Task.Yield() is a context-yield primitive.
            await Task.Yield();
        }
    }
}
"""

_FIXTURE_MED_TOOLS = """
namespace DINOForge.Fixture.Cli
{
    using System.Threading.Tasks;

    public class CliBad
    {
        public async Task DoAsync()
        {
            // Tools/Cli path -> MED.
            await Stream.ReadAsync(buf);
        }
    }
}
"""

_FIXTURE_LOW_INSTALLER = """
namespace DINOForge.Fixture.InstallerLib
{
    using System.Threading.Tasks;

    public class InstallerBad
    {
        public async Task DoAsync()
        {
            // InstallerLib path -> LOW.
            await Step.RunAsync();
        }
    }
}
"""

_FIXTURE_SKIP_TESTS = """
namespace DINOForge.Fixture.Tests
{
    using System.Threading.Tasks;

    public class TestSite
    {
        public async Task DoAsync()
        {
            // Tests scope is auto-skipped.
            await Other.WorkAsync();
        }
    }
}
"""

_FIXTURE_SKIP_GUI = """
namespace DINOForge.Fixture.Gui
{
    using System.Threading.Tasks;

    public class GuiSite
    {
        public async Task DoAsync()
        {
            // GUI scope is auto-skipped (Avalonia needs UI context).
            await ViewModel.LoadAsync();
        }
    }
}
"""

_FIXTURE_HIGH_AWAIT_USING = """
namespace DINOForge.Fixture.SdkAwaitUsing
{
    using System.Threading.Tasks;

    public class AwaitUsingSdk
    {
        public async Task DoAsync()
        {
            // ``await using`` is auto-skipped (different surface).
            await using var disposable = new MyAsyncDisposable();
            // But a follow-up bare await IS a hit.
            await disposable.WriteAsync();
        }
    }
}
"""

_FIXTURE_HIGH_CA_COMMENT = """
namespace DINOForge.Fixture.SdkCaComment
{
    using System.Threading.Tasks;

    public class CommentOptOut
    {
        public async Task DoAsync()
        {
            // Trailing CA-allowed comment opts the line out.
            await Foo.BarAsync(); // CA-allowed: pump test
        }
    }
}
"""


def _build_fixture_repo(td: Path) -> Path:
    """Lay out a synthetic repo under *td* matching the gate's expected
    structure: HIGH boundaries (SDK / Bridge.Client), MED (Cli /
    PackCompiler), LOW (InstallerLib), and auto-skipped (Tests / GUI /
    Runtime / DesktopCompanion / DinoforgeMcp)."""
    repo = td / "repo"
    sdk = repo / "src" / "SDK" / "Validation"
    sdk_healthy = repo / "src" / "SDK" / "Healthy"
    sdk_yield = repo / "src" / "SDK" / "Yield"
    sdk_using = repo / "src" / "SDK" / "Using"
    sdk_comment = repo / "src" / "SDK" / "Comment"
    cli = repo / "src" / "Tools" / "Cli"
    installerlib = repo / "src" / "Tools" / "Installer" / "InstallerLib"
    tests = repo / "src" / "Tests"
    gui = repo / "src" / "Tools" / "Installer" / "GUI"
    for d in (sdk, sdk_healthy, sdk_yield, sdk_using, sdk_comment,
              cli, installerlib, tests, gui):
        d.mkdir(parents=True, exist_ok=True)

    (sdk / "BadSdk.cs").write_text(_FIXTURE_HIGH_BARE, encoding="utf-8")
    (sdk_healthy / "GoodSdk.cs").write_text(
        _FIXTURE_HIGH_HEALTHY, encoding="utf-8"
    )
    (sdk_yield / "YieldSdk.cs").write_text(
        _FIXTURE_HIGH_TASK_YIELD, encoding="utf-8"
    )
    (sdk_using / "AwaitUsingSdk.cs").write_text(
        _FIXTURE_HIGH_AWAIT_USING, encoding="utf-8"
    )
    (sdk_comment / "CommentOptOut.cs").write_text(
        _FIXTURE_HIGH_CA_COMMENT, encoding="utf-8"
    )
    (cli / "CliBad.cs").write_text(_FIXTURE_MED_TOOLS, encoding="utf-8")
    (installerlib / "InstallerBad.cs").write_text(
        _FIXTURE_LOW_INSTALLER, encoding="utf-8"
    )
    (tests / "TestSite.cs").write_text(_FIXTURE_SKIP_TESTS, encoding="utf-8")
    (gui / "GuiSite.cs").write_text(_FIXTURE_SKIP_GUI, encoding="utf-8")

    return repo


def _self_test() -> int:  # noqa: C901
    import tempfile

    # 1) Regex sanity — direct string checks.
    assert AWAIT_RE.search("await foo.BarAsync();")
    # ``await using`` matched but ``after`` group set so we skip.
    m = AWAIT_RE.search("await using var x = ...;")
    assert m is not None and m.group("after") is not None
    # ``waiter`` should not match — word boundary protects us.
    assert not AWAIT_RE.search("var waiter = thing;")

    # _ends_with_configureawait detects healthy suffix.
    assert _ends_with_configureawait("foo.BarAsync().ConfigureAwait(false)")
    assert _ends_with_configureawait("Stream.ReadAsync(buf).ConfigureAwait( false )")
    assert not _ends_with_configureawait("foo.BarAsync()")
    assert not _ends_with_configureawait("Stream.ReadAsync().ConfigureAwait(true)")

    # _is_auto_allowed for Task.Yield().
    assert _is_auto_allowed("Task.Yield()")
    assert not _is_auto_allowed("Foo.Yield()")
    assert not _is_auto_allowed("Task.Run(...)")

    # 2) End-to-end scan against the synthetic repo.
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        repo = _build_fixture_repo(td_path)

        hits, n_files = scan_root(repo, "src")
        files_seen = {h.file for h in hits}

        # Healthy and auto-allowed sites must NOT appear.
        assert not any(
            "GoodSdk.cs" in f for f in files_seen
        ), f"healthy ConfigureAwait flagged: {files_seen}"
        assert not any(
            "YieldSdk.cs" in f for f in files_seen
        ), f"Task.Yield() flagged: {files_seen}"
        assert not any(
            "TestSite.cs" in f for f in files_seen
        ), f"Tests scope leaked: {files_seen}"
        assert not any(
            "GuiSite.cs" in f for f in files_seen
        ), f"GUI scope leaked: {files_seen}"
        assert not any(
            "CommentOptOut.cs" in f for f in files_seen
        ), f"// CA-allowed not honored: {files_seen}"

        # HIGH: BadSdk.cs (SDK).
        high = [h for h in hits if h.severity == SEV_HIGH]
        high_files = {h.file for h in high}
        assert any(
            "BadSdk.cs" in f for f in high_files
        ), f"missing HIGH for BadSdk.cs: {high_files}"

        # ``await using`` is auto-skipped, but the bare await ON THE
        # NEXT LINE in the same file is still flagged.
        assert any(
            "AwaitUsingSdk.cs" in f for f in high_files
        ), f"missing HIGH for follow-up bare await: {high_files}"

        # MED: CliBad.cs (Tools/Cli).
        med = [h for h in hits if h.severity == SEV_MED]
        med_files = {h.file for h in med}
        assert any(
            "CliBad.cs" in f for f in med_files
        ), f"missing MED for CliBad.cs: {med_files}"

        # LOW: InstallerBad.cs (InstallerLib).
        low = [h for h in hits if h.severity == SEV_LOW]
        low_files = {h.file for h in low}
        assert any(
            "InstallerBad.cs" in f for f in low_files
        ), f"missing LOW for InstallerBad.cs: {low_files}"

        # 3) Allowlist suppression — pick the BadSdk hit, allowlist it.
        report_pre = build_report(list(hits), set(), n_files)
        target = next(
            h for h in hits
            if h.severity == SEV_HIGH and "BadSdk.cs" in h.file
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

        # 4) Bare-path allowlist — listing a relative file path drops
        #    every hit in that file.
        hits3, _ = scan_root(repo, "src")
        bare_path = "src/Tools/Cli/CliBad.cs"
        report_bare = build_report(list(hits3), {bare_path}, n_files)
        bare_files = {h["file"] for h in report_bare["med_violations"]}
        assert bare_path not in bare_files, (
            f"bare-path allowlist did not suppress {bare_path}"
        )

        # 5) Strict mode promotes LOW to fail.
        hits4, _ = scan_root(repo, "src")
        non_strict = build_report(list(hits4), set(), n_files, strict=False)
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
            "missing-configureawait: "
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
