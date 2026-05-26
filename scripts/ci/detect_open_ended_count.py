#!/usr/bin/env python3
"""
Pattern #110: Open-Ended Count Assertion Detection

Detects brittle count assertions in test files that use lower-bound or
upper-bound assertions (>= or >) instead of exact-count assertions.

Patterns detected:
  - .Should().HaveCountGreaterThan(
  - .Should().HaveCountGreaterThanOrEqualTo(
  - .Count.Should().BeGreaterThan(
  - .Count.Should().BeGreaterThanOrEqualTo(
  - Assert.True(...Count >
  - .Should().NotBeEmpty() (when exact count is knowable)

Exit codes:
  - 0: HIGH count <= 50 (within baseline allowance)
  - 1: HIGH count > 50 or violations found
"""

import os
import re
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Set, Tuple

REPO_ROOT = Path(__file__).parent.parent.parent
TESTS_DIR = REPO_ROOT / "src" / "Tests"
ALLOWLIST_FILE = REPO_ROOT / "docs" / "qa" / "open_ended_count_allowlist.txt"

# Pattern configurations
PATTERNS = {
    "HaveCountGreaterThan": r"\.Should\(\)\.HaveCountGreaterThan\s*\(",
    "HaveCountGreaterThanOrEqualTo": r"\.Should\(\)\.HaveCountGreaterThanOrEqualTo\s*\(",
    "CountBeGreaterThan": r"\.Count\.Should\(\)\.BeGreaterThan\s*\(",
    "CountBeGreaterThanOrEqualTo": r"\.Count\.Should\(\)\.BeGreaterThanOrEqualTo\s*\(",
    "AssertTrue_Count_Gt": r"Assert\.True\s*\(\s*[^)]*\.Count\s*>\s*",
    "NotBeEmpty": r"\.Should\(\)\.NotBeEmpty\s*\(",
}

HIGH_SIGNAL_KEYWORDS = {
    "Loaded", "Has", "Returns", "Contains", "Exists", "Found",
    "Created", "Initialized", "Registered", "Resolved"
}

@dataclass
class Violation:
    file: Path
    line_num: int
    pattern: str
    severity: str  # HIGH or MED
    line_text: str
    reason: str = ""

    def __str__(self):
        return f"{self.file.relative_to(REPO_ROOT)}:{self.line_num} [{self.severity}] {self.pattern}"

    def allowlist_key(self):
        return f"{self.file.relative_to(REPO_ROOT)}:{self.line_num}"


class AllowlistManager:
    def __init__(self, allowlist_path: Path):
        self.allowlist_path = allowlist_path
        self.entries: Dict[str, str] = {}
        self.inline_okays: Set[int] = set()
        self._load()

    def _load(self):
        if not self.allowlist_path.exists():
            return

        with open(self.allowlist_path, 'r') as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith('#'):
                    continue
                # Format: <file>:<line> <reason>
                parts = line.split(maxsplit=1)
                if len(parts) >= 1:
                    self.entries[parts[0]] = parts[1] if len(parts) > 1 else ""

    def is_allowed(self, file_path: Path, line_num: int) -> bool:
        key = f"{file_path.relative_to(REPO_ROOT)}:{line_num}"
        return key in self.entries

    def get_reason(self, file_path: Path, line_num: int) -> str:
        key = f"{file_path.relative_to(REPO_ROOT)}:{line_num}"
        return self.entries.get(key, "")


class SeverityClassifier:
    @staticmethod
    def classify(method_name: str, pattern: str) -> str:
        """
        Classify violation severity based on method name and pattern.
        HIGH: method name contains keywords indicating exact counts should be known
        MED: generic test methods where count might legitimately be lower-bound
        """
        # NotBeEmpty is always MED unless method name strongly suggests HIGH
        if pattern == "NotBeEmpty":
            for keyword in HIGH_SIGNAL_KEYWORDS:
                if keyword.lower() in method_name.lower():
                    return "HIGH"
            return "MED"

        # Count assertions are HIGH if method name suggests determinism
        for keyword in HIGH_SIGNAL_KEYWORDS:
            if keyword.lower() in method_name.lower():
                return "HIGH"

        return "MED"


def extract_method_name(lines: List[str], line_idx: int) -> str:
    """Extract method name from surrounding context (look backwards)."""
    for i in range(line_idx, max(0, line_idx - 20), -1):
        match = re.search(r"public\s+(?:async\s+)?(?:\w+\s+)?(?:\w+\s+)*(\w+)\s*\(", lines[i])
        if match:
            return match.group(1)
    return "unknown"


