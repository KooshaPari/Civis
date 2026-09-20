# P3 agent-B

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


## Your slice: 21 IDs in 11 epics


### Epic `FR-CIV` — 1 IDs

#### FR-CIV-014

- Tests:
  - `crates/engine/src/emergence.rs:1627`
  - `crates/engine/src/save.rs:335`
  - `crates/engine/src/save.rs:375`
- Code:
  - `crates/engine/src/emergence.rs:1627`
  - `crates/engine/src/engine/engine_tests.rs:3157`
  - `crates/engine/src/save.rs:335`


### Epic `FR-CIV-BELIEF` — 1 IDs

#### FR-CIV-BELIEF-001

- Code:
  - `crates/engine/src/religion.rs:40`
  - `crates/engine/src/religion.rs:55`
  - `crates/engine/src/religion.rs:88`


### Epic `FR-CIV-CONSTRUCTION` — 1 IDs

#### FR-CIV-CONSTRUCTION-001

- Tests:
  - `crates/engine/tests/institutions_buildsites_econfocus_persistence.rs:4`
  - `crates/engine/tests/persistence_replay_coverage.rs:13`
- Code:
  - `crates/engine/src/engine.rs:466`
  - `crates/engine/src/engine.rs:2182`


### Epic `FR-CIV-ECON` — 1 IDs

#### FR-CIV-ECON-010

- Tests:
  - `crates/engine/tests/riot_migrant_taxation_persistence.rs:3`
- Code:
  - `crates/engine/src/engine.rs:532`
  - `crates/engine/src/engine.rs:723`
  - `crates/engine/src/engine.rs:2200`


### Epic `FR-CIV-GAME` — 3 IDs

#### FR-CIV-GAME-001

- Code:
  - `clients/bevy-ref/src/gameplay_hud.rs:3`
  - `clients/bevy-ref/src/lib.rs:541`
  - `clients/bevy-ref/src/outcome_overlay.rs:3`

#### FR-CIV-GAME-002

- Tests:
  - `crates/engine/src/gameplay.rs:888`
- Code:
  - `clients/bevy-ref/src/god_panel.rs:2`
  - `crates/engine/src/gameplay.rs:2`
  - `crates/engine/src/gameplay.rs:888`

#### FR-CIV-GAME-003

- Code:
  - `clients/bevy-ref/src/era_hud.rs:2`
  - `crates/engine/src/era.rs:1`


### Epic `FR-CIV-LEGENDS` — 1 IDs

#### FR-CIV-LEGENDS-010

- Code:
  - `crates/engine/src/engine.rs:998`


### Epic `FR-CIV-TEST` — 7 IDs

#### FR-CIV-TEST-001

- Tests:
  - `crates/engine/tests/n_series_coverage.rs:3`

#### FR-CIV-TEST-002

- Tests:
  - `crates/civis-mcp/tests/mcp_integration.rs:1`

#### FR-CIV-TEST-006

- Tests:
  - `crates/economy/tests/economy_coverage.rs:1`
  - `crates/economy/tests/economy_coverage.rs:18`
  - `crates/economy/tests/economy_coverage.rs:51`

#### FR-CIV-TEST-007

- Tests:
  - `crates/server/tests/server_coverage.rs:1`

#### FR-CIV-TEST-008

- Tests:
  - `crates/civ-emergence-metrics/tests/emergence_coverage.rs:2`
  - `crates/civis-mcp/tests/mcp_coverage.rs:1`

#### FR-CIV-TEST-009

- Tests:
  - `crates/protocol-3d/tests/protocol_coverage.rs:1`

#### FR-CIV-TEST-021

- Tests:
  - `crates/server/tests/save_load_e2e.rs:1`


### Epic `FR-LANGUAGE` — 1 IDs

#### FR-LANGUAGE-001

- Code:
  - `crates/engine/src/engine/culture_phases.rs:230`
  - `crates/engine/src/engine.rs:169`
  - `crates/engine/src/engine.rs:835`


### Epic `FR-NFR-CIV-LEGENDS-CONFIG` — 1 IDs

#### FR-NFR-CIV-LEGENDS-CONFIG-04



### Epic `FR-NFR-CIV-REL` — 3 IDs

#### FR-NFR-CIV-REL-001


#### FR-NFR-CIV-REL-002


#### FR-NFR-CIV-REL-003



### Epic `FR-NFR-SCALE` — 1 IDs

#### FR-NFR-SCALE-02


