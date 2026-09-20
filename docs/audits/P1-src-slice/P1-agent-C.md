# P1 agent-C

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


## Your slice: 57 IDs in 16 epics


### Epic `FR-CIV-0001-TICK` — 1 IDs

#### FR-CIV-0001-TICK

- Tests:
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:1`
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:11`
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:131`
- Spec/trace:
  - `docs/reference/ENGINEERING_PROCESS_SUMMARY.md:119`
  - `docs/traceability/fr-civ-0001-tick/fr-civ-0001-tick-adr.md:1`
  - `docs/traceability/fr-civ-0001-tick/fr-civ-0001-tick-adr.md:6`


### Epic `FR-CIV-ARCH-NOSVG` — 1 IDs

#### FR-CIV-ARCH-NOSVG-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_arch_nosvg_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_arch_nosvg_001.rs:5`
  - `crates/engine/tests/fr_fr_civ_arch_nosvg_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3218`
  - `docs/traceability/fr-civ-arch-nosvg-001/fr-civ-arch-nosvg-001-adr.md:1`
  - `docs/traceability/fr-civ-arch-nosvg-001/fr-civ-arch-nosvg-001-adr.md:6`


### Epic `FR-CIV-CORE-DET` — 3 IDs

#### FR-CIV-CORE-DET-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_det_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_det_001.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3206`
  - `docs/traceability/fr-civ-core-det-001/fr-civ-core-det-001-adr.md:1`
  - `docs/traceability/fr-civ-core-det-001/fr-civ-core-det-001-adr.md:6`

#### FR-CIV-CORE-DET-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_det_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_det_002.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3214`
  - `docs/traceability/fr-civ-core-det-002/fr-civ-core-det-002-adr.md:1`
  - `docs/traceability/fr-civ-core-det-002/fr-civ-core-det-002-adr.md:6`

#### FR-CIV-CORE-DET-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_det_003.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_det_003.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3223`
  - `docs/traceability/fr-civ-core-det-003/fr-civ-core-det-003-adr.md:1`
  - `docs/traceability/fr-civ-core-det-003/fr-civ-core-det-003-adr.md:6`


### Epic `FR-CIV-EMERGENCE` — 6 IDs

#### FR-CIV-EMERGENCE-003

- Tests:
  - `crates/civ-emergence-metrics/tests/fr_fr_civ_emergence_003.rs:1`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:97`
  - `docs/guides/voxel-emergent-vision-and-migration.md:139`
  - `docs/traceability/fr-civ-emergence-003/fr-civ-emergence-003-adr.md:1`

#### FR-CIV-EMERGENCE-005

- Tests:
  - `crates/civ-emergence-metrics/tests/fr_fr_civ_emergence_005.rs:1`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:141`
  - `docs/traceability/fr-civ-emergence-005/fr-civ-emergence-005-adr.md:1`
  - `docs/traceability/fr-civ-emergence-005/fr-civ-emergence-005-adr.md:6`

#### FR-CIV-EMERGENCE-006

- Tests:
  - `crates/civ-emergence-metrics/tests/fr_fr_civ_emergence_006.rs:1`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:142`
  - `docs/traceability/fr-civ-emergence-006/fr-civ-emergence-006-adr.md:1`
  - `docs/traceability/fr-civ-emergence-006/fr-civ-emergence-006-adr.md:6`

#### FR-CIV-EMERGENCE-011

- Tests:
  - `crates/civ-emergence-metrics/tests/fr_fr_civ_emergence_011.rs:1`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:98`
  - `docs/guides/voxel-emergent-vision-and-migration.md:144`
  - `docs/traceability/fr-civ-emergence-011/fr-civ-emergence-011-adr.md:1`

#### FR-CIV-EMERGENCE-012

- Tests:
  - `crates/civ-emergence-metrics/tests/fr_fr_civ_emergence_012.rs:1`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:98`
  - `docs/guides/voxel-emergent-vision-and-migration.md:145`
  - `docs/traceability/fr-civ-emergence-012/fr-civ-emergence-012-adr.md:1`

#### FR-CIV-EMERGENCE-013

- Tests:
  - `crates/civ-emergence-metrics/tests/fr_fr_civ_emergence_013.rs:1`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:98`
  - `docs/guides/voxel-emergent-vision-and-migration.md:133`
  - `docs/guides/voxel-emergent-vision-and-migration.md:146`


