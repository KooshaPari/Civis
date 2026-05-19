#!/usr/bin/env python3
"""
Pattern #226 Audit: Public Mutable Fields in Production Code

Detects public field declarations that lack encapsulation via properties,
excluding const, readonly static, interop, and serialization fields.

Severity classification:
- HIGH: SDK, Bridge.Protocol, Bridge.Client (NuGet-published, binary-compat risk)
- MED: Runtime, Domains (internal but public)
- LOW: Tools (CLI internal)
"""

import re
import os
import csv
from pathlib import Path
from collections import defaultdict

# Configuration
ROOT = Path("C:/Users/koosh/Dino")
SRC_DIR = ROOT / "src"
EXCLUDE_DIRS = {"bin", "obj", "Tests", "Generated", ".git"}

# Regex for public field declarations
# Matches: public [static|readonly] TYPE NAME [; = ...]
FIELD_PATTERN = re.compile(
    r"^\s*public(?:\s+(?:static|readonly|const))?\s+(?:(?:static|readonly)\s+)?[\w<>?,\s]+\s+(\w+)\s*[;=]",
    re.MULTILINE
)

# Exclusion markers
EXCLUSION_PATTERNS = [
    r"const\s+",           # const fields
    r"readonly\s+static",  # readonly static singletons
    r"\[FieldOffset",      # interop
    r"\[StructLayout",     # interop/serialization
    r"public-field-ok:",   # inline allowlist marker
]

def should_exclude(line_content):
    """Check if line should be excluded from audit."""
    for pattern in EXCLUSION_PATTERNS:
        if re.search(pattern, line_content):
            return True
    return False

def classify_severity(file_path):
    """Classify severity based on file location."""
    relative = file_path.relative_to(SRC_DIR).as_posix()
    if any(x in relative for x in ["SDK/", "Bridge/Protocol/", "Bridge/Client/"]):
        return "HIGH"
    elif any(x in relative for x in ["Runtime/", "Domains/"]):
        return "MED"
    elif "Tools/" in relative:
        return "LOW"
    return "LOW"

def audit_file(file_path):
    """Audit a single .cs file for public fields."""
    violations = []
    try:
        with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()

        for line_num, line in enumerate(lines, 1):
            # Skip excluded patterns
            if should_exclude(line):
                continue

            # Skip expression-bodied properties (public TYPE NAME => ...)
            if "=>" in line:
                continue

            # Skip properties with get/set accessors (public TYPE NAME { get; ... })
            if "{" in line or "get;" in line or "set;" in line or "init;" in line:
                continue

            # Match public field (non-property, non-method declaration)
            # Must end with ; or = (field assignment/declaration)
            match = re.search(r"^\s*public\s+(?!static\s+readonly|const\s)[\w<>?,\s]+\s+(\w+)\s*[;=]", line)
            if match:
                field_name = match.group(1)
                severity = classify_severity(file_path)
                violations.append({
                    "file": str(file_path.relative_to(SRC_DIR)),
                    "line": line_num,
                    "severity": severity,
                    "field": field_name,
                    "context": line.strip()[:80]  # First 80 chars of context
                })
    except Exception as e:
        print(f"Error reading {file_path}: {e}")

    return violations

def main():
    """Main audit loop."""
    all_violations = []
    severity_count = defaultdict(int)
    file_heat = defaultdict(int)

    # Walk src/ excluding unwanted directories
    for root, dirs, files in os.walk(SRC_DIR):
        # In-place filter to skip excluded directories
        dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]

        for file in files:
            if not file.endswith(".cs"):
                continue

            file_path = Path(root) / file
            violations = audit_file(file_path)

            if violations:
                all_violations.extend(violations)
                file_heat[violations[0]["file"].split("/")[0]] += len(violations)

                for v in violations:
                    severity_count[v["severity"]] += 1

    # Sort by severity (HIGH first) then by file
    all_violations.sort(key=lambda x: ({"HIGH": 0, "MED": 1, "LOW": 2}[x["severity"]], x["file"], x["line"]))

    # Write CSV
    output_csv = ROOT / "docs/qa/pattern_226_audit.csv"
    output_csv.parent.mkdir(parents=True, exist_ok=True)

    with open(output_csv, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=["file", "line", "severity", "field", "context"])
        writer.writeheader()
        writer.writerows(all_violations)

    # Write markdown report
    output_md = ROOT / "docs/qa/pattern_226_audit.md"

    with open(output_md, "w", encoding="utf-8") as f:
        f.write("# Pattern #226 Audit: Public Mutable Fields\n\n")
        f.write("**Audit Date**: 2026-05-18\n\n")
        f.write("## Detection Script\n\n")
        f.write("- **Path**: `scripts/ci/audit_public_fields.py`\n")
        f.write("- **LOC**: 137\n")
        f.write("- **Exclusions**: const, readonly static, [FieldOffset], [StructLayout], // public-field-ok: marker\n\n")

        f.write("## Summary\n\n")
        f.write(f"**Total Violations**: {len(all_violations)}\n\n")
        f.write("### Severity Breakdown\n")
        f.write(f"- **HIGH** (NuGet-published): {severity_count['HIGH']}\n")
        f.write(f"- **MED** (Internal but public): {severity_count['MED']}\n")
        f.write(f"- **LOW** (Tools/CLI): {severity_count['LOW']}\n\n")

        f.write("### Directory Heat Map\n")
        for dir_name in sorted(file_heat.keys(), key=lambda x: file_heat[x], reverse=True):
            f.write(f"- `src/{dir_name}/`: {file_heat[dir_name]} violations\n")
        f.write("\n")

        f.write("## Top 15 Violations\n\n")
        f.write("| File | Line | Severity | Field | Context |\n")
        f.write("|------|------|----------|-------|----------|\n")
        for v in all_violations[:15]:
            context = v["context"].replace("|", "\\|")
            f.write(f"| `{v['file']}` | {v['line']} | {v['severity']} | `{v['field']}` | {context} |\n")
        f.write("\n")

        f.write("## Tier Classification\n\n")
        high_count = severity_count.get("HIGH", 0)
        total = len(all_violations)

        if high_count < 10 and total < 50:
            tier = "**LOW** — Fix-as-touched. Encapsulation smell but not urgent. Enable as lint rule post-release."
        elif high_count < 30 and total < 150:
            tier = "**MODERATE** — Sweep before next NuGet release (v0.25.0). Promote to CI lint with allowlist."
        else:
            tier = "**ENDEMIC** — Promote to Pattern Catalog with deferred remediation plan (DF1018). Add CI gate with HIGH > 10 threshold."

        f.write(f"{tier}\n\n")

        f.write("## Recommendation\n\n")
        if high_count > 0:
            f.write(f"**{high_count} HIGH violations in NuGet-published assemblies require remediation before v0.25.0 release.**\n")
            f.write("Create internal properties (`public T Field { get; set; }`) with backing fields (`private T _field`).\n")
            f.write("For immutable data structures, use `init` accessor instead of `set`.\n")
        else:
            f.write("No NuGet-published violations. MED/LOW violations can be fixed opportunistically.\n")

    # Print summary
    print(f"[OK] Audit complete: {len(all_violations)} violations found")
    print(f"  - HIGH: {severity_count['HIGH']}")
    print(f"  - MED: {severity_count['MED']}")
    print(f"  - LOW: {severity_count['LOW']}")
    print(f"\n[OK] Results saved:")
    print(f"  - CSV: {output_csv}")
    print(f"  - Markdown: {output_md}")

if __name__ == "__main__":
    main()
