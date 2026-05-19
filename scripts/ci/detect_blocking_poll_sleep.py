#!/usr/bin/env python3
"""
Pattern #113: Blocking Polling with Hardcoded Sleep Intervals Detection

Detects Thread.Sleep() calls within loop structures (while, for, do) that may
indicate blocking polling without proper synchronization primitives.

Severity levels:
  - HIGH: Thread.Sleep found in loop without any guard condition
  - MED: Thread.Sleep found in loop with guard condition present
  - LOW: Thread.Sleep found outside loop context

Patterns detected:
  - Thread.Sleep(...) within while/for/do loops
  - Missing synchronization checks: _running, IsCancellationRequested, _destroyed, _stopEvent
  - Allowlist entries: path:line format with optional reason comment
"""

import re
import sys
import json
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import Dict, List, Tuple, Set


REPO_ROOT = Path(__file__).parent.parent.parent
SRC_DIR = REPO_ROOT / "src"
ALLOWLIST_FILE = REPO_ROOT / "docs" / "qa" / "blocking-poll-allowlist.txt"

# Guard keywords that indicate safe polling
GUARD_KEYWORDS = {
    "_running",
    "IsCancellationRequested",
    "_destroyed",
    "_stopEvent",
    "pattern-113-ok",
}


@dataclass
class Finding:
    path: str
    line: int
    severity: str  # HIGH, MED, LOW
    snippet: str
    in_loop: bool = False
    guarded: bool = False

    def to_dict(self):
        return asdict(self)


class AllowlistManager:
    def __init__(self, allowlist_path: Path):
        self.allowlist_path = allowlist_path
        self.entries: Set[str] = set()
        self._load()

    def _load(self):
        if not self.allowlist_path.exists():
            return

        with open(self.allowlist_path, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith('#'):
                    continue
                # Format: <file>:<line> # reason
                key = line.split('#')[0].strip()
                if key:
                    self.entries.add(key)

    def is_allowed(self, path: str, line_num: int) -> bool:
        key = f"{path}:{line_num}"
        return key in self.entries


def extract_loop_range(lines: List[str], sleep_line: int) -> Tuple[int, int]:
    """
    Find the loop header (while, for, do) that contains the Thread.Sleep call.
    Returns (loop_start_line, sleep_line) or (-1, -1) if not in loop.
    """
    brace_depth = 0

    # Scan backwards from sleep line to find matching loop header
    for i in range(sleep_line - 1, -1, -1):
        line = lines[i].strip()

        # Count braces (simple heuristic)
        for char in reversed(line):
            if char == '}':
                brace_depth += 1
            elif char == '{':
                brace_depth -= 1
                if brace_depth < 0:
                    # We've exited to an outer scope, check for loop keyword before this brace
                    if re.search(r'(while|for|do)\s*[\(\{]', line):
                        return i, sleep_line
                    brace_depth = 0
                    break

        # Check if this line has a loop keyword
        if re.search(r'(while|for|do)\s*[\(\{]', line):
            return i, sleep_line

    return -1, -1


def has_guard_in_scope(lines: List[str], loop_start: int, sleep_line: int) -> bool:
    """
    Check if the loop scope contains any guard keyword.
    """
    scope_text = "\n".join(lines[loop_start:sleep_line])

    for keyword in GUARD_KEYWORDS:
        if keyword in scope_text:
            return True

    return False


def scan_file(file_path: Path, allowlist: AllowlistManager) -> List[Finding]:
    """Scan a C# file for Thread.Sleep() calls."""
    findings = []

    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"[WARN] Could not read {file_path}: {e}", file=sys.stderr)
        return findings

    relative_path = str(file_path.relative_to(SRC_DIR)).replace("\\", "/")

    for line_idx, line in enumerate(lines, 1):
        # Match Thread.Sleep(...) calls
        match = re.search(r'\bThread\.Sleep\s*\(', line)
        if not match:
            continue

        # Skip if allowlisted
        if allowlist.is_allowed(relative_path, line_idx):
            continue

        # Extract snippet
        snippet = line.strip()[:120]

        # Check if we're in a loop
        loop_start, sleep_line = extract_loop_range(lines, line_idx)
        in_loop = loop_start >= 0

        if in_loop:
            # Check for guards
            guarded = has_guard_in_scope(lines, loop_start, sleep_line)
            severity = "MED" if guarded else "HIGH"
        else:
            severity = "LOW"
            guarded = False

        findings.append(Finding(
            path=relative_path,
            line=line_idx,
            severity=severity,
            snippet=snippet,
            in_loop=in_loop,
            guarded=guarded
        ))

    return findings


