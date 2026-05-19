#!/usr/bin/env python3
"""
Audit empty catch blocks in C# codebase.
Detects catch blocks with no logging, comment, or content.
"""

import re
import os
from pathlib import Path
from collections import defaultdict

def is_excluded_path(path: str) -> bool:
    """Check if path should be excluded from scan."""
    excluded = ['bin', 'obj', 'Tests', '.Generated.cs']
    return any(exc in path for exc in excluded)

def audit_file(filepath: str) -> list[tuple[int, str]]:
    """Scan file for empty catch blocks. Returns list of (line_num, context)."""
    violations = []
    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception:
        return violations

    i = 0
    while i < len(lines):
        line = lines[i]

        # Match catch clause (with optional exception type)
        if re.search(r'catch\s*(\([^)]*\))?\s*\{?\s*$', line):
            # Next non-blank line should indicate content
            j = i + 1
            while j < len(lines) and lines[j].strip() == '':
                j += 1

            # If next line is closing brace, it's empty
            if j < len(lines) and lines[j].strip() == '}':
                # But check: is there a comment or anything between catch and }?
                has_content = False
                for k in range(i + 1, j):
                    stripped = lines[k].strip()
                    if stripped and not stripped.startswith('//'):
                        has_content = True
                        break

                if not has_content:
                    violations.append((i + 1, line.strip()))

        i += 1

    return violations

def main():
    src_dir = Path('C:/Users/koosh/Dino/src')

    all_violations = []
    dir_counts = defaultdict(int)

    # Walk src/ tree
    for root, dirs, files in os.walk(src_dir):
        # Skip excluded paths
        if is_excluded_path(root):
            continue

        for filename in files:
            if not filename.endswith('.cs'):
                continue

            filepath = os.path.join(root, filename)
            if is_excluded_path(filepath):
                continue

            violations = audit_file(filepath)
            for line_num, context in violations:
                rel_path = os.path.relpath(filepath, src_dir)
                all_violations.append((rel_path, line_num, context))

                # Track directory
                top_dir = rel_path.split(os.sep)[0]
                dir_counts[top_dir] += 1

    # Sort by directory, then filename, then line
    all_violations.sort()

    # Report
    print(f"Empty Catch Block Audit")
    print(f"=" * 70)
    print(f"Total violations: {len(all_violations)}")
    print()

    print("Directory breakdown:")
    for dir_name in sorted(dir_counts.keys()):
        print(f"  {dir_name}: {dir_counts[dir_name]}")
    print()

    print("Top 10 violations (file:line):")
    for i, (rel_path, line_num, context) in enumerate(all_violations[:10], 1):
        print(f"  {i}. {rel_path}:{line_num}")
        print(f"     {context[:60]}")

    if len(all_violations) > 10:
        print(f"  ... and {len(all_violations) - 10} more")
    print()

    # Tier classification
    total = len(all_violations)
    if total < 5:
        tier = "LOW"
    elif total < 20:
        tier = "MODERATE"
    else:
        tier = "ENDEMIC"

    print(f"Tier: {tier} ({total} violations)")
    print()

    # Write to file
    with open('C:/Users/koosh/Dino/docs/qa/pattern_228_audit.md', 'w') as f:
        f.write("# Pattern #228 Audit: Empty Catch Blocks\n\n")
        f.write("## Definition\n")
        f.write("Empty `catch` blocks with no logging, comments, or statements between opening and closing braces.\n\n")
        f.write("## Detection Logic\n")
        f.write("- Regex: `catch\\s*(\\([^)]*\\))?\\s*\\{?\\s*$` (catch line with optional type)\n")
        f.write("- State: If next non-blank line is `}`, flag as empty\n")
        f.write("- Skip: Blocks with comments or statements\n\n")
        f.write("## Results\n\n")
        f.write(f"**Total violations**: {total}\n\n")
        f.write("**Directory breakdown**:\n")
        for dir_name in sorted(dir_counts.keys()):
            f.write(f"- {dir_name}: {dir_counts[dir_name]}\n")
        f.write(f"\n**Tier**: {tier}\n\n")
        f.write("## Top Violations\n\n")
        for i, (rel_path, line_num, context) in enumerate(all_violations[:10], 1):
            f.write(f"{i}. `{rel_path}:{line_num}`\n")
        if len(all_violations) > 10:
            f.write(f"\n... and {len(all_violations) - 10} more (see full output for complete list)\n")

    print("✓ Report written to docs/qa/pattern_228_audit.md")

if __name__ == '__main__':
    main()
