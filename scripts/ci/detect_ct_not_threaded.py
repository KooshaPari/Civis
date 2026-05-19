#!/usr/bin/env python3
"""
Pattern #114: CancellationToken Accepted But Not Threaded Detection

Detects async methods that accept a CancellationToken parameter but do not pass it
to any awaited calls within the method. This indicates incomplete cancellation support.
"""

import re
import sys
import json
from pathlib import Path


def extract_method_body(lines, start_idx):
    """Extract method body given the line where signature starts."""
    brace_depth = 0
    body_start = None

    # Find opening brace of method
    for i in range(start_idx, min(start_idx + 10, len(lines))):
        if '{' in lines[i]:
            body_start = i
            brace_depth = lines[i].count('{') - lines[i].count('}')
            break

    if body_start is None:
        return []

    # Collect lines until balanced braces
    body_lines = []
    for i in range(body_start + 1, len(lines)):
        line = lines[i]
        brace_depth += line.count('{') - line.count('}')
        body_lines.append(line)
        if brace_depth <= 0:
            break

    return body_lines


def find_ct_aliases(body_text):
    """
    Scan method body for CancellationTokenSource assignments and extract valid derived CTs.

    Recognizes patterns like:
    - CancellationTokenSource cts = new CancellationTokenSource(...)
    - using var cts = new CancellationTokenSource(...)
    - var cts = CancellationTokenSource.CreateLinkedTokenSource(...)
    - using var cts = CancellationTokenSource.CreateLinkedTokenSource(...)

    Returns a set of valid CT expressions like {"cts.Token", "linkedCts.Token", ...}
    """
    aliases = set()

    # Pattern 1: new CancellationTokenSource (direct assignment)
    for match in re.finditer(r'(?:using\s+)?(?:var|CancellationTokenSource)\s+(\w+)\s*=\s*new\s+CancellationTokenSource', body_text):
        var_name = match.group(1)
        aliases.add(f"{var_name}.Token")

    # Pattern 2: CancellationTokenSource.CreateLinkedTokenSource (static factory)
    for match in re.finditer(r'(?:using\s+)?(?:var|CancellationTokenSource)\s+(\w+)\s*=\s*CancellationTokenSource\.CreateLinkedTokenSource', body_text):
        var_name = match.group(1)
        aliases.add(f"{var_name}.Token")

    return aliases


