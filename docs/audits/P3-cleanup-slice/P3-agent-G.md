# P3 agent-G

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


## Your slice: 62 IDs in 10 epics


### Epic `FR-CIV-ARCH-C` — 4 IDs

#### FR-CIV-ARCH-C-001

- Tests:
  - `crates/build/src/tiers.rs:530`
- Code:
  - `crates/build/src/tiers.rs:530`

#### FR-CIV-ARCH-C-002

- Tests:
  - `crates/build/src/tiers.rs:558`
- Code:
  - `crates/build/src/tiers.rs:558`

#### FR-CIV-ARCH-C-003

- Tests:
  - `crates/build/src/tiers.rs:576`
- Code:
  - `crates/build/src/tiers.rs:576`

#### FR-CIV-ARCH-C-004

- Tests:
  - `crates/build/src/tiers.rs:586`
- Code:
  - `crates/build/src/tiers.rs:586`


### Epic `FR-CIV-CLIMATE` — 4 IDs

#### FR-CIV-CLIMATE-1

- Tests:
  - `crates/planet/src/seasonal.rs:181`
- Code:
  - `crates/planet/src/seasonal.rs:181`

#### FR-CIV-CLIMATE-2

- Tests:
  - `crates/planet/src/seasonal.rs:193`
- Code:
  - `crates/planet/src/seasonal.rs:193`

#### FR-CIV-CLIMATE-3

- Tests:
  - `crates/planet/src/seasonal.rs:209`
- Code:
  - `crates/planet/src/seasonal.rs:209`

#### FR-CIV-CLIMATE-4

- Tests:
  - `crates/planet/src/seasonal.rs:218`
- Code:
  - `crates/planet/src/seasonal.rs:218`


### Epic `FR-CIV-DIPLO` — 8 IDs

#### FR-CIV-DIPLO-003-006

- Tests:
  - `crates/diplomacy/src/shadow_networks.rs:660`
- Code:
  - `crates/diplomacy/src/shadow_networks.rs:660`

#### FR-CIV-DIPLO-003-01

- Tests:
  - `crates/diplomacy/src/shadow_networks.rs:403`
  - `crates/diplomacy/src/shadow_networks.rs:405`
  - `crates/diplomacy/src/shadow_networks.rs:433`
- Code:
  - `crates/diplomacy/src/shadow_networks.rs:403`
  - `crates/diplomacy/src/shadow_networks.rs:405`
  - `crates/diplomacy/src/shadow_networks.rs:433`

#### FR-CIV-DIPLO-003-02

- Tests:
  - `crates/diplomacy/src/shadow_networks.rs:470`
  - `crates/diplomacy/src/shadow_networks.rs:472`
  - `crates/diplomacy/src/shadow_networks.rs:509`
- Code:
  - `crates/diplomacy/src/shadow_networks.rs:470`
  - `crates/diplomacy/src/shadow_networks.rs:472`
  - `crates/diplomacy/src/shadow_networks.rs:509`

#### FR-CIV-DIPLO-003-03

- Tests:
  - `crates/diplomacy/src/shadow_networks.rs:520`
  - `crates/diplomacy/src/shadow_networks.rs:522`
  - `crates/diplomacy/src/shadow_networks.rs:560`
- Code:
  - `crates/diplomacy/src/shadow_networks.rs:520`
  - `crates/diplomacy/src/shadow_networks.rs:522`
  - `crates/diplomacy/src/shadow_networks.rs:560`

#### FR-CIV-DIPLO-003-04

- Tests:
  - `crates/diplomacy/src/shadow_networks.rs:594`
  - `crates/diplomacy/src/shadow_networks.rs:596`
- Code:
  - `crates/diplomacy/src/shadow_networks.rs:594`
  - `crates/diplomacy/src/shadow_networks.rs:596`

#### FR-CIV-DIPLO-003-05

- Tests:
  - `crates/diplomacy/src/shadow_networks.rs:620`
  - `crates/diplomacy/src/shadow_networks.rs:622`
- Code:
  - `crates/diplomacy/src/shadow_networks.rs:620`
  - `crates/diplomacy/src/shadow_networks.rs:622`

#### FR-CIV-DIPLO-003-06

- Tests:
  - `crates/diplomacy/src/shadow_networks.rs:658`
- Code:
  - `crates/diplomacy/src/shadow_networks.rs:658`

#### FR-CIV-DIPLO-003-07

- Tests:
  - `crates/diplomacy/src/shadow_networks.rs:710`
  - `crates/diplomacy/src/shadow_networks.rs:712`
