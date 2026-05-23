#!/usr/bin/env python3
"""Pattern #226 CI gate.

Pattern #226 says public fields in NuGet-published types should be properties.
This detector scans only the NuGet-published surface requested by the gate:

  * src/SDK/
  * src/Bridge/Protocol/
  * src/Bridge/Client/

It flags public field declarations and excludes:

  * const fields
  * static readonly fields
  * field-targeted attributes such as ``[field: ...]`` / ``[FieldOffset]``
  * inline ``// public-field-ok: ...`` suppressions

Allowlist entries live in ``docs/qa/pattern-226-allowlist.txt`` and may be
either ``HIGH|relative/path.cs|line`` or bare ``relative/path.cs``.
"""
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_ALLOWLIST = REPO_ROOT / "docs" / "qa" / "pattern-226-allowlist.txt"
TARGET_DIRS = (
    REPO_ROOT / "src" / "SDK",
    REPO_ROOT / "src" / "Bridge" / "Protocol",
    REPO_ROOT / "src" / "Bridge" / "Client",
)
EXCLUDED_DIR_PARTS = {"bin", "obj", "Tests", "tests", ".git"}

# Matches a public field declaration with an optional initializer.
PUBLIC_FIELD_RE = re.compile(
    r"\bpublic\s+"
    r"(?P<modifiers>(?:(?:static|readonly|const|required|volatile|unsafe|new|extern)\s+)*)"
    r"(?P<type>[^;{}()=]+?)\s+"
    r"(?P<name>[A-Za-z_]\w*)"
    r"(?:\s*=\s*(?!>)[^;]+)?\s*;"
)

PUBLIC_FIELD_OK_RE = re.compile(r"//\s*public-field-ok\b", re.IGNORECASE)
FIELD_ATTR_RE = re.compile(r"\[\s*(?:field\s*:)?[^\]]*Field[^\]]*\]", re.IGNORECASE)


@dataclass(frozen=True)
class Finding:
    file: str
    line: int
    text: str


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def load_allowlist(path: Path) -> set[str]:
    if not path.exists():
        return set()

    entries: set[str] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        line = line.split("#", 1)[0].strip().replace("\\", "/")
        if line:
            entries.add(line)
    return entries


def is_target_path(path: Path) -> bool:
    rel = path.relative_to(REPO_ROOT).as_posix()
    return any(rel.startswith(prefix) for prefix in (
        "src/SDK/",
        "src/Bridge/Protocol/",
        "src/Bridge/Client/",
    ))


def is_excluded(path: Path) -> bool:
    return bool(set(path.parts) & EXCLUDED_DIR_PARTS)


def is_field_attribute_block(lines: list[str], idx: int) -> bool:
    """Return True if the declaration is decorated with a field-targeted attribute."""
    start = max(0, idx - 3)
    for prev in lines[start:idx]:
        stripped = prev.strip()
        if not stripped:
            continue
        if PUBLIC_FIELD_OK_RE.search(stripped):
            continue
        if FIELD_ATTR_RE.search(stripped):
            return True
    return False


def is_violation_line(line: str) -> bool:
    stripped = line.strip()
    if not stripped or stripped.startswith("//") or stripped.startswith("/*") or stripped.startswith("*"):
        return False
    if PUBLIC_FIELD_OK_RE.search(stripped):
        return False
    if re.search(r"\b(?:event|class|struct|interface|enum|delegate)\b", stripped):
        return False

    match = PUBLIC_FIELD_RE.search(stripped)
    if not match:
        return False

    modifiers = match.group("modifiers") or ""
    if re.search(r"\bconst\b", modifiers):
        return False
    if re.search(r"\bstatic\b", modifiers) and re.search(r"\breadonly\b", modifiers):
        return False

    return True


def scan_file(path: Path) -> list[Finding]:
    text = read_text(path)
    if not text:
        return []

    lines = text.splitlines()
    rel = path.relative_to(REPO_ROOT).as_posix()
    findings: list[Finding] = []

    for idx, line in enumerate(lines):
        if not is_violation_line(line):
            continue

        match = PUBLIC_FIELD_RE.search(line)
        if not match:
            continue
        if FIELD_ATTR_RE.search(line[: match.start()]):
            continue
        if is_field_attribute_block(lines, idx):
            continue

        findings.append(
            Finding(
                file=rel,
                line=idx + 1,
                text=line.strip(),
            )
        )

    return findings


def is_allowlisted(rel_path: str, line_no: int, allowlist: set[str]) -> bool:
    candidates = (
        f"HIGH|{rel_path}|{line_no}",
        f"{rel_path}|{line_no}",
        f"{rel_path}:{line_no}",
        rel_path,
    )
    return any(candidate in allowlist for candidate in candidates)


def iter_source_files() -> list[Path]:
    files: list[Path] = []
    for root in TARGET_DIRS:
        if not root.exists():
            continue
        for path in root.rglob("*.cs"):
            if is_excluded(path):
                continue
            files.append(path)
    return sorted(files)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Pattern #226: public fields in NuGet-published types should be properties"
    )
    parser.add_argument(
        "--allowlist",
        type=Path,
        default=DEFAULT_ALLOWLIST,
        help="Allowlist file (default: docs/qa/pattern-226-allowlist.txt)",
    )
    args = parser.parse_args()

    allowlist = load_allowlist(args.allowlist)
    findings: list[Finding] = []
    allowlisted = 0

    for path in iter_source_files():
        if not is_target_path(path):
            continue
        for finding in scan_file(path):
            if is_allowlisted(finding.file, finding.line, allowlist):
                allowlisted += 1
                continue
            findings.append(finding)

    if findings:
        print(f"Pattern #226: {len(findings)} HIGH violation(s)")
        for finding in findings:
            print(f"  {finding.file}:{finding.line} [HIGH] {finding.text}")
    else:
        print("Pattern #226: 0 HIGH violation(s)")

    if allowlisted:
        print(f"Allowlisted: {allowlisted}")
    print(f"Total HIGH: {len(findings)}")

    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
