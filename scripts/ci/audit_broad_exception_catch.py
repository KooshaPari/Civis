#!/usr/bin/env python3
"""
Pattern #230 audit: Broad catch (Exception) without 'when' filter.
Detects catch blocks that capture Exception without exception-type filtering.
"""

import re
import os
from pathlib import Path
from collections import defaultdict

def should_skip(file_path):
    """Skip bin, obj, Generated, and Test files."""
    return any(part in file_path for part in ['\\bin\\', '\\obj\\', '\\Generated', '\\Tests\\'])

def get_severity(file_path):
    """Determine severity based on location."""
    if any(part in file_path for part in ['\\Runtime\\', '\\SDK\\', '\\Bridge\\', '\\Domains\\']):
        return 'HIGH'
    elif '\\Tools\\' in file_path:
        return 'MED'
    else:
        return 'LOW'

def is_exempt_context(lines, line_idx):
    """Check if catch has exemption marker or is in Main method."""
    # Check for inline exemption marker
    if '// broad-catch-ok:' in lines[line_idx]:
        return True

    # Check if in Main method (heuristic: scan backward for 'Main' in method signature)
    for i in range(max(0, line_idx - 10), line_idx):
        if 'static void Main' in lines[i] or 'static async Task Main' in lines[i]:
            return True

    return False

def audit_file(file_path):
    """Audit a single C# file for Pattern #230 violations."""
    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception:
        return []

    violations = []
    # Pattern: catch (Exception ...) without 'when' on same/next line
    pattern = re.compile(r'catch\s*\(\s*Exception\b[^)]*\)\s*(?:\{|when)', re.IGNORECASE)

    for idx, line in enumerate(lines):
        if pattern.search(line):
            # Check if it has a 'when' filter (indicates safe conditional catch)
            if 'when' in line:
                continue  # Has filter, skip

            # Check exemption
            if is_exempt_context(lines, idx):
                continue

            severity = get_severity(file_path)
            violations.append({
                'file': file_path,
                'line': idx + 1,
                'severity': severity,
                'text': line.strip()
            })

    return violations

def main():
    src_root = Path('C:\\Users\\koosh\\Dino\\src')
    all_violations = []
    severity_counts = defaultdict(int)

    # Walk all C# files
    for cs_file in src_root.rglob('*.cs'):
        file_path = str(cs_file)
        if should_skip(file_path):
            continue

        violations = audit_file(file_path)
        all_violations.extend(violations)
        for v in violations:
            severity_counts[v['severity']] += 1

    # Sort by severity (HIGH first), then file, then line
    severity_order = {'HIGH': 0, 'MED': 1, 'LOW': 2}
    all_violations.sort(
        key=lambda v: (severity_order[v['severity']], v['file'], v['line'])
    )

    # Generate report
    total = len(all_violations)
    high = severity_counts['HIGH']
    med = severity_counts['MED']
    low = severity_counts['LOW']

    # Tier classification
    if total < 50:
        tier = "LOW"
    elif total < 200:
        tier = "MODERATE"
    else:
        tier = "ENDEMIC"

    report = []
    report.append("# Pattern #230 Audit: Broad Exception Catch Without Filter\n")
    report.append(f"**Total violations**: {total} (HIGH: {high}, MED: {med}, LOW: {low})\n")
    report.append(f"**Tier**: {tier}\n\n")

    report.append("## Top 15 Violations\n")
    report.append("| File | Line | Severity |\n")
    report.append("|------|------|----------|\n")
    for v in all_violations[:15]:
        relpath = v['file'].replace('C:\\Users\\koosh\\Dino\\', '')
        report.append(f"| `{relpath}` | {v['line']} | {v['severity']} |\n")

    if len(all_violations) > 15:
        report.append(f"\n... and {len(all_violations) - 15} more violations.\n")

    # Write report
    report_path = Path('C:\\Users\\koosh\\Dino\\docs\\qa\\pattern_230_audit.md')
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(''.join(report))

    print(f"[OK] Audit complete. {total} violations found.")
    print(f"  HIGH: {high}, MED: {med}, LOW: {low}")
    print(f"  Tier: {tier}")
    print(f"  Report: {report_path}")

if __name__ == '__main__':
    main()
