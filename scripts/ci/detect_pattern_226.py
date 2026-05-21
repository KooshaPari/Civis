#!/usr/bin/env python3
"""
Pattern #226 Detection: Public mutable fields in NuGet-published surface.

Scans NuGet-published C# source (SDK, Bridge.Client, Bridge.Protocol, Domains)
for `public <Type> <Name>;` field declarations (not properties).

Why bad: Public fields bypass property getter/setter hooks (no validation,
no INotifyPropertyChanged, no breakpoints), and cannot be made virtual/abstract.
Consumers binding via reflection (JSON/YAML) silently lock the contract.

Severity: HIGH (NuGet surface).
Allowlist: docs/qa/pattern-226-allowlist.txt + inline `// public-field-ok: <reason>`
"""

import re
import sys
import json
from pathlib import Path
from typing import List, Tuple

REPO_ROOT = Path(__file__).resolve().parents[2]

SCAN_DIRS = [
    REPO_ROOT / 'src' / 'SDK',
    REPO_ROOT / 'src' / 'Bridge' / 'Client',
    REPO_ROOT / 'src' / 'Bridge' / 'Protocol',
    REPO_ROOT / 'src' / 'Domains',
]

# Match: `public [static] <Type> <Name>;` (field declaration, not property)
# Excludes property syntax `{` and method declarations `(`.
# Type may include generics, arrays, nullable: List<string>, int[], string?, Dictionary<string,int>
FIELD_RE = re.compile(
    r'^\s*public\s+'
    r'(?P<modifiers>(?:static\s+|readonly\s+|volatile\s+|unsafe\s+|extern\s+|new\s+)*)'
    r'(?P<type>[A-Za-z_][\w\.\?]*(?:\s*<[^<>;{}()]*(?:<[^<>;{}()]*>[^<>;{}()]*)*>)?(?:\s*\[\s*\])?(?:\s*\?)?)'
    r'\s+(?P<name>[A-Za-z_]\w*)\s*'
    r'(?:=\s*[^;]+)?\s*;'
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

def find_cs_files(dirs: List[Path]) -> List[Path]:
    files = []
    for d in dirs:
        if not d.exists():
            continue
        for cs_file in d.rglob('*.cs'):
            parts = cs_file.parts
            if 'Tests' in parts or 'bin' in parts or 'obj' in parts:
                continue
            if cs_file.name.endswith('.Generated.cs') or cs_file.name.endswith('.g.cs'):
                continue
            files.append(cs_file)
    return sorted(files)

# Reserved type-like tokens that should NOT be flagged (control-flow keywords)
KEYWORDS = {
    'class', 'struct', 'interface', 'enum', 'delegate', 'event',
    'const', 'return', 'if', 'else', 'while', 'for', 'foreach',
    'switch', 'case', 'default', 'void', 'override', 'virtual',
    'abstract', 'sealed', 'partial', 'async', 'using', 'namespace',
}

def is_field_violation(line: str, prev_lines: List[str]) -> bool:
    """Return True if line declares a public mutable field."""
    if '// public-field-ok:' in line:
        return False
    # Skip const declarations (those are constants, OK)
    if re.search(r'\bpublic\s+const\b', line):
        return False
    # Skip readonly+static (effectively constant, OK)
    if re.search(r'\bpublic\s+static\s+readonly\b', line) or \
       re.search(r'\bpublic\s+readonly\s+static\b', line):
        return False
    # Skip event declarations
    if re.search(r'\bpublic\s+(?:static\s+)?event\b', line):
        return False
    # Skip delegate / class / struct / enum / interface declarations
    if re.search(r'\bpublic\s+(?:static\s+|abstract\s+|sealed\s+|partial\s+)*(?:class|struct|interface|enum|delegate)\b', line):
        return False
    # Skip property (has `{` or `=>`)
    if '{' in line or '=>' in line:
        return False
    # Skip method declarations (has `(`)
    if '(' in line:
        return False

    m = FIELD_RE.match(line)
    if not m:
        return False

    name = m.group('name')
    type_token = m.group('type').strip()
    if name in KEYWORDS or type_token in KEYWORDS:
        return False

    # Check preceding lines for [JsonIgnore], [NonSerialized], or `// public-field-ok:` marker
    for prev in prev_lines[-3:]:
        s = prev.strip()
        if '[JsonIgnore]' in s or '[NonSerialized]' in s:
            return False
        if '// public-field-ok:' in s:
            return False

    return True

def scan_file(file_path: Path) -> List[Tuple[int, str]]:
    issues = []
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
    except Exception:
        return issues

    for i, line in enumerate(lines):
        prev_lines = lines[max(0, i-3):i]
        if is_field_violation(line, prev_lines):
            issues.append((i + 1, line.rstrip()))
    return issues

def check_allowlist(file_path: Path, line_num: int, allowlist: set) -> bool:
    try:
        rel = file_path.relative_to(REPO_ROOT).as_posix()
    except ValueError:
        rel = str(file_path)
    if f"{rel}:{line_num}" in allowlist:
        return True
    if rel in allowlist:
        return True
    return f"{file_path.name}:{line_num}" in allowlist

def main():
    import argparse
    parser = argparse.ArgumentParser(description='Pattern #226: Public mutable field detection (NuGet surface)')
    parser.add_argument('--json', action='store_true')
    parser.add_argument('--threshold', type=int, default=0,
                        help='Fail if HIGH violations exceed threshold (default: 0)')
    args = parser.parse_args()

    allowlist = load_allowlist(REPO_ROOT / 'docs' / 'qa' / 'pattern-226-allowlist.txt')

    all_issues = []
    high_count = 0

    for cs_file in find_cs_files(SCAN_DIRS):
        for line_num, text in scan_file(cs_file):
            if check_allowlist(cs_file, line_num, allowlist):
                continue
            all_issues.append({
                'file': str(cs_file.relative_to(REPO_ROOT).as_posix()),
                'line': line_num,
                'text': text.strip(),
                'severity': 'HIGH',
            })
            high_count += 1

    if args.json:
        print(json.dumps({'high': high_count, 'total': len(all_issues), 'issues': all_issues}, indent=2))
    else:
        if all_issues:
            print(f'Pattern #226: Found {len(all_issues)} HIGH violations')
            for issue in all_issues:
                print(f"  {issue['file']}:{issue['line']} [HIGH] {issue['text'][:100]}")
        else:
            print('Pattern #226: No public mutable fields found in NuGet surface')
        print(f'Total HIGH: {high_count}')

    if high_count > args.threshold:
        if not args.json:
            print(f'FAIL: HIGH ({high_count}) exceeds threshold ({args.threshold})', file=sys.stderr)
        sys.exit(1)
    sys.exit(0)

if __name__ == '__main__':
    main()
