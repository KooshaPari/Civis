# P3 agent-A

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


## Your slice: 23 IDs in 11 epics


### Epic `FR-ASSET-PIPELINE` — 2 IDs

#### FR-ASSET-PIPELINE-001

- Code:
  - `crates/asset-pipeline/src/lib.rs:3`
  - `crates/asset-pipeline/src/lib.rs:18`
  - `crates/asset-pipeline/src/lib.rs:57`

#### FR-ASSET-PIPELINE-002

- Code:
  - `crates/asset-pipeline/Cargo.toml:8`
  - `crates/asset-pipeline/src/bin/svg_export.rs:20`
  - `crates/asset-pipeline/src/error.rs:9`


### Epic `FR-CIV-ARCH-D` — 4 IDs

#### FR-CIV-ARCH-D-001

- Tests:
  - `crates/build/src/tiers.rs:602`
- Code:
  - `crates/build/src/tiers.rs:602`

#### FR-CIV-ARCH-D-002

- Tests:
  - `crates/build/src/tiers.rs:618`
- Code:
  - `crates/build/src/tiers.rs:618`

#### FR-CIV-ARCH-D-003

- Tests:
  - `crates/build/src/tiers.rs:633`
- Code:
  - `crates/build/src/tiers.rs:633`

#### FR-CIV-ARCH-D-004

- Tests:
  - `crates/build/src/tiers.rs:643`
- Code:
  - `crates/build/src/tiers.rs:643`


### Epic `FR-CIV-COHESION` — 1 IDs

#### FR-CIV-COHESION-001

- Code:
  - `crates/engine/src/engine/social_settlement_phases.rs:272`


### Epic `FR-CIV-DIPLOMACY` — 2 IDs

#### FR-CIV-DIPLOMACY-001

- Tests:
  - `crates/engine/tests/diplomacy_flow.rs:20`
  - `crates/engine/tests/persistence_replay_coverage.rs:15`
- Code:
  - `crates/engine/src/engine.rs:2168`
  - `crates/engine/src/engine.rs:2343`

#### FR-CIV-DIPLOMACY-004

- Code:
  - `crates/diplomacy/src/stance.rs:1`
  - `crates/engine/src/engine.rs:440`
  - `crates/engine/src/engine.rs:1003`


### Epic `FR-CIV-FEST` — 1 IDs

#### FR-CIV-FEST-001

- Code:
  - `crates/engine/src/festivals.rs:1`


### Epic `FR-CIV-INT` — 1 IDs

#### FR-CIV-INT-001

- Tests:
  - `crates/engine/tests/fr_engine_replay_integrity_tests.rs:5`
  - `crates/engine/tests/fr_engine_replay_integrity_tests.rs:113`
  - `crates/engine/tests/fr_engine_replay_integrity_tests.rs:116`


### Epic `FR-CIV-SERVER` — 1 IDs

#### FR-CIV-SERVER-003

- Tests:
  - `crates/server/src/jsonrpc.rs:4957`
  - `crates/server/src/jsonrpc.rs:4996`
  - `crates/server/src/jsonrpc.rs:5032`
- Code:
  - `crates/civis-mcp/src/server.rs:2287`
  - `crates/civis-mcp/src/server.rs:2310`
  - `crates/server/src/jsonrpc.rs:84`


### Epic `FR-FR-CORE` — 1 IDs

#### FR-FR-CORE-009

- Tests:
  - `crates/engine/tests/fr_core_cluster.rs:1`
  - `crates/engine/tests/fr_core_cluster.rs:15`
  - `crates/engine/tests/fr_core_cluster.rs:184`


### Epic `FR-NFR-CIV-DEV-HYGIENE` — 1 IDs

#### FR-NFR-CIV-DEV-HYGIENE-001



### Epic `FR-NFR-CIV-PORT` — 3 IDs

#### FR-NFR-CIV-PORT-001


#### FR-NFR-CIV-PORT-002


#### FR-NFR-CIV-PORT-003



### Epic `FR-NFR-S` — 6 IDs

#### FR-NFR-S-01


#### FR-NFR-S-02


#### FR-NFR-S-03


#### FR-NFR-S-04


#### FR-NFR-S-05


#### FR-NFR-S-06


