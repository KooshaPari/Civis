#!/usr/bin/env python3
"""
Pattern #111: Silent Exception Swallowing Detection

Detects bare catch blocks `catch { }` that may hide errors.
Allows safe-swallow annotations and configured allowlist for disposable cleanup.
"""

import re
import sys
from pathlib import Path


def main():
    repo_root = Path("C:/Users/koosh/Dino")
    src_dir = repo_root / "src"
    allowlist_file = repo_root / "docs/qa/silent-catch-allowlist.txt"

    # Load allowlist of files where bare catch is acceptable
    allowed_files = set()
    if allowlist_file.exists():
        with open(allowlist_file, encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#'):
                    allowed_files.add(line)

    violations = []
    safe_catches = []

    # Walk all C# files
    for cs_file in src_dir.rglob("*.cs"):
        # Skip obj and generated files
        if "\\obj\\" in str(cs_file) or ".g.cs" in str(cs_file):
            continue

        relative = str(cs_file.relative_to(src_dir)).replace("\\", "/")

        # Check if file is in allowlist
        if any(pattern in relative for pattern in allowed_files):
            continue

        try:
            with open(cs_file, encoding='utf-8', errors='ignore') as f:
                lines = f.readlines()
        except Exception as e:
            print(f"Error reading {cs_file}: {e}", file=sys.stderr)
            continue

        for i, line in enumerate(lines, 1):
            # Match bare catch blocks with or without exception type
            if re.search(r'catch\s*(\([A-Za-z_][A-Za-z0-9_]*\s*\))?\s*\{\s*\}', line):
                # Check for safe-swallow annotation on same line or line above
                annotation_found = False
                # Check same line first (most common pattern: `catch { } // safe-swallow: reason`)
                if "// safe-swallow:" in line or "/* safe-swallow:" in line:
                    annotation_found = True
                    safe_catches.append(f"{relative}:{i}")
                    continue
                # Check line above
                if i > 1 and ("// safe-swallow:" in lines[i-2] or "/* safe-swallow:" in lines[i-2]):
                    annotation_found = True
                    safe_catches.append(f"{relative}:{i}")
                    continue

                # Check context for disposable patterns
                context_start = max(0, i - 5)
                context_end = min(len(lines), i + 2)
                context = "".join(lines[context_start:context_end])

                # Safe patterns: pure cleanup
                if re.search(r'\.(Dispose|Close|Unsubscribe|Cancel)\s*\(\)', context):
                    safe_catches.append(f"{relative}:{i}")
                else:
                    violations.append(f"{relative}:{i}")

    # Report violations
    if violations:
        print(f"\n{'='*80}")
        print(f"Pattern #111 Violations: {len(violations)} bare catch blocks")
        print(f"{'='*80}")
        for v in sorted(violations):
            print(f"{v}")
        print(f"{'='*80}\n")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
