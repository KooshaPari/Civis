#!/usr/bin/env python3
"""
Pattern #105: Event-Subscription Lifecycle Asymmetry Detection

Detects event handler subscriptions without matching unsubscriptions,
which can cause memory leaks and prevent proper cleanup.

Patterns detected:
  - event += handler (subscription) without matching event -= handler (unsubscription)
  - In classes with Dispose/OnDestroy, missing -= is HIGH severity
  - In other classes with no cleanup hook, missing -= is MED severity

Exit codes:
  - 0: HIGH count <= 30 (within baseline)
  - 1: HIGH count > 30 or violations found
"""

import re
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Set, Tuple


REPO_ROOT = Path(__file__).parent.parent.parent
SRC_DIR = REPO_ROOT / "src"
ALLOWLIST_FILE = REPO_ROOT / "docs" / "qa" / "event-lifecycle-allowlist.txt"


@dataclass
class Violation:
    file: Path
    line_num: int
    severity: str  # HIGH or MED
    event_name: str
    line_text: str

    def __str__(self):
        return f"{self.file.relative_to(REPO_ROOT)}:{self.line_num} [{self.severity}] {self.event_name}"

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


def has_cleanup_hook(lines: List[str], class_start: int) -> bool:
    """Check if class has Dispose, OnDestroy, or similar cleanup method."""
    cleanup_patterns = [
        r'\bpublic\s+void\s+Dispose\s*\(',
        r'\bprivate\s+void\s+Dispose\s*\(',
        r'\bpublic\s+void\s+OnDestroy\s*\(',
        r'\bprivate\s+void\s+OnDestroy\s*\(',
        r'\bpublic\s+void\s+OnDisable\s*\(',
        r'\bprivate\s+void\s+OnDisable\s*\(',
        r'\bpublic\s+void\s+Close\s*\(',
        r'\bprivate\s+void\s+Close\s*\(',
    ]

    for i in range(class_start, min(len(lines), class_start + 300)):
        for pattern in cleanup_patterns:
            if re.search(pattern, lines[i]):
                return True
        # Stop at next class definition
        if i > class_start and re.search(r'^\s*(?:public|private|internal|protected)\s+(?:class|struct|interface)', lines[i]):
            break

    return False


def extract_class_name(lines: List[str], line_idx: int) -> str:
    """Extract class name from surrounding context."""
    for i in range(line_idx, max(0, line_idx - 30), -1):
        match = re.search(r'(?:public|private|internal|protected)\s+(?:class|struct)\s+(\w+)', lines[i])
        if match:
            return match.group(1)
    return "unknown"


def scan_file(file_path: Path, allowlist: AllowlistManager) -> List[Violation]:
    """Scan a C# file for event subscription/unsubscription asymmetry."""
    violations = []

    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"[WARN] Could not read {file_path}: {e}", file=sys.stderr)
        return violations

    # Find all subscriptions (+=) and unsubscriptions (-=)
    subscriptions: Dict[str, List[int]] = {}  # event_name -> [line_nums]
    unsubscriptions: Dict[str, List[int]] = {}

    for i, line in enumerate(lines):
        # Match event += handler
        match_sub = re.search(r'(\w+)\s*\+=', line)
        if match_sub:
            event_name = match_sub.group(1)
            if event_name not in subscriptions:
                subscriptions[event_name] = []
            subscriptions[event_name].append(i)

        # Match event -= handler
        match_unsub = re.search(r'(\w+)\s*-=', line)
        if match_unsub:
            event_name = match_unsub.group(1)
            if event_name not in unsubscriptions:
                unsubscriptions[event_name] = []
            unsubscriptions[event_name].append(i)

    # Find classes and check for cleanup hooks
    class_regions: List[Tuple[int, int, str]] = []  # (start, end, class_name)
    i = 0
    while i < len(lines):
        match = re.search(r'(?:public|private|internal|protected)\s+(?:class|struct)\s+(\w+)', lines[i])
        if match:
            class_name = match.group(1)
            class_start = i
            class_end = len(lines)

            # Find end of class (next class or end of file)
            brace_count = 0
            found_opening = False
            for j in range(i, len(lines)):
                brace_count += lines[j].count('{') - lines[j].count('}')
                if '{' in lines[j]:
                    found_opening = True
                if found_opening and brace_count == 0:
                    class_end = j
                    break

            has_cleanup = has_cleanup_hook(lines, class_start)
            class_regions.append((class_start, class_end, class_name))

        i += 1

    # For each subscription without unsubscription, report violation
    for event_name, sub_lines in subscriptions.items():
        unsub_lines = unsubscriptions.get(event_name, [])

        # Simple heuristic: if same count of += and -=, assume balanced
        if len(sub_lines) != len(unsub_lines):
            # Find which class this subscription belongs to
            for sub_line in sub_lines:
                if sub_line not in [u for unsubs in unsubscriptions.values() for u in unsubs]:
                    # This subscription has no matching unsubscription
                    if allowlist.is_allowed(file_path, sub_line + 1):
                        continue

                    # Determine severity based on cleanup hook in enclosing class
                    severity = "MED"
                    for class_start, class_end, class_name in class_regions:
                        if class_start <= sub_line <= class_end:
                            if has_cleanup_hook(lines, class_start):
                                severity = "HIGH"
                            break

                    violations.append(Violation(
                        file=file_path,
                        line_num=sub_line + 1,
                        severity=severity,
                        event_name=event_name,
                        line_text=lines[sub_line].strip()
                    ))

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
        print("[OK] No event lifecycle asymmetry found.")
        return 0

    print(f"\nPattern #105: Event-Subscription Lifecycle Asymmetry Detection\n")
    print(f"Summary:")
    print(f"  HIGH: {severity_counts['HIGH']}")
    print(f"  MED:  {severity_counts['MED']}")
    print(f"  TOTAL: {len(violations)}\n")

    if severity_counts['HIGH'] > 0:
        print("HIGH (class has Dispose/OnDestroy but no -= found):")
        for v in sorted(violations, key=lambda x: (x.file, x.line_num))[:15]:
            if v.severity == "HIGH":
                print(f"  {v}")
        high_count = len([v for v in violations if v.severity == "HIGH"])
        if high_count > 15:
            print(f"  ... and {high_count - 15} more")
        print()

    HIGH_THRESHOLD = 30
    if severity_counts['HIGH'] > HIGH_THRESHOLD:
        print(f"[FAIL] HIGH count ({severity_counts['HIGH']}) exceeds threshold ({HIGH_THRESHOLD})")
        return 1

    print(f"[PASS] HIGH count ({severity_counts['HIGH']}) within threshold ({HIGH_THRESHOLD})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
