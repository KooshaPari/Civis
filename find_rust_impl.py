import json

data = json.load(open('docs/audits/fr-matrix.json'))
rows = [r for r in data['rows'] if r['status'] == 'IMPL-NO-TEST']

for r in rows:
    rust_refs = [c for c in r['code_refs'] if c.startswith('crates/') or c.startswith('clients/') or '.rs' in c]
    if rust_refs:
        print(f"{r['id']}: {r['epic']}")
        for ref in rust_refs[:3]:
            print(f"  {ref}")
        if len(rust_refs) > 3:
            print(f"  ... ({len(rust_refs)} total)")