- Code:
  - `crates/diplomacy/src/shadow_networks.rs:710`
  - `crates/diplomacy/src/shadow_networks.rs:712`


### Epic `FR-CIV-FAMINE` — 1 IDs

#### FR-CIV-FAMINE-001

- Code:
  - `crates/engine/src/famine.rs:1`


### Epic `FR-CIV-INSTITUTIONS` — 1 IDs

#### FR-CIV-INSTITUTIONS-001

- Tests:
  - `crates/engine/tests/institutions_buildsites_econfocus_persistence.rs:3`
  - `crates/engine/tests/persistence_replay_coverage.rs:12`
- Code:
  - `crates/engine/src/engine.rs:458`
  - `crates/engine/src/engine.rs:2182`


### Epic `FR-CIV-REL` — 1 IDs

#### FR-CIV-REL-007

- Tests:
  - `crates/engine/tests/fr_civ_religion_007_phase_belief.rs:1`


### Epic `FR-EMG` — 24 IDs

#### FR-EMG-001

- Tests:
  - `crates/engine/tests/fr_emg_oracle.rs:7`
  - `crates/engine/tests/fr_emg_oracle.rs:40`
  - `crates/engine/tests/fr_emg_oracle.rs:208`
- Code:
  - `crates/emergence-oracle/src/lib.rs:15`
  - `crates/emergence-oracle/src/oracles/religion.rs:1`
  - `crates/emergence-oracle/src/oracles/religion.rs:17`

#### FR-EMG-002

- Tests:
  - `crates/engine/tests/fr_emg_oracle.rs:8`
  - `crates/engine/tests/fr_emg_oracle.rs:53`
  - `crates/engine/tests/fr_emg_oracle.rs:211`
- Code:
  - `crates/emergence-oracle/src/oracles/language.rs:1`
  - `crates/emergence-oracle/src/oracles/language.rs:19`

#### FR-EMG-003

- Tests:
  - `crates/engine/tests/fr_emg_oracle.rs:9`
  - `crates/engine/tests/fr_emg_oracle.rs:80`
  - `crates/engine/tests/fr_emg_oracle.rs:213`
- Code:
  - `crates/emergence-oracle/src/oracles/economy.rs:1`
  - `crates/emergence-oracle/src/oracles/economy.rs:17`

#### FR-EMG-004

- Tests:
  - `crates/engine/tests/fr_emg_oracle.rs:10`
  - `crates/engine/tests/fr_emg_oracle.rs:92`
  - `crates/engine/tests/fr_emg_oracle.rs:216`
- Code:
  - `crates/emergence-oracle/src/oracles/legends.rs:1`
  - `crates/emergence-oracle/src/oracles/legends.rs:18`

#### FR-EMG-005

- Tests:
  - `crates/engine/tests/fr_emg_oracle.rs:11`
  - `crates/engine/tests/fr_emg_oracle.rs:26`
  - `crates/engine/tests/fr_emg_oracle.rs:103`
- Code:
  - `crates/emergence-oracle/src/bin/oracle_report.rs:17`
  - `crates/emergence-oracle/src/oracles/diplomacy.rs:1`
  - `crates/emergence-oracle/src/oracles/diplomacy.rs:19`

#### FR-EMG-006

- Tests:
  - `crates/engine/tests/fr_emg_oracle.rs:12`
  - `crates/engine/tests/fr_emg_oracle.rs:132`
  - `crates/engine/tests/fr_emg_oracle.rs:222`
- Code:
  - `crates/emergence-oracle/src/oracles/psyche.rs:1`
  - `crates/emergence-oracle/src/oracles/psyche.rs:19`

#### FR-EMG-007

- Tests:
  - `crates/emergence-oracle/src/lib.rs:158`
  - `crates/emergence-oracle/src/lib.rs:159`
  - `crates/engine/tests/fr_emg_oracle.rs:13`
- Code:
  - `crates/emergence-oracle/src/lib.rs:158`
  - `crates/emergence-oracle/src/lib.rs:159`
  - `crates/emergence-oracle/src/oracles/architecture.rs:1`

#### FR-EMG-008

- Tests:
  - `crates/engine/tests/fr_emg_oracle.rs:14`
  - `crates/engine/tests/fr_emg_oracle.rs:26`
  - `crates/engine/tests/fr_emg_oracle.rs:175`
- Code:
  - `crates/emergence-oracle/src/bin/oracle_report.rs:18`
  - `crates/emergence-oracle/src/oracles/creature.rs:1`
  - `crates/emergence-oracle/src/oracles/creature.rs:24`

