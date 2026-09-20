# P1 agent-G

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


## Your slice: 52 IDs in 15 epics


### Epic `FR-CIV-ACTOR-001-LIFECYCLE` — 1 IDs

#### FR-CIV-ACTOR-001-LIFECYCLE

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:154`
  - `crates/build/tests/fr_matrix_batch12.rs:157`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:213`
  - `PLAN.md:145`
  - `PLAN.md:146`


### Epic `FR-CIV-CLIENT-GODOT` — 2 IDs

#### FR-CIV-CLIENT-GODOT-001

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:627`
  - `crates/build/tests/fr_matrix_batch12.rs:630`
- Spec/trace:
  - `agileplus-specs/civ-012-godot-secondary-client/plan.md:5`
  - `agileplus-specs/civ-012-godot-secondary-client/spec.md:26`
  - `docs/reference/agileplus-artifacts-index.md:220`

#### FR-CIV-CLIENT-GODOT-002

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:639`
  - `crates/build/tests/fr_matrix_batch12.rs:642`
- Spec/trace:
  - `agileplus-specs/civ-012-godot-secondary-client/plan.md:11`
  - `agileplus-specs/civ-012-godot-secondary-client/spec.md:27`
  - `docs/reference/agileplus-artifacts-index.md:220`


### Epic `FR-CIV-ECON` — 2 IDs

#### FR-CIV-ECON-003

- Tests:
  - `crates/economy/tests/fr_civ_econ_cluster.rs:2`
  - `crates/economy/tests/fr_civ_econ_cluster.rs:13`
  - `crates/economy/tests/fr_civ_econ_cluster.rs:74`
- Spec/trace:
  - `docs/design/civ-economy-emergent-markets.md:7`
  - `docs/reference/FR_TRACKER.md:9`
  - `docs/reports/STATUS_REPORT.md:91`

#### FR-CIV-ECON-004