def find_ct_violations(cs_file, allowlist):
    """Find CancellationToken threading violations in a C# file."""
    try:
        with open(cs_file, encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception as e:
        return [], []

    violations = []
    safe_methods = []
    relative = str(cs_file.relative_to(Path("C:/Users/koosh/Dino/src"))).replace("\\", "/")

    # Check if file is in allowlist
    if any(pattern in relative for pattern in allowlist):
        return [], []

    i = 0
    while i < len(lines):
        line = lines[i]

        # Match async method signature with CancellationToken parameter
        match = re.search(
            r'async\s+(?:Task|ValueTask)(?:<[^>]+>)?\s+(\w+)\s*\([^)]*\bCancellationToken\s+(\w+)\b[^)]*\)',
            line
        )

        if match:
            method_name = match.group(1)
            ct_param = match.group(2)

            # Extract method body
            body_lines = extract_method_body(lines, i)
            body_text = "".join(body_lines)

            # Build set of valid CT expressions: original param + any derived aliases (CTS.Token)
            valid_cts = {ct_param}  # Original CT parameter is always valid
            ct_aliases = find_ct_aliases(body_text)
            valid_cts.update(ct_aliases)

            # Conservative: only flag if there's an await that clearly lacks ANY valid CT
            # by searching for patterns like: await SomeMethod(...) where NO valid CT is in args
            found_violation = False
            await_pattern = r'await\s+[\w.]+\.([A-Za-z_]\w*)\s*\('

            for await_match in re.finditer(await_pattern, body_text):
                method_call = await_match.group(0)

                # Skip obvious Task.Delay, Task.Run — they may have CT but we're looking for actual async-await chains
                if 'Task.Delay' in method_call or 'Task.Run' in method_call or 'Task.WhenAny' in method_call:
                    continue

                # Extract args more carefully: find the matching closing paren
                start_idx = await_match.end() - 1
                paren_depth = 1
                end_idx = start_idx + 1

                while end_idx < len(body_text) and paren_depth > 0:
                    if body_text[end_idx] == '(':
                        paren_depth += 1
                    elif body_text[end_idx] == ')':
                        paren_depth -= 1
                    end_idx += 1

                if paren_depth == 0:
                    args = body_text[start_idx + 1:end_idx - 1]

                    # Check if ANY valid CT (original param or derived alias) is in args
                    has_valid_ct = False
                    for valid_ct in valid_cts:
                        # For aliases like "linkedCts.Token", we need to match the full expression
                        if '.' in valid_ct:
                            # Match as word boundary: linkedCts.Token (not as substring)
                            pattern = r'\b' + re.escape(valid_ct) + r'\b'
                        else:
                            # Simple param name
                            pattern = r'\b' + re.escape(valid_ct) + r'\b'
                        if re.search(pattern, args):
                            has_valid_ct = True
                            break

                    if not has_valid_ct:
                        # This is a real violation
                        found_violation = True
                        violations.append({
                            'file': relative,
                            'line': i + 1,
                            'method': method_name,
                            'ct_param': ct_param,
                            'severity': 'HIGH',
                            'detail': f"Async method accepts {ct_param} but not threaded to awaited calls"
                        })
                        break

            if not found_violation and len(body_lines) > 0:
                safe_methods.append(f"{relative}:{i+1}")

        i += 1

    return violations, safe_methods


def main():
    repo_root = Path("C:/Users/koosh/Dino")
    src_dir = repo_root / "src"
    allowlist_file = repo_root / "docs/qa/ct-threading-allowlist.txt"

    # Flags
    json_output = "--json" in sys.argv
    test_mode = "--test" in sys.argv
    threshold = 5

    if "--threshold" in sys.argv:
        try:
            idx = sys.argv.index("--threshold")
            threshold = int(sys.argv[idx + 1])
        except (IndexError, ValueError):
            pass

    # Load allowlist
    allowed_files = set()
    if allowlist_file.exists():
        with open(allowlist_file, encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#'):
                    parts = line.split('#')[0].strip()
                    if parts:
                        allowed_files.add(parts)

    # Run self-test if requested
    if test_mode:
        return run_self_test()

    # Scan all C# files
    all_violations = []
    all_safe = []

    for cs_file in src_dir.rglob("*.cs"):
        # Skip obj, bin, Tests, and generated files
        path_str = str(cs_file)
        if any(x in path_str for x in ["\\obj\\", "\\bin\\", "\\Tests\\", ".g.cs"]):
            continue

        violations, safe = find_ct_violations(cs_file, allowed_files)
        all_violations.extend(violations)
        all_safe.extend(safe)

    # Filter to HIGH severity only
    high_violations = [v for v in all_violations if v.get('severity') == 'HIGH']

    if json_output:
        output = {
            'total_high': len(high_violations),
            'total_safe': len(all_safe),
            'threshold': threshold,
            'violations': high_violations,
            'safe_methods': all_safe
        }
        print(json.dumps(output, indent=2))
    else:
        if high_violations:
            print(f"\n{'='*80}")
            print(f"Pattern #114 Violations: {len(high_violations)} CancellationToken not threaded")
            print(f"{'='*80}")
            for v in sorted(high_violations, key=lambda x: (x['file'], x['line'])):
                print(f"{v['file']}:{v['line']} in {v['method']} — {v['detail']}")
            print(f"{'='*80}\n")

        if len(high_violations) > threshold:
            print(f"FAIL: {len(high_violations)} violations exceed threshold of {threshold}")
            return 1

    return 0


def run_self_test():
    """Test the regex patterns against inline code samples."""
    test_cases = [
        # (code, should_match, description)
        ("async Task DoWork(CancellationToken ct) { await SomethingAsync(); }", True, "Basic violation: CT not passed"),
        ("async Task DoWork(CancellationToken ct) { await SomethingAsync(ct); }", False, "Basic safe: CT passed"),
        ("async ValueTask<int> Fetch(CancellationToken token) { return await repo.GetAsync(token); }", False, "ValueTask with CT"),
        ("async Task Execute(CancellationToken cancellationToken) { await Task.Delay(100); }", True, "Task.Delay (skipped)"),
        ("void SyncMethod(CancellationToken ct) { /* no await */ }", False, "No async method"),
        ("async Task Work() { await SomethingAsync(); }", False, "No CT param"),
        # New token aliasing tests
        ("async Task DoWork(CancellationToken ct) { using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(ct); await SomethingAsync(linkedCts.Token); }", False, "Token alias from CreateLinkedTokenSource is safe"),
        ("async Task DoWork(CancellationToken ct) { using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(ct); await SomethingAsync(); }", True, "Token alias exists but not used in awaits"),
    ]

    passed = 0
    failed = 0

    for code, should_match, description in test_cases:
        # Check if it matches async with CancellationToken
        has_ct_param = bool(re.search(r'async.*CancellationToken', code))

        if has_ct_param:
            # Extract CT param name
            ct_match = re.search(r'CancellationToken\s+(\w+)', code)
            if ct_match:
                ct_param = ct_match.group(1)
                # Build valid CT set: original param + aliases
                valid_cts = {ct_param}
                ct_aliases = find_ct_aliases(code)
                valid_cts.update(ct_aliases)

                # Check if any await lacks a valid CT
                awaits = re.findall(r'await\s+[\w.]+\s*\(([^)]*)\)', code)
                has_violation = True
                for args in awaits:
                    # Skip Task.Delay/Task.Run/Task.WhenAny
                    if 'Task.Delay' in args or 'Task.Run' in args or 'Task.WhenAny' in args:
                        continue
                    # Check if any valid CT is in args
                    found_valid_ct = False
                    for valid_ct in valid_cts:
                        if re.search(r'\b' + re.escape(valid_ct) + r'\b', args):
                            found_valid_ct = True
                            break
                    if not found_valid_ct:
                        has_violation = True
                        break
                    has_violation = False
            else:
                has_violation = False
        else:
            has_violation = False

        result = "PASS" if has_violation == should_match else "FAIL"
        if has_violation == should_match:
            passed += 1
        else:
            failed += 1

        print(f"{result}: {description:<60} (expect={should_match}, got={has_violation})")

    print(f"\nSelf-test: {passed} PASS, {failed} FAIL")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
