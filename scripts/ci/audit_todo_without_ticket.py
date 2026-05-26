#!/usr/bin/env python3
"""
Pattern #223 Audit: TODO/FIXME/HACK comments without ticket reference.

Scans src/ for untracked TODO/FIXME/HACK/XXX/NOTE comments.
Reports on violations and categorizes by severity.
"""

import os
import re
import sys
from pathlib import Path
from collections import defaultdict

# Patterns to match: TODO, FIXME, HACK, XXX, NOTE (case-insensitive)
TODO_PATTERN = re.compile(
    r'(//\s*(?P<kind>TODO|FIXME|HACK|XXX|NOTE)\s*:?\s*(?P<text>.+?)(?=\s*$|\s*//|$))',
    re.IGNORECASE
)

# Patterns that qualify as "referenced"
TICKET_PATTERN = re.compile(r'#\d+')  # #123
GITHUB_URL_PATTERN = re.compile(r'github\.com.*(?:/issues/|/pull/)')
OWNER_PATTERN = re.compile(r'@\w+|(\(.*?koosha.*?\))')
DATE_PATTERN = re.compile(r'202\d-\d{2}-\d{2}|202\d-')

def is_referenced(comment_line):
    """Check if a comment line contains a ticket reference, GitHub URL, owner, or date."""
    return (
        TICKET_PATTERN.search(comment_line) or
        GITHUB_URL_PATTERN.search(comment_line) or
        OWNER_PATTERN.search(comment_line) or
        DATE_PATTERN.search(comment_line)
    )

def audit_file(filepath):
    """Audit a single file for unreferenced TODOs."""
    violations = []
    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            for line_num, line in enumerate(f, 1):
                match = TODO_PATTERN.search(line)
                if match:
                    kind = match.group('kind').upper()
                    text = match.group('text').strip()[:60]
                    if not is_referenced(line):
                        violations.append({
                            'file': filepath,
                            'line': line_num,
                            'kind': kind,
                            'text': text,
                        })
    except Exception as e:
        print(f"ERROR reading {filepath}: {e}", file=sys.stderr)
    return violations

def main():
    """Main audit runner."""
    root = Path('src')
    if not root.exists():
        print(f"ERROR: src/ not found", file=sys.stderr)
        sys.exit(1)

    violations = []

    # Scan all .cs files in src/
    for filepath in root.rglob('*.cs'):
        # Skip bin, obj, .generated, Tests
        parts = filepath.parts
        if any(p in ('bin', 'obj') for p in parts):
            continue
        if 'Generated' in str(filepath):
            continue
        # INCLUDE Tests/ for visibility

        violations.extend(audit_file(filepath))

    # Categorize by kind
    by_kind = defaultdict(list)
    for v in violations:
        by_kind[v['kind']].append(v)

    # Categorize by directory
    by_dir = defaultdict(list)
    for v in violations:
        dir_name = str(v['file']).split('\\')[1] if '\\' in str(v['file']) else 'src'
        by_dir[dir_name].append(v)

    # Output report
    print(f"Pattern #223 Audit Results")
    print(f"========================\n")
    print(f"Total unreferenced comments: {len(violations)}\n")

    print("Breakdown by kind:")
    for kind in sorted(by_kind.keys()):
        print(f"  {kind}: {len(by_kind[kind])}")
    print()

    print("Top 30 violations (file:line:kind:excerpt):")
    for i, v in enumerate(sorted(violations, key=lambda x: (x['file'], x['line']))[:30], 1):
        filepath_short = str(v['file']).replace('\\', '/').replace('src/', '')
        text_safe = v['text'].encode('utf-8', errors='replace').decode('utf-8')
        print(f"  {i:2d}. {filepath_short}:{v['line']:4d} [{v['kind']:5s}] {text_safe}")
    print()

    print("Directory heat-map (count):")
    for dir_name in sorted(by_dir.keys()):
        print(f"  {dir_name}: {len(by_dir[dir_name])}")
    print()

    # Tier classification
    total = len(violations)
    if total < 30:
        tier = "LOW (handle as you touch the file)"
    elif total < 100:
        tier = "MODERATE (sweep before next minor release)"
    else:
        tier = "ENDEMIC (promote to Pattern Catalog with DF1015 deferred)"

    tier_safe = tier.encode('utf-8', errors='replace').decode('utf-8')
    print(f"Tier Classification: {tier_safe} ({total} violations)")

    return 0 if total == 0 else 1

if __name__ == '__main__':
    sys.exit(main())
