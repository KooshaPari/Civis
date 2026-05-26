#!/usr/bin/env python3
"""
Pattern #99 Family Follow-up (#735): StringComparer mismatch within the same file.

Detects when a single C# file constructs multiple Dictionary<string, T> instances
(either via `new Dictionary<string, T>(...)` or `.ToDictionary(... , StringComparer.*)`)
using DIFFERENT StringComparers (e.g. one Ordinal "primary" + one OrdinalIgnoreCase
"snapshot"). This is a primary-snapshot drift hazard: lookups against the snapshot
will find keys the primary did not (or vice versa), producing subtle correctness bugs.

Audit reference: a2ef18ad3ff527f84 (4 confirmed primary-snapshot drift bugs).

Severity:
  HIGH - 2+ different comparers in same file (including 'implicit Ordinal' counted as
         Ordinal when explicit Ordinal also present is FINE; but Ordinal + IgnoreCase
         in the same file is HIGH).
  MED  - file uses only IgnoreCase but mixes 'new Dictionary<string,...>()' (no comparer,
         implicit Ordinal) with the IgnoreCase form (still a drift hazard).

Allowlist: docs/qa/comparer-mismatch-allowlist.txt + inline `// comparer-mismatch-ok: <reason>`
"""

import re
import sys
import json
from pathlib import Path
from typing import List, Tuple, Dict, Set


ORDINAL = "Ordinal"
IGNORECASE = "OrdinalIgnoreCase"
IMPLICIT = "ImplicitOrdinal"  # `new Dictionary<string,...>()` with no comparer

# Constructor form: new [Concurrent]Dictionary<string, ...>(...)
CTOR_PATTERN = re.compile(
    r'new\s+(?:Concurrent)?Dictionary\s*<\s*string\s*,[^>]*>\s*\(([^;]*?)\)',
    re.IGNORECASE | re.DOTALL,
)

# LINQ form: .ToDictionary(... , StringComparer.X)
TO_DICT_PATTERN = re.compile(
    r'\.ToDictionary\s*\(([^;]*?)\)',
    re.IGNORECASE | re.DOTALL,
)


