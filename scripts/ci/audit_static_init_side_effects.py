#!/usr/bin/env python3
"""
Pattern #231 Audit: Static Field Initializers with Side Effects

Detects static field initializers that perform I/O, network, file system,
or process operations at static constructor time (program start).

These block application startup, throw if resources missing, and cannot be
easily deferred or mocked.
"""

import re
import os
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import List, Dict

EXCLUDE_DIRS = {'bin', 'obj', 'Tests', '.git', 'node_modules', 'Generated', '.vs'}

# Regex patterns for side-effect initializers
PATTERNS = [
    # File I/O
    (r'File\.(ReadAllText|ReadAllBytes|ReadAllLines|ReadLines|OpenRead)', 'File I/O'),
    (r'Directory\.(EnumerateFiles|EnumerateDirectories|GetFiles|GetDirectories)', 'Directory I/O'),

    # Network
    (r'new\s+HttpClient', 'HttpClient instantiation'),
    (r'HttpRequestMessage', 'HTTP request'),

    # Process/shell
    (r'new\s+Process', 'Process instantiation'),
    (r'ProcessStartInfo', 'Process start'),

    # Environment/registry
    (r'Environment\.GetEnvironment', 'Environment variable read'),
    (r'Registry\.LocalMachine|Registry\.CurrentUser', 'Registry access'),
    (r'Environment\.ExpandEnvironmentVariables', 'Environment expansion'),

    # Reflection (expensive)
    (r'Assembly\.Load(?!ed)', 'Assembly.Load'),
    (r'Type\.GetType', 'Type.GetType'),
]

@dataclass
class Violation:
    file: str
    line: int
    field_name: str
    reason: str
    severity: str  # HIGH, MED, LOW

def get_severity(file_path: str) -> str:
    """Determine severity based on file location."""
    normalized = file_path.replace('\\', '/').lower()
    if '/sdk/' in normalized or '/bridge/' in normalized:
        return 'HIGH'
    elif '/runtime/' in normalized or '/domains/' in normalized:
        return 'MED'
    else:
        return 'LOW'

def should_exclude(path: Path) -> bool:
    """Check if path should be excluded."""
    for part in path.parts:
        if part in EXCLUDE_DIRS:
            return True
    return False

def audit_file(file_path: Path) -> List[Violation]:
    """Audit a single C# file for Pattern #231 violations."""
    violations = []

    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"Warning: Could not read {file_path}: {e}", file=sys.stderr)
        return violations

    # Look for static field declarations with initializers
    in_static_field = False
    static_field_start = 0
    field_name = None

    for line_num, line in enumerate(lines, 1):
        # Detect static field declaration
        if re.search(r'(?:private|public|protected|internal)?\s*static\s+(?:readonly\s+)?', line):
            if '=' in line:
                # Single-line static field with initializer
                in_static_field = True
                static_field_start = line_num

                # Extract field name
                field_match = re.search(r'\s+(\w+)\s*=', line)
                field_name = field_match.group(1) if field_match else '?'

                # Check for side effects in this line
                for pattern, reason in PATTERNS:
                    if re.search(pattern, line):
                        severity = get_severity(str(file_path))
                        violations.append(Violation(
                            file=str(file_path),
                            line=line_num,
                            field_name=field_name,
                            reason=reason,
                            severity=severity
                        ))
                        break
            else:
                # Multi-line or no initializer, track for next lines
                in_static_field = True
                static_field_start = line_num
                field_match = re.search(r'\s+(\w+)\s*(?:;|,)', line)
                field_name = field_match.group(1) if field_match else '?'

        # Check continuation lines for side effects
        if in_static_field and ';' in line:
            in_static_field = False
            # Re-scan the full field declaration (from start to here)
            field_text = ''.join(lines[static_field_start-1:line_num])

            if '=' in field_text:  # Has initializer
                for pattern, reason in PATTERNS:
                    if re.search(pattern, field_text):
                        severity = get_severity(str(file_path))
                        violations.append(Violation(
                            file=str(file_path),
                            line=static_field_start,
                            field_name=field_name,
                            reason=reason,
                            severity=severity
                        ))
                        break

    return violations

def main():
    """Main entry point."""
    src_dir = Path('src')

    if not src_dir.exists():
        print(f"Error: src/ directory not found. Run from repo root.", file=sys.stderr)
        sys.exit(1)

    all_violations: List[Violation] = []

    # Audit all C# files
    for cs_file in src_dir.rglob('*.cs'):
        if should_exclude(cs_file):
            continue
        violations = audit_file(cs_file)
        all_violations.extend(violations)

    # Sort by severity (HIGH first), then file, then line
    severity_order = {'HIGH': 0, 'MED': 1, 'LOW': 2}
    all_violations.sort(
        key=lambda v: (severity_order.get(v.severity, 3), v.file, v.line)
    )

    # Summarize
    severity_counts = {}
    for v in all_violations:
        severity_counts[v.severity] = severity_counts.get(v.severity, 0) + 1

    # Determine tier
    total = len(all_violations)
    if total < 10:
        tier = 'low'
    elif total < 50:
        tier = 'mod'
    else:
        tier = 'endemic'

    # Generate report markdown
    report_lines = [
        '# Pattern #231 Audit: Static Field Initializers with Side Effects\n',
        f'**Date**: {Path("src").stat().st_mtime}\n',
        f'**Total Violations**: {total}\n',
        f'**Severity Breakdown**: HIGH={severity_counts.get("HIGH", 0)}, MED={severity_counts.get("MED", 0)}, LOW={severity_counts.get("LOW", 0)}\n',
        f'**Tier**: {tier}\n\n',
        '## Top Violations\n',
    ]

    for i, v in enumerate(all_violations[:10], 1):
        report_lines.append(f'{i}. `{v.file}:{v.line}` — `{v.field_name}` ({v.reason}) [{v.severity}]\n')

    if len(all_violations) > 10:
        report_lines.append(f'\n... and {len(all_violations) - 10} more violations\n')

    # Write report
    report_path = Path('docs/qa/pattern_231_audit.md')
    report_path.parent.mkdir(parents=True, exist_ok=True)

    with open(report_path, 'w', encoding='utf-8') as f:
        f.writelines(report_lines)

    # Print summary
    print(f'Pattern #231 Audit Complete')
    print(f'Total: {total} violations')
    print(f'HIGH: {severity_counts.get("HIGH", 0)}, MED: {severity_counts.get("MED", 0)}, LOW: {severity_counts.get("LOW", 0)}')
    print(f'Tier: {tier}')
    print(f'Report: {report_path}')

    # Exit with code 1 if HIGH violations exist (fail CI gate)
    if severity_counts.get('HIGH', 0) > 0:
        sys.exit(1)
    sys.exit(0)

if __name__ == '__main__':
    main()
