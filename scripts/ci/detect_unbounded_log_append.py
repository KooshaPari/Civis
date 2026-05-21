#!/usr/bin/env python3
"""
Pattern #232 enforcement: File.AppendAllText() calls must be preceded by a
size-check and rotation pattern within 30 lines.

Detects unbounded log-append patterns across the production source tree:
- File.AppendAllText(  / File.AppendAllLines(
- new StreamWriter(path, append: true)  / new StreamWriter(path, true)
- FileMode.Append (FileStream / File.Open)
- FileInfo.AppendText()  / File.AppendText()

Scope: src/Runtime/, src/Bridge/, src/SDK/, src/Tools/, src/Domains/
Extension (#708 audit, 2026-05-21): now also covers AppendAllLines,
StreamWriter append-mode, FileMode.Append, AppendText — previously only
File.AppendAllText was detected.

Each match is checked against the preceding 30 lines for a rotation guard
(file size check + rename/delete/move). Unguarded => HIGH violation.

Threshold: HIGH > 0 fails CI.
Allowlist: docs/qa/pattern-232-log-rotation-allowlist.txt (file:line format)
Inline marker: `// unbounded-log-ok: <reason>` on the offending line.
"""
import sys
import re
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent
SRC = REPO / "src"
SCOPES = ["Runtime", "Bridge", "SDK", "Tools", "Domains"]

# Patterns for unbounded log-append call sites.
APPEND_PATTERNS = [
    ("AppendAllText",   re.compile(r"File\.AppendAllText\s*\(")),
    ("AppendAllLines",  re.compile(r"File\.AppendAllLines\s*\(")),
    # StreamWriter with append=true (named or positional bool after path).
    ("StreamWriterAppend", re.compile(
        r"new\s+StreamWriter\s*\([^)]*?(?:append\s*:\s*true|,\s*true\b)"
    )),
    ("FileModeAppend",  re.compile(r"FileMode\.Append\b")),
    ("AppendText",      re.compile(r"\.AppendText\s*\(")),
]

# Inline suppression marker.
INLINE_OK = re.compile(r"//\s*unbounded-log-ok\s*:")

# Rotation guard patterns: size check or rename/overwrite
ROTATION_GUARD = re.compile(
    r"(FileInfo|File).*Length.*>=|"
    r"\.Length\s*>=.*Max|"
    r"File\.Move\(|"
    r"File\.Delete\(|"
    r"\.Rename\(|"
    r"Directory\.GetFiles.*\.OrderByDescending|"
    r"keep[^=]*<|"
    r"rotate|"
    r"max.*file.*size",
    re.IGNORECASE
)

def load_allowlist(allowlist_path):
    """Load allowlist from file (file:line format, skip blanks and #comments)."""
    if not allowlist_path.exists():
        return set()
    try:
        with open(allowlist_path, 'r') as f:
            lines = f.readlines()
        patterns = set()
        for line in lines:
            line = line.strip()
            if line and not line.startswith('#'):
                patterns.add(line)
        return patterns
    except Exception:
        return set()

def check_allowlist(file_path, line_num, allowlist):
    """Check if file:line is in allowlist."""
    try:
        rel_path = file_path.relative_to(REPO)
        # Normalize path separators to forward slashes for consistency
        key = str(rel_path).replace('\\', '/')
        key_with_line = f"{key}:{line_num}"
        if key_with_line in allowlist:
            return True
        # Also try with backslashes (Windows format in allowlist)
        key_backslash = str(rel_path)
        key_backslash_line = f"{key_backslash}:{line_num}"
        if key_backslash_line in allowlist:
            return True
    except ValueError:
        pass
    return False

def check_file(cs_file):
    """
    Check a .cs file for unguarded log-append calls.
    Return list of (line_number, severity, kind) tuples.
    """
    content = cs_file.read_text(encoding="utf-8", errors="replace")
    lines = content.split('\n')
    violations = []

    for line_idx, line in enumerate(lines):
        matched_kind = None
        for kind, pat in APPEND_PATTERNS:
            if pat.search(line):
                matched_kind = kind
                break
        if matched_kind is None:
            continue

        # Inline suppression marker on the same line.
        if INLINE_OK.search(line):
            continue

        # Check preceding 30 lines for a rotation guard.
        start = max(0, line_idx - 30)
        preceding = '\n'.join(lines[start:line_idx])

        if not ROTATION_GUARD.search(preceding):
            violations.append((line_idx + 1, "HIGH", matched_kind))

    return violations

def main():
    allowlist_path = REPO / "docs" / "qa" / "pattern-232-log-rotation-allowlist.txt"
    allowlist = load_allowlist(allowlist_path)

    violations = []
    allowlisted = 0

    for scope in SCOPES:
        scope_dir = SRC / scope
        if not scope_dir.exists():
            continue

        for cs_file in scope_dir.rglob("*.cs"):
            file_violations = check_file(cs_file)
            for line_no, severity, kind in file_violations:
                if check_allowlist(cs_file, line_no, allowlist):
                    allowlisted += 1
                else:
                    violations.append((cs_file.relative_to(REPO), line_no, severity, kind))

    if violations:
        high_count = sum(1 for v in violations if v[2] == "HIGH")
        med_count = len(violations) - high_count

        print(f"Pattern #232 violations ({high_count} HIGH, {med_count} MED, {allowlisted} allowlisted):")
        for path, line_no, sev, kind in violations:
            print(f"  {sev}  {path}:{line_no}: [{kind}] unbounded log-append without size check/rotation guard")

        sys.exit(1 if high_count > 0 else 0)

    print(f"Pattern #232: 0 violations ({allowlisted} allowlisted).")
    sys.exit(0)

if __name__ == "__main__":
    main()
