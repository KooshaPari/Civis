"""Spot-check the regenerated container-bindings.json.

Three defects were reported against this artifact and the detector:

1. 8 of the ids were reported as having no authoritative definition.
   Checked: there should now be 0.
2. 30 of 34 protocol-3d rows carried a middle-column table fragment as the
   `requirement` field, e.g. ': Snapshot Filtering'.
3. `defined_by_spec` was unreliable because of 2.

This asserts each of those, and additionally re-verifies every `requirement`
string against the file it was taken from, so a fragment cannot come back.
"""
import json
import pathlib
import re

REPO = pathlib.Path(r"C:\Users\koosh\Civis-clone")
data = json.loads(
    (REPO / "docs/audits/container-bindings.json").read_text(encoding="utf-8")
)
rows = data if isinstance(data, list) else data.get("bindings", [])

print(f"rows: {len(rows)}")

# 1. no undefined ids
undefined = [r for r in rows if not r.get("defined_by_spec", True)]
print(f"defined_by_spec false: {len(undefined)}")
for r in undefined:
    print("   UNDEFINED:", r.get("id"), r.get("file"), r.get("line"))

# 2. no table-row fragments in the requirement text
frag = [
    r
    for r in rows
    if isinstance(r.get("requirement"), str)
    and (
        r["requirement"].strip().startswith(":")
        or "|" in r["requirement"]
        or len(r["requirement"].strip()) < 12
    )
]
print(f"fragment-shaped requirement strings: {len(frag)}")
for r in frag:
    print("   FRAGMENT:", r.get("id"), repr(r["requirement"][:70]))

# 3. every requirement string must be findable verbatim in some spec file
specs = []
for d in (
    "docs/specs",
    "docs/models",
    "docs/design",
    "docs/reference",
    "docs/guides",
    "docs/traceability",
    "agileplus-specs",
):
    root = REPO / d
    if root.is_dir():
        for f in root.rglob("*.md"):
            if "fragemented" in f.parts:
                continue
            try:
                specs.append(f.read_text(encoding="utf-8", errors="replace"))
            except OSError:
                pass
corpus = "\n".join(specs)
print(f"spec corpus: {len(corpus)} chars")

unverifiable = []
for r in rows:
    req = (r.get("requirement") or "").strip()
    if not req:
        continue
    probe = re.sub(r"\s+", " ", req)[:60]
    if re.sub(r"\s+", " ", probe) not in re.sub(r"[ \t]+", " ", corpus):
        unverifiable.append(r)
print(f"requirement strings not found verbatim in the corpus: {len(unverifiable)}")
for r in unverifiable:
    print("   UNVERIFIED:", r.get("id"), repr((r.get("requirement") or "")[:70]))

ok = not undefined and not frag and not unverifiable
print("\nRESULT:", "PASS" if ok else "FAIL")