- Tests:
  - `crates/economy/tests/fr_civ_econ_tests.rs:3`
  - `crates/economy/tests/fr_civ_econ_tests.rs:20`
  - `crates/economy/tests/fr_civ_econ_tests.rs:23`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:76`
  - `docs/reference/CODE_ENTITY_MAP.md:8`
  - `docs/reference/FR_TRACKER.md:10`


### Epic `FR-CIV-INFOVIEW` — 8 IDs

#### FR-CIV-INFOVIEW-912

- Tests:
  - `crates/engine/tests/fr_civ_infoview_inspect_cluster.rs:32`
  - `crates/engine/tests/fr_civ_infoview_inspect_cluster.rs:472`
  - `crates/engine/tests/fr_civ_infoview_inspect_cluster.rs:475`
- Spec/trace:
  - `docs/agileplus/epics/civ-w4-perception.md:13`
  - `docs/agileplus/epics/civ-w4-perception.md:31`
  - `docs/agileplus/README.md:23`

#### FR-CIV-INFOVIEW-914

- Tests:
  - `crates/engine/tests/fr_civ_infoview_inspect_cluster.rs:40`
  - `crates/engine/tests/fr_civ_infoview_inspect_cluster.rs:72`
  - `crates/engine/tests/fr_civ_infoview_inspect_cluster.rs:691`
- Spec/trace:
  - `docs/agileplus/epics/civ-w4-perception.md:15`
  - `docs/agileplus/epics/civ-w4-perception.md:33`
  - `docs/agileplus/README.md:23`

#### FR-CIV-INFOVIEW-915

- Tests:
  - `crates/engine/tests/fr_fr_civ_infoview_915.rs:1`
  - `crates/engine/tests/fr_fr_civ_infoview_915.rs:4`
  - `crates/engine/tests/fr_fr_civ_infoview_915.rs:13`
- Spec/trace:
  - `docs/design/info-views.md:110`
  - `docs/traceability/fr-civ-infoview-915/fr-civ-infoview-915-adr.md:1`
  - `docs/traceability/fr-civ-infoview-915/fr-civ-infoview-915-adr.md:6`

#### FR-CIV-INFOVIEW-916

- Tests:
  - `crates/engine/tests/fr_fr_civ_infoview_916.rs:1`
  - `crates/engine/tests/fr_fr_civ_infoview_916.rs:4`
  - `crates/engine/tests/fr_fr_civ_infoview_916.rs:13`
- Spec/trace:
  - `docs/design/info-views.md:111`
  - `docs/traceability/fr-civ-infoview-916/fr-civ-infoview-916-adr.md:1`
  - `docs/traceability/fr-civ-infoview-916/fr-civ-infoview-916-adr.md:6`

#### FR-CIV-INFOVIEW-917

- Tests:
  - `crates/engine/tests/fr_fr_civ_infoview_917.rs:1`
  - `crates/engine/tests/fr_fr_civ_infoview_917.rs:4`
  - `crates/engine/tests/fr_fr_civ_infoview_917.rs:13`
- Spec/trace:
  - `docs/design/info-views.md:112`
  - `docs/traceability/fr-civ-infoview-917/fr-civ-infoview-917-adr.md:1`
  - `docs/traceability/fr-civ-infoview-917/fr-civ-infoview-917-adr.md:6`

#### FR-CIV-INFOVIEW-918

- Tests:
  - `crates/engine/tests/fr_fr_civ_infoview_918.rs:1`
  - `crates/engine/tests/fr_fr_civ_infoview_918.rs:4`
  - `crates/engine/tests/fr_fr_civ_infoview_918.rs:13`
- Spec/trace:
  - `docs/design/info-views.md:113`
  - `docs/traceability/fr-civ-infoview-918/fr-civ-infoview-918-adr.md:1`
  - `docs/traceability/fr-civ-infoview-918/fr-civ-infoview-918-adr.md:6`

#### FR-CIV-INFOVIEW-919

- Tests:
  - `crates/engine/tests/fr_fr_civ_infoview_919.rs:1`
  - `crates/engine/tests/fr_fr_civ_infoview_919.rs:4`
  - `crates/engine/tests/fr_fr_civ_infoview_919.rs:13`
- Spec/trace:
  - `docs/design/info-views.md:114`
  - `docs/traceability/fr-civ-infoview-919/fr-civ-infoview-919-adr.md:1`
  - `docs/traceability/fr-civ-infoview-919/fr-civ-infoview-919-adr.md:6`

#### FR-CIV-INFOVIEW-921

- Tests:
  - `crates/engine/tests/fr_fr_civ_infoview_921.rs:1`
  - `crates/engine/tests/fr_fr_civ_infoview_921.rs:4`
  - `crates/engine/tests/fr_fr_civ_infoview_921.rs:13`
- Spec/trace:
  - `docs/design/info-views.md:116`
  - `docs/traceability/fr-civ-infoview-921/fr-civ-infoview-921-adr.md:1`
  - `docs/traceability/fr-civ-infoview-921/fr-civ-infoview-921-adr.md:6`


### Epic `FR-CIV-LEGENDS-NARRATOR` — 1 IDs

#### FR-CIV-LEGENDS-NARRATOR-13

- Tests:
  - `crates/legends/tests/fr_fr_civ_legends_narrator_13.rs:1`
  - `crates/legends/tests/fr_fr_civ_legends_narrator_13.rs:10`
  - `crates/legends/tests/fr_fr_civ_legends_narrator_13.rs:21`
- Spec/trace:
  - `docs/design/legends-engine.md:448`
  - `docs/traceability/fr-civ-legends-narrator-13/fr-civ-legends-narrator-13-adr.md:1`
  - `docs/traceability/fr-civ-legends-narrator-13/fr-civ-legends-narrator-13-adr.md:6`


### Epic `FR-CIV-MARKET` — 8 IDs

#### FR-CIV-MARKET-001

- Tests:
  - `crates/economy/tests/fr_fr_civ_market_001.rs:1`
  - `crates/economy/tests/fr_fr_civ_market_001.rs:6`
- Spec/trace:
  - `docs/design/civ-economy-emergent-markets.md:7`
  - `docs/design/ECONOMY_EMERGENCE.md:25`
  - `docs/design/master-roadmap.md:25`

#### FR-CIV-MARKET-002

- Tests:
  - `crates/economy/tests/fr_fr_civ_market_002.rs:1`
  - `crates/economy/tests/fr_fr_civ_market_002.rs:6`
- Spec/trace:
  - `docs/design/civ-economy-emergent-markets.md:46`
  - `docs/design/polities-markets.md:111`
  - `docs/traceability/fr-civ-market-002/fr-civ-market-002-adr.md:1`

#### FR-CIV-MARKET-003

- Tests:
  - `crates/economy/tests/fr_fr_civ_market_003.rs:1`
  - `crates/economy/tests/fr_fr_civ_market_003.rs:6`
- Spec/trace:
  - `docs/design/polities-markets.md:122`
  - `docs/traceability/fr-civ-market-003/fr-civ-market-003-adr.md:1`
  - `docs/traceability/fr-civ-market-003/fr-civ-market-003-adr.md:6`

#### FR-CIV-MARKET-004

- Tests:
  - `crates/economy/tests/fr_fr_civ_market_004.rs:1`
  - `crates/economy/tests/fr_fr_civ_market_004.rs:6`
- Spec/trace:
  - `docs/design/polities-markets.md:126`
  - `docs/traceability/fr-civ-market-004/fr-civ-market-004-adr.md:1`
  - `docs/traceability/fr-civ-market-004/fr-civ-market-004-adr.md:6`

#### FR-CIV-MARKET-005

- Tests:
  - `crates/economy/tests/fr_fr_civ_market_005.rs:1`
  - `crates/economy/tests/fr_fr_civ_market_005.rs:6`
- Spec/trace:
  - `docs/design/polities-markets.md:138`
  - `docs/traceability/fr-civ-market-005/fr-civ-market-005-adr.md:1`
  - `docs/traceability/fr-civ-market-005/fr-civ-market-005-adr.md:6`

#### FR-CIV-MARKET-006

- Tests:
  - `crates/economy/tests/fr_fr_civ_market_006.rs:1`
  - `crates/economy/tests/fr_fr_civ_market_006.rs:6`
- Spec/trace:
  - `docs/design/civ-economy-emergent-markets.md:109`
  - `docs/design/ECONOMY_EMERGENCE.md:67`
  - `docs/design/polities-markets.md:140`

#### FR-CIV-MARKET-007

- Tests:
  - `crates/economy/tests/fr_fr_civ_market_007.rs:1`
  - `crates/economy/tests/fr_fr_civ_market_007.rs:6`
- Spec/trace:
  - `docs/design/civ-economy-emergent-markets.md:155`
  - `docs/design/polities-markets.md:144`
  - `docs/traceability/fr-civ-market-007/fr-civ-market-007-adr.md:1`

#### FR-CIV-MARKET-008

- Tests:
  - `crates/economy/tests/fr_fr_civ_market_008.rs:1`
  - `crates/economy/tests/fr_fr_civ_market_008.rs:6`
- Spec/trace:
  - `docs/design/civ-economy-emergent-markets.md:85`
  - `docs/design/polities-markets.md:146`
  - `docs/traceability/fr-civ-market-008/fr-civ-market-008-adr.md:1`


### Epic `FR-CIV-PERF-BUILD` — 1 IDs

#### FR-CIV-PERF-BUILD-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_build_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_build_001.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_build_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3215`
  - `docs/traceability/fr-civ-perf-build-001/fr-civ-perf-build-001-adr.md:1`
  - `docs/traceability/fr-civ-perf-build-001/fr-civ-perf-build-001-adr.md:6`


