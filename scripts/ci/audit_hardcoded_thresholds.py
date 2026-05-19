#!/usr/bin/env python3
"""
Pattern #221 — Hardcoded Numeric Thresholds (Audit-Only)

Scans src/ for hardcoded numeric literal comparisons, assignments, sleep calls,
and timeout values that should be named constants or injected parameters.
Excludes test fixtures, generated code, and lines marked with exemption markers.

Usage:
    python audit_hardcoded_thresholds.py

Exit code: Always 0 (audit-only, no CI gate yet)
Output: CSV to stdout + summary
"""

import re
import sys
from pathlib import Path
from collections import defaultdict


def should_exclude_path(path_str):
    """Return True if path should be excluded from scan."""
    exclude_patterns = ['bin/', 'obj/', 'Tests', 'Generated']
    return any(pattern in path_str for pattern in exclude_patterns)


def should_exclude_line(line):
    """Return True if line should be excluded from violations."""
    # Lines with const/readonly/static readonly declarations
    if re.search(r'\b(const|readonly|static\s+readonly)\b', line):
        return True
    # Attribute lines (test decorators)
    if re.match(r'^\s*\[(?:InlineData|Theory|MemberData|Property|Fact)\b', line):
        return True
    # Exemption markers
    if '// const-ok' in line or '// threshold-ok' in line:
        return True
    return False


def categorize_violation(literal, context):
    """Categorize violation by type."""
    if 'Sleep' in context or 'Delay' in context:
        return 'sleep'
    elif 'timeout' in context.lower() or 'Timeout' in context:
        return 'timeout'
    elif re.search(r'[<>]=?', context):
        return 'comparison'
    else:
        return 'size'


def scan_file(filepath):
    """Scan a single .cs file for violations. Returns list of (line_num, literal, context, category)."""
    violations = []
    try:
        with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
            for line_num, line in enumerate(f, start=1):
                if should_exclude_line(line):
                    continue

                # Pattern: comparison operators with >=2 digits
                for match in re.finditer(r'([><=]+)\s*(\d{2,})\b', line):
                    literal = match.group(2)
                    context = line.strip()
                    category = categorize_violation(literal, context)
                    violations.append((line_num, literal, context, category))

                # Pattern: Sleep/Delay with digits
                for match in re.finditer(r'\.Sleep\(\s*(\d+)\s*\)', line):
                    literal = match.group(1)
                    context = line.strip()
                    violations.append((line_num, literal, context, 'sleep'))

                # Pattern: named timeouts
                for match in re.finditer(r'\b\w*[Tt]imeout\b.*?:\s*(\d{3,})\b', line):
                    literal = match.group(1)
                    context = line.strip()
                    violations.append((line_num, literal, context, 'timeout'))

                # Pattern: assignment of 4+ digit literals
                for match in re.finditer(r'=\s*(\d{4,})\b', line):
                    literal = match.group(1)
                    context = line.strip()
                    violations.append((line_num, literal, context, 'size'))

    except Exception as e:
        print(f"Warning: {filepath} — {e}", file=sys.stderr)

    return violations


def main():
    src_dir = Path('src')
    if not src_dir.exists():
        print("Error: src/ directory not found", file=sys.stderr)
        sys.exit(0)

    # Collect all violations
    all_violations = []  # (filepath, line_num, literal, context, category)
    category_counts = defaultdict(int)
    file_counts = defaultdict(int)

    for cs_file in src_dir.rglob('*.cs'):
        rel_path = str(cs_file.relative_to(src_dir))
        if should_exclude_path(rel_path):
            continue

        violations = scan_file(cs_file)
        for line_num, literal, context, category in violations:
            all_violations.append((rel_path, line_num, literal, context, category))
            category_counts[category] += 1
            file_counts[rel_path] += 1

    # CSV header
    print("file,line,literal,context,category")

    # CSV rows (sorted by file, then line)
    all_violations.sort(key=lambda x: (x[0], x[1]))
    for file_path, line_num, literal, context, category in all_violations:
        # Escape context for CSV
        context_escaped = context.replace('"', '""')
        print(f'{file_path},{line_num},{literal},"{context_escaped}",{category}')

    # Summary
    total = len(all_violations)
    print(f"\n## Summary", file=sys.stderr)
    print(f"Total violations: {total}", file=sys.stderr)
    print(f"\nBy category:", file=sys.stderr)
    for category in sorted(category_counts.keys()):
        count = category_counts[category]
        print(f"  {category}: {count}", file=sys.stderr)

    # Top 30 offenders by file
    print(f"\nTop 30 offenders (by file):", file=sys.stderr)
    top_30 = sorted(file_counts.items(), key=lambda x: -x[1])[:30]
    for file_path, count in top_30:
        print(f"  {file_path}: {count}", file=sys.stderr)

    sys.exit(0)


if __name__ == '__main__':
    main()
