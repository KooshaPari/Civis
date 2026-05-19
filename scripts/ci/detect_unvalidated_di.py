#!/usr/bin/env python3
"""
Pattern #107: BuildServiceProvider without ValidateOnBuild Detection

Detects IServiceCollection.BuildServiceProvider() calls without
ServiceProviderOptions { ValidateOnBuild = true }, which can hide
DI container errors at runtime.

Patterns detected:
  - .BuildServiceProvider() - no options (missing validation)
  - .BuildServiceProvider(new ServiceProviderOptions()) - default ctor (missing ValidateOnBuild=true)

Exit codes:
  - 0: No violations found
  - 1: Any HIGH violations found (every site is a real risk)
"""

import re
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List


REPO_ROOT = Path(__file__).parent.parent.parent
SRC_DIR = REPO_ROOT / "src"
ALLOWLIST_FILE = REPO_ROOT / "docs" / "qa" / "di-validation-allowlist.txt"


@dataclass
class Violation:
    file: Path
    line_num: int
    severity: str  # Always HIGH
    pattern: str
    line_text: str

    def __str__(self):
        return f"{self.file.relative_to(REPO_ROOT)}:{self.line_num} [{self.severity}] {self.pattern}"

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


def scan_file(file_path: Path, allowlist: AllowlistManager) -> List[Violation]:
    """Scan a C# file for unvalidated BuildServiceProvider calls."""
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

        # Check for .BuildServiceProvider() with no args
        if re.search(r'\.BuildServiceProvider\s*\(\s*\)', line):
            if allowlist.is_allowed(file_path, i + 1):
                continue
            violations.append(Violation(
                file=file_path,
                line_num=i + 1,
                severity="HIGH",
                pattern="BuildServiceProvider() no args",
                line_text=line.strip()
            ))
            continue

        # Check for .BuildServiceProvider(new ServiceProviderOptions())
        # This is a violation because default ctor doesn't validate
        if re.search(r'\.BuildServiceProvider\s*\(\s*new\s+ServiceProviderOptions\s*\(\s*\)', line):
            if allowlist.is_allowed(file_path, i + 1):
                continue
            violations.append(Violation(
                file=file_path,
                line_num=i + 1,
                severity="HIGH",
                pattern="BuildServiceProvider(new ServiceProviderOptions())",
                line_text=line.strip()
            ))
            continue

    return violations


def scan_all_sources(allowlist: AllowlistManager) -> tuple:
    """Scan all C# files in src/."""
    all_violations = []
    severity_counts = {"HIGH": 0}

    if not SRC_DIR.exists():
        print(f"[WARN] Source directory not found: {SRC_DIR}", file=sys.stderr)
        return all_violations, severity_counts

    for cs_file in SRC_DIR.rglob("*.cs"):
        # Skip obj and generated files
        if "\\obj\\" in str(cs_file) or ".g.cs" in str(cs_file):
            continue

        violations = scan_file(cs_file, allowlist)
        all_violations.extend(violations)

        for v in violations:
            severity_counts[v.severity] += 1

    return all_violations, severity_counts


def main():
    """Main entry point."""
    allowlist = AllowlistManager(ALLOWLIST_FILE)
    violations, severity_counts = scan_all_sources(allowlist)

    if not violations:
        print("[OK] All BuildServiceProvider calls are properly validated.")
        return 0

    print(f"\nPattern #107: BuildServiceProvider without ValidateOnBuild Detection\n")
    print(f"Summary:")
    print(f"  HIGH: {severity_counts['HIGH']}\n")

    print("HIGH (every site is a real DI validation risk):")
    for v in sorted(violations, key=lambda x: (x.file, x.line_num)):
        print(f"  {v}")
        print(f"    > {v.line_text[:80]}")
    print()

    if severity_counts['HIGH'] > 0:
        print(f"[FAIL] Found {severity_counts['HIGH']} unvalidated DI sites")
        return 1

    print(f"[PASS] All DI sites properly validated")
    return 0


if __name__ == "__main__":
    sys.exit(main())
