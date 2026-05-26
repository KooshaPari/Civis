#!/usr/bin/env python3
"""
Audit Pattern #220: Unsealed concrete public classes with mutable state
but no inheritance contract (no virtual methods, not designed for subclassing).

Outputs CSV of violations to stdout.
"""

import os
import re
import csv
import sys
from pathlib import Path
from typing import List, Tuple

def scan_csharp_files(src_root: str) -> List[Tuple[str, int, str, str]]:
    """Walk src/ and find candidate violations."""
    violations = []
    
    # Excluded dirs
    excluded = {'bin', 'obj', 'Tests', 'Test'}
    
    for dirpath, dirnames, filenames in os.walk(src_root):
        # Filter dirs
        dirnames[:] = [d for d in dirnames if d not in excluded]
        
        for fname in filenames:
            if not fname.endswith('.cs'):
                continue
            
            fpath = os.path.join(dirpath, fname)
            try:
                with open(fpath, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
            except Exception as e:
                continue
            
            # Find all public class declarations (not sealed, not abstract, not static)
            # Pattern: public class Foo { ... } (simplistic, stops at first closing brace depth match)
            class_pattern = r'^(\s*)public\s+(?!sealed|abstract|static)class\s+(\w+)\s*(?:\([^)]*\))?\s*(?::\s*[^\{]+)?\s*\{'
            
            for m in re.finditer(class_pattern, content, re.MULTILINE):
                indent_level = len(m.group(1))
                class_name = m.group(2)
                class_start = m.start()
                class_line = content[:class_start].count('\n') + 1
                
                # Find class body (crude: scan forward until matching closing brace)
                open_pos = m.end() - 1  # position of opening {
                brace_count = 1
                i = open_pos + 1
                while i < len(content) and brace_count > 0:
                    if content[i] == '{':
                        brace_count += 1
                    elif content[i] == '}':
                        brace_count -= 1
                    i += 1
                
                if brace_count != 0:
                    # Couldn't match braces, skip
                    continue
                
                class_body = content[open_pos:i]
                
                # Check for mutable state: private field (list, dict, set, or general _field)
                has_mutable_state = bool(re.search(
                    r'private\s+(?:(?:List|Dictionary|HashSet|Queue|Stack|IList|IDictionary|ISet)\s*<|[\w\.]+\s+_\w+)',
                    class_body
                ))
                
                # Check for inheritance contract: protected virtual or protected abstract
                has_inheritance_contract = bool(re.search(
                    r'protected\s+(?:virtual|abstract)',
                    class_body
                ))
                
                # Violation: mutable state + no inheritance contract
                if has_mutable_state and not has_inheritance_contract:
                    reason = "mutable_state_no_inheritance_contract"
                    violations.append((fpath, class_line, class_name, reason))
    
    return violations

def main():
    src_root = os.path.join(os.path.dirname(__file__), '..', '..', 'src')
    src_root = os.path.normpath(src_root)
    
    if not os.path.isdir(src_root):
        print(f"Error: {src_root} not found", file=sys.stderr)
        sys.exit(1)
    
    violations = scan_csharp_files(src_root)
    
    # Sort by file, then line
    violations.sort(key=lambda x: (x[0], x[1]))
    
    # Output CSV
    writer = csv.writer(sys.stdout)
    writer.writerow(['file', 'line', 'classname', 'reason'])
    for v in violations:
        # Make path relative for readability
        rel_path = os.path.relpath(v[0], src_root)
        writer.writerow([rel_path, v[1], v[2], v[3]])
    
    # Summary to stderr
    print(f"\nTotal violations: {len(violations)}", file=sys.stderr)

if __name__ == '__main__':
    main()
