#!/usr/bin/env python3
"""
Pattern #104: Catch-Swallow-Default Erasure Detection

Detects catch blocks that swallow exceptions and return null/default values,
erasing the error state and making debugging impossible.

Patterns detected:
  - catch { return null; }
  - catch (Exception) { return default; }
  - catch (X ex) { return null; }
  - catch { return 0; } / return "";
  - catch { return default(T); }

Exit codes:
  - 0: HIGH count <= 50 (within baseline)
  - 1: HIGH count > 50 or violations found
"""

import re
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Set


REPO_ROOT = Path(__file__).parent.parent.parent
SRC_DIR = REPO_ROOT / "src"
ALLOWLIST_FILE = REPO_ROOT / "docs" / "qa" / "catch-swallow-default-allowlist.txt"


@dataclass
class Violation:
    file: Path
    line_num: int
    severity: str  # HIGH or LOW
    line_text: str
    reason: str = ""

    def __str__(self):
        return f"{self.file.relative_to(REPO_ROOT)}:{self.line_num} [{self.severity}]"

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


def classify_severity(context: str) -> str:
    """Classify severity based on return value."""
    # HIGH severity: returns null/default/0/empty string
    if re.search(r'\breturn\s+(null|default(?:\([^)]*\))?|0|""|\'\')\s*[;,]', context):
        return "HIGH"
    # LOW severity: returns non-default value or rethrow
    return "LOW"


def scan_file(file_path: Path, allowlist: AllowlistManager) -> List[Violation]:
    """Scan a C# file for catch-swallow-default patterns."""
    violations = []

    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"[WARN] Could not read {file_path}: {e}", file=sys.stderr)
        return violations

    i = 0
    while i < len(lines):
        line = lines[i]

        # Look for catch block opening
        if re.search(r'\bcatch\s*(\([^)]*\))?\s*\{', line):
            # Check if allowlisted
            if allowlist.is_allowed(file_path, i + 1):
                i += 1
                continue

            # Collect context: this line + next 10 lines
            context_start = i
            context_end = min(len(lines), i + 11)
            context = "".join(lines[context_start:context_end])

            # Check for safe-swallow annotation
            if i > 0 and "safe-swallow:" in lines[i - 1]:
                i += 1
                continue

            # Check for pure-cleanup patterns (Dispose, Close, etc.)
            if re.search(r'\.(Dispose|Close|Unsubscribe|Cancel|Cleanup)\s*\(\)', context):
                i += 1
                continue

            # Check for return default/null patterns
            if re.search(r'\breturn\s+(null|default(?:\([^)]*\))?|0|""|\'\')\s*[;,]', context):
                severity = "HIGH"
                violations.append(Violation(
                    file=file_path,
                    line_num=i + 1,
                    severity=severity,
                    line_text=line.strip()
                ))

        i += 1

    return violations


def scan_all_sources(allowlist: AllowlistManager) -> tuple:
    """Scan all C# files in src/."""
    all_violations = []
    severity_counts = {"HIGH": 0, "LOW": 0}

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
        print("[OK] No catch-swallow-default patterns found.")
        return 0

    print(f"\nPattern #104: Catch-Swallow-Default Erasure Detection\n")
    print(f"Summary:")
    print(f"  HIGH: {severity_counts['HIGH']}")
    print(f"  LOW:  {severity_counts['LOW']}")
    print(f"  TOTAL: {len(violations)}\n")

    if severity_counts['HIGH'] > 0:
        print("HIGH (returns null/default/0/''):")
        for v in sorted(violations, key=lambda x: (x.file, x.line_num))[:20]:
            if v.severity == "HIGH":
                print(f"  {v}")
                print(f"    > {v.line_text[:80]}")
        high_count = len([v for v in violations if v.severity == "HIGH"])
        if high_count > 20:
            print(f"  ... and {high_count - 20} more")
        print()

    HIGH_THRESHOLD = 50
    if severity_counts['HIGH'] > HIGH_THRESHOLD:
        print(f"[FAIL] HIGH count ({severity_counts['HIGH']}) exceeds threshold ({HIGH_THRESHOLD})")
        return 1

    print(f"[PASS] HIGH count ({severity_counts['HIGH']}) within threshold ({HIGH_THRESHOLD})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