### Epic `FR-CIV-RENDER` — 2 IDs

#### FR-CIV-RENDER-001

- Tests:
  - `crates/voxel/tests/fr_civ_render_001_chunk_stream_radius.rs:1`
  - `crates/voxel/tests/fr_civ_render_001_chunk_stream_radius.rs:51`
  - `crates/voxel/tests/fr_civ_render_001_chunk_stream_radius.rs:100`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:96`
  - `docs/guides/voxel-emergent-vision-and-migration.md:148`
  - `docs/guides/voxel-emergent-vision-and-migration.md:152`

#### FR-CIV-RENDER-002

- Tests:
  - `crates/voxel/tests/fr_civ_render_002_translucency.rs:1`
  - `crates/voxel/tests/fr_civ_render_002_translucency.rs:46`
  - `crates/voxel/tests/fr_civ_render_002_translucency.rs:74`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:96`
  - `docs/guides/voxel-emergent-vision-and-migration.md:148`
  - `docs/guides/voxel-emergent-vision-and-migration.md:153`


### Epic `FR-CIV-RTS-NATION` — 2 IDs

#### FR-CIV-RTS-NATION-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_nation_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_nation_001.rs:4`
  - `crates/engine/tests/fr_fr_civ_rts_nation_001.rs:11`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3209`
  - `docs/traceability/fr-civ-rts-nation-001/fr-civ-rts-nation-001-adr.md:1`
  - `docs/traceability/fr-civ-rts-nation-001/fr-civ-rts-nation-001-adr.md:6`

#### FR-CIV-RTS-NATION-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_nation_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_nation_002.rs:4`
  - `crates/engine/tests/fr_fr_civ_rts_nation_002.rs:11`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3220`
  - `docs/traceability/fr-civ-rts-nation-002/fr-civ-rts-nation-002-adr.md:1`
  - `docs/traceability/fr-civ-rts-nation-002/fr-civ-rts-nation-002-adr.md:6`


### Epic `FR-CIV-TERRAIN` — 6 IDs

#### FR-CIV-TERRAIN-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_terrain_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_terrain_001.rs:5`
- Spec/trace:
  - `agileplus-specs/civ-014-terrain-playable-hardening/spec.md:33`
  - `docs/traceability/fr-civ-terrain-001/fr-civ-terrain-001-adr.md:1`
  - `docs/traceability/fr-civ-terrain-001/fr-civ-terrain-001-adr.md:6`

