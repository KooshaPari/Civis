# P3 agent-D

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


## Your slice: 25 IDs in 11 epics


### Epic `FR-CIV-ARCH` — 1 IDs

#### FR-CIV-ARCH-00

- Tests:
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:2`


### Epic `FR-CIV-CA` — 1 IDs

#### FR-CIV-CA-011

- Code:
  - `crates/voxel/src/fluid_ca.rs:1360`
  - `crates/voxel/src/fluid_ca.rs:1698`


### Epic `FR-CIV-CORE` — 1 IDs

#### FR-CIV-CORE-021

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:765`


### Epic `FR-CIV-EMERGE-DASH` — 1 IDs

#### FR-CIV-EMERGE-DASH-001

- Code:
  - `clients/bevy-ref/src/emergence_dashboard.rs:3`


### Epic `FR-CIV-GODTOOL` — 1 IDs

#### FR-CIV-GODTOOL-001

- Code:
  - `crates/civis-mcp/src/server.rs:148`


### Epic `FR-CIV-PBR` — 3 IDs

#### FR-CIV-PBR-009

- Tests:
  - `crates/voxel/src/material_pbr.rs:1461`
- Code:
  - `crates/voxel/src/material_pbr.rs:754`
  - `crates/voxel/src/material_pbr.rs:1461`

#### FR-CIV-PBR-010

- Tests:
  - `crates/voxel/src/material_pbr.rs:1560`
  - `crates/voxel/src/material_pbr.rs:1562`
  - `crates/voxel/src/material_pbr.rs:1594`
- Code:
  - `CHANGELOG.md:16`
  - `crates/voxel/src/material_pbr.rs:14`
  - `crates/voxel/src/material_pbr.rs:1560`

#### FR-CIV-PBR-011

- Code:
  - `CHANGELOG.md:17`
  - `crates/voxel/src/atlas/gpu_atlas.rs:1`
  - `crates/voxel/src/atlas/gpu_atlas.rs:40`


### Epic `FR-CIV-WARFARE` — 4 IDs

#### FR-CIV-WARFARE-001

- Code:
  - `crates/tactics/src/war_from_diplomacy.rs:1`

#### FR-CIV-WARFARE-002

- Code:
  - `crates/tactics/src/doctrine_evolution.rs:1`

#### FR-CIV-WARFARE-003

- Code:
  - `crates/tactics/src/war_economy.rs:1`

#### FR-CIV-WARFARE-004

- Code:
  - `crates/tactics/src/war_legends.rs:1`


### Epic `FR-NFR-C` — 7 IDs

#### FR-NFR-C-01


#### FR-NFR-C-02


#### FR-NFR-C-03


#### FR-NFR-C-04


#### FR-NFR-C-05


#### FR-NFR-C-06


#### FR-NFR-C-07



### Epic `FR-NFR-CIV-LEGENDS-PERF` — 1 IDs

#### FR-NFR-CIV-LEGENDS-PERF-01



### Epic `FR-NFR-CIV-SEC` — 4 IDs

#### FR-NFR-CIV-SEC-001


#### FR-NFR-CIV-SEC-002


#### FR-NFR-CIV-SEC-003


#### FR-NFR-CIV-SEC-004



### Epic `NFR-CIV-SCALE-PERF` — 1 IDs

#### NFR-CIV-SCALE-PERF-900

- Tests:
  - `crates/voxel/src/scale_stream.rs:427`
  - `crates/voxel/src/scale_stream.rs:479`
  - `crates/voxel/src/scale_stream.rs:509`
- Code:
  - `crates/voxel/src/scale_stream.rs:1`
  - `crates/voxel/src/scale_stream.rs:16`
  - `crates/voxel/src/scale_stream.rs:58`

