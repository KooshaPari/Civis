# P3 agent-E

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


## Your slice: 24 IDs in 10 epics


### Epic `FR-CIV-ARCH-A` — 3 IDs

#### FR-CIV-ARCH-A-001

- Tests:
  - `crates/build/src/tiers.rs:405`
  - `scripts/traceability/test_fr_audit_classification.py:101`
- Code:
  - `crates/build/src/tiers.rs:405`

#### FR-CIV-ARCH-A-002

- Tests:
  - `crates/build/src/tiers.rs:426`
- Code:
  - `crates/build/src/tiers.rs:426`

#### FR-CIV-ARCH-A-003

- Tests:
  - `crates/build/src/tiers.rs:448`
- Code:
  - `crates/build/src/tiers.rs:448`


### Epic `FR-CIV-CARAVAN` — 1 IDs

#### FR-CIV-CARAVAN-001

- Code:
  - `crates/engine/src/caravan.rs:3`


### Epic `FR-CIV-CULTURE` — 1 IDs

#### FR-CIV-CULTURE-001

- Tests:
  - `crates/engine/tests/culture_ideology_aggression_persistence.rs:2`
- Code:
  - `crates/engine/src/engine.rs:478`
  - `crates/engine/src/engine.rs:2190`


### Epic `FR-CIV-EMERGENT-MIGRATION` — 1 IDs

#### FR-CIV-EMERGENT-MIGRATION-001

- Code:
  - `crates/engine/src/emergent_migration.rs:1`
  - `crates/engine/src/emergent_migration.rs:25`


### Epic `FR-CIV-GOV` — 5 IDs

#### FR-CIV-GOV-003

- Tests:
  - `crates/engine/tests/fr_civ_gov_institutions.rs:19`
  - `crates/engine/tests/fr_civ_gov_institutions.rs:132`
  - `crates/engine/tests/fr_civ_gov_institutions.rs:133`
- Code:
  - `crates/civ-institutions/src/lib.rs:38`
  - `crates/engine/src/engine.rs:856`
  - `crates/engine/src/engine.rs:865`

#### FR-CIV-GOV-010

- Tests:
  - `crates/engine/tests/fr_civ_gov_mood.rs:1`
- Code:
  - `crates/engine/src/engine.rs:3431`

#### FR-CIV-GOV-020

- Tests:
  - `crates/engine/tests/fr_civ_gov_stratification.rs:1`
  - `crates/engine/tests/fr_civ_gov_stratification.rs:28`
  - `crates/engine/tests/fr_civ_gov_stratification.rs:29`
- Code:
  - `crates/engine/src/social_types.rs:5`
  - `crates/engine/src/social_types.rs:69`

#### FR-CIV-GOV-100

- Tests:
  - `crates/engine/tests/fr_emergence_quality.rs:347`
- Code:
  - `crates/engine/src/engine.rs:870`
  - `crates/engine/src/engine.rs:875`
  - `crates/engine/src/engine.rs:880`

#### FR-CIV-GOV-200

- Code:
  - `crates/engine/src/engine/social_settlement_phases.rs:179`


### Epic `FR-CIV-PLANET` — 1 IDs

#### FR-CIV-PLANET-050

- Tests:
  - `crates/planet/src/geology.rs:651`
  - `crates/planet/src/geology.rs:658`
  - `crates/planet/src/geology.rs:665`
- Code:
  - `crates/planet/src/geology.rs:651`
  - `crates/planet/src/geology.rs:658`
  - `crates/planet/src/geology.rs:665`


### Epic `FR-DIP` — 1 IDs

#### FR-DIP-002

- Code:
  - `crates/diplomacy/src/effects.rs:1`


### Epic `FR-NFR-CIV-ACC` — 4 IDs

#### FR-NFR-CIV-ACC-001


#### FR-NFR-CIV-ACC-002


#### FR-NFR-CIV-ACC-003


#### FR-NFR-CIV-ACC-004



### Epic `FR-NFR-CIV-LEGENDS-SCALE` — 1 IDs

#### FR-NFR-CIV-LEGENDS-SCALE-02



### Epic `FR-NFR-O` — 6 IDs

#### FR-NFR-O-01


#### FR-NFR-O-02


#### FR-NFR-O-03


#### FR-NFR-O-04


#### FR-NFR-O-05


#### FR-NFR-O-06


