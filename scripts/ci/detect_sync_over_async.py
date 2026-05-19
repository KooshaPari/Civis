#!/usr/bin/env python3
"""
Pattern #116 Detection: Sync-over-Async Blocking Anti-Pattern.

Detects instances where async methods are blocked synchronously using .Result or .Wait(),
which can cause deadlocks, thread pool starvation, and UI freezes. Identifies violations
by severity level and flags critical contexts (GameBridgeServer, SystemBase, ECS, etc.)
where sync-over-async is especially dangerous.

Severity levels:
  CRITICAL: In GameBridgeServer, SystemBase subclass, MainThreadDispatcher, or EcsTypeDiscovery
  HIGH:     In any other src/ file (catch block, method body, property)
  MEDIUM:   File/class already marked with sync-over-async-unavoidable comment nearby

Exclusions:
  - .ResultType, .ResultSummary, .Results (property access, not Task.Result)
  - response.Result (DTO property access)
  - Normal await expressions
  - Comments with // sync-over-async-ok: reason

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
    severity: str  # CRITICAL, HIGH, MEDIUM
    reason: str
    context: str  # class name or method context


@dataclass
class Report:
    total_violations: int
    severity_breakdown: dict
    violations: list
    allowlist_entries: list


def is_false_positive(line: str) -> bool:
    """
    Check if line contains a false positive (property access, not Task.Result/.Wait()).
    """
    # Exclude property accesses: .ResultType, .ResultSummary, .Results (plural)
    if re.search(r'\.(ResultType|ResultSummary|Results|Result\[)\b', line):
        return True
    # Exclude DTO response.Result pattern
    if re.search(r'\w+\.Result\b', line) and 'response' in line.lower():
        return True
    # Exclude if marked as OK inline
    if '// sync-over-async-ok:' in line:
        return True
    return False


def extract_context(file_path: Path, line_num: int, lines: list) -> str:
    """
    Extract class/method context for a violation by walking backwards from line.
    """
    # Walk backwards to find class or method
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


def classify_severity(file_path: Path, class_context: str, has_unavoidable_marker: bool) -> str:
    """
    Classify violation severity based on context.
    """
    path_str = str(file_path)

    # CRITICAL: GameBridgeServer, SystemBase, MainThreadDispatcher, EcsTypeDiscovery
    if any(x in path_str for x in ['GameBridgeServer', 'MainThreadDispatcher', 'EcsTypeDiscovery']):
        return "CRITICAL"
    if 'SystemBase' in class_context:
        return "CRITICAL"

    # MEDIUM: Already marked as unavoidable
    if has_unavoidable_marker:
        return "MEDIUM"

    # HIGH: Default for all other src/ files
    return "HIGH"


def scan_csharp_file(file_path: Path) -> list[Violation]:
    """
    Scan a C# file for sync-over-async blocking violations (.Result, .Wait()).
    """
    violations = []

    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
        lines = f.readlines()

    for line_num, line in enumerate(lines, 1):
        # Skip pure comments
        if line.strip().startswith("//"):
            continue

        # Skip if false positive
        if is_false_positive(line):
            continue

        # Check for .Result (but not property access)
        if re.search(r'\.Result\b(?!Type|Summary|s\b)', line):
            # Verify it's Task.Result, not DTO property
            if not any(x in line for x in ['.ResultType', '.ResultSummary', '.Results']):
                # Check for unavoidable marker nearby
                has_marker = False
                for i in range(max(0, line_num - 4), min(len(lines), line_num + 1)):
                    if 'sync-over-async-unavoidable:' in lines[i]:
                        has_marker = True
                        break

                class_context = extract_context(file_path, line_num, lines)
                severity = classify_severity(file_path, class_context, has_marker)

                violations.append(Violation(
                    file=str(file_path),
                    line=line_num,
                    code_snippet=line.strip(),
                    severity=severity,
                    reason="Synchronous block on async Task (.Result)",
                    context=class_context
                ))
                continue

        # Check for .Wait() on Task/SemaphoreSlim
        if re.search(r'\.Wait\s*\(', line):
            # Verify it's on a Task, not a different method
            if not re.search(r'\.Result\b', line):  # Avoid double-counting
                # Check for unavoidable marker nearby
                has_marker = False
                for i in range(max(0, line_num - 4), min(len(lines), line_num + 1)):
                    if 'sync-over-async-unavoidable:' in lines[i]:
                        has_marker = True
                        break

                class_context = extract_context(file_path, line_num, lines)
                severity = classify_severity(file_path, class_context, has_marker)

                violations.append(Violation(
                    file=str(file_path),
                    line=line_num,
                    code_snippet=line.strip(),
                    severity=severity,
                    reason="Synchronous block on async method (.Wait())",
                    context=class_context
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
    Generate recommended allowlist entries for MEDIUM violations (unavoidable cases).
    """
    entries = []
    for v in violations:
        if v.severity == "MEDIUM":
            entries.append(f"{v.file}:{v.line}  # {v.context}: {v.reason}")
    return entries