#### FR-CIV-TERRAIN-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_terrain_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_terrain_002.rs:5`
- Spec/trace:
  - `agileplus-specs/civ-014-terrain-playable-hardening/spec.md:38`
  - `docs/traceability/fr-civ-terrain-002/fr-civ-terrain-002-adr.md:1`
  - `docs/traceability/fr-civ-terrain-002/fr-civ-terrain-002-adr.md:6`

#### FR-CIV-TERRAIN-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_terrain_003.rs:1`
  - `crates/engine/tests/fr_fr_civ_terrain_003.rs:5`
- Spec/trace:
  - `agileplus-specs/civ-014-terrain-playable-hardening/spec.md:41`
  - `docs/traceability/fr-civ-terrain-003/fr-civ-terrain-003-adr.md:1`
  - `docs/traceability/fr-civ-terrain-003/fr-civ-terrain-003-adr.md:6`

#### FR-CIV-TERRAIN-004

- Tests:
  - `crates/engine/tests/fr_fr_civ_terrain_004.rs:1`
  - `crates/engine/tests/fr_fr_civ_terrain_004.rs:5`
- Spec/trace:
  - `agileplus-specs/civ-014-terrain-playable-hardening/spec.md:44`
  - `docs/traceability/fr-civ-terrain-004/fr-civ-terrain-004-adr.md:1`
  - `docs/traceability/fr-civ-terrain-004/fr-civ-terrain-004-adr.md:6`

#### FR-CIV-TERRAIN-005

- Tests:
  - `crates/engine/tests/fr_fr_civ_terrain_005.rs:1`
  - `crates/engine/tests/fr_fr_civ_terrain_005.rs:5`
- Spec/trace:
  - `agileplus-specs/civ-014-terrain-playable-hardening/spec.md:47`
  - `docs/traceability/fr-civ-terrain-005/fr-civ-terrain-005-adr.md:1`
  - `docs/traceability/fr-civ-terrain-005/fr-civ-terrain-005-adr.md:6`

#### FR-CIV-TERRAIN-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_terrain_006.rs:1`
  - `crates/engine/tests/fr_fr_civ_terrain_006.rs:5`
