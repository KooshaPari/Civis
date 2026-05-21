#!/usr/bin/env python3
"""
Pattern #99 Detection: Unprotected Dictionary<string, T> without StringComparer.Ordinal

Scans C# source for Dictionary<string, T> declarations lacking explicit StringComparer.
Severity: HIGH if no StringComparer argument within 3 lines; MED otherwise.
Allowlist: docs/qa/string-dict-allowlist.txt + inline // string-dict-ok: <reason>
"""

import re
import sys
import json
from pathlib import Path
from typing import List, Tuple, Dict

def load_allowlist(allowlist_path: Path) -> set:
    """Load allowlist from file."""
    if not allowlist_path.exists():
        return set()
    try:
        with open(allowlist_path, 'r') as f:
            lines = f.readlines()
        # Extract patterns: file:line format or file format
        patterns = set()
        for line in lines:
            line = line.strip()
            if line and not line.startswith('#'):
                patterns.add(line)
        return patterns
    except Exception:
        return set()

def find_cs_files(src_dir: Path) -> List[Path]:
    """Find all .cs files under src/, excluding Tests/, bin/, obj/."""
    files = []
    for cs_file in src_dir.rglob('*.cs'):
        parts = cs_file.parts
        # Skip excluded directories
        # Analyzers/ and Roslyn/ contain analyzer source whose comments/strings
        # describe the very pattern being detected (Pattern #99 self-match false positive).
        if 'Tests' in parts or 'bin' in parts or 'obj' in parts or 'Analyzers' in parts or 'Roslyn' in parts:
            continue
        files.append(cs_file)
    return sorted(files)

def extract_dictionary_declarations(content: str, file_path: Path) -> List[Tuple[int, str, str]]:
    """
    Extract CONSTRUCTOR instantiation of Dictionary<string, T> without StringComparer.
    Only matches: new Dictionary<string, T>(...) or new ConcurrentDictionary<string, T>(...)
    Ignores: type declarations like 'private Dictionary<string, T> _field;'
            method parameters like 'void M(Dictionary<string, int> arg)'
            method declarations like 'Dictionary<string, int> GetMap()'
    Returns: [(line_num, declaration_text, severity)]
    """
    issues = []
    lines = content.split('\n')

    # Pattern: new Dictionary<string, T> or new ConcurrentDictionary<string, T>
    # Must NOT match:
    # - Type declarations: 'private Dictionary<string, T> _field;' or 'Dictionary<string, T> { get; }'
    # - Method params: 'void M(Dictionary<string, T> arg)'
    # - Method returns: 'Dictionary<string, T> Method()'
    constructor_pattern = re.compile(r'new\s+(?:Concurrent)?Dictionary\s*<\s*string\s*,', re.IGNORECASE)

    for i, line in enumerate(lines):
        # Skip lines with inline allowlist marker
        if '// string-dict-ok:' in line:
            continue

        # Must contain 'new Dictionary<string, ...>' or 'new ConcurrentDictionary<string, ...>'
        if not constructor_pattern.search(line):
            continue

        # Additional validation: ensure this is a constructor call, not a type declaration.
        # Skip if the line looks like a type declaration (no 'new' keyword really present on a declaration context).
        # By matching 'new', we're already filtering to constructor contexts.
        # However, we still need to reject false patterns like:
        #   'new\s+(Concurrent)?Dictionary' followed by '(' instead of '()' or '(capacity)'
        # The real issue is distinguishing:
        #   var x = new Dictionary<string, int>();              <- REAL constructor (should match)
        #   private Dictionary<string, int> _x;                 <- Type decl (should NOT match, but we skip it because no 'new')
        #   void M(Dictionary<string, int> arg)                 <- Type decl (should NOT match, no 'new')

        # Since we only matched lines with 'new Dictionary<string, ...>', we're already filtering correctly.
        # The remaining check is to avoid false positives on type declarations.
        # Type declarations won't have 'new' keyword, so we're already safe.

        # Additional validation: reject lines that look like type declarations (no 'new' on line).
        # This catches edge cases where our main regex might have matched on a type declaration line.
        if 'new' not in line:
            continue

        # Look ahead up to 3 lines for StringComparer (starting from current line)
        comparer_found = False
        look_ahead_lines = lines[i:min(i+3, len(lines))]
        look_ahead_text = '\n'.join(look_ahead_lines)

        if 'StringComparer.Ordinal' in look_ahead_text or 'StringComparer.OrdinalIgnoreCase' in look_ahead_text:
            comparer_found = True

        severity = 'MED' if comparer_found else 'HIGH'
        issues.append((i+1, line.strip(), severity))  # i+1 for 1-based line number

    return issues

def check_allowlist(file_path: Path, line_num: int, allowlist: set) -> bool:
    """Check if this file:line is in allowlist."""
    try:
        rel_path = file_path.relative_to(Path.cwd())
        key = f"{rel_path}:{line_num}"
        if key in allowlist:
            return True
    except ValueError:
        pass
    # Also check just filename:line
    key_file = f"{file_path.name}:{line_num}"
    return key_file in allowlist