def main():
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--threshold", type=int, default=5, help="Fail if HIGH+CRITICAL violations >= threshold")
    parser.add_argument("--json", action="store_true", help="Output JSON report")
    parser.add_argument("--test", action="store_true", help="Run self-test mode")
    args = parser.parse_args()

    if args.test:
        # Self-test: create temp C# files and scan them
        test_content = """
using System;
using System.Threading;
using System.Threading.Tasks;

public class BadSyncClass {
    public void SyncMethod() {
        // POSITIVE: .Result blocking
        var task = GetDataAsync();
        var result = task.Result;  // should trigger HIGH
    }

    public void WaitMethod() {
        // POSITIVE: .Wait() blocking
        var task = GetDataAsync();
        task.Wait();  // should trigger HIGH
    }

    private async Task<string> GetDataAsync() {
        return await Task.FromResult("data");
    }
}

public class WithMarker {
    // sync-over-async-unavoidable: legacy API compatibility
    public void LegacySync() {
        var task = GetDataAsync();
        var result = task.Result;  // should trigger MEDIUM (marked)
    }

    private async Task<string> GetDataAsync() {
        return await Task.FromResult("data");
    }
}

public class FalsePositives {
    public void PropertyAccess() {
        // NEGATIVE: property access, not Task.Result
        var obj = GetObject();
        var type = obj.ResultType;  // should NOT trigger
        var summary = obj.ResultSummary;  // should NOT trigger
        var results = obj.Results;  // should NOT trigger
    }

    public void ResponseDto() {
        // NEGATIVE: DTO property access
        var response = new ApiResponse();
        var data = response.Result;  // should NOT trigger (response.Result pattern)
    }

    private class ApiResponse {
        public string Result { get; set; }
    }

    private object GetObject() {
        return null;
    }
}
"""
        import tempfile
        with tempfile.NamedTemporaryFile(mode='w', suffix='.cs', delete=False) as f:
            f.write(test_content)
            temp_file = f.name

        try:
            violations = scan_csharp_file(Path(temp_file))

            # Expected: 3 positives (2 HIGH from BadSyncClass, 1 MEDIUM from WithMarker)
            # False positives should NOT appear
            high_count = len([v for v in violations if v.severity == "HIGH"])
            med_count = len([v for v in violations if v.severity == "MEDIUM"])

            print(f"Self-test: Found {len(violations)} violations (expected 3: 2 HIGH + 1 MEDIUM)")
            print(f"  HIGH: {high_count} (expected 2)")
            print(f"  MEDIUM: {med_count} (expected 1)")
            for v in violations:
                print(f"    {v.severity} @ {v.file.split('/')[-1]}:{v.line} - {v.reason}")

            if len(violations) == 3 and high_count == 2 and med_count == 1:
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
    critical_count = len([v for v in violations if v.severity == "CRITICAL"])
    high_count = len([v for v in violations if v.severity == "HIGH"])
    med_count = len([v for v in violations if v.severity == "MEDIUM"])

    report = Report(
        total_violations=len(violations),
        severity_breakdown={"CRITICAL": critical_count, "HIGH": high_count, "MEDIUM": med_count},
        violations=[asdict(v) for v in violations],
        allowlist_entries=generate_allowlist_entries(violations)
    )

    if args.json:
        print(json.dumps(asdict(report), indent=2))
    else:
        print(f"Sync-over-Async Pattern #116 Scan Results")
        print(f"==========================================")
        print(f"Total violations: {report.total_violations}")
        print(f"CRITICAL: {report.severity_breakdown['CRITICAL']}, HIGH: {report.severity_breakdown['HIGH']}, MEDIUM: {report.severity_breakdown['MEDIUM']}")
        print()
        for v in sorted([asdict(v) for v in violations], key=lambda x: (x['severity'] != 'CRITICAL', x['severity'] != 'HIGH')):
            print(f"{v['file']}:{v['line']} [{v['severity']}] {v['context']}")
            print(f"  {v['reason']}")
            print(f"  {v['code_snippet'][:80]}")

    # Exit code: fail if HIGH + CRITICAL >= threshold
    combined_high_critical = high_count + critical_count
    if combined_high_critical >= args.threshold:
        sys.exit(1)
    sys.exit(0)


if __name__ == "__main__":
    main()