- Spec/trace:
  - `agileplus-specs/civ-014-terrain-playable-hardening/spec.md:50`
  - `docs/traceability/fr-civ-terrain-006/fr-civ-terrain-006-adr.md:1`
  - `docs/traceability/fr-civ-terrain-006/fr-civ-terrain-006-adr.md:6`


### Epic `FR-DET` — 7 IDs

#### FR-DET-001

- Tests:
  - `crates/engine/tests/fr_fr_det_001.rs:1`
  - `crates/engine/tests/fr_fr_det_001.rs:5`
  - `crates/engine/tests/fr_fr_det_001.rs:9`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:439`
  - `docs/traceability/fr-det-001/fr-det-001-adr.md:1`
  - `docs/traceability/fr-det-001/fr-det-001-adr.md:6`

#### FR-DET-002

- Tests:
  - `crates/engine/tests/fr_fr_det_002.rs:1`
  - `crates/engine/tests/fr_fr_det_002.rs:5`
  - `crates/engine/tests/fr_fr_det_002.rs:9`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:290`
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:440`
  - `docs/traceability/fr-det-002/fr-det-002-adr.md:1`

#### FR-DET-003

- Tests:
  - `crates/engine/tests/fr_fr_det_003.rs:1`
  - `crates/engine/tests/fr_fr_det_003.rs:5`
  - `crates/engine/tests/fr_fr_det_003.rs:9`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:441`
  - `docs/traceability/fr-det-003/fr-det-003-adr.md:1`
  - `docs/traceability/fr-det-003/fr-det-003-adr.md:6`

#### FR-DET-004

- Tests:
  - `crates/engine/tests/fr_fr_det_004.rs:1`
  - `crates/engine/tests/fr_fr_det_004.rs:5`
  - `crates/engine/tests/fr_fr_det_004.rs:9`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:442`
  - `docs/traceability/fr-det-004/fr-det-004-adr.md:1`
  - `docs/traceability/fr-det-004/fr-det-004-adr.md:6`

#### FR-DET-005

- Tests:
  - `crates/engine/tests/fr_fr_det_005.rs:1`
  - `crates/engine/tests/fr_fr_det_005.rs:5`
  - `crates/engine/tests/fr_fr_det_005.rs:9`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:443`
  - `docs/traceability/fr-det-005/fr-det-005-adr.md:1`
  - `docs/traceability/fr-det-005/fr-det-005-adr.md:6`

#### FR-DET-006

- Tests:
  - `crates/engine/tests/fr_fr_det_006.rs:1`
  - `crates/engine/tests/fr_fr_det_006.rs:5`
  - `crates/engine/tests/fr_fr_det_006.rs:9`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:342`
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:444`
  - `docs/traceability/fr-det-006/fr-det-006-adr.md:1`

#### FR-DET-007

- Tests:
  - `crates/engine/tests/fr_fr_det_007.rs:1`
  - `crates/engine/tests/fr_fr_det_007.rs:5`
  - `crates/engine/tests/fr_fr_det_007.rs:9`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:445`
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:451`
  - `docs/traceability/fr-det-007/fr-det-007-adr.md:1`


### Epic `FR-METRICS` — 2 IDs

#### FR-METRICS-004

- Tests:
  - `crates/engine/tests/fr_fr_metrics_004.rs:1`
  - `crates/engine/tests/fr_fr_metrics_004.rs:8`
  - `crates/engine/tests/fr_fr_metrics_004.rs:18`
- Spec/trace:
  - `docs/FR.md:36`
  - `docs/traceability/fr-metrics-004/fr-metrics-004-adr.md:1`
  - `docs/traceability/fr-metrics-004/fr-metrics-004-adr.md:6`

#### FR-METRICS-005

- Tests:
  - `crates/engine/tests/fr_fr_metrics_005.rs:1`
  - `crates/engine/tests/fr_fr_metrics_005.rs:8`
  - `crates/engine/tests/fr_fr_metrics_005.rs:16`
