"""Strip legacy `Epic: auto-generated` and `Stub: TDD-red` docstring lines
from FR test files.

The detection logic in _gather_ids.py classifies a file as STUB-TEST-ONLY if
either marker appears in the first 30 lines. The agent fan-out replaced stub
test bodies but didn't always remove these header markers, so the files
still classify as stubs even though the bodies are real.

Run this idempotently after the stub-fill agents have committed.
"""
import os
import re
import sys

ROOT = sys.argv[1] if len(sys.argv) > 1 else 'crates'
DRY_RUN = '--dry-run' in sys.argv

# Lines to strip from docstring headers.
STUB_LINE_PATTERNS = [
    re.compile(r'^\s*//!.*Epic:\s*auto-generated\s*$'),
    re.compile(r'^\s*//!.*Stub:\s*TDD-red.*$'),
]


def clean(path: str) -> tuple[bool, int]:
    """Return (changed, lines_removed)."""
    with open(path, encoding='utf-8', errors='replace') as f:
        original = f.read()
    lines = original.splitlines(keepends=True)
    new_lines = []
    removed = 0
    for line in lines:
        if any(p.match(line) for p in STUB_LINE_PATTERNS):
            removed += 1
            continue
        new_lines.append(line)
    if removed == 0:
        return False, 0
    new = ''.join(new_lines)
    if not DRY_RUN:
        with open(path, 'w', encoding='utf-8') as f:
            f.write(new)
    return True, removed


def main():
    files = []
    for r, _, fs in os.walk(ROOT):
        for f in fs:
            if f.startswith('fr_fr_') and f.endswith('.rs'):
                files.append(os.path.join(r, f))
    changed = 0
    total_removed = 0
    for p in files:
        c, n = clean(p)
        if c:
            changed += 1
            total_removed += n
    print(f'Inspected: {len(files)}')
    print(f'Changed:   {changed}')
    print(f'Lines removed: {total_removed}')
    if DRY_RUN:
        print('(dry-run)')


if __name__ == '__main__':
    main()
