# P3 agent-F

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


## Your slice: 36 IDs in 10 epics


### Epic `FR-CIV-ARCH-B` — 4 IDs

#### FR-CIV-ARCH-B-001

- Tests:
  - `crates/build/src/tiers.rs:472`
- Code:
  - `crates/build/src/tiers.rs:472`

#### FR-CIV-ARCH-B-002

- Tests:
  - `crates/build/src/tiers.rs:491`
- Code:
  - `crates/build/src/tiers.rs:491`

#### FR-CIV-ARCH-B-003

- Tests:
  - `crates/build/src/tiers.rs:505`
- Code:
  - `crates/build/src/tiers.rs:505`

#### FR-CIV-ARCH-B-004

- Tests:
  - `crates/build/src/tiers.rs:514`
- Code:
  - `crates/build/src/tiers.rs:514`


### Epic `FR-CIV-CLIENT` — 3 IDs

#### FR-CIV-CLIENT-006

- Code:
  - `crates/civis-mcp/src/server.rs:346`
  - `crates/civis-mcp/src/server.rs:2264`
  - `crates/server/src/jsonrpc.rs:82`

#### FR-CIV-CLIENT-011

- Code:
  - `clients/bevy-ref/src/tutorial.rs:3`

#### FR-CIV-CLIENT-013

- Code:
  - `clients/bevy-ref/src/civ_history.rs:2`


### Epic `FR-CIV-DET` — 6 IDs

#### FR-CIV-DET-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_det_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_det_002.rs:8`
  - `crates/engine/tests/fr_fr_civ_det_002.rs:18`

#### FR-CIV-DET-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_det_003.rs:1`
  - `crates/engine/tests/fr_fr_civ_det_003.rs:8`
  - `crates/engine/tests/fr_fr_civ_det_003.rs:16`

#### FR-CIV-DET-004

- Tests:
  - `crates/engine/tests/fr_fr_civ_det_004.rs:1`
  - `crates/engine/tests/fr_fr_civ_det_004.rs:8`
  - `crates/engine/tests/fr_fr_civ_det_004.rs:15`

#### FR-CIV-DET-005

- Tests:
  - `crates/engine/tests/fr_fr_civ_det_005.rs:1`
  - `crates/engine/tests/fr_fr_civ_det_005.rs:8`
  - `crates/engine/tests/fr_fr_civ_det_005.rs:19`

#### FR-CIV-DET-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_det_006.rs:1`
  - `crates/engine/tests/fr_fr_civ_det_006.rs:8`
  - `crates/engine/tests/fr_fr_civ_det_006.rs:16`

#### FR-CIV-DET-007

- Tests:
  - `crates/engine/tests/fr_fr_civ_det_007.rs:1`
  - `crates/engine/tests/fr_fr_civ_det_007.rs:8`
  - `crates/engine/tests/fr_fr_civ_det_007.rs:19`


### Epic `FR-CIV-ERA` — 1 IDs

#### FR-CIV-ERA-001

- Tests:
  - `crates/engine/tests/era_emergence_significance_persistence.rs:2`
- Code:
  - `crates/engine/src/engine.rs:538`


### Epic `FR-CIV-IDEOLOGY` — 1 IDs

#### FR-CIV-IDEOLOGY-001

- Tests:
  - `crates/engine/tests/culture_ideology_aggression_persistence.rs:2`
- Code:
  - `crates/engine/src/engine.rs:486`
  - `crates/engine/src/engine.rs:2190`


### Epic `FR-CIV-PSYCHE-N11` — 1 IDs

#### FR-CIV-PSYCHE-N11

- Code:
  - `crates/engine/src/dormant_phases.rs:88`


### Epic `FR-ECON-EMERGE` — 3 IDs

#### FR-ECON-EMERGE-001

- Code:
  - `crates/economy/src/prices.rs:1`

#### FR-ECON-EMERGE-002

- Code:
  - `crates/economy/src/trade.rs:1`

#### FR-ECON-EMERGE-004

- Code:
  - `crates/economy/src/shocks.rs:1`


### Epic `FR-NFR-CIV-AI` — 3 IDs

#### FR-NFR-CIV-AI-001


#### FR-NFR-CIV-AI-002


#### FR-NFR-CIV-AI-003



### Epic `FR-NFR-CIV-MAINT` — 6 IDs

#### FR-NFR-CIV-MAINT-001


#### FR-NFR-CIV-MAINT-002


#### FR-NFR-CIV-MAINT-003


#### FR-NFR-CIV-MAINT-004


#### FR-NFR-CIV-MAINT-005


#### FR-NFR-CIV-MAINT-006



### Epic `FR-NFR-P` — 8 IDs

#### FR-NFR-P-01


#### FR-NFR-P-02


#### FR-NFR-P-03


#### FR-NFR-P-04


#### FR-NFR-P-05


#### FR-NFR-P-06


#### FR-NFR-P-07


#### FR-NFR-P-08


