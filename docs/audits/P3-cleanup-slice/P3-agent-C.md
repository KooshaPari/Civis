# P3 agent-C

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


## Your slice: 24 IDs in 11 epics


### Epic `FR-CIV-AGGRESSION` — 1 IDs

#### FR-CIV-AGGRESSION-001

- Tests:
  - `crates/engine/tests/culture_ideology_aggression_persistence.rs:3`
- Code:
  - `crates/engine/src/engine.rs:492`
  - `crates/engine/src/engine.rs:2191`


### Epic `FR-CIV-BEVY` — 4 IDs

#### FR-CIV-BEVY-028

- Tests:
  - `crates/server/tests/ws_smoke.rs:1513`
  - `crates/server/tests/ws_smoke.rs:1522`
- Code:
  - `crates/server/src/ws_bridge.rs:57`

#### FR-CIV-BEVY-034

- Tests:
  - `clients/bevy-ref/src/diplomacy_ui.rs:625`
- Code:
  - `clients/bevy-ref/src/diplomacy_ui.rs:625`

#### FR-CIV-BEVY-035

- Tests:
  - `clients/bevy-ref/src/lib.rs:1973`
- Code:
  - `clients/bevy-ref/src/lib.rs:1973`

#### FR-CIV-BEVY-036

- Code:
  - `clients/bevy-ref/src/menus.rs:4`
  - `clients/bevy-ref/src/menus.rs:1348`


### Epic `FR-CIV-CONTENT` — 1 IDs

#### FR-CIV-CONTENT-001

- Code:
  - `crates/engine/src/emergence_coupling.rs:471`


### Epic `FR-CIV-ECON-FOCUS` — 1 IDs

#### FR-CIV-ECON-FOCUS-001

- Tests:
  - `crates/engine/tests/institutions_buildsites_econfocus_persistence.rs:4`
  - `crates/engine/tests/persistence_replay_coverage.rs:14`
- Code:
  - `crates/engine/src/engine.rs:472`
  - `crates/engine/src/engine.rs:2183`


### Epic `FR-CIV-GENETICS-SEED` — 3 IDs

#### FR-CIV-GENETICS-SEED-001

- Code:
  - `crates/engine/src/engine/engine_tests.rs:2941`

#### FR-CIV-GENETICS-SEED-002

- Code:
  - `crates/engine/src/engine/engine_tests.rs:2991`

#### FR-CIV-GENETICS-SEED-003

- Code:
  - `crates/engine/src/engine/engine_tests.rs:3037`


### Epic `FR-CIV-NEEDS-DECAY` — 1 IDs

#### FR-CIV-NEEDS-DECAY-01

- Tests:
  - `crates/needs/src/decay.rs:336`
  - `crates/needs/src/decay.rs:361`
  - `crates/needs/src/decay.rs:386`
- Code:
  - `crates/needs/src/decay.rs:32`
  - `crates/needs/src/decay.rs:219`
  - `crates/needs/src/decay.rs:336`


### Epic `FR-CIV-UNREST` — 1 IDs

#### FR-CIV-UNREST-002

- Tests:
  - `crates/engine/tests/riot_migrant_taxation_persistence.rs:2`
- Code:
  - `crates/engine/src/engine.rs:504`
  - `crates/engine/src/engine.rs:526`
  - `crates/engine/src/engine.rs:2200`


### Epic `FR-MUSIC` — 1 IDs

#### FR-MUSIC-001

- Tests:
  - `crates/engine/src/engine/engine_tests.rs:3732`
- Code:
  - `crates/engine/src/engine/engine_tests.rs:3732`


### Epic `FR-NFR-CIV-LEGENDS-LOUD` — 1 IDs

#### FR-NFR-CIV-LEGENDS-LOUD-03



### Epic `FR-NFR-CIV-SCALE` — 9 IDs

#### FR-NFR-CIV-SCALE-001


#### FR-NFR-CIV-SCALE-002


#### FR-NFR-CIV-SCALE-003


#### FR-NFR-CIV-SCALE-004


#### FR-NFR-CIV-SCALE-900


#### FR-NFR-CIV-SCALE-901


#### FR-NFR-CIV-SCALE-902


#### FR-NFR-CIV-SCALE-910


#### FR-NFR-CIV-SCALE-920



### Epic `FR-VIEWPORT` — 1 IDs

#### FR-VIEWPORT-001

- Code:
  - `crates/civis-cli/src/bin/three_d_quality.rs:54`

