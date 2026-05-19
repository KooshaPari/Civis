#!/usr/bin/env python3
"""
Pattern #117 Detection: StringBuilder Capacity Not Pre-sized.

Detects instances where StringBuilder is constructed with default capacity,
which causes repeated allocations and memory fragmentation when many .Append()
calls are made in a loop. Pre-sizing capacity avoids these reallocs.

Severity levels:
  HIGH:    new StringBuilder() with >10 appends AND a loop (while/for/foreach) in next 30 lines
  MED:     new StringBuilder() with >10 appends, no loop
  LOW:     new StringBuilder() with <=10 appends (acceptable, rarely causes realloc)

Exclusions:
  - StringBuilder(capacity) where capacity is non-zero (already sized)
  - new StringBuilder(string) — string-initialized (pre-sized by ctor)
  - Comments with // stringbuilder-capacity-ok: reason
  - Test files (Tests/)

Output: JSON with violations list, severity breakdown, and allowlist recommendations.
"""

import json
import re
import sys
from pathlib import Path
from dataclasses import dataclass, asdict


@dataclass
class Violation:
    file: str
    line: int
    code_snippet: str
    severity: str  # HIGH, MED, LOW
    reason: str
    append_count: int
    has_loop: bool


@dataclass
class Report:
    total_violations: int
    severity_breakdown: dict
    violations: list
    allowlist_entries: list


def extract_context(file_path: Path, line_num: int, lines: list) -> str:
    """
    Extract class/method context for a violation by walking backwards from line.
    """
    for i in range(line_num - 2, max(0, line_num - 50), -1):
        line = lines[i] if i < len(lines) else ""
        if re.search(r'^\s*(public|private|protected|internal)?\s*class\s+\w+', line):
            match = re.search(r'class\s+(\w+)', line)
            if match:
                return match.group(1)
        if re.search(r'^\s*(public|private|protected|internal)?\s*(async\s+)?[\w<>.]+\s+\w+\s*\(', line):
            match = re.search(r'([\w<>.]+)\s*\(', line)
            if match:
                return f"method: {match.group(1)}"
    return "unknown"


def count_appends_and_loop(file_path: Path, start_line: int, lines: list, scan_ahead: int = 30) -> tuple[int, bool]:
    """
    Count .Append/.AppendLine calls and detect loops in the next N lines.
    Returns (append_count, has_loop).
    """
    append_count = 0
    has_loop = False

    end_line = min(start_line + scan_ahead, len(lines))

    for i in range(start_line, end_line):
        line = lines[i] if i < len(lines) else ""

        # Count appends
        append_count += len(re.findall(r'\.Append(?:Line)?\s*\(', line))

        # Detect loops (while, for, foreach)
        if re.search(r'\b(while|for|foreach)\s*[\s({]', line):
            has_loop = True

    return append_count, has_loop


def classify_severity(append_count: int, has_loop: bool) -> str:
    """
    Classify violation severity.
    """
    if append_count > 10 and has_loop:
        return "HIGH"
    elif append_count > 10:
        return "MED"
    else:
        return "LOW"