def main():
    import argparse

    parser = argparse.ArgumentParser(description='Pattern #99 Detection: Unprotected Dictionary<string, T>')
    parser.add_argument('--json', action='store_true', help='Output JSON format')
    parser.add_argument('--threshold', type=int, default=10, help='Fail if HIGH violations exceed threshold (default: 10)')
    parser.add_argument('--test', action='store_true', help='Run self-test with fixtures')
    args = parser.parse_args()

    if args.test:
        return run_self_test()

    src_dir = Path('src')
    if not src_dir.exists():
        print('Error: src/ directory not found', file=sys.stderr)
        sys.exit(1)

    allowlist_path = Path('docs/qa/string-dict-allowlist.txt')
    allowlist = load_allowlist(allowlist_path)

    all_issues = []
    high_count = 0
    med_count = 0

    for cs_file in find_cs_files(src_dir):
        try:
            with open(cs_file, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            if args.json:
                print(json.dumps({'error': f'Failed to read {cs_file}: {e}'}))
            continue

        issues = extract_dictionary_declarations(content, cs_file)
        for line_num, text, severity in issues:
            # Check allowlist
            if check_allowlist(cs_file, line_num, allowlist):
                continue

            all_issues.append({
                'file': str(cs_file),
                'line': line_num,
                'text': text,
                'severity': severity
            })
            if severity == 'HIGH':
                high_count += 1
            else:
                med_count += 1

    if args.json:
        output = {
            'high': high_count,
            'med': med_count,
            'issues': all_issues
        }
        print(json.dumps(output, indent=2))
    else:
        if all_issues:
            print(f'Found {len(all_issues)} issues (HIGH: {high_count}, MED: {med_count})')
            for issue in all_issues:
                print(f"  {issue['file']}:{issue['line']} [{issue['severity']}] {issue['text'][:80]}")
        else:
            print('No unprotected Dictionary<string, T> found')

    # Exit with failure if HIGH count exceeds threshold
    if high_count > args.threshold:
        if not args.json:
            print(f'FAIL: HIGH violations ({high_count}) exceed threshold ({args.threshold})', file=sys.stderr)
        sys.exit(1)

    sys.exit(0)

def run_self_test():
    """Self-test with positive and negative fixtures."""
    test_cases = [
        # (code, should_detect, expected_severity, description)
        # Positive cases (should detect as HIGH - no StringComparer)
        ('var cache = new Dictionary<string, Unit>();', True, 'HIGH', 'Constructor no comparer'),
        ('var x = new Dictionary<string, int>();', True, 'HIGH', 'Constructor with parens, no comparer'),
        ('var map = new ConcurrentDictionary<string, int>();', True, 'HIGH', 'Concurrent constructor no comparer'),

        # Positive cases (should detect as MED - has StringComparer within 3 lines)
        ('var dict = new Dictionary<string, Item>(StringComparer.Ordinal);', True, 'MED', 'Constructor with Ordinal'),
        ('var dict = new Dictionary<string, int>(StringComparer.OrdinalIgnoreCase);', True, 'MED', 'Constructor with OrdinalIgnoreCase'),

        # Negative cases (should NOT detect - type declarations without 'new' keyword)
        ('private Dictionary<string, int> _map;', False, 'N/A', 'Field declaration (no new)'),
        ('public Dictionary<string, Item> Items { get; }', False, 'N/A', 'Property declaration (no new)'),
        ('void ProcessMap(Dictionary<string, int> input)', False, 'N/A', 'Method parameter (no new)'),
        ('Dictionary<string, int> GetMap()', False, 'N/A', 'Method return type (no new)'),
        ('protected IReadOnlyDictionary<string, int> Cache { get; }', False, 'N/A', 'IReadOnlyDictionary type (no new)'),

        # Negative cases (other types)
        ('var list = new List<string>();', False, 'N/A', 'List, not dict'),
        ('var set = new HashSet<string>();', False, 'N/A', 'HashSet, not dict'),
    ]

    passed = 0
    failed = 0

    for i, (code, should_detect, expected_severity, desc) in enumerate(test_cases, start=1):
        issues = extract_dictionary_declarations(code, Path('test.cs'))
        detected = len(issues) > 0

        if should_detect:
            if detected and issues[0][2] == expected_severity:
                passed += 1
                print(f'  Test {i}: PASS ({desc})')
            else:
                failed += 1
                expected = f'{expected_severity}' if expected_severity != 'N/A' else 'detection'
                actual = f'{issues[0][2]}' if issues else 'no detection'
                print(f'  Test {i}: FAIL ({desc}: expected {expected}, got {actual})')
        else:
            if not detected:
                passed += 1
                print(f'  Test {i}: PASS ({desc})')
            else:
                failed += 1
                print(f'  Test {i}: FAIL ({desc}: unexpected detection {issues})')

    print(f'\nSelf-test: {passed} passed, {failed} failed')
    return 0 if failed == 0 else 1

if __name__ == '__main__':
    sys.exit(main())
