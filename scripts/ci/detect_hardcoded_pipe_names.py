#!/usr/bin/env python3
"""
Pattern #441: Hardcoded Pipe Name Detection

Detects hardcoded string literals in PipeName assignments in test fixtures,
which violate test isolation policy and can cause pipe-name collisions.
Each test should use GUID-randomized pipe names for isolation.

Patterns detected:
  - PipeName = "literal-string" (string literal, no Guid, no interpolation)
  - Severity: HIGH (test isolation policy violation)

Inline marker (allowed): // hardcoded-pipe-name-ok: <reason>

Exit codes:
  - 0: violations <= threshold (default 2)
  - 1: violations > threshold or --test fails
"""

import re
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Optional


REPO_ROOT = Path(__file__).parent.parent.parent
TESTS_DIR = REPO_ROOT / "src" / "Tests"
ALLOWLIST_FILE = REPO_ROOT / "docs" / "qa" / "hardcoded-pipe-names-allowlist.txt"


@dataclass
class Violation:
    file: Path
    line_num: int
    severity: str  # HIGH
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


def has_inline_allowlist_marker(line: str) -> bool:
    """Check if line has // hardcoded-pipe-name-ok marker."""
    return '// hardcoded-pipe-name-ok' in line


def is_hardcoded_pipe(line: str) -> bool:
    """Check if line contains a hardcoded PipeName assignment."""
    # Skip if it's a comment
    if line.strip().startswith('//'):
        return False

    # Skip if it has inline marker
    if has_inline_allowlist_marker(line):
        return False

    # Skip if it contains Guid.NewGuid() or interpolation
    if 'Guid.NewGuid()' in line or '${' in line or '$"' in line:
        return False

    # Check if it matches PipeName = "..."
    if re.search(r'PipeName\s*=\s*"[^"]+"', line):
        return True

    return False


def scan_file(file_path: Path, allowlist: AllowlistManager) -> List[Violation]:
    """Scan a test file for hardcoded pipe names."""
    violations = []

    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"[WARN] Could not read {file_path}: {e}", file=sys.stderr)
        return violations

    for i, line in enumerate(lines):
        if is_hardcoded_pipe(line):
            # Reject if already in allowlist
            if allowlist.is_allowed(file_path, i + 1):
                continue

            # Extract the quoted string
            match = re.search(r'PipeName\s*=\s*"([^"]+)"', line)
            if match:
                pipe_name = match.group(1)
                method_name = extract_method_name(lines, i)

                violations.append(Violation(
                    file=file_path,
                    line_num=i + 1,
                    severity="HIGH",
                    pattern=f"PipeName = \"{pipe_name}\"",
                    method_name=method_name,
                    line_text=line.strip()
                ))

    return violations


def scan_all_tests(allowlist: AllowlistManager) -> tuple:
    """Scan all test files in src/Tests/."""
    all_violations = []
    severity_counts = {"HIGH": 0}

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


def self_test():
    """Run self-tests on known patterns."""
    test_cases = [
        # Positive fixtures (should detect)
        ('PipeName = "hardcoded"', True, 'simple literal'),
        ('PipeName = "test-pipe-123"', True, 'hyphenated literal'),
        ('PipeName = "pipe"', True, 'single word'),
        # Negative fixtures (should NOT detect)
        ('PipeName = Guid.NewGuid().ToString()', False, 'Guid.NewGuid()'),
        ('PipeName = $"pipe-{Guid.NewGuid()}"', False, 'interpolated Guid'),
        ('// PipeName = "commented-out"', False, 'comment'),
    ]

    passed = 0
    failed = 0

    for code, should_detect, desc in test_cases:
        result = is_hardcoded_pipe(code)
        if result == should_detect:
            print(f"  [PASS] {desc}: '{code}'")
            passed += 1
        else:
            print(f"  [FAIL] {desc}: '{code}' (expected {should_detect}, got {result})")
            failed += 1

    return passed, failed


def main():
    """Main entry point."""
    if len(sys.argv) > 1:
        if sys.argv[1] == '--test':
            print("Running self-tests...\n")
            passed, failed = self_test()
            print(f"\nSelf-test: {passed} passed, {failed} failed")
            return 0 if failed == 0 else 1

    allowlist = AllowlistManager(ALLOWLIST_FILE)
    violations, severity_counts = scan_all_tests(allowlist)

    if not violations:
        print("[OK] No hardcoded pipe names found.")
        return 0

    print(f"\nPattern #441: Hardcoded Pipe Name Detection\n")
    print(f"Summary:")
    print(f"  HIGH: {severity_counts['HIGH']} (test isolation violations)\n")

    print("HIGH (hardcoded pipe names - use Guid.NewGuid().ToString()):")
    for v in sorted(violations, key=lambda x: (x.file, x.line_num)):
        print(f"  {v}")
    print()

    THRESHOLD = 2
    if severity_counts['HIGH'] > THRESHOLD:
        print(f"[FAIL] HIGH count ({severity_counts['HIGH']}) exceeds threshold ({THRESHOLD})")
        return 1

    print(f"[PASS] HIGH count ({severity_counts['HIGH']}) within threshold ({THRESHOLD})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