def has_inline_comment_ok(lines: List[str], line_idx: int) -> bool:
    """Check if line has inline comment opt-out: // open-ended-count-ok"""
    if line_idx > 0:
        prev_line = lines[line_idx - 1]
        if "open-ended-count-ok" in prev_line or "open-ended-count-ok:" in prev_line:
            return True

    current_line = lines[line_idx]
    if "open-ended-count-ok" in current_line:
        return True

    return False


def scan_test_file(file_path: Path, allowlist: AllowlistManager) -> List[Violation]:
    """Scan a test file for open-ended count violations."""
    violations = []

    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"[WARN] Could not read {file_path}: {e}", file=sys.stderr)
        return violations

    method_name = ""

    for line_idx, line in enumerate(lines, 1):
        # Update current method name
        if re.search(r"public\s+(?:async\s+)?(?:\w+\s+)?(?:\w+\s+)*\w+\s*\(", line):
            method_name = extract_method_name(lines, line_idx - 1)

        # Skip if allowlisted
        if allowlist.is_allowed(file_path, line_idx):
            continue

        # Skip if inline-comment opt-out
        if has_inline_comment_ok(lines, line_idx - 1):
            continue

        # Check all patterns
        for pattern_name, pattern_re in PATTERNS.items():
            if re.search(pattern_re, line):
                severity = SeverityClassifier.classify(method_name, pattern_name)
                violations.append(Violation(
                    file=file_path,
                    line_num=line_idx,
                    pattern=pattern_name,
                    severity=severity,
                    line_text=line.strip()
                ))

    return violations


def scan_all_tests(allowlist: AllowlistManager) -> Tuple[List[Violation], Dict[str, int]]:
    """Scan all test files in src/Tests/."""
    all_violations = []
    severity_counts = {"HIGH": 0, "MED": 0}

    if not TESTS_DIR.exists():
        print(f"[WARN] Tests directory not found: {TESTS_DIR}", file=sys.stderr)
        return all_violations, severity_counts

    # Recursively find all .cs test files
    for cs_file in TESTS_DIR.rglob("*.cs"):
        # Skip non-test files (heuristic: end with Tests.cs or *Test.cs)
        if "Test" not in cs_file.name:
            continue

        violations = scan_test_file(cs_file, allowlist)
        all_violations.extend(violations)

        for v in violations:
            severity_counts[v.severity] += 1

    return all_violations, severity_counts


def report_violations(violations: List[Violation], severity_counts: Dict[str, int]):
    """Report violations grouped by severity."""
    if not violations:
        print("[OK] No open-ended count assertions found.")
        return 0

    print(f"\nPattern #110: Open-Ended Count Assertion Detection\n")
    print(f"Summary:")
    print(f"  HIGH: {severity_counts['HIGH']}")
    print(f"  MED:  {severity_counts['MED']}")
    print(f"  TOTAL: {len(violations)}\n")

    # Report HIGH violations
    if severity_counts['HIGH'] > 0:
        print("HIGH (should be exact-count):")
        high_violations = [v for v in violations if v.severity == "HIGH"]
        for v in sorted(high_violations, key=lambda x: (x.file, x.line_num))[:20]:
            print(f"  {v}")
            if len(v.line_text) > 100:
                print(f"    > {v.line_text[:100]}...")
            else:
                print(f"    > {v.line_text}")
        if len(high_violations) > 20:
            print(f"  ... and {len(high_violations) - 20} more")
        print()

    # Report MED violations (sample only)
    if severity_counts['MED'] > 0:
        print(f"MED (may be legitimate lower-bound):")
        med_violations = [v for v in violations if v.severity == "MED"]
        for v in sorted(med_violations, key=lambda x: (x.file, x.line_num))[:10]:
            print(f"  {v}")
        if len(med_violations) > 10:
            print(f"  ... and {len(med_violations) - 10} more")
        print()


def main():
    allowlist = AllowlistManager(ALLOWLIST_FILE)
    violations, severity_counts = scan_all_tests(allowlist)

    report_violations(violations, severity_counts)

    # Gate: fail if HIGH > 50
    HIGH_THRESHOLD = 50
    if severity_counts['HIGH'] > HIGH_THRESHOLD:
        print(f"[FAIL] HIGH count ({severity_counts['HIGH']}) exceeds threshold ({HIGH_THRESHOLD})")
        return 1

    print(f"[PASS] HIGH count ({severity_counts['HIGH']}) within threshold ({HIGH_THRESHOLD})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
