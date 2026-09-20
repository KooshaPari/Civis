"""Distribution of IDs by status, for sizing fan-out slices."""
import json
import sys
from collections import Counter, defaultdict

PATH = sys.argv[1] if len(sys.argv) > 1 else 'docs/audits/fr-matrix.json'

m = json.load(open(PATH, encoding='utf-8'))
rows = m['rows']
epic_of = defaultdict(list)
for r in rows:
    epic_of[r['epic']].append(r)

for status in ['TEST-NO-CODE-REF', 'SPEC-ONLY', 'CODE-ONLY-no-spec']:
    rows_s = [r for r in rows if r['status'] == status]
    by_epic = Counter(r['epic'] for r in rows_s)
    print(f'\n=== {status} ({len(rows_s)} IDs across {len(by_epic)} epics) ===')
    for epic, n in sorted(by_epic.items(), key=lambda x: -x[1])[:25]:
        print(f'  {epic:40s} {n}')
