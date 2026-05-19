#!/usr/bin/env python3
"""TaskCompletionSource sync-continuation detector — Pattern #97 CI gate.

Pattern #97 ("TaskCompletionSource without RunContinuationsAsynchronously")
is the failure mode where C# code constructs a ``TaskCompletionSource``
or ``TaskCompletionSource<T>`` without passing
``TaskCreationOptions.RunContinuationsAsynchronously``. The default
behavior is "synchronous continuation": when the producer calls
``TrySetResult`` / ``TrySetException`` / ``TrySetCanceled``, the await
continuation runs INLINE on the producer's thread.

That is fine for a quick consumer but disastrous for any of the workloads
DINOForge actually has:

  * Cross-thread marshalling (Win32 input thread → Unity main thread,
    BepInEx loader thread → ECS world). The continuation steals the
    producer's thread, blocking the next signal it should be servicing.
  * Re-entrancy through the bridge (``GameClient`` consumer awaits a
    response → ``TrySetResult`` runs the continuation → continuation
    enqueues another request synchronously → potential deadlock).
  * Long synchronous waiters (e.g. ``await tcs.Task`` followed by a
    blocking I/O call) starve the producer thread until they finish.

The healthy pattern is ALWAYS to pass
``TaskCreationOptions.RunContinuationsAsynchronously`` so the
continuation is queued back to the threadpool and the producer thread
returns immediately:

    var tcs = new TaskCompletionSource<int>(
        TaskCreationOptions.RunContinuationsAsynchronously);
    var tcs = new TaskCompletionSource(
        TaskCreationOptions.RunContinuationsAsynchronously);

This gate scans ``src/`` for ``TaskCompletionSource`` constructions and
classifies each site:

  * **HIGH** — call site lives under ``src/Runtime/`` or
    ``src/Bridge/``. These are the cross-thread marshalling boundaries
    where inlined continuations cause main-thread starvation /
    cross-thread deadlocks. Most painful in practice.
  * **MED**  — call site lives under ``src/Tools/``, ``src/SDK/``, or
    ``src/Domains/``. Generally async code paths; inlining is still a
    bug but the blast radius is narrower.
  * **LOW**  — call site lives under ``src/Tests/``. Tests can usually
    tolerate inlining (the test thread is the only consumer) but the
    inconsistency hurts policy enforcement. Reported for visibility;
    ``--strict`` promotes to fail the gate.

Auto-skipped sites (zero recorded as hits):

  * Constructions whose argument list contains the literal token
    ``RunContinuationsAsynchronously`` — healthy pattern.

Allowlisting (one entry per line, ``#`` for comments):
``docs/qa/tcs-sync-continuation-allowlist.txt``. Two entry forms:

  1. ``severity|file|line|generic`` — line-locked allowlist key.
  2. Bare ``relative/path.cs`` — suppress every site in that file.

CLI:
    python scripts/ci/detect_tcs_sync_continuations.py
        [--root <src/>]
        [--allowlist <path>]
        [--output <json>]
        [--strict]
        [--quiet|--verbose]
        [--self-test]

Exit 0 = no NEW (un-allowlisted) HIGH/MED hits; 1 = new violations
detected (CI fails); 2 = scan/usage error. ``--strict`` promotes LOW
findings to fail the gate as well.

Modeled on ``scripts/ci/detect_logerror_no_stack.py`` (#268) and
``scripts/ci/detect_unguarded_deserialize.py`` (#265). Pairs with the
TCS sync-continuation sweep being driven by #271 (3 known sites).

This is task #272.
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

# ``new TaskCompletionSource<T>(...)`` or ``new TaskCompletionSource(...)``.
# We capture the optional generic argument and the argument list so the
# call-body inspection can decide whether the healthy options token is
# present.
#
# The regex anchors on ``new TaskCompletionSource`` then optionally
# absorbs a generic argument (``<...>`` with balanced angle brackets is
# unnecessary here because TCS only takes a single type parameter, so a
# greedy non-newline match is sufficient). Then we anchor on the opening
# paren ``(`` and let _find_call_close walk the brackets to find the
# matching ``)``.
TCS_CTOR_RE = re.compile(
    r"\bnew\s+TaskCompletionSource"
    r"(?P<generic><[^<>\n]+>)?"
    r"\s*\(",
)

# Healthy token — any presence in the argument list disqualifies the hit.
HEALTHY_TOKEN = "RunContinuationsAsynchronously"

# Default scan root.
DEFAULT_SCAN_ROOT = "src"

# Path fragments (POSIX-relative, lowercased) that promote a hit to a
# given severity. First-match wins.
HIGH_BOUNDARY_PARTS = (
    "src/runtime/",
    "src/bridge/",
)
MED_BOUNDARY_PARTS = (
    "src/tools/",
    "src/sdk/",
    "src/domains/",
)
LOW_BOUNDARY_PARTS = (
    "src/tests/",
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


def _classify(rel_posix: str) -> str:
    """Return SEV_HIGH / SEV_MED / SEV_LOW for a file path. First match
    along the (HIGH, LOW, MED) order wins. (LOW is checked before MED so
    ``src/Tests/`` does not get caught by the ``src/`` MED catch-all
    if MED were extended.)"""
    lo = rel_posix.lower()
    if any(p in lo for p in HIGH_BOUNDARY_PARTS):
        return SEV_HIGH
    if any(p in lo for p in LOW_BOUNDARY_PARTS):
        return SEV_LOW
    if any(p in lo for p in MED_BOUNDARY_PARTS):
        return SEV_MED
    # Anything outside src/{Runtime,Bridge,Tools,SDK,Domains,Tests} but
    # still under src/ — bucket as MED (conservative; new dirs default
    # to "needs review").
    return SEV_MED


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


@dataclass
class Hit:
    file: str            # POSIX-relative to repo root
    line: int
    generic: str         # ``<int>`` etc. (empty for non-generic form)
    args: str            # raw argument list as written (without parens)
    severity: str
    detail: str
    boundary: str        # high|med|low — coarse path-bucket tag
    allowlist_key: str = field(default="")
    in_allowlist: bool = field(default=False)


def _scan_one_match(
    text: str,
    rel: str,
    m: re.Match,
) -> Hit | None:
    """For a TCS_CTOR_RE match, locate the matching close paren, inspect
    the argument list, and emit a hit if the healthy token is missing."""
    paren_idx = m.end() - 1  # the ``(`` is the last char of the match
    if paren_idx < 0 or paren_idx >= len(text) or text[paren_idx] != "(":
        return None
    close_idx = _find_call_close(text, paren_idx)
    if close_idx == -1:
        return None
    args = text[paren_idx + 1:close_idx - 1]

    # Healthy: argument list mentions RunContinuationsAsynchronously.
    if HEALTHY_TOKEN in args:
        return None

    generic = m.group("generic") or ""
    line = line_of(text, m.start())
    sev = _classify(rel)

    if generic:
        ctor_form = f"new TaskCompletionSource{generic}({args.strip()})"
    else:
        ctor_form = f"new TaskCompletionSource({args.strip()})"

    detail = (
        f"{ctor_form} runs continuations synchronously on the producer's "
        f"thread. Pass TaskCreationOptions.RunContinuationsAsynchronously "
        f"to queue continuations to the threadpool and avoid main-thread "
        f"starvation / cross-thread deadlocks."
    )

    boundary_map = {SEV_HIGH: "high", SEV_MED: "med", SEV_LOW: "low"}
    return Hit(
        file=rel,
        line=line,
        generic=generic,
        args=args.strip(),
        severity=sev,
        detail=detail,
        boundary=boundary_map[sev],
    )


def scan_file(path: Path, repo_root: Path) -> list[Hit]:
    text = read_text_safe(path)
    if not text:
        return []
    rel = path.relative_to(repo_root).as_posix()
    hits: list[Hit] = []
    for m in TCS_CTOR_RE.finditer(text):
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
    for f in files:
        hits.extend(scan_file(f, repo_root))
    return hits, len(files)


# ----------------------------------------------------------------------------
# Report
# ----------------------------------------------------------------------------


def _hit_key(h: Hit) -> str:
    """Stable allowlist key: ``severity|file|line|generic``. The generic
    component is ``<T>`` literally (or empty for the non-generic ctor)."""
    g = h.generic if h.generic else "<>"
    return f"{h.severity}|{h.file}|{h.line}|{g}"


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
        "generic": h.generic,
        "args": h.args,
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
    print("tcs-sync-continuation gate (Pattern #97)")
    print(f"  files scanned          : {report['files_scanned']}")
    print(f"  total hits             : {report['total_hits']}")
    print(f"  allowlist size         : {report['allowlist_size']}")
    print(f"  NEW hits               : {report['new_hits']}")
    print(f"    HIGH (Runtime/Bridge)     : {report['high_count']}")
    print(f"    MED  (Tools/SDK/Domains)  : {report['med_count']}")
    print(f"    LOW  (Tests)              : {report['low_count']}")
    if report["new_hits"]:
        print()
        print("NEW tcs-sync-continuation sites:")
        for sev, items in (
            ("HIGH", report["high_violations"]),
            ("MED", report["med_violations"]),
            ("LOW", report["low_violations"]),
        ):
            if not items:
                continue
            print(f"  -- {sev} --")
            for h in items:
                gen = h["generic"] if h["generic"] else ""
                print(
                    f"    [{h['severity']}] TaskCompletionSource{gen}"
                    f" {h['file']}:{h['line']}"
                )
    print()
    print(f"JSON report: {output_path}")


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Detect TaskCompletionSource constructions without "
            "RunContinuationsAsynchronously (Pattern #97). HIGH = "
            "Runtime/Bridge (cross-thread marshalling); MED = "
            "Tools/SDK/Domains; LOW = Tests."
        )
    )
    p.add_argument(
        "--root",
        default=DEFAULT_SCAN_ROOT,
        help=f"Source root to scan (default: {DEFAULT_SCAN_ROOT})",
    )
    p.add_argument(
        "--allowlist",
        default="docs/qa/tcs-sync-continuation-allowlist.txt",
        help=(
            "Allowlist file; one ``severity|file|line|generic`` key or "
            "bare relative path per line; ``#`` for comments "
            "(default: docs/qa/tcs-sync-continuation-allowlist.txt)"
        ),
    )
    p.add_argument(
        "--output",
        default="docs/qa/tcs-sync-continuation-report.json",
        help="JSON report output path",
    )
    p.add_argument(
        "--strict",
        action="store_true",
        help=(
            "Promote LOW (Tests) findings to fail the gate. "
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


_FIXTURE_HIGH_GENERIC = """
namespace DINOForge.Fixture.Runtime
{
    using System.Threading.Tasks;