### Epic `FR-CIV-LEGENDS-BROWSER` — 1 IDs

#### FR-CIV-LEGENDS-BROWSER-09

- Tests:
  - `crates/legends/tests/fr_fr_civ_legends_browser_09.rs:1`
  - `crates/legends/tests/fr_fr_civ_legends_browser_09.rs:10`
  - `crates/legends/tests/fr_fr_civ_legends_browser_09.rs:18`
- Spec/trace:
  - `docs/design/legends-engine.md:444`
  - `docs/traceability/fr-civ-legends-browser-09/fr-civ-legends-browser-09-adr.md:1`
  - `docs/traceability/fr-civ-legends-browser-09/fr-civ-legends-browser-09-adr.md:6`


### Epic `FR-CIV-LEGENDS-PRODUCER` — 1 IDs

#### FR-CIV-LEGENDS-PRODUCER-03

- Tests:
  - `crates/legends/tests/fr_fr_civ_legends_producer_03.rs:1`
  - `crates/legends/tests/fr_fr_civ_legends_producer_03.rs:10`
  - `crates/legends/tests/fr_fr_civ_legends_producer_03.rs:19`
- Spec/trace:
  - `docs/design/legends-engine.md:438`
  - `docs/traceability/fr-civ-legends-producer-03/fr-civ-legends-producer-03-adr.md:1`
  - `docs/traceability/fr-civ-legends-producer-03/fr-civ-legends-producer-03-adr.md:6`


### Epic `FR-CIV-METRICS-001-TIMESERIES` — 1 IDs

#### FR-CIV-METRICS-001-TIMESERIES

- Tests:
  - `crates/engine/tests/fr_engine_metrics_replay_tests.rs:89`
  - `crates/engine/tests/fr_engine_metrics_replay_tests.rs:92`
  - `crates/observability/tests/fr_civ_metrics_tests.rs:3`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:216`
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:217`
  - `PLAN.md:151`


### Epic `FR-CIV-POLITY` — 8 IDs

#### FR-CIV-POLITY-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_polity_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_polity_001.rs:5`
  - `crates/engine/tests/fr_fr_civ_polity_001.rs:9`
- Spec/trace:
  - `docs/design/master-roadmap.md:25`
  - `docs/design/polities-markets.md:37`
  - `docs/traceability/fr-civ-polity-001/fr-civ-polity-001-adr.md:1`

#### FR-CIV-POLITY-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_polity_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_polity_002.rs:5`
  - `crates/engine/tests/fr_fr_civ_polity_002.rs:9`
- Spec/trace:
  - `docs/design/polities-markets.md:50`
  - `docs/traceability/fr-civ-polity-002/fr-civ-polity-002-adr.md:1`
  - `docs/traceability/fr-civ-polity-002/fr-civ-polity-002-adr.md:6`

#### FR-CIV-POLITY-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_polity_003.rs:1`
  - `crates/engine/tests/fr_fr_civ_polity_003.rs:5`
  - `crates/engine/tests/fr_fr_civ_polity_003.rs:9`
- Spec/trace:
  - `docs/design/polities-markets.md:54`
  - `docs/traceability/fr-civ-polity-003/fr-civ-polity-003-adr.md:1`
  - `docs/traceability/fr-civ-polity-003/fr-civ-polity-003-adr.md:6`

#### FR-CIV-POLITY-004

- Tests:
  - `crates/engine/tests/fr_fr_civ_polity_004.rs:1`
  - `crates/engine/tests/fr_fr_civ_polity_004.rs:5`
  - `crates/engine/tests/fr_fr_civ_polity_004.rs:9`
- Spec/trace:
  - `docs/design/polities-markets.md:68`
  - `docs/traceability/fr-civ-polity-004/fr-civ-polity-004-adr.md:1`
  - `docs/traceability/fr-civ-polity-004/fr-civ-polity-004-adr.md:6`

#### FR-CIV-POLITY-005

- Tests:
  - `crates/engine/tests/fr_fr_civ_polity_005.rs:1`
  - `crates/engine/tests/fr_fr_civ_polity_005.rs:5`
  - `crates/engine/tests/fr_fr_civ_polity_005.rs:9`
- Spec/trace:
  - `docs/design/polities-markets.md:82`
  - `docs/traceability/fr-civ-polity-005/fr-civ-polity-005-adr.md:1`
  - `docs/traceability/fr-civ-polity-005/fr-civ-polity-005-adr.md:6`

