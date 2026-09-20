"""Generic plan generator: split fr-matrix rows of a given status across N agents,
producing one markdown file per agent with full ID context."""
import json
import os
import sys
from collections import defaultdict

MATRIX = sys.argv[1] if len(sys.argv) > 1 else 'docs/audits/fr-matrix.json'
STATUS = sys.argv[2] if len(sys.argv) > 2 else 'TEST-NO-CODE-REF'
OUT_DIR = sys.argv[3] if len(sys.argv) > 3 else 'docs/audits/next-slice'
N_AGENTS = int(sys.argv[4]) if len(sys.argv) > 4 else 7
PHASE = sys.argv[5] if len(sys.argv) > 5 else 'P1'

os.makedirs(OUT_DIR, exist_ok=True)

m = json.load(open(MATRIX, encoding='utf-8'))
rows = [r for r in m['rows'] if r['status'] == STATUS]

# Sort by epic then id for stable round-robin
rows.sort(key=lambda r: (r['epic'], r['id']))

# Bucket by epic to keep each agent's work contiguous per epic
by_epic = defaultdict(list)
for r in rows:
    by_epic[r['epic']].append(r)

# Round-robin epic buckets across agents (preserves epic-locality within an agent)
epic_names = sorted(by_epic.keys())
slots = [[] for _ in range(N_AGENTS)]
agent_epics = [[] for _ in range(N_AGENTS)]
for i, epic in enumerate(epic_names):
    slot = i % N_AGENTS
    slots[slot].extend(by_epic[epic])
    agent_epics[slot].append(epic)

# Per-phase instructions
INSTRUCTIONS = {
    'TEST-NO-CODE-REF': """\
# Phase: TEST-NO-CODE-REF → COVERED

Each ID has a real test (`test_refs`) but no source file with a `// FR-XYZ`
comment (`code_refs: []`). Your job is to find the source function the test
exercises and add a single-line `// FR-XYZ` comment near its definition so
the audit picks it up as `code_ref`.

## Approach
1. Open the test file at the listed `test_refs` paths. Find what
   function/struct/method it calls.
2. Grep for the function definition in `crates/`. Common targets:
   `crates/<crate>/src/<module>.rs`.
3. Add `// FR-XYZ` comment on the line directly above the function/struct
   definition. Single-line comment, no doc comment needed.
4. Repeat for every ID in your slice.
5. `cargo build -p <crate>` after edits to catch syntax errors.
6. Commit with `test(<crate>): tag FR-XYZ on <brief summary>`.

## What counts as "code_ref"
- Any `fn`, `pub fn`, `pub struct`, `pub enum`, `pub trait`, `impl`,
  `const`, `static`, `mod` definition.
- Single-line `// FR-XYZ` above the item.
- Multiple FRs per file? Multiple comments are fine.

## Quality
- Don't change the source code itself. Only add the comment.
- Make sure you tag the *actual* function the test calls, not just any
  function in the file.
- If a test calls a public API across multiple modules, tag the entry
  point first.

""",
    'SPEC-ONLY': """\
# Phase: SPEC-ONLY → IMPL-NO-TEST or COVERED

Each ID has spec/trace references but no source code. You have two options:

## Option A: Write minimal source
1. Read the spec at the listed `spec_refs` paths.
2. If the spec is well-defined and small (≤1 function, ≤20 lines), write
   a minimal source implementation that satisfies it.
3. Add the function to the appropriate `crates/<crate>/src/<file>.rs`.
4. Add `// FR-XYZ` comment above the function.
5. Optionally write a minimal test in `crates/<crate>/tests/fr_<id>.rs`.

## Option B: Defer or delete
1. If the spec is large / unclear / depends on systems we don't have
   yet, **skip the ID** — leave it as SPEC-ONLY for a future agent.
2. Add a comment to `docs/audits/spec-only-deferred.md` listing
   deferred IDs with a brief reason.

## Bias
- Prefer Option A for FRs with `spec_refs` pointing to small ADR/plan
  files (< 100 lines).
- Prefer skip for FRs whose spec requires infrastructure not yet built.

""",
    'CODE-ONLY-no-spec': """\
# Phase: CODE-ONLY-no-spec → COVERED (write spec) or remove

Each ID has source code (`code_refs`) but no spec/traceability reference
(`spec_refs: []`). These are orphaned code fragments — they exist but no
one wrote down what they're for.

## Option A: Write spec (preferred)
1. Open the source file at the listed `code_refs` paths.
2. Read the function/code to understand what it does.
3. Write a minimal spec doc at
   `docs/traceability/<id-lower>/<id-lower>-intent.md` (≤ 30 lines).
   Use existing spec files as templates — see
   `docs/traceability/fr-civ-brush-01/fr-civ-brush-01-intent.md`.
4. Add `// Covers: FR-XYZ` to the spec body so audit picks it up.
5. Tag the source file with `// FR-XYZ` comment too if not already.

## Option B: Delete orphaned code
1. If the code is unused / dead / only called by tests of equal orphan
   status, delete it.
2. Add a one-line note in `docs/audits/code-only-deleted.md` so we
   don't waste cycles later.

## Bias
- Default to Option A. Most FR-NFR-prefixed orphans are intentional.
- Default to Option B for FR-NFR-P/NFR-C/NFR-R/NFR-S etc. that look
  like generic categories without substance.

""",
}

# Write summary
summary_path = os.path.join(OUT_DIR, f'_plan-{PHASE}-summary.md')
with open(summary_path, 'w', encoding='utf-8') as f:
    f.write(f"# {PHASE} plan\n\n")
    f.write(f"Status: `{STATUS}` — {len(rows)} IDs across {len(by_epic)} epics\n\n")
    f.write(f"Distributed across {N_AGENTS} agents.\n\n")
    f.write(INSTRUCTIONS.get(STATUS, ''))
    f.write(f"\n## Agent load distribution\n\n")
    f.write("| Agent | IDs | Epics |\n")
    f.write("|------:|----:|-------|\n")
    for i, slot in enumerate(slots):
        f.write(f"| {chr(65+i)} | {len(slot)} | {len(agent_epics[i])} |\n")

# Per-agent slice files
for i, slot in enumerate(slots):
    agent = chr(65 + i)
    path = os.path.join(OUT_DIR, f'{PHASE}-agent-{agent}.md')
    with open(path, 'w', encoding='utf-8') as f:
        f.write(f"# {PHASE} agent-{agent}\n\n")
        f.write(INSTRUCTIONS.get(STATUS, ''))
        f.write(f"\n## Your slice: {len(slot)} IDs in {len(agent_epics[i])} epics\n\n")
        for epic in sorted(agent_epics[i]):
            epic_rows = [r for r in slot if r['epic'] == epic]
            f.write(f"\n### Epic `{epic}` — {len(epic_rows)} IDs\n\n")
            for r in epic_rows:
                f.write(f"#### {r['id']}\n\n")
                if r.get('test_refs'):
                    f.write("- Tests:\n")
                    for ref in r['test_refs'][:3]:
                        f.write(f"  - `{ref}`\n")
                if r.get('code_refs'):
                    f.write("- Code:\n")
                    for ref in r['code_refs'][:3]:
                        f.write(f"  - `{ref}`\n")
                if r.get('spec_refs'):
                    f.write("- Spec/trace:\n")
                    for ref in r['spec_refs'][:3]:
                        f.write(f"  - `{ref}`\n")
                f.write("\n")

print(f'Wrote {summary_path} ({len(rows)} IDs, {N_AGENTS} agents)')
print(f'Wrote {N_AGENTS} per-agent files in {OUT_DIR}/')
