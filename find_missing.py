import json
import os
import re

# Load the fr-matrix
with open('docs/audits/fr-matrix.json', 'r', encoding='utf-8') as f:
    data = json.load(f)

impl_no_test = [r['id'] for r in data['rows'] if r['status'] == 'IMPL-NO-TEST']

# Search for Covers annotations in all .rs files
covered_ids = set()
for root, dirs, files in os.walk('.'):
    # Skip target and .git
    if 'target' in root or '.git' in root or 'node_modules' in root:
        continue
    for file in files:
        if file.endswith('.rs'):
            path = os.path.join(root, file)
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                for match in re.finditer(r'Covers (FR-[^\s]+)', content):
                    covered_ids.add(match.group(1))
            except Exception:
                pass

# Find IMPL-NO-TEST IDs without Covers annotation
missing = [id for id in impl_no_test if id not in covered_ids]
print(f"Total IMPL-NO-TEST: {len(impl_no_test)}")
print(f"Already covered: {len(covered_ids & set(impl_no_test))}")
print(f"Missing coverage: {len(missing)}")
for id in missing:
    print(id)