- Spec/trace:
  - `docs/FR.md:37`
  - `docs/traceability/fr-metrics-005/fr-metrics-005-adr.md:1`
  - `docs/traceability/fr-metrics-005/fr-metrics-005-adr.md:6`


### Epic `FR-SAVE` — 3 IDs

#### FR-SAVE-001

- Tests:
  - `crates/save-db/tests/fr_save_tests.rs:3`
  - `crates/save-db/tests/fr_save_tests.rs:11`
  - `crates/save-db/tests/fr_save_tests.rs:121`
- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2800`
  - `docs/traceability/TRACEABILITY_MATRIX.md:273`
  - `docs/traceability/fr-save-001/fr-save-001-adr.md:1`

#### FR-SAVE-004

- Tests:
  - `crates/save-db/tests/fr_save_tests.rs:3`
  - `crates/save-db/tests/fr_save_tests.rs:53`
- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2803`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2935`
  - `docs/traceability/TRACEABILITY_MATRIX.md:276`

#### FR-SAVE-010

- Tests:
  - `crates/save-db/tests/fr_save_tests.rs:3`
  - `crates/save-db/tests/fr_save_tests.rs:93`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:159`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2809`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2955`


### Epic `FR-SOC-IDE` — 6 IDs

#### FR-SOC-IDE-001

- Tests:
  - `crates/engine/tests/fr_fr_soc_ide_001.rs:1`
  - `crates/engine/tests/fr_fr_soc_ide_001.rs:5`
  - `crates/engine/tests/fr_fr_soc_ide_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1588`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1990`
  - `docs/traceability/fr-soc-ide-001/fr-soc-ide-001-adr.md:1`

#### FR-SOC-IDE-002

- Tests:
  - `crates/engine/tests/fr_fr_soc_ide_002.rs:1`
  - `crates/engine/tests/fr_fr_soc_ide_002.rs:5`
  - `crates/engine/tests/fr_fr_soc_ide_002.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1601`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1991`
  - `docs/traceability/fr-soc-ide-002/fr-soc-ide-002-adr.md:1`

#### FR-SOC-IDE-003

- Tests:
  - `crates/engine/tests/fr_fr_soc_ide_003.rs:1`
  - `crates/engine/tests/fr_fr_soc_ide_003.rs:5`
  - `crates/engine/tests/fr_fr_soc_ide_003.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1612`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1992`
  - `docs/traceability/fr-soc-ide-003/fr-soc-ide-003-adr.md:1`

#### FR-SOC-IDE-004

- Tests:
  - `crates/engine/tests/fr_fr_soc_ide_004.rs:1`
  - `crates/engine/tests/fr_fr_soc_ide_004.rs:5`
  - `crates/engine/tests/fr_fr_soc_ide_004.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1624`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1993`
  - `docs/traceability/fr-soc-ide-004/fr-soc-ide-004-adr.md:1`

#### FR-SOC-IDE-005

- Tests:
  - `crates/engine/tests/fr_fr_soc_ide_005.rs:1`
  - `crates/engine/tests/fr_fr_soc_ide_005.rs:5`
  - `crates/engine/tests/fr_fr_soc_ide_005.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4373`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4572`
  - `docs/traceability/fr-soc-ide-005/fr-soc-ide-005-adr.md:1`

#### FR-SOC-IDE-006

- Tests:
  - `crates/engine/tests/fr_fr_soc_ide_006.rs:1`
  - `crates/engine/tests/fr_fr_soc_ide_006.rs:5`
  - `crates/engine/tests/fr_fr_soc_ide_006.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4390`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4573`
  - `docs/traceability/fr-soc-ide-006/fr-soc-ide-006-adr.md:1`


### Epic `FR-VAL` — 1 IDs

#### FR-VAL-001

- Tests:
  - `crates/engine/tests/fr_fr_val_001.rs:1`
  - `crates/engine/tests/fr_fr_val_001.rs:5`
  - `crates/engine/tests/fr_fr_val_001.rs:9`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:170`
  - `docs/traceability/fr-val-001/fr-val-001-adr.md:1`
  - `docs/traceability/fr-val-001/fr-val-001-adr.md:6`

