#!/usr/bin/env python3
"""
Pattern #115 Detection: HttpClient Per-Instance / Per-Call Anti-Pattern.

Detects instances where HttpClient is created per-request or per-constructor instead of
being reused as a singleton or DI-injected static field. This causes socket exhaustion
and connection pool starvation.

Severity levels:
  HIGH:   Using 'using var http = new HttpClient()' or per-method instantiation
  HIGH:   Per-constructor 'new HttpClient()' where class is instantiated frequently
  MED:    Per-constructor without 'static readonly' singleton fallback
  LOW:    Assigned to 'static readonly' (acceptable, if timeout + headers properly managed)

Output: JSON with violations list, severity breakdown, and allowlist recommendations.
"""

import json
import re
import sys
from pathlib import Path
from typing import TypedDict, Optional
from dataclasses import dataclass, asdict


@dataclass
class Violation:
    file: str
    line: int
    code_snippet: str
    severity: str  # HIGH, MED, LOW
    reason: str


@dataclass
class Report:
    total_violations: int
    severity_breakdown: dict
    violations: list
    allowlist_entries: list


def scan_csharp_file(file_path: Path) -> list[Violation]:
    """
    Scan a C# file for HttpClient per-instance/per-call violations.
    """
    violations = []

    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
        lines = f.readlines()

    for line_num, line in enumerate(lines, 1):
        # Skip comments and allowlisted lines
        if line.strip().startswith("//") or "// http-client-ok:" in line:
            continue

        # Pattern 1: using var http = new HttpClient()
        if re.search(r'using\s+var\s+\w+\s*=\s*new\s+HttpClient\s*[{(]', line):
            violations.append(Violation(
                file=str(file_path),
                line=line_num,
                code_snippet=line.strip(),
                severity="HIGH",
                reason="Per-call HttpClient instantiation in using statement"
            ))
            continue

        # Pattern 2: var http = new HttpClient() in method body
        if re.search(r'var\s+\w+\s*=\s*new\s+HttpClient\s*\(', line) and "static readonly" not in line:
            # Check if in method (simple heuristic: not a field declaration)
            if line.lstrip().startswith(("var ", "_") + tuple()):
                violations.append(Violation(
                    file=str(file_path),
                    line=line_num,
                    code_snippet=line.strip(),
                    severity="HIGH",
                    reason="Per-method HttpClient instantiation"
                ))
                continue

        # Pattern 3: new HttpClient() without static readonly in ctor/method
        if re.search(r'new\s+HttpClient\s*\(', line):
            # Check context: static readonly = LOW, ctor/method = MED/HIGH
            if "static readonly" in line:
                violations.append(Violation(
                    file=str(file_path),
                    line=line_num,
                    code_snippet=line.strip(),
                    severity="LOW",
                    reason="Static readonly HttpClient (acceptable if timeout/headers managed)"
                ))
            elif "public " in line or "private " in line or "protected " in line:
                # Likely a field in constructor param default
                violations.append(Violation(
                    file=str(file_path),
                    line=line_num,
                    code_snippet=line.strip(),
                    severity="MED",
                    reason="Per-instance HttpClient in constructor (creates on each instantiation)"
                ))
            elif re.search(r'(public|private|protected).*\(.*\)', line):
                # In constructor signature
                violations.append(Violation(
                    file=str(file_path),
                    line=line_num,
                    code_snippet=line.strip(),
                    severity="MED",
                    reason="HttpClient default in constructor parameter"
                ))

    return violations


def scan_codebase(src_root: Path) -> list[Violation]:
    """
    Scan src/ excluding Tests/, bin/, obj/.
    """
    all_violations = []

    for cs_file in src_root.rglob("*.cs"):
        # Skip test files, bin, obj
        path_str = str(cs_file)
        if any(skip in path_str.lower() for skip in ["/tests/", "/bin/", "/obj/", "\\tests\\", "\\bin\\", "\\obj\\"]):
            continue

        violations = scan_csharp_file(cs_file)
        all_violations.extend(violations)

    return all_violations


def generate_allowlist_entries(violations: list[Violation]) -> list[str]:
    """
    Generate recommended allowlist entries for violations.
    """
    entries = []
    for v in violations:
        if v.severity == "LOW":
            entries.append(f"{v.file}:{v.line} # {v.reason}")
    return entries


def main():
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--threshold", type=int, default=3, help="Fail if HIGH violations >= threshold")
    parser.add_argument("--json", action="store_true", help="Output JSON report")
    parser.add_argument("--test", action="store_true", help="Run self-test mode")
    args = parser.parse_args()

    if args.test:
        # Self-test: create temp C# file and scan it
        test_content = """
using System.Net.Http;

public class BadClass {
    public BadClass() {
        using var http = new HttpClient { Timeout = TimeSpan.FromSeconds(30) };  // HIGH
    }
}

public class SemiGood {
    public SemiGood(string url) : this(url, new HttpClient()) { }  // MED

    public SemiGood(string url, HttpClient client) { }
}

public class Good {
    private static readonly HttpClient SharedHttp = new();  // LOW (acceptable)
}
"""
        import tempfile
        with tempfile.NamedTemporaryFile(mode='w', suffix='.cs', delete=False) as f:
            f.write(test_content)
            temp_file = f.name

        try:
            violations = scan_csharp_file(Path(temp_file))
            print(f"Self-test: Found {len(violations)} violations (expected 2)")
            for v in violations:
                print(f"  {v.severity}: {v.reason}")
            print("PASS" if len(violations) >= 2 else "FAIL")
        finally:
            Path(temp_file).unlink()
        return

    # Real scan
    repo_root = Path(__file__).parent.parent.parent
    src_root = repo_root / "src"

    violations = scan_codebase(src_root)

    # Categorize
    high_count = len([v for v in violations if v.severity == "HIGH"])
    med_count = len([v for v in violations if v.severity == "MED"])
    low_count = len([v for v in violations if v.severity == "LOW"])

    report = Report(
        total_violations=len(violations),
        severity_breakdown={"HIGH": high_count, "MED": med_count, "LOW": low_count},
        violations=[asdict(v) for v in violations],
        allowlist_entries=generate_allowlist_entries(violations)
    )

    if args.json:
        print(json.dumps(asdict(report), indent=2))
    else:
        print(f"HttpClient Pattern #115 Scan Results")
        print(f"====================================")
        print(f"Total violations: {report['total_violations']}")
        print(f"HIGH: {report['severity_breakdown']['HIGH']}, MED: {report['severity_breakdown']['MED']}, LOW: {report['severity_breakdown']['LOW']}")
        print()
        for v in report['violations']:
            print(f"{v['file']}:{v['line']} [{v['severity']}] {v['reason']}")
            print(f"  {v['code_snippet'][:80]}")

    # Exit code
    if high_count >= args.threshold:
        sys.exit(1)
    sys.exit(0)


if __name__ == "__main__":
    main()