    public class Producer
    {
        public TaskCompletionSource<int> Make()
        {
            // Zero-arg generic ctor -> HIGH (Runtime path).
            var tcs = new TaskCompletionSource<int>();
            return tcs;
        }
    }
}
"""

_FIXTURE_HIGH_NONGENERIC = """
namespace DINOForge.Fixture.Bridge
{
    using System.Threading.Tasks;

    public class Producer
    {
        public TaskCompletionSource Make()
        {
            // Non-generic zero-arg ctor -> HIGH (Bridge path).
            var tcs = new TaskCompletionSource();
            return tcs;
        }
    }
}
"""

_FIXTURE_HEALTHY_GENERIC = """
namespace DINOForge.Fixture.Healthy
{
    using System.Threading.Tasks;

    public class GoodProducer
    {
        public TaskCompletionSource<int> Make()
        {
            // Healthy: option flag present.
            return new TaskCompletionSource<int>(
                TaskCreationOptions.RunContinuationsAsynchronously);
        }
    }
}
"""

_FIXTURE_HEALTHY_NONGENERIC = """
namespace DINOForge.Fixture.HealthyNg
{
    using System.Threading.Tasks;

    public class GoodProducer
    {
        public TaskCompletionSource Make()
        {
            return new TaskCompletionSource(
                TaskCreationOptions.RunContinuationsAsynchronously);
        }
    }
}
"""

_FIXTURE_MED_TOOLS = """
namespace DINOForge.Fixture.Tools
{
    using System.Threading.Tasks;

