#!/usr/bin/env python3
"""
Detector for Pattern #235: BepInEx plugin GraphicRaycaster without EventSystem guard.

Scans for GraphicRaycaster instantiation/addition without nearby EventSystem null checks.
Severity: HIGH if no guard within ±20 lines of the GraphicRaycaster site.
"""

import re
import sys
from pathlib import Path

ALLOWLIST_PATH = Path("docs/qa/pattern-235-graphicraycaster-eventsystem-allowlist.txt")

def load_allowlist():
    """Load allowlisted file paths (one per line)."""
    if not ALLOWLIST_PATH.exists():
        return set()
    return set(line.strip() for line in ALLOWLIST_PATH.read_text().splitlines() if line.strip() and not line.startswith("#"))

def scan_file(filepath):
    """
    Scan a C# file for GraphicRaycaster instantiation without EventSystem guard.
    Returns (high_count, violations).
    """
    content = filepath.read_text(encoding='utf-8', errors='ignore')
    lines = content.splitlines()

    violations = []
    high_count = 0

    # Pattern: AddComponent<GraphicRaycaster>() or new GraphicRaycaster()
    graphicraycaster_pattern = re.compile(r'(AddComponent<GraphicRaycaster>\s*\(\)|new\s+GraphicRaycaster\s*\()')

    # Pattern: EventSystem guard within context
    eventsystem_guard_pattern = re.compile(r'EventSystem\.(current\s*!=\s*null|current\s*==\s*null|\w+EventSystem)')

    for idx, line in enumerate(lines):
        if graphicraycaster_pattern.search(line):
            # Check ±20 lines for EventSystem guard
            start = max(0, idx - 20)
            end = min(len(lines), idx + 21)
            context = '\n'.join(lines[start:end])

            if not eventsystem_guard_pattern.search(context):
                high_count += 1
                violations.append({
                    'file': str(filepath),
                    'line': idx + 1,
                    'text': line.strip(),
                    'severity': 'HIGH'
                })

    return high_count, violations

def main():
    allowlist = load_allowlist()

    # Scan directories
    scan_dirs = [
        Path('src/Runtime/UI'),
        Path('src/Domains/UI'),
        Path('src/Runtime'),
    ]

    all_high = 0
    all_violations = []

    for scan_dir in scan_dirs:
        if not scan_dir.exists():
            continue
        for cs_file in scan_dir.rglob('*.cs'):
            # Skip if allowlisted
            if str(cs_file) in allowlist:
                continue

            high_count, violations = scan_file(cs_file)
            all_high += high_count
            all_violations.extend(violations)

    # Report
    if all_violations:
        print(f"\n=== Pattern #235: GraphicRaycaster without EventSystem Guard ===")
        print(f"Total HIGH violations: {all_high}\n")
        for v in all_violations:
            print(f"  {v['file']}:{v['line']}")
            print(f"    {v['text']}")
            print(f"    Severity: {v['severity']}\n")
    else:
        print("[OK] Pattern #235: No violations found (all GraphicRaycaster sites have EventSystem guards)")

    # Exit with error if HIGH > 0
    if all_high > 0:
        sys.exit(1)
    sys.exit(0)

if __name__ == '__main__':
    main()
