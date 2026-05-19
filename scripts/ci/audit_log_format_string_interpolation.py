#!/usr/bin/env python3
"""
Pattern #232 Audit: Unstructured Logger String Interpolation Detection

Detects logger calls that use $-string interpolation instead of structured-logging
placeholders. Exempt: explicit inline markers and non-interpolation calls.
"""

import re
import sys
from pathlib import Path
from typing import NamedTuple


class Violation(NamedTuple):
    file: str
    line: int
    text: str
    severity: str


def classify_severity(file_path: Path) -> str:
    """Classify severity by file location."""
    parts = file_path.parts
    if "Runtime" in parts or "SDK" in parts or "Bridge" in parts:
        return "HIGH"
    elif "Tools" in parts:
        return "MED"
    elif "Domains" in parts:
        return "LOW"
    return "LOW"


def audit_file(file_path: Path) -> list[Violation]:
    """Scan file for Pattern #232 violations."""
    violations = []
    try:
        content = file_path.read_text(encoding="utf-8", errors="ignore")
        lines = content.splitlines()
    except Exception:
        return violations

    # Regex: logger call with $ interpolation as first arg
    # Pattern: (_logger|logger|log).(LogTrace|LogDebug|LogInformation|LogWarning|LogError|LogCritical)\s*\(\s*\$
    pattern = re.compile(
        r"(_logger|logger|log)\.(LogTrace|LogDebug|LogInformation|LogWarning|LogError|LogCritical)\s*\(\s*\$"
    )

    for idx, line in enumerate(lines, start=1):
        # Skip exempt markers
        if "log-interpolation-ok:" in line:
            continue

        # Check for match
        if pattern.search(line):
            violations.append(
                Violation(
                    file=str(file_path),
                    line=idx,
                    text=line.strip(),
                    severity=classify_severity(file_path),
                )
            )

    return violations


def main():
    repo_root = Path(__file__).parent.parent.parent
    src_dir = repo_root / "src"

    # Walk source tree
    all_violations = []
    for csharp_file in src_dir.rglob("*.cs"):
        # Skip excluded paths
        if any(x in csharp_file.parts for x in ["bin", "obj", "Tests", "Generated"]):
            continue

        violations = audit_file(csharp_file)
        all_violations.extend(violations)

    # Categorize
    high = [v for v in all_violations if v.severity == "HIGH"]
    med = [v for v in all_violations if v.severity == "MED"]
    low = [v for v in all_violations if v.severity == "LOW"]

    # Sort by file:line for consistent output
    all_violations.sort(key=lambda v: (v.file, v.line))

    # Determine tier
    total = len(all_violations)
    if total < 30:
        tier = "LOW"
    elif total <= 100:
        tier = "MODERATE"
    else:
        tier = "ENDEMIC"

    # Write report
    report_file = repo_root / "docs" / "qa" / "pattern_232_audit.md"
    report_file.parent.mkdir(parents=True, exist_ok=True)

    with open(report_file, "w", encoding="utf-8") as f:
        f.write("# Pattern #232 Audit: Logger String Interpolation\n\n")
        f.write("## Summary\n\n")
        f.write(f"- **Total Violations**: {total}\n")
        f.write(f"- **HIGH**: {len(high)}\n")
        f.write(f"- **MED**: {len(med)}\n")
        f.write(f"- **LOW**: {len(low)}\n")
        f.write(f"- **Tier**: {tier}\n\n")

        f.write("## Top 10 Violations\n\n")
        for v in all_violations[:10]:
            f.write(f"- `{v.file}:{v.line}` [{v.severity}]\n")
            f.write(f"  ```csharp\n")
            f.write(f"  {v.text}\n")
            f.write(f"  ```\n\n")

        if len(all_violations) > 10:
            f.write(f"## All {total} Violations (CSV)\n\n")
            f.write("| File | Line | Severity | Text |\n")
            f.write("|------|------|----------|------|\n")
            for v in all_violations:
                # Escape pipe chars in text
                text_safe = v.text.replace("|", "\\|")[:80]
                f.write(f"| {v.file} | {v.line} | {v.severity} | `{text_safe}...` |\n")

    # Console output
    print(f"Pattern #232 Audit Complete")
    print(f"Total: {total} | HIGH: {len(high)} | MED: {len(med)} | LOW: {len(low)} | Tier: {tier}")
    print(f"Report: {report_file}")


if __name__ == "__main__":
    main()