    public class CliProducer
    {
        public TaskCompletionSource<bool> Make()
        {
            // Tools path -> MED.
            return new TaskCompletionSource<bool>();
        }
    }
}
"""

_FIXTURE_LOW_TESTS = """
namespace DINOForge.Fixture.Tests
{
    using System.Threading.Tasks;

    public class TestProducer
    {
        public TaskCompletionSource<string> Make()
        {
            // Tests path -> LOW.
            return new TaskCompletionSource<string>();
        }
    }
}
"""


def _build_fixture_repo(td: Path) -> Path:
    """Lay out a synthetic repo under *td* matching the gate's expected
    structure: HIGH boundaries (Runtime/Bridge), MED boundaries
    (Tools/SDK/Domains), LOW boundary (Tests), and healthy variants."""
    repo = td / "repo"
    runtime = repo / "src" / "Runtime"
    bridge = repo / "src" / "Bridge" / "Client"
    tools = repo / "src" / "Tools" / "Cli"
    sdk = repo / "src" / "SDK" / "Validation"
    tests = repo / "src" / "Tests"
    for d in (runtime, bridge, tools, sdk, tests):
        d.mkdir(parents=True, exist_ok=True)

    # HIGH boundary: Runtime + Bridge.
    (runtime / "Producer.cs").write_text(
        _FIXTURE_HIGH_GENERIC, encoding="utf-8"
    )
    (bridge / "ProducerNg.cs").write_text(
        _FIXTURE_HIGH_NONGENERIC, encoding="utf-8"
    )

    # Healthy sites in HIGH boundary — should NOT be flagged.
    (runtime / "GoodProducer.cs").write_text(
        _FIXTURE_HEALTHY_GENERIC, encoding="utf-8"
    )
    (bridge / "GoodProducerNg.cs").write_text(
        _FIXTURE_HEALTHY_NONGENERIC, encoding="utf-8"
    )

    # MED boundary: Tools.
    (tools / "CliProducer.cs").write_text(
        _FIXTURE_MED_TOOLS, encoding="utf-8"
    )

    # LOW boundary: Tests.
    (tests / "TestProducer.cs").write_text(
        _FIXTURE_LOW_TESTS, encoding="utf-8"
    )

    return repo


def _self_test() -> int:  # noqa: C901
    import tempfile

    # 1) Regex sanity — direct string checks.
    # Generic zero-arg.
    assert TCS_CTOR_RE.search("new TaskCompletionSource<int>()")
    assert TCS_CTOR_RE.search("new TaskCompletionSource<MyDto>(  )")
    # Non-generic zero-arg.
    assert TCS_CTOR_RE.search("new TaskCompletionSource()")
    # Generic with options arg (regex still matches; the post-filter
    # rejects it via HEALTHY_TOKEN).
    m = TCS_CTOR_RE.search(
        "new TaskCompletionSource<int>("
        "TaskCreationOptions.RunContinuationsAsynchronously)"
    )
    assert m is not None
    # Should NOT match TaskCompletionSourceFactory or similar prefix-extended
    # identifiers.
    assert not TCS_CTOR_RE.search("new TaskCompletionSourceFactory<int>()")

    # 2) End-to-end scan against the synthetic repo.
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        repo = _build_fixture_repo(td_path)

        hits, n_files = scan_root(repo, "src")
        files_seen = {h.file for h in hits}

        # Healthy sites must NOT appear.
        assert not any(
            "GoodProducer.cs" in f for f in files_seen
        ), f"healthy generic flagged: {files_seen}"
        assert not any(
            "GoodProducerNg.cs" in f for f in files_seen
        ), f"healthy non-generic flagged: {files_seen}"

        # HIGH: Runtime/Producer.cs (generic) + Bridge/ProducerNg.cs (non-generic).
        high = [h for h in hits if h.severity == SEV_HIGH]
        high_files = {h.file for h in high}
        assert any(
            "Runtime/Producer.cs" in f for f in high_files
        ), f"missing HIGH for Runtime/Producer.cs: {high_files}"
        assert any(
            "Bridge/Client/ProducerNg.cs" in f for f in high_files
        ), f"missing HIGH for Bridge/Client/ProducerNg.cs: {high_files}"

        # Generic discrimination — generic vs non-generic captured.
        forms = {(h.file.split("/")[-1], h.generic) for h in high}
        assert ("Producer.cs", "<int>") in forms, forms
        assert ("ProducerNg.cs", "") in forms, forms

        # MED: Tools site.
        med = [h for h in hits if h.severity == SEV_MED]
        med_files = {h.file for h in med}
        assert any(
            "CliProducer.cs" in f for f in med_files
        ), f"missing MED for CliProducer.cs: {med_files}"

        # LOW: Tests site.
        low = [h for h in hits if h.severity == SEV_LOW]
        low_files = {h.file for h in low}
        assert any(
            "TestProducer.cs" in f for f in low_files
        ), f"missing LOW for TestProducer.cs: {low_files}"

        # 3) Allowlist suppression — pick the Runtime hit, allowlist it.
        report_pre = build_report(list(hits), set(), n_files)
        target = next(
            h for h in hits
            if h.severity == SEV_HIGH and "Runtime/Producer.cs" in h.file
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
        bare_path = "src/Tools/Cli/CliProducer.cs"
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
            "tcs-sync-continuation: "
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