def scan_csharp_file(file_path: Path) -> list[Violation]:
    """
    Scan a C# file for StringBuilder capacity violations.
    """
    violations = []

    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
        lines = f.readlines()

    for line_num, line in enumerate(lines):
        # Skip pure comments
        if line.strip().startswith("//"):
            continue

        # Look for new StringBuilder() with NO capacity argument
        # Pattern: new StringBuilder() or new StringBuilder(  ) with only whitespace
        if re.search(r'new\s+StringBuilder\s*\(\s*\)', line):
            # Skip if marked as OK on same line
            if '// stringbuilder-capacity-ok:' in line:
                continue
            # Verify it's not already sized (e.g., new StringBuilder(1024))
            if re.search(r'new\s+StringBuilder\s*\(\s*\d+\s*\)', line):
                continue

            # Verify it's not initialized with a string (e.g., new StringBuilder(str))
            if re.search(r'new\s+StringBuilder\s*\(\s*"', line):
                continue
            if re.search(r'new\s+StringBuilder\s*\(\s*\w+\)', line):
                # This could be a string variable — skip (false positive)
                continue

            # Count appends and detect loops
            append_count, has_loop = count_appends_and_loop(file_path, line_num + 1, lines, scan_ahead=30)
            severity = classify_severity(append_count, has_loop)

            context = extract_context(file_path, line_num, lines)
            reason = f"Default capacity StringBuilder with {append_count} append calls"
            if has_loop:
                reason += " in a loop"

            violations.append(Violation(
                file=str(file_path),
                line=line_num + 1,  # 1-indexed for reporting
                code_snippet=line.strip(),
                severity=severity,
                reason=reason,
                append_count=append_count,
                has_loop=has_loop
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
    Generate recommended allowlist entries for LOW severity violations (acceptable cases).
    """
    entries = []
    for v in violations:
        if v.severity == "LOW":
            entries.append(f"{v.file}:{v.line}  # append_count={v.append_count}, has_loop={v.has_loop}")
    return entries


def main():
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--threshold", type=int, default=5, help="Fail if HIGH violations >= threshold")
    parser.add_argument("--json", action="store_true", help="Output JSON report")
    parser.add_argument("--test", action="store_true", help="Run self-test mode")
    args = parser.parse_args()

    if args.test:
        # Self-test: create temp C# files and scan them
        test_content = """
using System;
using System.Text;

public class BadStringBuilderClass {
    public void ManyAppends() {
        // POSITIVE HIGH: default capacity + 15 appends + loop
        var sb = new StringBuilder();
        for (int i = 0; i < 100; i++) {
            sb.Append("item");
            sb.Append(" ");
            sb.AppendLine("value");
            sb.Append("more");
            sb.Append("data");
            sb.AppendLine("line");
            sb.Append("x");
            sb.AppendLine("y");
            sb.Append("z");
            sb.AppendLine("end");
            sb.Append("extra1");
            sb.Append("extra2");
            sb.Append("extra3");
            sb.Append("extra4");
            sb.Append("extra5");
        }
        return sb.ToString();
    }

    public void ManyAppendsNoLoop() {
        // POSITIVE MED: default capacity + 12 appends, no loop
        var sb = new StringBuilder();
        sb.Append("a");
        sb.Append("b");
        sb.Append("c");
        sb.Append("d");
        sb.Append("e");
        sb.Append("f");
        sb.Append("g");
        sb.Append("h");
        sb.Append("i");
        sb.Append("j");
        sb.Append("k");
        sb.Append("l");
        return sb.ToString();
    }

    public void FewAppends() {
        // POSITIVE LOW: default capacity + 3 appends (acceptable)
        var sb = new StringBuilder();
        sb.Append("x");
        sb.Append("y");
        sb.Append("z");
        return sb.ToString();
    }
}

public class GoodStringBuilderClass {
    public void PreSized() {
        // NEGATIVE: already sized
        var sb = new StringBuilder(1024);
        sb.Append("data");
        return sb.ToString();
    }

    public void StringInitialized() {
        // NEGATIVE: initialized with string
        var sb = new StringBuilder("initial");
        sb.Append("more");
        return sb.ToString();
    }

    public void MarkedOk() {
        var sb = new StringBuilder();  // stringbuilder-capacity-ok: this small builder is acceptable
        sb.Append("x");
        return sb.ToString();
    }
}
"""
        import tempfile
        with tempfile.NamedTemporaryFile(mode='w', suffix='.cs', delete=False) as f:
            f.write(test_content)
            temp_file = f.name

        try:
            violations = scan_csharp_file(Path(temp_file))

            # Expected: 3 positives (1 HIGH + 1 MED + 1 LOW)
            high_count = len([v for v in violations if v.severity == "HIGH"])
            med_count = len([v for v in violations if v.severity == "MED"])
            low_count = len([v for v in violations if v.severity == "LOW"])

            print(f"Self-test: Found {len(violations)} violations (expected 3: 1 HIGH + 1 MED + 1 LOW)")
            print(f"  HIGH: {high_count} (expected 1)")
            print(f"  MED: {med_count} (expected 1)")
            print(f"  LOW: {low_count} (expected 1)")
            for v in violations:
                print(f"    {v.severity} @ line {v.line} - append_count={v.append_count}, has_loop={v.has_loop}")

            if len(violations) == 3 and high_count == 1 and med_count == 1 and low_count == 1:
                print("PASS")
                sys.exit(0)
            else:
                print("FAIL")
                sys.exit(1)
        finally:
            Path(temp_file).unlink()

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
        print(f"StringBuilder Capacity Pattern #117 Scan Results")
        print(f"==============================================")
        print(f"Total violations: {report.total_violations}")
        print(f"HIGH: {report.severity_breakdown['HIGH']}, MED: {report.severity_breakdown['MED']}, LOW: {report.severity_breakdown['LOW']}")
        print()
        for v in sorted([asdict(v) for v in violations], key=lambda x: (x['severity'] != 'HIGH', x['severity'] != 'MED')):
            print(f"{v['file']}:{v['line']} [{v['severity']}]")
            print(f"  {v['reason']}")
            print(f"  {v['code_snippet'][:80]}")

    # Exit code: fail if HIGH violations >= threshold
    if high_count >= args.threshold:
        sys.exit(1)
    sys.exit(0)


if __name__ == "__main__":
    main()
