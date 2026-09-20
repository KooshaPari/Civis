# P2 plan

Status: `SPEC-ONLY` — 294 IDs across 51 epics

Distributed across 7 agents.

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


## Agent load distribution

| Agent | IDs | Epics |
|------:|----:|-------|
| A | 54 | 8 |
| B | 36 | 8 |
| C | 22 | 7 |
| D | 25 | 7 |
| E | 35 | 7 |
| F | 87 | 7 |
| G | 35 | 7 |