def scan_file_with_test_rules(file_path: Path, allowlist: AllowlistManager) -> List[Finding]:
    """
    Scan a C# file with special rules for test fixtures.

    Test fixtures (src/Tests/**/*.cs) have stricter rules:
    - Thread.Sleep(Timeout.Infinite) is ALWAYS HIGH (no marker can exempt)
    - Thread.Sleep(5000+) without guards is HIGH (not just unguarded loop)
    """
    findings = []
    is_test_file = "\\Tests\\" in str(file_path) or "/Tests/" in str(file_path)

    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"[WARN] Could not read {file_path}: {e}", file=sys.stderr)
        return findings

    relative_path = str(file_path.relative_to(SRC_DIR)).replace("\\", "/")

    for line_idx, line in enumerate(lines, 1):
        # Match Thread.Sleep(...) calls
        match = re.search(r'\bThread\.Sleep\s*\((.*?)\)', line)
        if not match:
            continue

        sleep_arg = match.group(1).strip()

        # Skip if allowlisted (except Timeout.Infinite in tests — never allowed)
        if allowlist.is_allowed(relative_path, line_idx):
            if is_test_file and "Timeout.Infinite" in sleep_arg:
                # Thread.Sleep(Timeout.Infinite) in test fixture CANNOT be allowlisted
                pass
            else:
                continue

        # Extract snippet
        snippet = line.strip()[:120]

        # Special rule 1: Thread.Sleep(Timeout.Infinite) is ALWAYS HIGH in test files
        if is_test_file and "Timeout.Infinite" in sleep_arg:
            findings.append(Finding(
                path=relative_path,
                line=line_idx,
                severity="HIGH",
                snippet=snippet,
                in_loop=False,  # Not relevant; it's HIGH regardless
                guarded=False
            ))
            continue

        # Special rule 2: Thread.Sleep(5000+) in test files without guards is HIGH
        if is_test_file:
            # Extract numeric value
            numeric_match = re.search(r'Thread\.Sleep\s*\((\d+)', line)
            if numeric_match:
                sleep_ms = int(numeric_match.group(1))
                if sleep_ms >= 5000:
                    loop_start, _ = extract_loop_range(lines, line_idx)
                    in_loop = loop_start >= 0
                    guarded = has_guard_in_scope(lines, loop_start, line_idx) if in_loop else False
                    severity = "HIGH" if not guarded else "MED"
                    findings.append(Finding(
                        path=relative_path,
                        line=line_idx,
                        severity=severity,
                        snippet=snippet,
                        in_loop=in_loop,
                        guarded=guarded
                    ))
                    continue

        # Standard rules (non-test or test with < 5000ms):
        loop_start, sleep_line = extract_loop_range(lines, line_idx)
        in_loop = loop_start >= 0

        if in_loop:
            guarded = has_guard_in_scope(lines, loop_start, sleep_line)
            severity = "MED" if guarded else "HIGH"
        else:
            severity = "LOW"
            guarded = False

        findings.append(Finding(
            path=relative_path,
            line=line_idx,
            severity=severity,
            snippet=snippet,
            in_loop=in_loop,
            guarded=guarded
        ))

    return findings