#### FR-CIV-POLITY-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_polity_006.rs:1`
  - `crates/engine/tests/fr_fr_civ_polity_006.rs:5`
  - `crates/engine/tests/fr_fr_civ_polity_006.rs:9`
- Spec/trace:
  - `docs/design/polities-markets.md:84`
  - `docs/traceability/fr-civ-polity-006/fr-civ-polity-006-adr.md:1`
  - `docs/traceability/fr-civ-polity-006/fr-civ-polity-006-adr.md:6`

#### FR-CIV-POLITY-007

- Tests:
  - `crates/engine/tests/fr_fr_civ_polity_007.rs:1`
  - `crates/engine/tests/fr_fr_civ_polity_007.rs:5`
  - `crates/engine/tests/fr_fr_civ_polity_007.rs:9`
- Spec/trace:
  - `docs/design/polities-markets.md:86`
  - `docs/traceability/fr-civ-polity-007/fr-civ-polity-007-adr.md:1`
  - `docs/traceability/fr-civ-polity-007/fr-civ-polity-007-adr.md:6`

#### FR-CIV-POLITY-008

- Tests:
  - `crates/engine/tests/fr_fr_civ_polity_008.rs:1`
  - `crates/engine/tests/fr_fr_civ_polity_008.rs:5`
  - `crates/engine/tests/fr_fr_civ_polity_008.rs:9`
- Spec/trace:
  - `docs/design/civ-economy-emergent-markets.md:109`
  - `docs/design/polities-markets.md:90`
  - `docs/design/polities-markets.md:140`


### Epic `FR-CIV-RESEARCH-002-SNAPSHOT` — 1 IDs

#### FR-CIV-RESEARCH-002-SNAPSHOT

- Tests:
  - `crates/research/tests/fr_civ_research_tests.rs:3`
  - `crates/research/tests/fr_civ_research_tests.rs:26`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:219`
  - `PLAN.md:235`
  - `PLAN.md:236`


### Epic `FR-CIV-SERVER` — 2 IDs

#### FR-CIV-SERVER-001

- Tests:
  - `crates/server/tests/fr_civ_server_tests.rs:3`
  - `crates/server/tests/fr_civ_server_tests.rs:8`
  - `crates/server/tests/fr_fr_civ_server_001.rs:1`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:221`
  - `PLAN.md:174`
  - `PLAN.md:175`

#### FR-CIV-SERVER-002

- Tests:
  - `crates/server/tests/fr_civ_server_tests.rs:3`
  - `crates/server/tests/fr_civ_server_tests.rs:27`
  - `crates/server/tests/fr_fr_civ_server_002.rs:1`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:223`
  - `PLAN.md:176`
  - `PLAN.md:177`


### Epic `FR-CIV-VOXEL` — 6 IDs

#### FR-CIV-VOXEL-023

- Tests:
  - `crates/voxel/tests/fr_civ_voxel_ca_fluid_gas_heat.rs:1`
  - `crates/voxel/tests/fr_civ_voxel_ca_fluid_gas_heat.rs:6`
  - `crates/voxel/tests/fr_civ_voxel_ca_fluid_gas_heat.rs:86`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:126`
  - `docs/traceability/fr-civ-voxel-023/fr-civ-voxel-023-adr.md:1`
  - `docs/traceability/fr-civ-voxel-023/fr-civ-voxel-023-adr.md:6`

#### FR-CIV-VOXEL-024

- Tests:
  - `crates/voxel/tests/fr_civ_voxel_ca_fluid_gas_heat.rs:8`
  - `crates/voxel/tests/fr_civ_voxel_ca_fluid_gas_heat.rs:165`
  - `crates/voxel/tests/fr_civ_voxel_ca_fluid_gas_heat.rs:168`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:127`
  - `docs/traceability/fr-civ-voxel-024/fr-civ-voxel-024-adr.md:1`
  - `docs/traceability/fr-civ-voxel-024/fr-civ-voxel-024-adr.md:6`

#### FR-CIV-VOXEL-025

- Tests:
  - `crates/voxel/tests/fr_civ_voxel_ca_fluid_gas_heat.rs:10`
  - `crates/voxel/tests/fr_civ_voxel_ca_fluid_gas_heat.rs:219`
  - `crates/voxel/tests/fr_civ_voxel_ca_fluid_gas_heat.rs:222`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:128`
  - `docs/traceability/fr-civ-voxel-025/fr-civ-voxel-025-adr.md:1`
  - `docs/traceability/fr-civ-voxel-025/fr-civ-voxel-025-adr.md:6`