def load_allowlist(allowlist_path: Path) -> set:
    if not allowlist_path.exists():
        return set()
    try:
        patterns = set()
        with open(allowlist_path, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#'):
                    patterns.add(line)
        return patterns
    except Exception:
        return set()


def find_cs_files(src_dir: Path) -> List[Path]:
    files = []
    for cs_file in src_dir.rglob('*.cs'):
        parts = cs_file.parts
        if any(p in parts for p in ('Tests', 'bin', 'obj', 'Analyzers', 'Roslyn')):
            continue
        files.append(cs_file)
    return sorted(files)


def classify_comparer(arg_text: str) -> str:
    """Classify which comparer (if any) appears in the argument string."""
    if 'StringComparer.OrdinalIgnoreCase' in arg_text:
        return IGNORECASE
    if 'StringComparer.Ordinal' in arg_text:
        return ORDINAL
    if 'StringComparer.InvariantCultureIgnoreCase' in arg_text:
        return IGNORECASE
    if 'StringComparer.InvariantCulture' in arg_text:
        return ORDINAL  # cultural, but treated as "case-sensitive" for drift purposes
    return ""


def scan_file(content: str) -> List[Tuple[int, str, str]]:
    """
    Return list of (line_num, comparer_classification, snippet).
    Lines tagged with `// comparer-mismatch-ok:` are skipped.
    """
    findings: List[Tuple[int, str, str]] = []
    lines = content.split('\n')

    # Build per-line allowlist set
    skip_lines: Set[int] = set()
    for i, line in enumerate(lines, start=1):
        if '// comparer-mismatch-ok:' in line:
            skip_lines.add(i)

    # Constructor matches
    for m in CTOR_PATTERN.finditer(content):
        line_num = content[:m.start()].count('\n') + 1
        if line_num in skip_lines:
            continue
        arg = m.group(1) or ""
        kind = classify_comparer(arg)
        if not kind:
            # No explicit comparer in argument → implicit Ordinal
            kind = IMPLICIT
        snippet = lines[line_num - 1].strip()[:120] if line_num - 1 < len(lines) else ""
        findings.append((line_num, kind, snippet))

    # .ToDictionary matches — only flag when an explicit comparer is passed
    for m in TO_DICT_PATTERN.finditer(content):
        line_num = content[:m.start()].count('\n') + 1
        if line_num in skip_lines:
            continue
        arg = m.group(1) or ""
        kind = classify_comparer(arg)
        if not kind:
            continue  # no comparer in ToDictionary == we can't infer key type cheaply
        snippet = lines[line_num - 1].strip()[:120] if line_num - 1 < len(lines) else ""
        findings.append((line_num, kind, snippet))

    return findings


def evaluate_file(findings: List[Tuple[int, str, str]]) -> Tuple[str, Set[str]]:
    """
    Decide severity for a file given its findings.
    Returns (severity, set_of_comparer_kinds) where severity is "" | "MED" | "HIGH".
    """
    kinds = {k for _, k, _ in findings}
    explicit = kinds - {IMPLICIT}
    if ORDINAL in explicit and IGNORECASE in explicit:
        return "HIGH", kinds
    if IGNORECASE in explicit and IMPLICIT in kinds:
        # Implicit Ordinal + explicit IgnoreCase in same file = drift hazard
        return "MED", kinds
    return "", kinds


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description='Pattern #99 follow-up (#735): StringComparer mismatch within a file.'
    )
    parser.add_argument('--json', action='store_true', help='Output JSON')
    parser.add_argument('--threshold', type=int, default=5,
                        help='Fail if HIGH violations exceed threshold (default: 5)')
    parser.add_argument('--test', action='store_true', help='Run self-test')
    args = parser.parse_args()

    if args.test:
        return run_self_test()

    src_dir = Path('src')
    if not src_dir.exists():
        print('Error: src/ directory not found', file=sys.stderr)
        sys.exit(2)

    allowlist = load_allowlist(Path('docs/qa/comparer-mismatch-allowlist.txt'))

    all_issues = []
    high_count = 0
    med_count = 0

    for cs_file in find_cs_files(src_dir):
        try:
            with open(cs_file, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception:
            continue

        findings = scan_file(content)
        if len(findings) < 2:
            continue

        severity, kinds = evaluate_file(findings)
        if not severity:
            continue

        try:
            rel = str(cs_file.relative_to(Path.cwd()))
        except ValueError:
            rel = str(cs_file)

        if rel in allowlist:
            continue

        entry = {
            'file': rel,
            'severity': severity,
            'comparers': sorted(kinds),
            'sites': [
                {'line': ln, 'comparer': k, 'text': txt}
                for ln, k, txt in findings
            ],
        }
        all_issues.append(entry)
        if severity == 'HIGH':
            high_count += 1
        else:
            med_count += 1

    if args.json:
        print(json.dumps(
            {'high': high_count, 'med': med_count, 'issues': all_issues},
            indent=2,
        ))
    else:
        if all_issues:
            print(f'Found {len(all_issues)} files with comparer mismatch '
                  f'(HIGH: {high_count}, MED: {med_count})')
            for issue in all_issues:
                print(f"  {issue['file']} [{issue['severity']}] "
                      f"comparers={issue['comparers']}")
                for s in issue['sites']:
                    print(f"    L{s['line']:>4} [{s['comparer']}] {s['text']}")
        else:
            print('No comparer mismatches found.')

    if high_count > args.threshold:
        if not args.json:
            print(f'FAIL: HIGH violations ({high_count}) exceed threshold '
                  f'({args.threshold})', file=sys.stderr)
        sys.exit(1)
    sys.exit(0)


def run_self_test():
    fixtures = [
        # (content, expected_severity, description)
        (
            'var a = new Dictionary<string, int>(StringComparer.Ordinal);\n'
            'var b = new Dictionary<string, int>(StringComparer.OrdinalIgnoreCase);\n',
            'HIGH',
            'Ordinal + IgnoreCase in same file',
        ),
        (
            'var a = new Dictionary<string, int>();\n'
            'var b = installed.ToDictionary(p => p.Id, StringComparer.OrdinalIgnoreCase);\n',
            'MED',
            'Implicit Ordinal + explicit IgnoreCase via ToDictionary',
        ),
        (
            'var a = new Dictionary<string, int>(StringComparer.Ordinal);\n'
            'var b = new Dictionary<string, int>(StringComparer.Ordinal);\n',
            '',
            'Both Ordinal – clean',
        ),
        (
            'var a = new Dictionary<string, int>();\n'
            'var b = new Dictionary<string, int>();\n',
            '',
            'Both implicit – not a drift since identical',
        ),
        (
            'var a = new Dictionary<string, int>(StringComparer.OrdinalIgnoreCase);\n'
            'var b = new Dictionary<string, int>(StringComparer.OrdinalIgnoreCase);\n',
            '',
            'Both IgnoreCase – clean',
        ),
    ]
    passed = 0
    failed = 0
    for i, (content, expected, desc) in enumerate(fixtures, start=1):
        findings = scan_file(content)
        severity, _ = evaluate_file(findings)
        if severity == expected:
            passed += 1
            print(f'  Test {i}: PASS ({desc})')
        else:
            failed += 1
            print(f'  Test {i}: FAIL ({desc}: expected "{expected}", got "{severity}")')
    print(f'\nSelf-test: {passed} passed, {failed} failed')
    return 0 if failed == 0 else 1


if __name__ == '__main__':
    sys.exit(main())
