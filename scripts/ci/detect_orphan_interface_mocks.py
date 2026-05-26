#!/usr/bin/env python3
"""
Pattern #125 D2: Detect orphan interface mocks.

An orphan interface mock is a public interface with:
- 3+ production implementations (not mocked)
- 0 test double classes (Mock*, Fake*, Stub*)

This indicates the interface is real enough to produce implementations,
but lacks dedicated test doubles, forcing tests to mock ad-hoc.

Scope:
- Interfaces: src/SDK/ + src/Bridge/Protocol/
- Production refs: entire src/ except Tests/, bin/, obj/
- Test doubles: src/Tests/Mocks/ + src/Tests/Doubles/

Exit codes:
  0 = OK (violations <= threshold)
  1 = FAIL (violations > threshold)
  2 = USAGE
"""

import argparse
import json
import os
import re
import sys
from collections import defaultdict
from pathlib import Path


def find_interfaces(sdk_path, protocol_path):
    """Enumerate public interfaces in SDK and Bridge.Protocol."""
    interfaces = {}
    pattern = re.compile(r'public\s+interface\s+(I\w+)')

    for root_path in [sdk_path, protocol_path]:
        if not root_path.exists():
            continue
        for cs_file in root_path.rglob('*.cs'):
            with open(cs_file, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                for match in pattern.finditer(content):
                    iface_name = match.group(1)
                    interfaces[iface_name] = str(cs_file)

    return interfaces


def count_implementations(iface_name, src_root):
    """Count production implementations of interface (grep for ': I<Name>' or ', I<Name>')."""
    pattern = re.compile(rf':\s*{iface_name}\b|,\s*{iface_name}\b')
    count = 0

    # Exclude test folders
    exclude = {'Tests', 'bin', 'obj', '.git'}

    for cs_file in src_root.rglob('*.cs'):
        # Skip test and build dirs
        if any(part in exclude for part in cs_file.parts):
            continue

        try:
            with open(cs_file, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                count += len(pattern.findall(content))
        except Exception:
            pass

    return count


def find_test_doubles(iface_name, mocks_path, doubles_path):
    """Check if Mock<Name>, Fake<Name>, or Stub<Name> exist in test directories."""
    prefixes = ['Mock', 'Fake', 'Stub']

    for test_dir in [mocks_path, doubles_path]:
        if not test_dir.exists():
            continue
        for prefix in prefixes:
            class_name = f"{prefix}{iface_name[1:]}"  # Remove 'I' prefix from interface
            for cs_file in test_dir.rglob('*.cs'):
                with open(cs_file, 'r', encoding='utf-8', errors='ignore') as f:
                    if re.search(rf'class\s+{class_name}\b', f.read()):
                        return True

    return False


def check_allowlist(iface_name, allowlist_file):
    """Check if interface has an allowlist entry."""
    if not allowlist_file.exists():
        return False

    with open(allowlist_file, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            if iface_name in line:
                return True

    return False


def main():
    parser = argparse.ArgumentParser(
        description='Detect orphan interface mocks (Pattern #125)',
        epilog='Exit 0 if violations <= threshold, 1 if > threshold, 2 for usage.'
    )
    parser.add_argument('--repo', default='.', help='Repository root')
    parser.add_argument('--threshold', type=int, default=5, help='Fail if HIGH violations > threshold (default 5)')
    parser.add_argument('--json', action='store_true', help='Output as JSON')
    parser.add_argument('--test', action='store_true', help='Run self-test')

    args = parser.parse_args()

    repo = Path(args.repo).resolve()
    sdk_path = repo / 'src' / 'SDK'
    protocol_path = repo / 'src' / 'Bridge' / 'Protocol'
    src_root = repo / 'src'
    mocks_path = repo / 'src' / 'Tests' / 'Mocks'
    doubles_path = repo / 'src' / 'Tests' / 'Doubles'
    allowlist_file = repo / 'docs' / 'qa' / 'orphan-interface-mocks-allowlist.txt'

    if args.test:
        # Self-test: verify paths exist and basic structure
        print('Self-test: Checking paths...')
        assert sdk_path.exists(), f'SDK path not found: {sdk_path}'
        assert protocol_path.exists(), f'Protocol path not found: {protocol_path}'
        print('Self-test: PASS')
        return 0

    print(f'Scanning interfaces in {sdk_path} and {protocol_path}...')
    interfaces = find_interfaces(sdk_path, protocol_path)

    violations = []

    for iface_name, iface_file in sorted(interfaces.items()):
        impl_count = count_implementations(iface_name, src_root)
        has_double = find_test_doubles(iface_name, mocks_path, doubles_path)
        allowlisted = check_allowlist(iface_name, allowlist_file)

        # HIGH: 3+ implementations AND no test doubles
        if impl_count >= 3 and not has_double and not allowlisted:
            violations.append({
                'interface': iface_name,
                'file': iface_file,
                'implementation_count': impl_count,
                'has_test_double': has_double,
                'severity': 'HIGH'
            })

    if args.json:
        output = {
            'total_violations': len(violations),
            'threshold': args.threshold,
            'violations': violations,
            'passed': len(violations) <= args.threshold
        }
        print(json.dumps(output, indent=2))
    else:
        print(f'\nFound {len(violations)} HIGH violations (threshold: {args.threshold})\n')
        for v in violations:
            print(f"  {v['interface']} ({v['implementation_count']} impls, 0 mocks)")
            print(f"    {v['file']}")

        if violations:
            print(f'\nFail: {len(violations)} > {args.threshold}' if len(violations) > args.threshold else f'Pass: {len(violations)} <= {args.threshold}')

    return 1 if len(violations) > args.threshold else 0


if __name__ == '__main__':
    sys.exit(main())
