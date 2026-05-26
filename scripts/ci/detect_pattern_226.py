#!/usr/bin/env python3
"""Pattern #226 drift gate — public fields in NuGet-published types.

Pattern #226 was RETIRED at iter-133 (#494) after the JsonRpcMessage
public-field → property migration cleared 34 HIGH violations to zero. The
migration is fragile: any new ``public string Foo;`` declaration in a
NuGet-published assembly silently re-introduces the pattern.

This detector is a drift gate. Threshold is zero — ANY new HIGH violation
re-opens the pattern.

Scans the NuGet-published surface:

  * src/SDK/
  * src/Bridge/Protocol/
  * src/Bridge/Client/
  * src/Domains/

Excludes:

  * const fields
  * static readonly fields
  * field-targeted attributes such as ``[field: ...]`` / ``[FieldOffset]``
  * inline ``// pattern-226-ok: <reason>`` (or legacy ``// public-field-ok:``)

Allowlist entries live in ``docs/qa/pattern-226-allowlist.txt`` and may be
either ``HIGH|relative/path.cs|line`` or bare ``relative/path.cs``.

JSON report is written to ``docs/qa/pattern-226-report.json`` mirroring the
peer-detector schema.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_ALLOWLIST = REPO_ROOT / "docs" / "qa" / "pattern-226-allowlist.txt"
DEFAULT_REPORT = REPO_ROOT / "docs" / "qa" / "pattern-226-report.json"
TARGET_DIRS = (
    REPO_ROOT / "src" / "SDK",
    REPO_ROOT / "src" / "Bridge" / "Protocol",
    REPO_ROOT / "src" / "Bridge" / "Client",
    REPO_ROOT / "src" / "Domains",
)
NUGET_SURFACE_PREFIXES = (
    "src/SDK/",
    "src/Bridge/Protocol/",
    "src/Bridge/Client/",
    "src/Domains/",
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

PUBLIC_FIELD_OK_RE = re.compile(
    r"//\s*(?:pattern-226-ok|public-field-ok)\b", re.IGNORECASE
)
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
    return any(rel.startswith(prefix) for prefix in NUGET_SURFACE_PREFIXES)


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
        description=(
            "Pattern #226 drift gate: detect public mutable fields in "
            "NuGet-published assemblies. Pattern is RETIRED; ANY HIGH "
            "violation re-opens it."
        )
    )
    parser.add_argument(
        "--allowlist",
        type=Path,
        default=DEFAULT_ALLOWLIST,
        help="Allowlist file (default: docs/qa/pattern-226-allowlist.txt)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_REPORT,
        help="JSON report path (default: docs/qa/pattern-226-report.json)",
    )
    parser.add_argument(
        "--threshold",
        type=int,
        default=0,
        help="Max unallowlisted HIGH findings before CI fails (default: 0)",
    )
    args = parser.parse_args()

    allowlist = load_allowlist(args.allowlist)
    findings: list[Finding] = []
    allowlisted: list[Finding] = []
    files_scanned = 0

    for path in iter_source_files():
        if not is_target_path(path):
            continue
        files_scanned += 1
        for finding in scan_file(path):
            if is_allowlisted(finding.file, finding.line, allowlist):
                allowlisted.append(finding)
                continue
            findings.append(finding)

    high_count = len(findings)
    med_count = 0

    def _f2d(f: Finding) -> dict:
        return {
            "file": f.file,
            "line": f.line,
            "severity": "HIGH",
            "rule": "public_mutable_field_nuget",
            "line_excerpt": f.text[:200],
            "allowlist_key": f"HIGH|{f.file}|{f.line}",
        }

    fail = high_count > args.threshold
    report = {
        "scan_utc": datetime.utcnow().isoformat() + "Z",
        "scan_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "pattern": "226",
        "pattern_status": "RETIRED (drift gate)",
        "scan_paths": list(NUGET_SURFACE_PREFIXES),
        "files_scanned": files_scanned,
        "total_hits": high_count + len(allowlisted),
        "new_hits": high_count,
        "allowlist_size": len(allowlist),
        "allowlist_path": args.allowlist.as_posix(),
        "threshold": args.threshold,
        "high_count": high_count,
        "med_count": med_count,
        "public_mutable_field_nuget": [_f2d(f) for f in findings],
        "allowlisted_hits": [_f2d(f) for f in allowlisted],
        "exit_code": 1 if fail else 0,
    }

    try:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2), encoding="utf-8")
    except OSError as exc:
        print(f"WARNING: failed to write JSON report {args.output}: {exc}", file=sys.stderr)

    if findings:
        print(
            f"pattern-226 drift gate: {high_count} HIGH, {med_count} MED — "
            f"DRIFT DETECTED (Pattern #226 is RETIRED)"
        )
        for finding in findings:
            print(f"  {finding.file}:{finding.line} [HIGH] {finding.text}")
    else:
        print(
            f"pattern-226 drift gate: {high_count} HIGH, {med_count} MED — "
            f"drift-gate CLEAN"
        )

    if allowlisted:
        print(f"Allowlisted: {len(allowlisted)}")
    print(f"JSON report: {args.output}")

    return 1 if fail else 0


if __name__ == "__main__":
    sys.exit(main())