#### FR-CIV-VOXEL-030

- Tests:
  - `crates/voxel/tests/fr_civ_voxel_ca_fluid_gas_heat.rs:273`
  - `crates/voxel/tests/fr_civ_voxel_ca_fluid_gas_heat.rs:276`
  - `crates/voxel/tests/fr_civ_voxel_ca_fluid_gas_heat.rs:303`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:95`
  - `docs/guides/voxel-emergent-vision-and-migration.md:129`
  - `docs/traceability/fr-civ-voxel-030/fr-civ-voxel-030-adr.md:1`

#### FR-CIV-VOXEL-031

- Tests:
  - `crates/voxel/tests/fr_civ_voxel_hydrology_atmosphere.rs:1`
  - `crates/voxel/tests/fr_civ_voxel_hydrology_atmosphere.rs:5`
  - `crates/voxel/tests/fr_civ_voxel_hydrology_atmosphere.rs:54`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:95`
  - `docs/guides/voxel-emergent-vision-and-migration.md:130`
  - `docs/traceability/fr-civ-voxel-031/fr-civ-voxel-031-adr.md:1`

#### FR-CIV-VOXEL-032

- Tests:
  - `crates/voxel/tests/fr_civ_voxel_hydrology_atmosphere.rs:7`
  - `crates/voxel/tests/fr_civ_voxel_hydrology_atmosphere.rs:162`
  - `crates/voxel/tests/fr_civ_voxel_hydrology_atmosphere.rs:165`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:95`
  - `docs/guides/voxel-emergent-vision-and-migration.md:119`
  - `docs/guides/voxel-emergent-vision-and-migration.md:131`


### Epic `FR-ECO` — 10 IDs

#### FR-ECO-001

- Tests:
  - `docs/specs/CIV-0100-economy-v1.md:1651`
- Spec/trace:
  - `docs/traceability/fr-eco-001/fr-eco-001-adr.md:1`
  - `docs/traceability/fr-eco-001/fr-eco-001-adr.md:6`
  - `docs/traceability/fr-eco-001/fr-eco-001-adr.md:11`

#### FR-ECO-002

- Tests:
  - `docs/specs/CIV-0100-economy-v1.md:1662`
- Spec/trace:
  - `docs/traceability/fr-eco-002/fr-eco-002-adr.md:1`
  - `docs/traceability/fr-eco-002/fr-eco-002-adr.md:6`
  - `docs/traceability/fr-eco-002/fr-eco-002-adr.md:11`

#### FR-ECO-003

- Tests:
  - `docs/specs/CIV-0100-economy-v1.md:1682`
- Spec/trace:
  - `docs/traceability/fr-eco-003/fr-eco-003-adr.md:1`
  - `docs/traceability/fr-eco-003/fr-eco-003-adr.md:6`
  - `docs/traceability/fr-eco-003/fr-eco-003-adr.md:11`

#### FR-ECO-004

- Tests:
  - `docs/specs/CIV-0100-economy-v1.md:1696`
- Spec/trace:
  - `docs/traceability/fr-eco-004/fr-eco-004-adr.md:1`
  - `docs/traceability/fr-eco-004/fr-eco-004-adr.md:6`
  - `docs/traceability/fr-eco-004/fr-eco-004-adr.md:11`

#### FR-ECO-005

- Tests:
  - `docs/specs/CIV-0100-economy-v1.md:1719`
- Spec/trace:
  - `docs/traceability/fr-eco-005/fr-eco-005-adr.md:1`
  - `docs/traceability/fr-eco-005/fr-eco-005-adr.md:6`
  - `docs/traceability/fr-eco-005/fr-eco-005-adr.md:11`

#### FR-ECO-006

- Tests:
  - `docs/specs/CIV-0100-economy-v1.md:1739`
- Spec/trace:
  - `docs/traceability/fr-eco-006/fr-eco-006-adr.md:1`
  - `docs/traceability/fr-eco-006/fr-eco-006-adr.md:6`
  - `docs/traceability/fr-eco-006/fr-eco-006-adr.md:11`

#### FR-ECO-007

- Tests:
  - `docs/specs/CIV-0100-economy-v1.md:1766`
- Spec/trace:
  - `docs/traceability/fr-eco-007/fr-eco-007-adr.md:1`
  - `docs/traceability/fr-eco-007/fr-eco-007-adr.md:6`
  - `docs/traceability/fr-eco-007/fr-eco-007-adr.md:11`

#### FR-ECO-008

- Tests:
  - `docs/specs/CIV-0100-economy-v1.md:1792`
- Spec/trace:
  - `docs/traceability/fr-eco-008/fr-eco-008-adr.md:1`
  - `docs/traceability/fr-eco-008/fr-eco-008-adr.md:6`
  - `docs/traceability/fr-eco-008/fr-eco-008-adr.md:11`

#### FR-ECO-009

- Tests:
  - `docs/specs/CIV-0100-economy-v1.md:1821`
- Spec/trace:
  - `docs/traceability/fr-eco-009/fr-eco-009-adr.md:1`
  - `docs/traceability/fr-eco-009/fr-eco-009-adr.md:6`
  - `docs/traceability/fr-eco-009/fr-eco-009-adr.md:11`

#### FR-ECO-010

- Tests:
  - `docs/specs/CIV-0100-economy-v1.md:1845`
- Spec/trace:
  - `docs/traceability/fr-eco-010/fr-eco-010-adr.md:1`
  - `docs/traceability/fr-eco-010/fr-eco-010-adr.md:6`
  - `docs/traceability/fr-eco-010/fr-eco-010-adr.md:11`


### Epic `FR-PROT` — 4 IDs

#### FR-PROT-001

- Tests:
  - `crates/engine/tests/fr_fr_prot_001.rs:1`
  - `crates/engine/tests/fr_fr_prot_001.rs:9`
  - `crates/engine/tests/fr_fr_prot_001.rs:18`
- Spec/trace:
  - `docs/traceability/TRACEABILITY_MATRIX.md:178`
  - `docs/traceability/fr-prot-001/fr-prot-001-adr.md:1`
  - `docs/traceability/fr-prot-001/fr-prot-001-adr.md:6`

#### FR-PROT-002

- Tests:
  - `crates/engine/tests/fr_fr_prot_002.rs:1`
  - `crates/engine/tests/fr_fr_prot_002.rs:9`
  - `crates/engine/tests/fr_fr_prot_002.rs:19`
- Spec/trace:
  - `docs/traceability/TRACEABILITY_MATRIX.md:179`
  - `docs/traceability/fr-prot-002/fr-prot-002-adr.md:1`
  - `docs/traceability/fr-prot-002/fr-prot-002-adr.md:6`

#### FR-PROT-003

- Tests:
  - `crates/engine/tests/fr_fr_prot_003.rs:1`
  - `crates/engine/tests/fr_fr_prot_003.rs:9`
  - `crates/engine/tests/fr_fr_prot_003.rs:19`
- Spec/trace:
  - `docs/traceability/TRACEABILITY_MATRIX.md:180`
  - `docs/traceability/fr-prot-003/fr-prot-003-adr.md:1`
  - `docs/traceability/fr-prot-003/fr-prot-003-adr.md:6`

#### FR-PROT-005

- Tests:
  - `crates/engine/tests/fr_fr_prot_005.rs:1`
  - `crates/engine/tests/fr_fr_prot_005.rs:9`
  - `crates/engine/tests/fr_fr_prot_005.rs:16`
- Spec/trace:
  - `docs/traceability/TRACEABILITY_MATRIX.md:182`
  - `docs/traceability/fr-prot-005/fr-prot-005-adr.md:1`
  - `docs/traceability/fr-prot-005/fr-prot-005-adr.md:6`


### Epic `FR-SOC-COH` — 4 IDs

#### FR-SOC-COH-001

- Tests:
  - `crates/engine/tests/fr_fr_soc_coh_001.rs:1`
  - `crates/engine/tests/fr_fr_soc_coh_001.rs:5`
  - `crates/engine/tests/fr_fr_soc_coh_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1533`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1986`
  - `docs/traceability/fr-soc-coh-001/fr-soc-coh-001-adr.md:1`