def scan_all_files(allowlist: AllowlistManager) -> Tuple[List[Finding], Dict[str, int]]:
    """Scan all C# files in src/ directory (including Tests/ with stricter rules)."""
    all_findings = []
    severity_counts = {"HIGH": 0, "MED": 0, "LOW": 0}

    if not SRC_DIR.exists():
        print(f"[WARN] src directory not found: {SRC_DIR}", file=sys.stderr)
        return all_findings, severity_counts

    # Scan all .cs files
    for cs_file in SRC_DIR.rglob("*.cs"):
        # Skip obj, bin, generated
        path_str = str(cs_file)
        if any(x in path_str for x in ["\\obj\\", "\\bin\\", ".g.cs", ".zig-cache"]):
            continue

        # Use special scanning for test files; standard scanning for others
        if "\\Tests\\" in path_str or "/Tests/" in path_str:
            findings = scan_file_with_test_rules(cs_file, allowlist)
        else:
            findings = scan_file(cs_file, allowlist)

        all_findings.extend(findings)

        for f in findings:
            severity_counts[f.severity] += 1

    return all_findings, severity_counts


def report_violations(findings: List[Finding], severity_counts: Dict[str, int]) -> None:
    """Print human-readable report."""
    if not findings:
        print("[OK] No blocking poll patterns found.")
        return

    print("\nPattern #113: Blocking Polling with Hardcoded Sleep Detection\n")
    print(f"Summary:")
    print(f"  HIGH: {severity_counts['HIGH']}")
    print(f"  MED:  {severity_counts['MED']}")
    print(f"  LOW:  {severity_counts['LOW']}")
    print(f"  TOTAL: {len(findings)}\n")

    # Report HIGH violations
    if severity_counts['HIGH'] > 0:
        print("HIGH (Thread.Sleep in unguarded loop):")
        high_findings = [f for f in findings if f.severity == "HIGH"]
        for f in sorted(high_findings, key=lambda x: (x.path, x.line))[:20]:
            print(f"  {f.path}:{f.line}")
            print(f"    > {f.snippet}")
        if len(high_findings) > 20:
            print(f"  ... and {len(high_findings) - 20} more")
        print()

    # Report MED violations (sample)
    if severity_counts['MED'] > 0:
        print(f"MED (Thread.Sleep in guarded loop):")
        med_findings = [f for f in findings if f.severity == "MED"]
        for f in sorted(med_findings, key=lambda x: (x.path, x.line))[:10]:
            print(f"  {f.path}:{f.line}")
        if len(med_findings) > 10:
            print(f"  ... and {len(med_findings) - 10} more")
        print()

    # Report LOW violations (sample)
    if severity_counts['LOW'] > 0:
        print(f"LOW (Thread.Sleep outside loop):")
        low_findings = [f for f in findings if f.severity == "LOW"]
        for f in sorted(low_findings, key=lambda x: (x.path, x.line))[:5]:
            print(f"  {f.path}:{f.line}")
        if len(low_findings) > 5:
            print(f"  ... and {len(low_findings) - 5} more")
        print()


