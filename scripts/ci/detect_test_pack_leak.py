#!/usr/bin/env python3
"""
Pattern #234: Test Fixture IDs Leaking Into Deployed Packs

Detects when test pack IDs (Test*, test-invalid, test-valid, etc.) reach
production deployment directories (packs/ or MSBuild DeployPacks outputs).
"""

import re
import sys
from pathlib import Path

def scan_csproj_deploy_packs(csproj_path):
    """Check if csproj lacks test-pack exclusion in DeployPacks."""
    violations = []
    try:
        content = csproj_path.read_text(encoding='utf-8')
        # Look for PackFiles Include without test exclusion
        if '<PackFiles Include=' in content and 'packs' in content:
            if 'test' not in content.lower() or 'Exclude=' not in content:
                violations.append({
                    'type': 'csproj_missing_exclusion',
                    'file': str(csproj_path),
                    'severity': 'HIGH'
                })
    except Exception as e:
        print(f"Warning: Could not read {csproj_path}: {e}", file=sys.stderr)
    return violations

def scan_pack_ids():
    """Scan packs/ for test fixture IDs in non-test locations."""
    violations = []
    packs_dir = Path('packs')
    test_fixtures_dir = Path('src/Tests/Fixtures')

    if not packs_dir.exists():
        return violations

    # Test ID patterns
    test_patterns = [
        r'^Test',
        r'^test-invalid',
        r'^test-valid',
        r'^test-bad',
        r'TestInvalidID',
        r'MockTest',
        r'FakeTest'
    ]

    for pack_yaml in packs_dir.rglob('pack.yaml'):
        pack_dir = pack_yaml.parent

        # Skip if in test fixtures
        if str(pack_dir).startswith(str(test_fixtures_dir)):
            continue

        try:
            content = pack_yaml.read_text(encoding='utf-8')
            # Extract id: field
            match = re.search(r'^\s*id:\s*["\']?([^\s"\']+)["\']?', content, re.MULTILINE)
            if match:
                pack_id = match.group(1)
                if any(re.match(pattern, pack_id) for pattern in test_patterns):
                    violations.append({
                        'type': 'test_id_in_packs_dir',
                        'file': str(pack_yaml),
                        'pack_id': pack_id,
                        'severity': 'HIGH'
                    })
        except Exception as e:
            print(f"Warning: Could not read {pack_yaml}: {e}", file=sys.stderr)

    return violations

def main():
    csproj_violations = []
    for csproj in Path('src').rglob('*.csproj'):
        if 'DeployPacks' in csproj.read_text(encoding='utf-8', errors='ignore'):
            csproj_violations.extend(scan_csproj_deploy_packs(csproj))

    pack_id_violations = scan_pack_ids()
    all_violations = csproj_violations + pack_id_violations

    high_count = sum(1 for v in all_violations if v['severity'] == 'HIGH')
    med_count = sum(1 for v in all_violations if v['severity'] == 'MED')

    print(f"Pattern #234 Test Pack Leak Detection")
    print(f"HIGH: {high_count}, MED: {med_count}")
    print()

    if all_violations:
        print("First 5 violations:")
        for v in all_violations[:5]:
            if v['type'] == 'csproj_missing_exclusion':
                print(f"  [{v['severity']}] {v['file']}: DeployPacks lacks test exclusion")
            else:
                print(f"  [{v['severity']}] {v['file']}: test ID '{v['pack_id']}' in production packs/")

    return 1 if high_count > 0 else 0

if __name__ == '__main__':
    sys.exit(main())