#### FR-SOC-COH-002

- Tests:
  - `crates/engine/tests/fr_fr_soc_coh_002.rs:1`
  - `crates/engine/tests/fr_fr_soc_coh_002.rs:5`
  - `crates/engine/tests/fr_fr_soc_coh_002.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1546`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1987`
  - `docs/traceability/fr-soc-coh-002/fr-soc-coh-002-adr.md:1`

#### FR-SOC-COH-003

- Tests:
  - `crates/engine/tests/fr_fr_soc_coh_003.rs:1`
  - `crates/engine/tests/fr_fr_soc_coh_003.rs:5`
  - `crates/engine/tests/fr_fr_soc_coh_003.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1559`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1988`
  - `docs/traceability/fr-soc-coh-003/fr-soc-coh-003-adr.md:1`

#### FR-SOC-COH-004

- Tests:
  - `crates/engine/tests/fr_fr_soc_coh_004.rs:1`
  - `crates/engine/tests/fr_fr_soc_coh_004.rs:5`
  - `crates/engine/tests/fr_fr_soc_coh_004.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1571`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1989`
  - `docs/traceability/fr-soc-coh-004/fr-soc-coh-004-adr.md:1`


### Epic `FR-SOC-INTG` — 7 IDs

#### FR-SOC-INTG-001

- Tests:
  - `crates/engine/tests/fr_fr_soc_intg_001.rs:1`
  - `crates/engine/tests/fr_fr_soc_intg_001.rs:5`
  - `crates/engine/tests/fr_fr_soc_intg_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1812`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2007`
  - `docs/traceability/fr-soc-intg-001/fr-soc-intg-001-adr.md:1`

