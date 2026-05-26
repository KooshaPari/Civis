#!/usr/bin/env python3
"""
Pattern #108: Sleep-Based Test Sync Detection

Detects Thread.Sleep() and Task.Delay() calls in test methods,
which are brittle, unreliable, and should use proper async/await patterns.

Patterns detected:
  - await Task.Delay(N) in test methods
  - Thread.Sleep(N) in test methods
  - Thread.Sleep in test setup/teardown

Severity:
  - HIGH: inside [Fact] or [Theory] test method
  - LOW: in test helper/fixture methods

Exit codes:
  - 0: HIGH count <= 20 (within baseline for brittle tests)
  - 1: HIGH count > 20 or violations found
"""

import re
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Optional


REPO_ROOT = Path(__file__).parent.parent.parent
TESTS_DIR = REPO_ROOT / "src" / "Tests"
ALLOWLIST_FILE = REPO_ROOT / "docs" / "qa" / "test-sleep-sync-allowlist.txt"


@dataclass
class Violation:
    file: Path
    line_num: int
    severity: str  # HIGH or LOW
    pattern: str
    method_name: Optional[str]
    line_text: str

    def __str__(self):
        method = f" in {self.method_name}" if self.method_name else ""
        return f"{self.file.relative_to(REPO_ROOT)}:{self.line_num} [{self.severity}] {self.pattern}{method}"

    def allowlist_key(self):
        return f"{self.file.relative_to(REPO_ROOT)}:{self.line_num}"


class AllowlistManager:
    def __init__(self, allowlist_path: Path):
        self.allowlist_path = allowlist_path
        self.entries: Dict[str, str] = {}
        self._load()

    def _load(self):
        if not self.allowlist_path.exists():
            return

        with open(self.allowlist_path, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith('#'):
                    continue
                parts = line.split(maxsplit=1)
                if len(parts) >= 1:
                    self.entries[parts[0]] = parts[1] if len(parts) > 1 else ""

    def is_allowed(self, file_path: Path, line_num: int) -> bool:
        key = f"{file_path.relative_to(REPO_ROOT)}:{line_num}"
        return key in self.entries


def extract_method_name(lines: List[str], line_idx: int) -> Optional[str]:
    """Extract method name by searching backwards from line_idx."""
    for i in range(line_idx, max(0, line_idx - 30), -1):
        # Match [Fact] or [Theory] decoration
        if re.search(r'^\s*\[\s*(?:Fact|Theory)\s*\]', lines[i]):
            # Now look for method signature
            for j in range(i + 1, min(len(lines), i + 5)):
                match = re.search(r'(?:public|private)\s+(?:async\s+)?(?:\w+\s+)?(\w+)\s*\(', lines[j])
                if match:
                    return match.group(1)
            break

    return None


def is_test_method(lines: List[str], line_idx: int) -> bool:
    """Check if line is inside a test method (decorated with [Fact] or [Theory])."""
    # Look backwards for test attribute
    for i in range(line_idx, max(0, line_idx - 30), -1):
        if re.search(r'^\s*\[\s*(?:Fact|Theory)\s*\]', lines[i]):
            return True
        # Stop at previous method definition
        if i < line_idx and re.search(r'(?:public|private)\s+(?:async\s+)?(?:\w+\s+)?(\w+)\s*\(', lines[i]):
            break

    return False


def scan_file(file_path: Path, allowlist: AllowlistManager) -> List[Violation]:
    """Scan a test file for sleep patterns."""
    violations = []

    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"[WARN] Could not read {file_path}: {e}", file=sys.stderr)
        return violations

    for i, line in enumerate(lines):
        # Skip comments
        if line.strip().startswith("//"):
            continue

        # Check for await Task.Delay(
        if re.search(r'await\s+Task\.Delay\s*\(', line):
            if allowlist.is_allowed(file_path, i + 1):
                continue

            method_name = extract_method_name(lines, i)
            in_test = is_test_method(lines, i)
            severity = "HIGH" if in_test else "LOW"

            violations.append(Violation(
                file=file_path,
                line_num=i + 1,
                severity=severity,
                pattern="await Task.Delay()",
                method_name=method_name,
                line_text=line.strip()
            ))
            continue

        # Check for Thread.Sleep(
        if re.search(r'Thread\.Sleep\s*\(', line):
            if allowlist.is_allowed(file_path, i + 1):
                continue

            method_name = extract_method_name(lines, i)
            in_test = is_test_method(lines, i)
            severity = "HIGH" if in_test else "LOW"

            violations.append(Violation(
                file=file_path,
                line_num=i + 1,
                severity=severity,
                pattern="Thread.Sleep()",
                method_name=method_name,
                line_text=line.strip()
            ))
            continue

    return violations


def scan_all_tests(allowlist: AllowlistManager) -> tuple:
    """Scan all test files in src/Tests/."""
    all_violations = []
    severity_counts = {"HIGH": 0, "LOW": 0}

    if not TESTS_DIR.exists():
        print(f"[WARN] Tests directory not found: {TESTS_DIR}", file=sys.stderr)
        return all_violations, severity_counts

    for cs_file in TESTS_DIR.rglob("*.cs"):
        # Skip non-test files
        if "Test" not in cs_file.name:
            continue

        violations = scan_file(cs_file, allowlist)
        all_violations.extend(violations)

        for v in violations:
            severity_counts[v.severity] += 1

    return all_violations, severity_counts


def main():
    """Main entry point."""
    allowlist = AllowlistManager(ALLOWLIST_FILE)
    violations, severity_counts = scan_all_tests(allowlist)

    if not violations:
        print("[OK] No sleep-based test sync found.")
        return 0

    print(f"\nPattern #108: Sleep-Based Test Sync Detection\n")
    print(f"Summary:")
    print(f"  HIGH: {severity_counts['HIGH']} (inside test methods)")
    print(f"  LOW:  {severity_counts['LOW']} (in helpers)")
    print(f"  TOTAL: {len(violations)}\n")

    if severity_counts['HIGH'] > 0:
        print("HIGH (in test methods - replace with proper async/await):")
        for v in sorted(violations, key=lambda x: (x.file, x.line_num)):
            if v.severity == "HIGH":
                print(f"  {v}")
        print()

    HIGH_THRESHOLD = 20
    if severity_counts['HIGH'] > HIGH_THRESHOLD:
        print(f"[FAIL] HIGH count ({severity_counts['HIGH']}) exceeds threshold ({HIGH_THRESHOLD})")
        return 1

    print(f"[PASS] HIGH count ({severity_counts['HIGH']}) within threshold ({HIGH_THRESHOLD})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
