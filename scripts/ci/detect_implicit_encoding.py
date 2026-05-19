#!/usr/bin/env python3
"""
Pattern #106: Implicit File.ReadAllText Encoding Detection

Detects File.ReadAllText/ReadAllLines without explicit Encoding parameter,
which can cause encoding mismatches and data corruption.

Patterns detected:
  - File.ReadAllText(path) - missing Encoding
  - File.ReadAllLines(path) - missing Encoding
  - File.WriteAllText(path, content) - missing Encoding
  - File.WriteAllLines(path, lines) - missing Encoding

Exit codes:
  - 0: HIGH count <= 100 (read paths, encoding mismatch risk)
  - 1: HIGH count > 100 or violations found
"""

import re
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Tuple


REPO_ROOT = Path(__file__).parent.parent.parent
SRC_DIR = REPO_ROOT / "src"
ALLOWLIST_FILE = REPO_ROOT / "docs" / "qa" / "implicit-encoding-allowlist.txt"


@dataclass
class Violation:
    file: Path
    line_num: int
    severity: str  # HIGH or MED
    method: str
    line_text: str

    def __str__(self):
        return f"{self.file.relative_to(REPO_ROOT)}:{self.line_num} [{self.severity}] {self.method}"

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
    """Scan a C# file for implicit encoding patterns."""
    violations = []

    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"[WARN] Could not read {file_path}: {e}", file=sys.stderr)
        return violations

    for i, line in enumerate(lines):
        # Skip comments and strings (basic heuristic)
        if line.strip().startswith("//") or line.strip().startswith("/*"):
            continue

        # Check for File.ReadAllText(path) - no encoding
        match = re.search(r'File\.ReadAllText\s*\(\s*([^,)]+)\s*\)', line)
        if match:
            if allowlist.is_allowed(file_path, i + 1):
                continue
            violations.append(Violation(
                file=file_path,
                line_num=i + 1,
                severity="HIGH",
                method="File.ReadAllText",
                line_text=line.strip()
            ))
            continue

        # Check for File.ReadAllLines(path) - no encoding
        match = re.search(r'File\.ReadAllLines\s*\(\s*([^,)]+)\s*\)', line)
        if match:
            if allowlist.is_allowed(file_path, i + 1):
                continue
            violations.append(Violation(
                file=file_path,
                line_num=i + 1,
                severity="HIGH",
                method="File.ReadAllLines",
                line_text=line.strip()
            ))
            continue

        # Check for File.WriteAllText(path, content) - no encoding
        match = re.search(r'File\.WriteAllText\s*\(\s*([^,]+),\s*([^,)]+)\s*\)', line)
        if match:
            if allowlist.is_allowed(file_path, i + 1):
                continue
            violations.append(Violation(
                file=file_path,
                line_num=i + 1,
                severity="MED",
                method="File.WriteAllText",
                line_text=line.strip()
            ))
            continue

        # Check for File.WriteAllLines(path, lines) - no encoding
        match = re.search(r'File\.WriteAllLines\s*\(\s*([^,]+),\s*([^,)]+)\s*\)', line)
        if match:
            if allowlist.is_allowed(file_path, i + 1):
                continue
            violations.append(Violation(
                file=file_path,
                line_num=i + 1,
                severity="MED",
                method="File.WriteAllLines",
                line_text=line.strip()
            ))
            continue

    return violations


def scan_all_sources(allowlist: AllowlistManager) -> tuple:
    """Scan all C# files in src/."""
    all_violations = []
    severity_counts = {"HIGH": 0, "MED": 0}

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
        print("[OK] No implicit encoding patterns found.")
        return 0

    print(f"\nPattern #106: Implicit File Encoding Detection\n")
    print(f"Summary:")
    print(f"  HIGH: {severity_counts['HIGH']} (read operations, data corruption risk)")
    print(f"  MED:  {severity_counts['MED']} (write operations)")
    print(f"  TOTAL: {len(violations)}\n")

    if severity_counts['HIGH'] > 0:
        print("HIGH (File.ReadAllText/ReadAllLines without Encoding):")
        high_violations = [v for v in violations if v.severity == "HIGH"]
        for v in sorted(high_violations, key=lambda x: (x.file, x.line_num))[:20]:
            print(f"  {v}")
        if len(high_violations) > 20:
            print(f"  ... and {len(high_violations) - 20} more")
        print()

    HIGH_THRESHOLD = 100
    if severity_counts['HIGH'] > HIGH_THRESHOLD:
        print(f"[FAIL] HIGH count ({severity_counts['HIGH']}) exceeds threshold ({HIGH_THRESHOLD})")
        return 1

    print(f"[PASS] HIGH count ({severity_counts['HIGH']}) within threshold ({HIGH_THRESHOLD})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
