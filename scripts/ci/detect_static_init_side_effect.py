#!/usr/bin/env python3
"""
Pattern #231 Detector: Static Constructor / Static Field Initializer with I/O Side Effect

Flags static field initializers and static constructors that perform I/O, process spawn,
env-var reads, or HttpClient instantiation at class-load time.

Usage:
  python detect_static_init_side_effect.py

Exit codes:
  0 = no HIGH violations
  1 = HIGH violations exist (or parse error)
"""

import re
import sys
from pathlib import Path
from collections import defaultdict

REPO_ROOT = Path(__file__).resolve().parents[2]
ALLOWLIST_PATH = REPO_ROOT / "docs" / "qa" / "pattern-231-static-init-allowlist.txt"

# Target directories (NuGet-published surfaces)
SOURCES = [
    "src/SDK/",
    "src/Bridge/Client/",
    "src/Bridge/Protocol/",
    "src/Domains/",
]


def load_allowlist() -> set:
    """Load allowlist entries as set of "filepath:line" strings (POSIX paths)."""
    allow: set = set()
    if not ALLOWLIST_PATH.exists():
        return allow
    for line in ALLOWLIST_PATH.read_text(encoding="utf-8").splitlines():
        s = line.strip()
        if not s or s.startswith("#"):
            continue
        # Format: filepath:line_number | reason
        entry = s.split("|", 1)[0].strip()
        if not entry:
            continue
        # Normalize backslashes to forward slashes
        allow.add(entry.replace("\\", "/"))
    return allow

# Patterns indicating I/O side effects
IO_PATTERNS = [
    r"File\.",           # File.ReadAllText, File.WriteAllBytes, etc.
    r"Directory\.",      # Directory.CreateDirectory, etc.
    r"Path\.",           # Path.Combine (usually safe, but context-dependent)
    r"Process\.Start",   # Process spawn
    r"Environment\.",    # Environment variable read/write
    r"new HttpClient",   # HttpClient instantiation
    r"Path\.GetFullPath",
    r"System\.Net\.",    # Networking I/O
]

# Determine if a source file is in NuGet-published assembly
NUGET_DIRS = {"src/SDK/", "src/Bridge/Client/", "src/Bridge/Protocol/", "src/Domains/"}

def is_nuget_surface(filepath: str) -> bool:
    # Normalize Windows backslashes to forward slashes for path matching
    normalized = filepath.replace('\\', '/')
    for nuget_dir in NUGET_DIRS:
        if nuget_dir in normalized:
            return True
    return False

def scan_file(filepath: Path) -> list:
    """Scan a C# file for Pattern #231 violations."""
    violations = []

    try:
        content = filepath.read_text(encoding='utf-8')
    except Exception as e:
        print(f"WARN: Could not read {filepath}: {e}", file=sys.stderr)
        return violations

    lines = content.split('\n')

    # Pattern 1: static readonly Foo = <IO>(...)
    static_field_pattern = r'static\s+readonly\s+\w+\s+\w+\s*=\s*(?!new\s+\w+\(\))'

    # Pattern 2: static {  ... } (static constructor)
    static_ctor_pattern = r'static\s*\{'

    in_static_ctor = False
    ctor_depth = 0

    for line_num, line in enumerate(lines, start=1):
        # Check for static constructor entry
        if re.search(static_ctor_pattern, line) and 'static' in line and '{' in line:
            in_static_ctor = True
            ctor_depth = line.count('{') - line.count('}')

        # Track brace depth in static ctor
        if in_static_ctor:
            ctor_depth += line.count('{') - line.count('}')
            if ctor_depth <= 0:
                in_static_ctor = False

        # Check for suppression marker on this line or the immediately preceding line
        prev_line = lines[line_num - 2] if line_num >= 2 else ''
        has_marker = ('static-init-ok' in line) or ('static-init-ok' in prev_line)

        # Check static field initializer
        if re.search(static_field_pattern, line) and not has_marker:
            for io_pattern in IO_PATTERNS:
                if re.search(io_pattern, line):
                    is_nuget = is_nuget_surface(str(filepath))
                    severity = "HIGH" if is_nuget else "MED"
                    violations.append({
                        'file': filepath.relative_to(Path.cwd()),
                        'line': line_num,
                        'severity': severity,
                        'text': line.strip()[:80],
                    })
                    break

        # Check inside static ctor
        if in_static_ctor and not has_marker:
            for io_pattern in IO_PATTERNS:
                if re.search(io_pattern, line):
                    is_nuget = is_nuget_surface(str(filepath))
                    severity = "HIGH" if is_nuget else "MED"
                    violations.append({
                        'file': filepath.relative_to(Path.cwd()),
                        'line': line_num,
                        'severity': severity,
                        'text': line.strip()[:80],
                    })
                    break

    return violations

def main():
    """Main entry point."""
    repo_root = Path.cwd()
    allow = load_allowlist()

    all_violations = defaultdict(list)
    suppressed_count = 0

    for source_dir in SOURCES:
        source_path = repo_root / source_dir
        if not source_path.exists():
            continue

        for cs_file in source_path.rglob("*.cs"):
            # Skip test, bin, obj directories
            if any(part in cs_file.parts for part in ['Tests', 'bin', 'obj', '.git']):
                continue

            violations = scan_file(cs_file)
            for v in violations:
                # Build allowlist key: "<posix-relpath>:<line>"
                rel = str(v['file']).replace('\\', '/')
                key = f"{rel}:{v['line']}"
                if key in allow:
                    suppressed_count += 1
                    continue
                all_violations[v['severity']].append(v)

    if suppressed_count:
        print(f"[detect_static_init_side_effect] suppressed via allowlist: {suppressed_count}")

    # Sort by severity and file
    high_count = len(all_violations['HIGH'])
    med_count = len(all_violations['MED'])

    print(f"\n=== Pattern #231: Static Init Side Effect ===\n")
    print(f"HIGH (NuGet surface): {high_count}")
    print(f"MED (Internal):       {med_count}\n")

    if high_count > 0:
        print("HIGH Violations:")
        for v in sorted(all_violations['HIGH'], key=lambda x: str(x['file'])):
            print(f"  {v['file']}:{v['line']} — {v['text']}")

    if med_count > 0:
        print("\nMED Violations:")
        for v in sorted(all_violations['MED'], key=lambda x: str(x['file']))[:10]:
            print(f"  {v['file']}:{v['line']} — {v['text']}")
        if med_count > 10:
            print(f"  ... and {med_count - 10} more")

    return 1 if high_count > 0 else 0

if __name__ == '__main__':
    sys.exit(main())
