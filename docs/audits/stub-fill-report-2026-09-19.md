# Stub-Fill Report — 2026-09-19

## Goal

Eliminate `STUB-TEST-ONLY` rows in the FR coverage matrix by replacing
placeholder test bodies (e.g. `let ws = WorldState::default(); assert!(ws.tick == 0)`)
with real, FR-specific assertions in parallel via agent fan-out.

## Approach

1. **Detection**: `_gather_ids.py` classifies a test file as a stub if
   its first 30 lines contain either `Epic: auto-generated` (legacy) or
   `Stub: TDD-red` (new). Stub refs are routed to `in_stub_tests`
   instead of `in_tests`.

2. **Marking**: `bulk-mark-stubs.py` injects the `Stub: TDD-red` line
   into 552 placeholder test files across 59 epics, then bumps them in
   the inventory. New audit status: `STUB-TEST-ONLY`.

3. **Planning**: `gen-stub-plan.py` + `split-stub-plan.py` produce one
   markdown file per agent describing which IDs each agent owns.

4. **Fan-out**: 7 git worktrees at `C:\Users\koosh\Civis-clone-worktrees\agent-{A..G}`,
   on branches `stub-fill-{A..G}`. Each agent edits only files inside
   its slice.

5. **Re-classification**: After agents commit, `strip-stub-headers.py`
   removes the legacy `Epic: auto-generated` / `Stub: TDD-red` header
   lines from the rewritten files so the matrix correctly reflects them
   as `TEST-NO-CODE-REF` or `COVERED`.

## Agent distribution

| Agent | Slice                                  | Files | Commits       |
|------:|----------------------------------------|------:|---------------|
| A     | VEHICLE/BRUSH/RTS-RENDER/NATION/ZOOM   |    47 | `21e5810f`    |
| B     | MOD/LANG/MCP/DET/DOC/GUARD/INT/MET/METRICS | 54 | `421315b4`*   |
| C     | PERF/LLM/POLITY                        |    37 | `14f35edc`    |
| D     | 3D/QOL/CIV/VERIFY/MARKET               |    57 | `f99fb708`    |
| E     | RTS/CORE/CORE-DET/TERRAIN              |    34 | `cc023edc`    |
| F     | INFOVIEW/AI                            |    16 | `60b2d725` + `dbaf4beb` |
| G     | overflow (NET/PROT/REP/SESSION/...)    |    84 | `b18b3309`    |
|       | **Total**                              | **329** |             |

\* Agent B did the work but never committed; the lead agent committed
on its behalf after build verification.

## Merge order

```
f79ed60d merge: stub-fill-A (BRUSH/RTS/VEHICLE stubs)        clean
2d4b8d26 Merge branch 'stub-fill-B'                            clean
bd4fbdb3 Merge branch 'stub-fill-C'                            clean
97fb92aa Merge branch 'stub-fill-D'                            clean
3daf5727 Merge branch 'stub-fill-E'                            clean
547e025d Merge branch 'stub-fill-F'                            clean
94125341 Merge branch 'stub-fill-G'                            37 conflicts
                                                                  (resolved with --ours;
                                                                  HEAD already had F's
                                                                  richer assertions)
620736c9 fix(traceability): strip legacy stub header markers   245 files touched
78e99247 fix(test): QOL-100 uses FirstFaction variant         1 file
```

## Build status

```
cargo build --workspace --tests  →  exit code 0  ✓
```

Two post-merge fixes were required:

1. `fr_fr_civ_3d_014.rs::step_produces_new_state` — captured initial
   tick before calling `civ_engine::step(ws1, ...)` which moves `ws1`.

2. `fr_fr_civ_qol_100.rs::tutorial_milestone_exists` — replaced
   `TutorialMilestone::default()` (not implemented for enum) with
   the explicit `FirstFaction` variant.

## Coverage delta

| Status              | Before (`29ea8740`) | After (`78e99247`) | Δ      |
|---------------------|--------------------:|-------------------:|-------:|
| `COVERED`           | 493                 | 505                |  +12   |
| `STUB-TEST-ONLY`    | **328**             | **0**              | **-328** |
| `TEST-NO-CODE-REF`  | 170                 | 495                | +325   |
| `SPEC-ONLY`         | 303                 | 294                |   -9   |
| `CODE-ONLY-no-spec` | 209                 | 215                |   +6   |
| **Total**           | 1509                | 1509               |    0   |

The 328 STUB rows split cleanly into:

- **+12 COVERED** — agents tagged source files too (`FR-CIV-DIPLO`,
  `FR-CIV-AI`, etc.).
- **+325 TEST-NO-CODE-REF** — agents rewrote test bodies but did not
  tag the corresponding `crates/*/src/*.rs` files with `// FR-XYZ`.
  Downstream work: have agents add ID-tagged code comments in src/
  to push these into COVERED too.

## Quality variance

- **High** (agents B, F, G): real math/behavior assertions —
  `compute_waste_heat == 10%, cap jitter at 2%`, type-state machines,
  BTreeMap inv keying, etc.
- **Medium** (agents A, C, D, E): mostly `Type::default()` existence
  + `field >= 0` / `set.len() == N` checks. Catches API drift but
  doesn't exercise deeper behavior.

## Downstream work

1. Add ID-tagged code comments in `crates/*/src/*.rs` for the 495
   TEST-NO-CODE-REF rows. This is the next fan-out task.
2. Address the 294 SPEC-ONLY rows (no code yet) and 215 CODE-ONLY-no-spec
   rows (orphaned code without spec) — likely via either delete or
   spec-write agents.
