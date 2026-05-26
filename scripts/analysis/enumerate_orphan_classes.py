#!/usr/bin/env python3
"""Orphan-class enumerator — finds public classes that have zero (or test-only) call sites.

Usage:
    python scripts/analysis/enumerate_orphan_classes.py [options]

Exit 0 = success (always; baseline is informational, not a gate)
Exit 1 = baseline-compare detected NEW orphan classes (only with --baseline-compare)
Exit 2 = scan error (root path missing, etc.)

Background: see Pattern #86 in TRUTH_TABLE — "class exists, zero call sites".
Feature classes get added with self-tests but Plugin.cs / production callers never
wire them. This is the discipline tool: enumerate every public class under src/,
filter out intentional orphan-by-design categories (interfaces, DTOs, attributes,
BepInEx plugin entry points, MonoBehaviours), and grep the rest for non-self,
non-doc-comment references. Any class with zero production references is flagged.

This is task #229. Modeled on enumerate_mock_theater.py and check_trait_fraud.py.

Categories:
    DEAD       — 0 total references anywhere in src/
    TEST_ONLY  — 0 production references, >0 test references
    PROD       — >0 production references (healthy)
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime
from pathlib import Path
from typing import Iterable


# ----------------------------------------------------------------------------
# Regex catalogue
# ----------------------------------------------------------------------------

# Capture nested + top-level class declarations. We accept any access modifier
# combination and any of the partial/sealed/abstract/static markers.
CLASS_DECL_RE = re.compile(
    r"^[ \t]*"
    r"(?P<attrs>(?:\[[^\]]*\][ \t]*\r?\n[ \t]*)*)"  # leading attribute lines
    r"(?P<modifiers>(?:public|internal|private|protected)"
    r"(?:[ \t]+(?:sealed|abstract|static|partial|new|unsafe))*)"
    r"[ \t]+class[ \t]+"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
    r"(?P<rest>[^\r\n{]*)",
    re.MULTILINE,
)

INTERFACE_DECL_RE = re.compile(
    r"^[ \t]*(?:public|internal)[ \t]+interface[ \t]+(?P<name>[A-Za-z_]\w*)",
    re.MULTILINE,
)

BEPIN_PLUGIN_ATTR_RE = re.compile(r"\[\s*BepInPlugin\s*\(")
MONOBEHAVIOUR_BASE_RE = re.compile(r":\s*[^{}]*\bMonoBehaviour\b")
JSONRPC_HANDLER_HINT_RE = re.compile(r"\[\s*JsonRpcMethod\s*\(|RegisterMethod\s*\(|RegisterHandler\s*\(")

# Iter-51 D3: reflection-bound false-positive suppressors.
# Each is intentionally narrow — match only the specific reflection contract.
IVALUE_CONVERTER_RE = re.compile(r"\bIValueConverter\b")
BURST_COMPILE_ATTR_RE = re.compile(r"\[\s*BurstCompile\b")
ECS_SYSTEM_BASE_RE = re.compile(
    r":\s*[^{}]*\b(SystemBase|ComponentSystem|ISystem|JobComponentSystem)\b"
)
SYSTEM_COMMAND_BASE_RE = re.compile(r":\s*[^{}]*\b(Command|CommandHandler)\b")
XAML_CODEBEHIND_FILENAME_RE = re.compile(r"\.(?:axaml|xaml)\.cs$", re.IGNORECASE)
SYSTEM_COMMANDLINE_PATH_RE = re.compile(r"(?:^|/)Tools/Cli/(?:[^/]+/)*Commands/")

# Heuristic for "DTO" — body contains few public method-like signatures.
# We catch:
#   public ReturnType Name(...)
#   public static ReturnType Name(...)
#   public async Task Name(...)
#   public ReturnType Name<T>(...) where T : ...
#   public ReturnType Name(...) =>
# but we DO NOT count properties (which never use `(`) and ctors (which match
# the class name and have no return type — hard to distinguish without a parser,
# so we accept that ctor matches inflate slightly).
PUBLIC_METHOD_RE = re.compile(
    r"\bpublic\b[^;{=]*?\b[A-Za-z_][A-Za-z0-9_<>\?,\s]*?\s+"
    r"(?P<name>[A-Z][A-Za-z0-9_]*)"
    r"\s*(?:<[^>]*>)?\s*\("
)

GENERATED_FILENAME_SUFFIXES = (
    ".designer.cs",
    ".g.cs",
    ".g.i.cs",
    ".AssemblyInfo.cs",
    ".AssemblyAttributes.cs",
    ".GlobalUsings.g.cs",
)

EXCLUDED_DIR_PARTS = {"bin", "obj"}


# ----------------------------------------------------------------------------
# Data model (plain dicts; matches enumerate_mock_theater.py style)
# ----------------------------------------------------------------------------


def is_excluded_path(path: Path, root: Path) -> bool:
    """True if the path should be skipped from class extraction."""
    parts = set(path.parts)
    if parts & EXCLUDED_DIR_PARTS:
        return True
    name = path.name.lower()
    for suffix in GENERATED_FILENAME_SUFFIXES:
        if name.endswith(suffix.lower()):
            return True
    return False


def is_test_path(path: Path, root: Path) -> bool:
    rel = path.relative_to(root).as_posix()
    return rel.startswith("Tests/") or "/Tests/" in ("/" + rel)


def read_text_safe(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def find_class_body(text: str, decl_end: int) -> str:
    """Given the offset just after `class Name<...>(...)` rest-of-line,
    return the brace-balanced body text. Returns "" if not found."""
    # Locate the opening brace
    brace_open = text.find("{", decl_end)
    if brace_open == -1:
        return ""
    depth = 0
    i = brace_open
    end = len(text)
    in_string = False
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
            elif c == '"':
                in_string = True
            elif c == "'":
                in_char = True
            elif c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    return text[brace_open + 1 : i]
        i += 1
    return text[brace_open + 1 :]


def is_attribute_class(name: str) -> bool:
    return name.endswith("Attribute")


def is_dto_like(body: str) -> bool:
    """Heuristic: a DTO is a class that holds data only — body has zero
    detectable public method signatures. We deliberately keep this strict so
    feature classes with even one Run/Inject/Wire method are NOT classified
    as DTOs. Empty body → DTO-like (rare; usually a marker class).

    Note: this still accepts ``static`` helper classes that have only static
    methods because PUBLIC_METHOD_RE matches them. Pure DTOs typically expose
    only ``public T Name { get; set; }`` properties (no ``(``)."""
    if not body.strip():
        return True
    methods = list(PUBLIC_METHOD_RE.finditer(body))
    return len(methods) == 0


def has_bepin_plugin_attr(text_above: str) -> bool:
    return bool(BEPIN_PLUGIN_ATTR_RE.search(text_above))


def is_monobehaviour(rest_of_decl: str) -> bool:
    return bool(MONOBEHAVIOUR_BASE_RE.search(rest_of_decl))


def has_jsonrpc_handler_hint(body: str) -> bool:
    return bool(JSONRPC_HANDLER_HINT_RE.search(body))


def collect_classes(root: Path, verbose: bool = False) -> tuple[list[dict], int]:
    """Walk all *.cs under root (including tests), return list of class records.

    Each record has: class, file, line, is_test_file, body, modifiers, rest_of_decl,
    attribute_block (text immediately preceding decl), category_skip (str | None).
    """
    classes: list[dict] = []
    files_scanned = 0
    interface_names: set[str] = set()

    for cs_file in sorted(root.rglob("*.cs")):
        if is_excluded_path(cs_file, root):
            continue
        text = read_text_safe(cs_file)
        if not text:
            continue
        files_scanned += 1

        # Capture interfaces so we can filter callers that match an interface name
        # (we never want to flag an interface as orphan; we also don't filter usages
        # that happen to be the interface name vs class name — names are distinct).
        for im in INTERFACE_DECL_RE.finditer(text):
            interface_names.add(im.group("name"))

        rel = cs_file.relative_to(root).as_posix()
        is_test = is_test_path(cs_file, root)

        for m in CLASS_DECL_RE.finditer(text):
            name = m.group("name")
            modifiers = m.group("modifiers")
            rest_of_decl = m.group("rest")
            decl_start = m.start()
            decl_end = m.end()
            line = text.count("\n", 0, decl_start) + 1

            # Captured-attribute block + a small lookback for cases where the
            # attribute lines are split by extra blank lines or doc comments.
            inline_attrs = m.group("attrs") or ""
            lookback_start = max(0, decl_start - 800)
            lookback = text[lookback_start:decl_start]
            attribute_block_tail = lookback + inline_attrs

            body = find_class_body(text, decl_end)

            classes.append(
                {
                    "class": name,
                    "file": rel,
                    "line": line,
                    "is_test_file": is_test,
                    "modifiers": modifiers,
                    "rest_of_decl": rest_of_decl,
                    "attribute_block": attribute_block_tail,
                    "body": body,
                }
            )

        if verbose and files_scanned % 200 == 0:
            print(f"  scanned {files_scanned} files, {len(classes)} classes so far",
                  file=sys.stderr)

    return classes, files_scanned


def classify_skip(record: dict) -> str | None:
    """Return a non-None reason string if the class should be excluded from the
    orphan check (orphan-by-design).

    Order is priority — cheapest checks first, then reflection-bound suppressors
    added in iter-51 D3 (XAML, Burst, ECS, System.CommandLine).
    """
    name = record["class"]
    rest_of_decl = record.get("rest_of_decl", "") or ""
    attribute_block = record.get("attribute_block", "") or ""
    body = record.get("body", "") or ""
    file_path = record.get("file", "") or ""
    modifiers = record.get("modifiers", "") or ""

    if is_attribute_class(name):
        return "attribute_class"
    if has_bepin_plugin_attr(attribute_block):
        return "bepin_plugin_entry"
    if is_monobehaviour(rest_of_decl):
        return "monobehaviour"
    if record["is_test_file"]:
        return "test_fixture"

    # --- Iter-51 D3: reflection-bound suppressors (before DTO catch-all) ---

    # XAML-bound IValueConverter (Avalonia + WinUI). Invisible to C# grep.
    if IVALUE_CONVERTER_RE.search(rest_of_decl):
        return "value_converter"

    # Unity ECS systems are auto-discovered via World.GetOrCreateSystem<T>().
    if ECS_SYSTEM_BASE_RE.search(rest_of_decl):
        return "ecs_system_subclass"

    # Burst-compiled jobs/systems are reflection-bound by Unity Burst.
    if BURST_COMPILE_ATTR_RE.search(attribute_block):
        return "burst_compiled"
    # Static helper class with a [BurstCompile] method body.
    if "static" in modifiers and BURST_COMPILE_ATTR_RE.search(body):
        return "burst_compiled"

    # XAML code-behind partial class (paired to .axaml/.xaml root).
    if XAML_CODEBEHIND_FILENAME_RE.search(file_path):
        return "xaml_codebehind"

    # System.CommandLine handler reflection-bound via CommandHandler.Create().
    if (
        SYSTEM_COMMAND_BASE_RE.search(rest_of_decl)
        and SYSTEM_COMMANDLINE_PATH_RE.search(file_path)
    ):
        return "system_command_handler"

    if is_dto_like(body):
        return "dto_like"
    return None


def count_references(
    name: str,
    declaring_file: str,
    src_root: Path,
    all_files: list[Path],
) -> tuple[int, int, int]:
    """Return (total_refs, prod_refs, test_refs) for the class name across src/.

    Doc-comment lines and self-referential lines (within declaring_file) are
    excluded. Reference matched by \\bName\\b on a non-comment, non-decl line.
    """
    pattern = re.compile(r"\b" + re.escape(name) + r"\b")
    total = 0
    test_refs = 0
    prod_refs = 0

    for path in all_files:
        rel = path.relative_to(src_root).as_posix()
        # Skip the declaring file entirely (self-reference)
        if rel == declaring_file:
            continue
        text = read_text_safe(path)
        if not text or name not in text:
            continue
        is_test = rel.startswith("Tests/")
        for line in text.splitlines():
            stripped = line.lstrip()
            if stripped.startswith("///") or stripped.startswith("//"):
                continue
            if "/*" in stripped and "*/" in stripped:
                # crude single-line block comment skip
                continue
            if not pattern.search(line):
                continue
            total += 1
            if is_test:
                test_refs += 1
            else:
                prod_refs += 1
    return total, prod_refs, test_refs


def categorize(total: int, prod: int, test: int) -> str:
    if total == 0:
        return "DEAD"
    if prod == 0 and test > 0:
        return "TEST_ONLY"
    return "PROD"


# ----------------------------------------------------------------------------
# CLI
# ----------------------------------------------------------------------------


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Enumerate orphan public classes (Pattern #86) under src/."
    )
    parser.add_argument("--root", default="src", help="Source root (default: src)")
    parser.add_argument(
        "--output",
        default=None,
        help="JSON output path (default: docs/proof/orphan-classes-YYYYMMDD.json)",
    )
    parser.add_argument("--quiet", action="store_true", help="Minimal console output")
    parser.add_argument("--verbose", action="store_true", help="Verbose progress")
    parser.add_argument(
        "--json-only",
        action="store_true",
        help="Emit JSON file, skip the human summary",
    )
    parser.add_argument(
        "--baseline-compare",
        default=None,
        help=(
            "Path to a prior baseline JSON. Exits 1 if any DEAD classes are new "
            "vs the baseline (CI gate mode)."
        ),
    )
    parser.add_argument(
        "--show-suppressed",
        action="store_true",
        help=(
            "Include `suppressed_classes` in the JSON output, grouped by "
            "skip-category. Default off (keeps JSON small). Useful for auditing "
            "false-positive suppressors."
        ),
    )
    return parser.parse_args(argv)


def repo_root_from_script() -> Path:
    """scripts/analysis/<this>.py → repo root is parents[2]."""
    return Path(__file__).resolve().parents[2]


def resolve_paths(args: argparse.Namespace) -> tuple[Path, Path]:
    repo_root = repo_root_from_script()
    root = Path(args.root)
    if not root.is_absolute():
        root = (repo_root / root).resolve()
    if args.output:
        output = Path(args.output)
        if not output.is_absolute():
            output = (repo_root / output).resolve()
    else:
        date = datetime.utcnow().strftime("%Y%m%d")
        output = (repo_root / "docs" / "proof" / f"orphan-classes-{date}.json").resolve()
    return root, output


def collect_cs_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for p in root.rglob("*.cs"):
        if is_excluded_path(p, root):
            continue
        files.append(p)
    return files


def build_report(
    root: Path,
    classes: list[dict],
    cs_files: list[Path],
    verbose: bool,
    show_suppressed: bool = False,
) -> dict:
    dead: list[dict] = []
    test_only: list[dict] = []
    prod_count = 0
    skipped_by_category: dict[str, int] = {}
    suppressed_by_category: dict[str, list[dict]] = {}
    indirect_callsite_candidates: list[dict] = []

    # Pre-index file lists outside src/Tests for faster sequential scan.
    candidates = []
    for rec in classes:
        skip = classify_skip(rec)
        if skip:
            skipped_by_category[skip] = skipped_by_category.get(skip, 0) + 1
            if show_suppressed:
                suppressed_by_category.setdefault(skip, []).append(
                    {
                        "class": rec["class"],
                        "file": rec["file"],
                        "line": rec["line"],
                    }
                )
            continue
        candidates.append(rec)

    if verbose:
        print(f"  {len(candidates)} candidate classes after skip filter "
              f"(of {len(classes)} total)", file=sys.stderr)

    for i, rec in enumerate(candidates):
        if verbose and i % 50 == 0 and i:
            print(f"  ref-scanned {i}/{len(candidates)}", file=sys.stderr)
        total, prod, test = count_references(
            rec["class"], rec["file"], root, cs_files
        )
        category = categorize(total, prod, test)
        out_record = {
            "class": rec["class"],
            "file": rec["file"],
            "line": rec["line"],
            "total_refs": total,
            "prod_refs": prod,
            "test_refs": test,
        }
        if has_jsonrpc_handler_hint(rec["body"]):
            out_record["indirect_callsite_hint"] = "jsonrpc_handler"
            indirect_callsite_candidates.append(out_record)
        if category == "DEAD":
            dead.append(out_record)
        elif category == "TEST_ONLY":
            test_only.append(out_record)
        else:
            prod_count += 1

    dead.sort(key=lambda r: (r["file"], r["class"]))
    test_only.sort(key=lambda r: (-r["test_refs"], r["file"], r["class"]))
    if show_suppressed:
        for cat in suppressed_by_category:
            suppressed_by_category[cat].sort(key=lambda r: (r["file"], r["class"]))

    report: dict = {
        "scan_utc": datetime.utcnow().isoformat() + "Z",
        "scan_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "scan_root": root.as_posix(),
        "exclusions": [
            "src/Tests/** (test fixtures excluded from candidate list)",
            "**/bin/**",
            "**/obj/**",
            "*.designer.cs",
            "*.g.cs",
            "*.AssemblyInfo.cs",
        ],
        "skip_categories": skipped_by_category,
        "category_counts": {
            "DEAD": len(dead),
            "TEST_ONLY": len(test_only),
            "PROD": prod_count,
        },
        "totals": {
            "files_scanned": len(cs_files),
            "classes_seen": len(classes),
            "candidates_after_skip": len(candidates),
        },
        "dead_classes": dead,
        "test_only_classes": test_only,
        "indirect_callsite_candidates": indirect_callsite_candidates,
        "summary_top_dead": dead[:10],
    }
    if show_suppressed:
        report["suppressed_classes"] = suppressed_by_category
    return report


def print_summary(report: dict, output_path: Path) -> None:
    counts = report["category_counts"]
    totals = report["totals"]
    print("orphan-class enumeration")
    print(f"  scan_root              : {report['scan_root']}")
    print(f"  files scanned          : {totals['files_scanned']}")
    print(f"  classes seen           : {totals['classes_seen']}")
    print(f"  candidates (post-skip) : {totals['candidates_after_skip']}")
    print(f"  DEAD classes           : {counts['DEAD']}")
    print(f"  TEST_ONLY classes      : {counts['TEST_ONLY']}")
    print(f"  PROD classes           : {counts['PROD']}")
    print(f"  skip categories        : {report['skip_categories']}")
    print(f"  indirect-callsite candidates: {len(report['indirect_callsite_candidates'])}")
    if report["dead_classes"]:
        print()
        print("Top 10 DEAD classes:")
        for r in report["summary_top_dead"]:
            print(f"  - {r['class']:<40} {r['file']}:{r['line']}")
    if report["test_only_classes"]:
        print()
        print("Top 10 TEST_ONLY classes:")
        for r in report["test_only_classes"][:10]:
            print(f"  - {r['class']:<40} test_refs={r['test_refs']}  {r['file']}:{r['line']}")
    print()
    print(f"JSON output: {output_path}")


def compare_baseline(report: dict, baseline_path: Path) -> int:
    """Return exit code; print newly-orphan classes vs baseline."""
    try:
        prior = json.loads(baseline_path.read_text(encoding="utf-8"))
    except OSError as exc:
        print(f"ERROR: cannot read baseline {baseline_path}: {exc}", file=sys.stderr)
        return 2
    prior_dead = {(c["class"], c["file"]) for c in prior.get("dead_classes", [])}
    current_dead = {(c["class"], c["file"]) for c in report["dead_classes"]}
    newly_dead = sorted(current_dead - prior_dead)
    if newly_dead:
        print("NEWLY-DEAD classes vs baseline:")
        for cls, fp in newly_dead:
            print(f"  - {cls}  {fp}")
        return 1
    print("No new DEAD classes since baseline.")
    return 0


# ----------------------------------------------------------------------------
# Self-tests (run with: python enumerate_orphan_classes.py --self-test)
# ----------------------------------------------------------------------------


def _self_test() -> int:
    """Lightweight assertions over the regex + helpers."""
    sample = """
namespace Foo
{
    /// <summary>doc</summary>
    [BepInPlugin("a","b","1")]
    public sealed class PluginEntry : MonoBehaviour
    {
        public void Awake() { }
    }

    public class FooAdapter
    {
        public void DoThing() { }
        public int Compute(int x) { return x + 1; }
        public bool Check() { return true; }
    }

    public sealed class FooDto
    {
        public int Id { get; set; }
        public string Name { get; set; }
    }

    public class MyAttribute : Attribute { }

    internal sealed class HelperImpl { public void Run() { } public void Run2(){} public void Run3(){} }
}
"""
    matches = [m.group("name") for m in CLASS_DECL_RE.finditer(sample)]
    assert "PluginEntry" in matches, matches
    assert "FooAdapter" in matches, matches
    assert "FooDto" in matches, matches
    assert "MyAttribute" in matches, matches
    assert "HelperImpl" in matches, matches

    # body extraction
    decl_match = next(m for m in CLASS_DECL_RE.finditer(sample) if m.group("name") == "FooAdapter")
    body = find_class_body(sample, decl_match.end())
    assert "DoThing" in body
    assert "Compute" in body
    assert "Check" in body
    assert not is_dto_like(body), "FooAdapter has public methods, should NOT be DTO-like"

    # DTO heuristic — properties only, no methods
    dto_match = next(m for m in CLASS_DECL_RE.finditer(sample) if m.group("name") == "FooDto")
    dto_body = find_class_body(sample, dto_match.end())
    assert is_dto_like(dto_body), f"FooDto is DTO-like, body={dto_body!r}"

    # Attribute class
    assert is_attribute_class("MyAttribute")
    assert not is_attribute_class("FooAdapter")

    # MonoBehaviour
    plug_match = next(m for m in CLASS_DECL_RE.finditer(sample) if m.group("name") == "PluginEntry")
    assert is_monobehaviour(plug_match.group("rest"))

    # BepInPlugin attribute lookback (captured attrs group + lookback)
    attr_block = sample[max(0, plug_match.start() - 800) : plug_match.start()] + (plug_match.group("attrs") or "")
    assert has_bepin_plugin_attr(attr_block), f"BepInPlugin attr not found in: {attr_block!r}"

    # Doc-comment / self filter via count_references is harder to unit test
    # without filesystem fixtures, so we verify the regex on a string:
    pattern = re.compile(r"\b" + re.escape("FooAdapter") + r"\b")
    assert pattern.search("var x = new FooAdapter();")
    assert not pattern.search("var x = new FooAdapter2();")

    # ------------------------------------------------------------------
    # Iter-51 D3: 5 reflection-bound suppressors
    # ------------------------------------------------------------------

    def _mk_record(
        cls: str,
        rest: str = "",
        attrs: str = "",
        body: str = "",
        file: str = "src/Foo/Bar.cs",
        modifiers: str = "public",
        is_test: bool = False,
    ) -> dict:
        return {
            "class": cls,
            "rest_of_decl": rest,
            "attribute_block": attrs,
            "body": body,
            "file": file,
            "modifiers": modifiers,
            "is_test_file": is_test,
            "line": 1,
        }

    # 1. value_converter — IValueConverter on declaration line
    rec_vc = _mk_record(
        "BoolToOpacityConverter",
        rest=" : IValueConverter",
        body="public object Convert(...) { } public object ConvertBack(...) { }",
        file="src/Tools/Installer/GUI/Converters/BoolToOpacityConverter.cs",
    )
    res_vc = classify_skip(rec_vc)
    assert res_vc == "value_converter", f"expected value_converter, got {res_vc!r}"

    # 2. ecs_system_subclass — SystemBase / ComponentSystem / ISystem / JobComponentSystem
    rec_ecs = _mk_record(
        "MyHudSystem",
        rest=" : SystemBase",
        body="protected override void OnUpdate() { }",
        file="src/Runtime/Systems/MyHudSystem.cs",
    )
    res_ecs = classify_skip(rec_ecs)
    assert res_ecs == "ecs_system_subclass", f"expected ecs_system_subclass, got {res_ecs!r}"

    rec_ecs2 = _mk_record(
        "BurstJobSys",
        rest=" : ISystem",
        body="public void OnCreate(ref SystemState s) { }",
        file="src/Runtime/Systems/BurstJobSys.cs",
    )
    assert classify_skip(rec_ecs2) == "ecs_system_subclass"

    # 3. burst_compiled — [BurstCompile] on declaration attributes
    rec_burst_attr = _mk_record(
        "MyBurstJob",
        rest=" : IJob",
        attrs="[BurstCompile]\n",
        body="public void Execute() { }",
        file="src/Runtime/Jobs/MyBurstJob.cs",
    )
    # Note: ECS check triggers first only if rest_of_decl matches an ECS base.
    # IJob is not in that list, so burst_compiled should win.
    res_burst = classify_skip(rec_burst_attr)
    assert res_burst == "burst_compiled", f"expected burst_compiled, got {res_burst!r}"

    # 3b. burst_compiled via static helper class with [BurstCompile] in body
    rec_burst_static = _mk_record(
        "BurstHelpers",
        rest="",
        body="[BurstCompile] public static int Add(int a, int b) => a+b;",
        file="src/Runtime/Math/BurstHelpers.cs",
        modifiers="public static",
    )
    res_burst_static = classify_skip(rec_burst_static)
    assert res_burst_static == "burst_compiled", (
        f"expected burst_compiled (static), got {res_burst_static!r}"
    )

    # 4. xaml_codebehind — filename ends with .axaml.cs / .xaml.cs
    rec_xaml_avalonia = _mk_record(
        "MainWindow",
        rest="",
        body="public MainWindow() { InitializeComponent(); }",
        file="src/Tools/Installer/GUI/Views/MainWindow.axaml.cs",
    )
    res_xaml = classify_skip(rec_xaml_avalonia)
    assert res_xaml == "xaml_codebehind", f"expected xaml_codebehind, got {res_xaml!r}"

    rec_xaml_winui = _mk_record(
        "ShellPage",
        rest="",
        body="public ShellPage() { }",
        file="src/Tools/DesktopCompanion/Views/ShellPage.xaml.cs",
    )
    assert classify_skip(rec_xaml_winui) == "xaml_codebehind"

    # 5. system_command_handler — Command/CommandHandler base + Tools/Cli/.../Commands/ path
    rec_cmd = _mk_record(
        "DeployCommand",
        rest=" : Command",
        body="public DeployCommand() : base(\"deploy\") { }",
        file="src/Tools/Cli/Commands/DeployCommand.cs",
    )
    res_cmd = classify_skip(rec_cmd)
    assert res_cmd == "system_command_handler", (
        f"expected system_command_handler, got {res_cmd!r}"
    )

    # Negative: same base class but path is NOT under Tools/Cli/.../Commands/ —
    # must NOT be suppressed (could be an unrelated `Command` base elsewhere).
    rec_cmd_neg = _mk_record(
        "FakeCommand",
        rest=" : Command",
        body="public void Execute() { }",
        file="src/Tools/Other/FakeCommand.cs",
    )
    assert classify_skip(rec_cmd_neg) != "system_command_handler", (
        "non-Cli/Commands path should not be suppressed by system_command_handler"
    )

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
        print("ERROR: --quiet and --verbose are mutually exclusive", file=sys.stderr)
        return 2

    root, output = resolve_paths(args)
    if not root.exists():
        print(f"ERROR: scan root not found: {root}", file=sys.stderr)
        return 2

    if not args.quiet:
        print(f"scanning {root} ...", file=sys.stderr)

    classes, _ = collect_classes(root, verbose=args.verbose)
    cs_files = collect_cs_files(root)

    report = build_report(
        root,
        classes,
        cs_files,
        verbose=args.verbose,
        show_suppressed=args.show_suppressed,
    )

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2), encoding="utf-8")

    if not args.json_only and not args.quiet:
        print_summary(report, output)
    elif args.quiet:
        # one line for CI logs
        c = report["category_counts"]
        print(
            f"orphan-classes: DEAD={c['DEAD']} TEST_ONLY={c['TEST_ONLY']} "
            f"PROD={c['PROD']} -> {output}"
        )

    if args.baseline_compare:
        return compare_baseline(report, Path(args.baseline_compare))

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