#### FR-EMG-009

- Code:
  - `crates/emergence-oracle/src/oracles/migration.rs:1`
  - `crates/emergence-oracle/src/oracles/migration.rs:17`

#### FR-EMG-010

- Code:
  - `crates/emergence-oracle/src/oracles/epidemic.rs:1`
  - `crates/emergence-oracle/src/oracles/epidemic.rs:17`
  - `crates/emergence-oracle/src/oracles/trade.rs:1`

#### FR-EMG-012

- Code:
  - `crates/emergence-oracle/src/oracles/festival.rs:1`
  - `crates/emergence-oracle/src/oracles/festival.rs:18`

#### FR-EMG-013

- Code:
  - `crates/emergence-oracle/src/oracles/disaster.rs:1`
  - `crates/emergence-oracle/src/oracles/disaster.rs:17`

#### FR-EMG-014

- Code:
  - `crates/emergence-oracle/src/oracles/mood.rs:1`
  - `crates/emergence-oracle/src/oracles/mood.rs:17`

#### FR-EMG-015

- Code:
  - `crates/emergence-oracle/src/oracles/stratification.rs:1`
  - `crates/emergence-oracle/src/oracles/stratification.rs:18`

#### FR-EMG-016

- Code:
  - `crates/emergence-oracle/src/oracles/religious_conflict.rs:1`
  - `crates/emergence-oracle/src/oracles/religious_conflict.rs:17`

#### FR-EMG-017

- Code:
  - `crates/emergence-oracle/src/oracles/expansion.rs:1`
  - `crates/emergence-oracle/src/oracles/expansion.rs:17`

#### FR-EMG-018

- Code:
  - `crates/emergence-oracle/src/oracles/migration_flow.rs:1`
  - `crates/emergence-oracle/src/oracles/migration_flow.rs:17`

#### FR-EMG-019

- Code:
  - `crates/emergence-oracle/src/oracles/coastal_settlement.rs:1`
  - `crates/emergence-oracle/src/oracles/coastal_settlement.rs:17`

#### FR-EMG-020

- Code:
  - `crates/emergence-oracle/src/oracles/river_trade.rs:1`
  - `crates/emergence-oracle/src/oracles/river_trade.rs:17`

#### FR-EMG-021

- Code:
  - `crates/emergence-oracle/src/oracles/mountain_pass.rs:1`
  - `crates/emergence-oracle/src/oracles/mountain_pass.rs:17`

#### FR-EMG-022

- Code:
  - `crates/emergence-oracle/src/oracles/desert_caravan.rs:1`
  - `crates/emergence-oracle/src/oracles/desert_caravan.rs:17`

#### FR-EMG-023

- Code:
  - `crates/emergence-oracle/src/oracles/genetics.rs:1`
  - `crates/emergence-oracle/src/oracles/genetics.rs:60`

#### FR-EMG-024

- Code:
  - `crates/emergence-oracle/src/oracles/i18n.rs:1`
  - `crates/emergence-oracle/src/oracles/i18n.rs:46`
  - `crates/emergence-oracle/src/oracles/powers.rs:1`

#### FR-EMG-025

- Tests:
  - `crates/emergence-oracle/src/oracles/migration_pressure.rs:54`
- Code:
  - `crates/emergence-oracle/src/oracles/migration_pressure.rs:1`
  - `crates/emergence-oracle/src/oracles/migration_pressure.rs:16`
  - `crates/emergence-oracle/src/oracles/migration_pressure.rs:54`


### Epic `FR-NFR-CIV-DET` — 2 IDs

#### FR-NFR-CIV-DET-001


#### FR-NFR-CIV-DET-002



### Epic `FR-NFR-CIV-PERF` — 11 IDs

#### FR-NFR-CIV-PERF-001


#### FR-NFR-CIV-PERF-002


#### FR-NFR-CIV-PERF-003


#### FR-NFR-CIV-PERF-004


#### FR-NFR-CIV-PERF-005


#### FR-NFR-CIV-PERF-006


#### FR-NFR-CIV-PERF-007


#### FR-NFR-CIV-PERF-008


#### FR-NFR-CIV-PERF-900


#### FR-NFR-CIV-PERF-901


#### FR-NFR-CIV-PERF-902



### Epic `FR-NFR-R` — 6 IDs

#### FR-NFR-R-01


#### FR-NFR-R-02


#### FR-NFR-R-03


#### FR-NFR-R-04


#### FR-NFR-R-05


#### FR-NFR-R-06


