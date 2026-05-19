#!/usr/bin/env python3
"""Unnecessary LINQ Terminal Allocation detector — Pattern #121 CI gate.

Pattern #121 ("Unnecessary LINQ Terminal Allocation") is the failure mode where
.ToList() or .ToArray() is called on an IEnumerable sequence when the result is:

  1. **HIGH** — Used in a lock() block (within 10 preceding lines) OR is the entire
     body of a property getter / arrow expression (e.g. => x.ToList()).
     These introduce allocation waste in critical paths (synchronization) and
     on every property access. Must be removed or replaced with .AsReadOnly().

  2. **MED** — Immediately consumed by foreach/for on the next line only.
     The result is materialized but not stored. Can usually be inlined or use
     yield return. Flagged for manual review.

  3. **LOW** — All other uses. May be justified (external API contract, immutability
     requirement). Not flagged for action.

Allowlisting (one entry per line, # for comments):
  ``docs/qa/unnecessary-allocation-allowlist.txt``. Three suppression mechanisms:

  1. ``severity|file|line`` — line-locked allowlist key. Pull from the
     ``allowlist_key`` field of the JSON report. Moving the line forces
     a fresh review (intentional — prevents silent drift).
  2. Bare ``relative/path.cs`` — suppress every site in that file.
  3. Trailing ``// allocation-ok: <reason>`` comment on the same source line.
     Inline self-documenting suppression.

Scans: .cs files under src/Runtime/, src/Tools/, src/Bridge/, src/SDK/
Excludes: Tests/, bin/, obj/, .git/

CLI:
    python scripts/ci/detect_unnecessary_allocation.py
        [--root <repo>]
        [--allowlist <path>]
        [--output <json>]
        [--threshold N]
        [--strict]
        [--quiet|--verbose]
        [--test]

Exit 0 = no new HIGH hits or (HIGH count <= threshold); 1 = too many NEW
unallowlisted hits (CI fails); 2 = usage error. ``--strict`` fails on any
unallowlisted HIGH, regardless of threshold.
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

# Match .ToList() or .ToArray() — capture the method and context
ALLOCATION_TERMINAL_RE = re.compile(
    r"\.To(List|Array)\s*\(\)",
    re.MULTILINE
)


@dataclass
class AllocationHit:
    """Single .ToList()/.ToArray() occurrence."""
    severity: str  # HIGH, MED, LOW
    file: str  # relative path
    line_num: int  # 1-based
    method: str  # ToList or ToArray
    line_content: str  # the actual line
    context_lines: list[str] = field(default_factory=list)  # surrounding lines for HIGH classification
    allowlist_key: str = ""  # severity|file|line for lookup

    def to_dict(self) -> dict:
        return {
            "severity": self.severity,
            "file": self.file,
            "line_num": self.line_num,
            "method": self.method,
            "line_content": self.line_content.strip(),
            "allowlist_key": self.allowlist_key,
        }


def classify_severity(
    line_num: int,
    line_content: str,
    lines: list[str],
    file_path: str
) -> str:
    """Classify severity: HIGH, MED, or LOW."""

    # HIGH: property getter / arrow expression
    # Pattern: => ... .ToList() or { return ... .ToList(); }
    if "=>" in line_content:
        # Arrow expression with .ToList()/.ToArray() at the end
        stripped = line_content.strip()
        if (".ToList()" in stripped or ".ToArray()" in stripped) and \
           (stripped.endswith(".ToList();") or stripped.endswith(".ToArray();") or \
            stripped.endswith(".ToList()") or stripped.endswith(".ToArray()")):
            return "HIGH"

    # HIGH: property getter body
    if line_num > 0:
        for i in range(max(0, line_num - 5), line_num):
            if i < len(lines) and ("get {" in lines[i] or "get" in lines[i]):
                stripped = line_content.strip()
                if stripped.endswith(".ToList());") or stripped.endswith(".ToArray());"):
                    return "HIGH"

    # HIGH: inside lock(...) block — check 10 preceding lines
    for i in range(max(0, line_num - 10), line_num):
        if i < len(lines) and "lock" in lines[i] and "(" in lines[i]:
            return "HIGH"

    # MED: consumed by foreach/for on the next line
    if line_num + 1 < len(lines):
        next_line = lines[line_num + 1].strip()
        if next_line.startswith("foreach") or next_line.startswith("for"):
            return "MED"

    # LOW: everything else
    return "LOW"


def scan_file(file_path: Path, root: Path) -> list[AllocationHit]:
    """Scan a single .cs file for .ToList()/.ToArray() calls."""
    try:
        content = file_path.read_text(encoding="utf-8")
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
        return []

    lines = content.split("\n")
    hits: list[AllocationHit] = []

    for line_num, line in enumerate(lines):
        match = ALLOCATION_TERMINAL_RE.search(line)
        if not match:
            continue

        method = match.group(1)  # ToList or ToArray
        rel_path = str(file_path.relative_to(root))

        # Check for inline allowlist comment
        has_inline_ok = "// allocation-ok:" in line

        severity = classify_severity(line_num, line, lines, str(file_path))

        # Skip if already allowlisted inline
        if has_inline_ok:
            continue

        allowlist_key = f"{severity}|{rel_path}|{line_num + 1}"

        hit = AllocationHit(
            severity=severity,
            file=rel_path,
            line_num=line_num + 1,
            method=method,
            line_content=line,
            context_lines=lines[max(0, line_num - 2):min(len(lines), line_num + 3)],
            allowlist_key=allowlist_key,
        )
        hits.append(hit)

    return hits


def load_allowlist(allowlist_path: Path) -> set[str]:
    """Load allowlist keys and file-level suppressions."""
    keys = set()
    if not allowlist_path.exists():
        return keys

    try:
        content = allowlist_path.read_text(encoding="utf-8")
        for line in content.split("\n"):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            keys.add(line)
    except Exception as e:
        print(f"Error reading allowlist {allowlist_path}: {e}", file=sys.stderr)

    return keys


def is_allowlisted(hit: AllocationHit, allowlist_keys: set[str]) -> bool:
    """Check if a hit is allowlisted."""
    # Check exact key match (severity|file|line)
    if hit.allowlist_key in allowlist_keys:
        return True

    # Check file-level suppression
    if hit.file in allowlist_keys:
        return True

    return False


def scan_tree(root: Path, exclude_tests: bool = True) -> list[AllocationHit]:
    """Scan src/Runtime/, src/Tools/, src/Bridge/, src/SDK/ for .cs files."""
    scan_dirs = [
        root / "src" / "Runtime",
        root / "src" / "Tools",
        root / "src" / "Bridge",
        root / "src" / "SDK",
    ]

    all_hits: list[AllocationHit] = []

    for scan_dir in scan_dirs:
        if not scan_dir.exists():
            continue

        for cs_file in scan_dir.rglob("*.cs"):
            # Exclude Tests, bin, obj
            if "Tests" in cs_file.parts or "bin" in cs_file.parts or "obj" in cs_file.parts:
                continue

            hits = scan_file(cs_file, root)
            all_hits.extend(hits)

    return all_hits


def run_self_test() -> bool:
    """Self-test: 3 positive + 3 negative cases."""
    print("[Self-test] Pattern #121 Unnecessary Allocation Detector")

    test_cases = [
        # HIGH: arrow expression
        ("Items => collection.ToList();", "HIGH", True),
        # HIGH: property getter + line content
        ("public List<T> Items => collection.ToList();", "HIGH", True),
        # HIGH: inside lock — use multi-line context
        (["lock(sync) {", "    var x = list.ToArray();"], 1, "HIGH", True),
        # MED: foreach next line
        ([".ToList();", "foreach(var x in y)"], 0, "MED", True),
        # LOW: plain assignment
        ("var result = query.ToList();", "LOW", True),
        # NEGATIVE: no allocation
        ("var items = GetItems();", None, False),
    ]

    passed = 0
    for test_case in test_cases:
        if len(test_case) == 3:
            # Simple case: (content, expected_severity, should_match)
            content, expected_severity, should_match = test_case
            lines = [content]
            line_num = 0
        else:
            # Multi-line case: (lines, line_num, expected_severity, should_match)
            lines, line_num, expected_severity, should_match = test_case
            content = lines[line_num]

        match = ALLOCATION_TERMINAL_RE.search(content)
        if should_match and not match:
            print(f"  FAIL: Expected match in '{content}'")
            continue
        if not should_match and match:
            print(f"  FAIL: Unexpected match in '{content}'")
            continue

        # For matched cases, verify severity classification (basic check)
        if should_match:
            severity = classify_severity(line_num, content, lines, "test.cs")
            if severity == expected_severity:
                print(f"  PASS: '{content[:40]}...' -> {severity}")
                passed += 1
            else:
                print(f"  FAIL: Expected {expected_severity}, got {severity}")
        else:
            print(f"  PASS: Correctly rejected '{content[:40]}...'")
            passed += 1

    success = passed == len(test_cases)
    print(f"\n[Self-test] Result: {passed}/{len(test_cases)} ({'PASS' if success else 'FAIL'})")
    return success


def main():
    parser = argparse.ArgumentParser(
        description="Pattern #121: Unnecessary LINQ Terminal Allocation detector"
    )
    parser.add_argument("--root", type=Path, default=Path.cwd(), help="Repo root")
    parser.add_argument("--allowlist", type=Path, help="Allowlist file path")
    parser.add_argument("--output", type=Path, help="JSON output file")
    parser.add_argument("--threshold", type=int, default=10, help="Fail if HIGH count > threshold")
    parser.add_argument("--strict", action="store_true", help="Fail on any unallowlisted HIGH")
    parser.add_argument("--quiet", action="store_true", help="Suppress console output")
    parser.add_argument("--verbose", action="store_true", help="Verbose output")
    parser.add_argument("--json", action="store_true", help="Output JSON to stdout")
    parser.add_argument("--test", action="store_true", help="Run self-test and exit")

    args = parser.parse_args()

    if args.test:
        success = run_self_test()
        sys.exit(0 if success else 1)

    root = args.root
    allowlist_path = args.allowlist or (root / "docs" / "qa" / "unnecessary-allocation-allowlist.txt")

    if not args.quiet and not args.json:
        print(f"[detect_unnecessary_allocation] Scanning {root}")

    # Scan
    hits = scan_tree(root)
    allowlist_keys = load_allowlist(allowlist_path)

    # Classify
    high_unallowlisted = []
    med_unallowlisted = []
    low_unallowlisted = []

    for hit in hits:
        if is_allowlisted(hit, allowlist_keys):
            continue

        if hit.severity == "HIGH":
            high_unallowlisted.append(hit)
        elif hit.severity == "MED":
            med_unallowlisted.append(hit)
        else:
            low_unallowlisted.append(hit)

    # Report
    if not args.quiet and not args.json:
        print(f"  HIGH:     {len(high_unallowlisted)}")
        print(f"  MED:      {len(med_unallowlisted)}")
        print(f"  LOW:      {len(low_unallowlisted)}")

    if args.json or args.verbose:
        report = {
            "timestamp": datetime.now().isoformat(),
            "root": str(root),
            "threshold": args.threshold,
            "high": [h.to_dict() for h in high_unallowlisted],
            "med": [h.to_dict() for h in med_unallowlisted],
            "low": [h.to_dict() for h in low_unallowlisted],
            "summary": {
                "high_count": len(high_unallowlisted),
                "med_count": len(med_unallowlisted),
                "low_count": len(low_unallowlisted),
            }
        }

        if args.json:
            print(json.dumps(report, indent=2))

        if args.output:
            args.output.write_text(json.dumps(report, indent=2))
            if not args.quiet:
                print(f"  Report written to {args.output}")

    # Exit code logic
    if args.strict and len(high_unallowlisted) > 0:
        if not args.quiet:
            print(f"\n[FAIL] --strict: {len(high_unallowlisted)} unallowlisted HIGH violations")
        sys.exit(1)

    if len(high_unallowlisted) > args.threshold:
        if not args.quiet:
            print(f"\n[FAIL] HIGH violations ({len(high_unallowlisted)}) exceed threshold ({args.threshold})")
        sys.exit(1)

    if not args.quiet:
        print("[PASS] Threshold check passed")
    sys.exit(0)


if __name__ == "__main__":
    main()