def run_self_tests() -> bool:
    """Run self-tests on synthetic code samples."""
    print("\n=== Running Self-Tests ===\n")

    tests_passed = 0
    tests_total = 0

    # Test 1: Basic unguarded while loop
    tests_total += 1
    test_lines = [
        "",
        "while (true) {",
        "    Thread.Sleep(100);",
        "}"
    ]
    loop_start, _ = extract_loop_range(test_lines, 3)
    guarded = has_guard_in_scope(test_lines, loop_start, 3) if loop_start >= 0 else False
    severity = "MED" if (loop_start >= 0 and guarded) else ("HIGH" if loop_start >= 0 else "LOW")
    if severity == "HIGH":
        print("[PASS] test_unguarded_while: HIGH")
        tests_passed += 1
    else:
        print(f"[FAIL] test_unguarded_while: expected HIGH, got {severity}")

    # Test 2: Guarded while with _running
    tests_total += 1
    test_lines = [
        "",
        "while (_running) {",
        "    Thread.Sleep(100);",
        "}"
    ]
    loop_start, _ = extract_loop_range(test_lines, 3)
    guarded = has_guard_in_scope(test_lines, loop_start, 3) if loop_start >= 0 else False
    severity = "MED" if (loop_start >= 0 and guarded) else ("HIGH" if loop_start >= 0 else "LOW")
    if severity == "MED":
        print("[PASS] test_guarded_while: MED")
        tests_passed += 1
    else:
        print(f"[FAIL] test_guarded_while: expected MED, got {severity}")

    # Test 3: Sleep outside any loop
    tests_total += 1
    test_lines = [
        "",
        "public void Init() {",
        "    Thread.Sleep(100);",
        "}"
    ]
    loop_start, _ = extract_loop_range(test_lines, 3)
    in_loop = loop_start >= 0
    severity = "LOW" if not in_loop else ("HIGH" if not has_guard_in_scope(test_lines, loop_start, 3) else "MED")
    if severity == "LOW":
        print("[PASS] test_outside_loop: LOW")
        tests_passed += 1
    else:
        print(f"[FAIL] test_outside_loop: expected LOW, got {severity}")

    # Test 4: For loop unguarded
    tests_total += 1
    test_lines = [
        "",
        "for (int i = 0; i < 10; i++) {",
        "    Thread.Sleep(100);",
        "}"
    ]
    loop_start, _ = extract_loop_range(test_lines, 3)
    guarded = has_guard_in_scope(test_lines, loop_start, 3) if loop_start >= 0 else False
    severity = "MED" if (loop_start >= 0 and guarded) else ("HIGH" if loop_start >= 0 else "LOW")
    if severity == "HIGH":
        print("[PASS] test_for_loop: HIGH")
        tests_passed += 1
    else:
        print(f"[FAIL] test_for_loop: expected HIGH, got {severity}")

    # Test 5: CancellationRequested guard
    tests_total += 1
    test_lines = [
        "",
        "while (!ct.IsCancellationRequested) {",
        "    Thread.Sleep(100);",
        "}"
    ]
    loop_start, _ = extract_loop_range(test_lines, 3)
    guarded = has_guard_in_scope(test_lines, loop_start, 3) if loop_start >= 0 else False
    severity = "MED" if (loop_start >= 0 and guarded) else ("HIGH" if loop_start >= 0 else "LOW")
    if severity == "MED":
        print("[PASS] test_cancellation_guard: MED")
        tests_passed += 1
    else:
        print(f"[FAIL] test_cancellation_guard: expected MED, got {severity}")

    # Test 6: Do-while unguarded
    tests_total += 1
    test_lines = [
        "",
        "do {",
        "    Thread.Sleep(50);",
        "} while (true);"
    ]
    loop_start, _ = extract_loop_range(test_lines, 3)
    guarded = has_guard_in_scope(test_lines, loop_start, 3) if loop_start >= 0 else False
    severity = "MED" if (loop_start >= 0 and guarded) else ("HIGH" if loop_start >= 0 else "LOW")
    if severity == "HIGH":
        print("[PASS] test_do_while: HIGH")
        tests_passed += 1
    else:
        print(f"[FAIL] test_do_while: expected HIGH, got {severity}")

    print(f"\nSelf-Test Results: {tests_passed}/{tests_total} passed\n")
    return tests_passed == tests_total


def main():
    # Check for --test flag
    if "--test" in sys.argv:
        success = run_self_tests()
        return 0 if success else 1

    # Check for --json flag
    json_mode = "--json" in sys.argv

    # Check for --threshold flag
    threshold = 8
    if "--threshold" in sys.argv:
        idx = sys.argv.index("--threshold")
        if idx + 1 < len(sys.argv):
            try:
                threshold = int(sys.argv[idx + 1])
            except ValueError:
                print("[ERROR] --threshold requires an integer argument", file=sys.stderr)
                return 1

    # Load allowlist and scan
    allowlist = AllowlistManager(ALLOWLIST_FILE)
    findings, severity_counts = scan_all_files(allowlist)

    if json_mode:
        output = {
            "summary": severity_counts,
            "findings": [f.to_dict() for f in findings]
        }
        print(json.dumps(output, indent=2))
    else:
        report_violations(findings, severity_counts)

    # Gate: fail if HIGH > threshold
    if severity_counts['HIGH'] > threshold:
        if not json_mode:
            print(f"[FAIL] HIGH count ({severity_counts['HIGH']}) exceeds threshold ({threshold})")
        return 1

    if not json_mode:
        print(f"[PASS] HIGH count ({severity_counts['HIGH']}) within threshold ({threshold})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