#### FR-SOC-INTG-002

- Tests:
  - `crates/engine/tests/fr_fr_soc_intg_002.rs:1`
  - `crates/engine/tests/fr_fr_soc_intg_002.rs:5`
  - `crates/engine/tests/fr_fr_soc_intg_002.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1822`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2008`
  - `docs/traceability/fr-soc-intg-002/fr-soc-intg-002-adr.md:1`

#### FR-SOC-INTG-003

- Tests:
  - `crates/engine/tests/fr_fr_soc_intg_003.rs:1`
  - `crates/engine/tests/fr_fr_soc_intg_003.rs:5`
  - `crates/engine/tests/fr_fr_soc_intg_003.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1830`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2009`
  - `docs/traceability/fr-soc-intg-003/fr-soc-intg-003-adr.md:1`

#### FR-SOC-INTG-004

- Tests:
  - `crates/engine/tests/fr_fr_soc_intg_004.rs:1`
  - `crates/engine/tests/fr_fr_soc_intg_004.rs:5`
  - `crates/engine/tests/fr_fr_soc_intg_004.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4478`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4577`
  - `docs/traceability/fr-soc-intg-004/fr-soc-intg-004-adr.md:1`

#### FR-SOC-INTG-005

- Tests:
  - `crates/engine/tests/fr_fr_soc_intg_005.rs:1`
  - `crates/engine/tests/fr_fr_soc_intg_005.rs:5`
  - `crates/engine/tests/fr_fr_soc_intg_005.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4495`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4578`
  - `docs/traceability/fr-soc-intg-005/fr-soc-intg-005-adr.md:1`

#### FR-SOC-INTG-006

- Tests:
  - `crates/engine/tests/fr_fr_soc_intg_006.rs:1`
  - `crates/engine/tests/fr_fr_soc_intg_006.rs:5`
  - `crates/engine/tests/fr_fr_soc_intg_006.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4517`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4579`
  - `docs/traceability/fr-soc-intg-006/fr-soc-intg-006-adr.md:1`

#### FR-SOC-INTG-007

- Tests:
  - `crates/engine/tests/fr_fr_soc_intg_007.rs:1`
  - `crates/engine/tests/fr_fr_soc_intg_007.rs:5`
  - `crates/engine/tests/fr_fr_soc_intg_007.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4537`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4580`
  - `docs/traceability/fr-soc-intg-007/fr-soc-intg-007-adr.md:1`


### Epic `NFR-CIV-REL` — 1 IDs

#### NFR-CIV-REL-004

- Tests:
  - `crates/engine/tests/fr_nfr_civ_rel_004.rs:1`
- Spec/trace:
  - `docs/reference/non-functional-requirements.md:278`
  - `docs/reference/non-functional-requirements.md:568`
  - `docs/reference/non-functional-requirements.md:605`

