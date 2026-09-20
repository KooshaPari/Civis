# P1 agent-A

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


## Your slice: 98 IDs in 16 epics


### Epic `FR-API` — 3 IDs

#### FR-API-002

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:68`
  - `crates/build/tests/fr_matrix_batch12.rs:71`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `agileplus-specs/civ-013-research-api/plan.md:17`
  - `agileplus-specs/civ-013-research-api/spec.md:26`

#### FR-API-003

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:84`
  - `crates/build/tests/fr_matrix_batch12.rs:87`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `agileplus-specs/civ-013-research-api/plan.md:17`
  - `agileplus-specs/civ-013-research-api/spec.md:27`

#### FR-API-004

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:94`
  - `crates/build/tests/fr_matrix_batch12.rs:97`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `agileplus-specs/civ-013-research-api/plan.md:24`
  - `agileplus-specs/civ-013-research-api/spec.md:28`


### Epic `FR-CIV-AI` — 5 IDs

#### FR-CIV-AI-011

- Tests:
  - `crates/ai/tests/fr_fr_civ_ai_011.rs:1`
  - `crates/ai/tests/fr_fr_civ_ai_011.rs:4`
  - `crates/ai/tests/fr_fr_civ_ai_011.rs:18`
- Spec/trace:
  - `docs/design/civ-ai-crate.md:43`
  - `docs/design/civ-ai-crate.md:173`
  - `docs/design/civ-ai-crate.md:262`

#### FR-CIV-AI-012

- Tests:
  - `crates/ai/tests/fr_fr_civ_ai_012.rs:1`
  - `crates/ai/tests/fr_fr_civ_ai_012.rs:4`
  - `crates/ai/tests/fr_fr_civ_ai_012.rs:17`
- Spec/trace:
  - `docs/design/civ-ai-crate.md:44`
  - `docs/design/civ-ai-crate.md:171`
  - `docs/design/civ-ai-crate.md:263`

#### FR-CIV-AI-013

- Tests:
  - `crates/ai/tests/fr_fr_civ_ai_013.rs:1`
  - `crates/ai/tests/fr_fr_civ_ai_013.rs:4`
  - `crates/ai/tests/fr_fr_civ_ai_013.rs:26`
- Spec/trace:
  - `docs/design/civ-ai-crate.md:45`
  - `docs/design/civ-ai-crate.md:264`
  - `docs/traceability/fr-civ-ai-013/fr-civ-ai-013-adr.md:1`

#### FR-CIV-AI-014

- Tests:
  - `crates/ai/tests/fr_fr_civ_ai_014.rs:1`
  - `crates/ai/tests/fr_fr_civ_ai_014.rs:4`
  - `crates/ai/tests/fr_fr_civ_ai_014.rs:19`
- Spec/trace:
  - `docs/design/civ-ai-crate.md:46`
  - `docs/design/civ-ai-crate.md:172`
  - `docs/design/civ-ai-crate.md:265`

#### FR-CIV-AI-015

- Tests:
  - `crates/ai/tests/fr_fr_civ_ai_015.rs:1`
  - `crates/ai/tests/fr_fr_civ_ai_015.rs:4`
  - `crates/ai/tests/fr_fr_civ_ai_015.rs:18`
- Spec/trace:
  - `docs/design/civ-ai-crate.md:47`
  - `docs/design/civ-ai-crate.md:266`
  - `docs/traceability/fr-civ-ai-015/fr-civ-ai-015-adr.md:1`


### Epic `FR-CIV-CLIMATE` — 2 IDs

#### FR-CIV-CLIMATE-001

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:651`
  - `crates/build/tests/fr_matrix_batch12.rs:654`
- Spec/trace:
  - `agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:41`
  - `agileplus-specs/civ-005-climate-disasters-seasons/plan.md:5`
  - `agileplus-specs/civ-005-climate-disasters-seasons/spec.md:24`

#### FR-CIV-CLIMATE-003

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:702`
  - `crates/build/tests/fr_matrix_batch12.rs:705`
- Spec/trace:
  - `agileplus-specs/civ-005-climate-disasters-seasons/plan.md:19`
  - `agileplus-specs/civ-005-climate-disasters-seasons/spec.md:26`
  - `docs/reference/agileplus-artifacts-index.md:105`


### Epic `FR-CIV-ECON-001-MARKET` — 1 IDs

#### FR-CIV-ECON-001-MARKET

- Tests:
  - `crates/economy/tests/fr_civ_econ_tests.rs:3`
  - `crates/economy/tests/fr_civ_econ_tests.rs:10`
  - `crates/economy/tests/fr_civ_econ_tests.rs:13`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/plan.md:39`
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:64`
  - `docs/guides/COPILOT_L3_AGENTS.md:90`


### Epic `FR-CIV-INSPECT` — 2 IDs

#### FR-CIV-INSPECT-902

- Tests:
  - `crates/engine/tests/fr_civ_infoview_inspect_cluster.rs:1`
  - `crates/engine/tests/fr_civ_infoview_inspect_cluster.rs:48`
  - `crates/engine/tests/fr_civ_infoview_inspect_cluster.rs:76`
- Spec/trace:
  - `docs/agileplus/epics/civ-w4-perception.md:19`
  - `docs/agileplus/epics/civ-w4-perception.md:36`
  - `docs/agileplus/README.md:23`

#### FR-CIV-INSPECT-920

- Tests:
  - `crates/engine/tests/fr_civ_infoview_inspect_cluster.rs:57`
  - `crates/engine/tests/fr_civ_infoview_inspect_cluster.rs:83`
  - `crates/engine/tests/fr_civ_infoview_inspect_cluster.rs:1176`
- Spec/trace:
  - `docs/agileplus/epics/civ-w4-perception.md:22`
  - `docs/agileplus/epics/civ-w4-perception.md:37`
  - `docs/agileplus/README.md:23`


### Epic `FR-CIV-LEGENDS-PERSIST` — 1 IDs

#### FR-CIV-LEGENDS-PERSIST-11

- Tests:
  - `crates/legends/tests/fr_fr_civ_legends_persist_11.rs:1`
  - `crates/legends/tests/fr_fr_civ_legends_persist_11.rs:10`
  - `crates/legends/tests/fr_fr_civ_legends_persist_11.rs:31`
- Spec/trace:
  - `docs/design/legends-engine.md:446`
  - `docs/traceability/fr-civ-legends-persist-11/fr-civ-legends-persist-11-adr.md:1`
  - `docs/traceability/fr-civ-legends-persist-11/fr-civ-legends-persist-11-adr.md:6`


### Epic `FR-CIV-MCP` — 4 IDs

#### FR-CIV-MCP-002

- Tests:
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_002.rs:1`
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_002.rs:8`
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_002.rs:15`
- Spec/trace:
  - `agileplus-specs/civ-017-civis-mcp-server/spec.md:38`
  - `docs/traceability/fr-civ-mcp-002/fr-civ-mcp-002-adr.md:1`
  - `docs/traceability/fr-civ-mcp-002/fr-civ-mcp-002-adr.md:6`

#### FR-CIV-MCP-004

- Tests:
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_004.rs:1`
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_004.rs:8`
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_004.rs:17`
- Spec/trace:
  - `agileplus-specs/civ-017-civis-mcp-server/spec.md:46`
  - `docs/traceability/fr-civ-mcp-004/fr-civ-mcp-004-adr.md:1`
  - `docs/traceability/fr-civ-mcp-004/fr-civ-mcp-004-adr.md:6`

#### FR-CIV-MCP-005

- Tests:
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_005.rs:1`
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_005.rs:8`
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_005.rs:17`
- Spec/trace:
  - `agileplus-specs/civ-017-civis-mcp-server/spec.md:49`
  - `docs/traceability/fr-civ-mcp-005/fr-civ-mcp-005-adr.md:1`
  - `docs/traceability/fr-civ-mcp-005/fr-civ-mcp-005-adr.md:6`

#### FR-CIV-MCP-006

- Tests:
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_006.rs:1`
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_006.rs:8`
  - `crates/civis-mcp/tests/fr_fr_civ_mcp_006.rs:18`
- Spec/trace:
  - `agileplus-specs/civ-017-civis-mcp-server/spec.md:52`
  - `docs/traceability/fr-civ-mcp-006/fr-civ-mcp-006-adr.md:1`
  - `docs/traceability/fr-civ-mcp-006/fr-civ-mcp-006-adr.md:6`


### Epic `FR-CIV-PERF-RT` — 3 IDs

#### FR-CIV-PERF-RT-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_rt_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_rt_001.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_rt_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3216`
  - `docs/traceability/fr-civ-perf-rt-001/fr-civ-perf-rt-001-adr.md:1`
  - `docs/traceability/fr-civ-perf-rt-001/fr-civ-perf-rt-001-adr.md:6`

#### FR-CIV-PERF-RT-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_rt_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_rt_002.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_rt_002.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3217`
  - `docs/traceability/fr-civ-perf-rt-002/fr-civ-perf-rt-002-adr.md:1`
  - `docs/traceability/fr-civ-perf-rt-002/fr-civ-perf-rt-002-adr.md:6`

#### FR-CIV-PERF-RT-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_rt_003.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_rt_003.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_rt_003.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3221`
  - `docs/traceability/fr-civ-perf-rt-003/fr-civ-perf-rt-003-adr.md:1`
  - `docs/traceability/fr-civ-perf-rt-003/fr-civ-perf-rt-003-adr.md:6`


### Epic `FR-CIV-RES` — 1 IDs

#### FR-CIV-RES-001

- Tests:
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:2`
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:21`
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:1182`
- Spec/trace:
  - `docs/reference/CODE_ENTITY_MAP.md:17`
  - `docs/reference/FR_TRACKER.md:40`
  - `docs/traceability/fr-civ-res-001/fr-civ-res-001-adr.md:1`


### Epic `FR-CIV-RTS-RENDER` — 5 IDs

#### FR-CIV-RTS-RENDER-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_render_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_render_001.rs:4`
  - `crates/engine/tests/fr_fr_civ_rts_render_001.rs:11`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3205`
  - `docs/traceability/fr-civ-rts-render-001/fr-civ-rts-render-001-adr.md:1`
  - `docs/traceability/fr-civ-rts-render-001/fr-civ-rts-render-001-adr.md:6`

#### FR-CIV-RTS-RENDER-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_render_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_render_002.rs:4`
  - `crates/engine/tests/fr_fr_civ_rts_render_002.rs:11`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3207`
  - `docs/traceability/fr-civ-rts-render-002/fr-civ-rts-render-002-adr.md:1`
  - `docs/traceability/fr-civ-rts-render-002/fr-civ-rts-render-002-adr.md:6`

#### FR-CIV-RTS-RENDER-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_render_003.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_render_003.rs:4`
  - `crates/engine/tests/fr_fr_civ_rts_render_003.rs:11`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3208`
  - `docs/traceability/fr-civ-rts-render-003/fr-civ-rts-render-003-adr.md:1`
  - `docs/traceability/fr-civ-rts-render-003/fr-civ-rts-render-003-adr.md:6`

#### FR-CIV-RTS-RENDER-004

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_render_004.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_render_004.rs:4`
  - `crates/engine/tests/fr_fr_civ_rts_render_004.rs:11`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3210`
  - `docs/traceability/fr-civ-rts-render-004/fr-civ-rts-render-004-adr.md:1`
  - `docs/traceability/fr-civ-rts-render-004/fr-civ-rts-render-004-adr.md:6`

#### FR-CIV-RTS-RENDER-005

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_render_005.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_render_005.rs:4`
  - `crates/engine/tests/fr_fr_civ_rts_render_005.rs:11`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3211`
  - `docs/traceability/fr-civ-rts-render-005/fr-civ-rts-render-005-adr.md:1`
  - `docs/traceability/fr-civ-rts-render-005/fr-civ-rts-render-005-adr.md:6`


### Epic `FR-CIV-VEHICLE` — 23 IDs

#### FR-CIV-VEHICLE-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_002.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_002.rs:12`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:105`
  - `docs/traceability/fr-civ-vehicle-002/fr-civ-vehicle-002-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-002/fr-civ-vehicle-002-adr.md:6`

#### FR-CIV-VEHICLE-005

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_005.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_005.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_005.rs:12`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:111`
  - `docs/traceability/fr-civ-vehicle-005/fr-civ-vehicle-005-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-005/fr-civ-vehicle-005-adr.md:6`

#### FR-CIV-VEHICLE-010

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_010.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_010.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_010.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:161`
  - `docs/design/vehicles-logistics.md:162`
  - `docs/traceability/fr-civ-vehicle-010/fr-civ-vehicle-010-adr.md:1`

#### FR-CIV-VEHICLE-011

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_011.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_011.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_011.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:164`
  - `docs/traceability/fr-civ-vehicle-011/fr-civ-vehicle-011-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-011/fr-civ-vehicle-011-adr.md:6`

#### FR-CIV-VEHICLE-012

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_012.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_012.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_012.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:166`
  - `docs/traceability/fr-civ-vehicle-012/fr-civ-vehicle-012-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-012/fr-civ-vehicle-012-adr.md:6`

#### FR-CIV-VEHICLE-013

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_013.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_013.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_013.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:168`
  - `docs/traceability/fr-civ-vehicle-013/fr-civ-vehicle-013-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-013/fr-civ-vehicle-013-adr.md:6`

#### FR-CIV-VEHICLE-014

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_014.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_014.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_014.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:170`
  - `docs/traceability/fr-civ-vehicle-014/fr-civ-vehicle-014-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-014/fr-civ-vehicle-014-adr.md:6`

#### FR-CIV-VEHICLE-020

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_020.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_020.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_020.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:198`
  - `docs/design/vehicles-logistics.md:199`
  - `docs/traceability/fr-civ-vehicle-020/fr-civ-vehicle-020-adr.md:1`

#### FR-CIV-VEHICLE-021

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_021.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_021.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_021.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:201`
  - `docs/traceability/fr-civ-vehicle-021/fr-civ-vehicle-021-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-021/fr-civ-vehicle-021-adr.md:6`

#### FR-CIV-VEHICLE-022

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_022.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_022.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_022.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:203`
  - `docs/traceability/fr-civ-vehicle-022/fr-civ-vehicle-022-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-022/fr-civ-vehicle-022-adr.md:6`

#### FR-CIV-VEHICLE-023

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_023.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_023.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_023.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:205`
  - `docs/traceability/fr-civ-vehicle-023/fr-civ-vehicle-023-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-023/fr-civ-vehicle-023-adr.md:6`

#### FR-CIV-VEHICLE-024

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_024.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_024.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_024.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:206`
  - `docs/traceability/fr-civ-vehicle-024/fr-civ-vehicle-024-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-024/fr-civ-vehicle-024-adr.md:6`

#### FR-CIV-VEHICLE-030

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_030.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_030.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_030.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:222`
  - `docs/traceability/fr-civ-vehicle-030/fr-civ-vehicle-030-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-030/fr-civ-vehicle-030-adr.md:6`

#### FR-CIV-VEHICLE-040

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_040.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_040.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_040.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:277`
  - `docs/design/vehicles-logistics.md:278`
  - `docs/traceability/fr-civ-vehicle-040/fr-civ-vehicle-040-adr.md:1`

#### FR-CIV-VEHICLE-041

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_041.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_041.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_041.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:280`
  - `docs/traceability/fr-civ-vehicle-041/fr-civ-vehicle-041-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-041/fr-civ-vehicle-041-adr.md:6`

#### FR-CIV-VEHICLE-042

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_042.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_042.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_042.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:282`
  - `docs/traceability/fr-civ-vehicle-042/fr-civ-vehicle-042-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-042/fr-civ-vehicle-042-adr.md:6`

#### FR-CIV-VEHICLE-043

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_043.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_043.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_043.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:284`
  - `docs/traceability/fr-civ-vehicle-043/fr-civ-vehicle-043-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-043/fr-civ-vehicle-043-adr.md:6`

#### FR-CIV-VEHICLE-044

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_044.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_044.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_044.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:286`
  - `docs/traceability/fr-civ-vehicle-044/fr-civ-vehicle-044-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-044/fr-civ-vehicle-044-adr.md:6`

#### FR-CIV-VEHICLE-045

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_045.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_045.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_045.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:288`
  - `docs/traceability/fr-civ-vehicle-045/fr-civ-vehicle-045-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-045/fr-civ-vehicle-045-adr.md:6`

#### FR-CIV-VEHICLE-046

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_046.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_046.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_046.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:290`
  - `docs/traceability/fr-civ-vehicle-046/fr-civ-vehicle-046-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-046/fr-civ-vehicle-046-adr.md:6`

#### FR-CIV-VEHICLE-047

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_047.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_047.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_047.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:292`
  - `docs/traceability/fr-civ-vehicle-047/fr-civ-vehicle-047-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-047/fr-civ-vehicle-047-adr.md:6`

#### FR-CIV-VEHICLE-050

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_050.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_050.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_050.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:309`
  - `docs/traceability/fr-civ-vehicle-050/fr-civ-vehicle-050-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-050/fr-civ-vehicle-050-adr.md:6`

#### FR-CIV-VEHICLE-060

- Tests:
  - `crates/engine/tests/fr_fr_civ_vehicle_060.rs:1`
  - `crates/engine/tests/fr_fr_civ_vehicle_060.rs:4`
  - `crates/engine/tests/fr_fr_civ_vehicle_060.rs:11`
- Spec/trace:
  - `docs/design/vehicles-logistics.md:342`
  - `docs/traceability/fr-civ-vehicle-060/fr-civ-vehicle-060-adr.md:1`
  - `docs/traceability/fr-civ-vehicle-060/fr-civ-vehicle-060-adr.md:6`


### Epic `FR-DIPL` — 1 IDs

#### FR-DIPL-007

- Tests:
  - `crates/diplomacy/tests/fr_fr_dipl_007.rs:1`
  - `crates/diplomacy/tests/fr_fr_dipl_007.rs:15`
  - `crates/diplomacy/tests/fr_fr_dipl_007.rs:36`
- Spec/trace:
  - `docs/traceability/TRACEABILITY_MATRIX.md:137`
  - `docs/traceability/fr-dipl-007/fr-dipl-007-adr.md:1`
  - `docs/traceability/fr-dipl-007/fr-dipl-007-adr.md:6`


### Epic `FR-NET` — 3 IDs

#### FR-NET-001

- Tests:
  - `crates/engine/tests/fr_fr_net_001.rs:1`
  - `crates/engine/tests/fr_fr_net_001.rs:9`
  - `crates/engine/tests/fr_fr_net_001.rs:18`
- Spec/trace:
  - `docs/FR.md:52`
  - `docs/FR_DETAILED.md:267`
  - `docs/traceability/fr-net-001/fr-net-001-adr.md:1`

#### FR-NET-002

- Tests:
  - `crates/engine/tests/fr_fr_net_002.rs:1`
  - `crates/engine/tests/fr_fr_net_002.rs:8`
  - `crates/engine/tests/fr_fr_net_002.rs:31`
- Spec/trace:
  - `docs/FR.md:53`
  - `docs/FR_DETAILED.md:281`
  - `docs/traceability/fr-net-002/fr-net-002-adr.md:1`

#### FR-NET-003

- Tests:
  - `crates/engine/tests/fr_fr_net_003.rs:1`
  - `crates/engine/tests/fr_fr_net_003.rs:9`
  - `crates/engine/tests/fr_fr_net_003.rs:20`
- Spec/trace:
  - `docs/FR.md:54`
  - `docs/traceability/fr-net-003/fr-net-003-adr.md:1`
  - `docs/traceability/fr-net-003/fr-net-003-adr.md:6`


### Epic `FR-SESSION` — 33 IDs

#### FR-SESSION-001

- Tests:
  - `crates/server/tests/fr_fr_session_001.rs:1`
  - `crates/server/tests/fr_fr_session_001.rs:5`
  - `crates/server/tests/fr_fr_session_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2020`
  - `docs/traceability/fr-session-001/fr-session-001-adr.md:1`
  - `docs/traceability/fr-session-001/fr-session-001-adr.md:6`

#### FR-SESSION-002

- Tests:
  - `crates/server/tests/fr_fr_session_002.rs:1`
  - `crates/server/tests/fr_fr_session_002.rs:5`
  - `crates/server/tests/fr_fr_session_002.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2022`
  - `docs/traceability/fr-session-002/fr-session-002-adr.md:1`
  - `docs/traceability/fr-session-002/fr-session-002-adr.md:6`

#### FR-SESSION-003

- Tests:
  - `crates/server/tests/fr_fr_session_003.rs:1`
  - `crates/server/tests/fr_fr_session_003.rs:5`
  - `crates/server/tests/fr_fr_session_003.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2024`
  - `docs/traceability/fr-session-003/fr-session-003-adr.md:1`
  - `docs/traceability/fr-session-003/fr-session-003-adr.md:6`

#### FR-SESSION-004

- Tests:
  - `crates/server/tests/fr_fr_session_004.rs:1`
  - `crates/server/tests/fr_fr_session_004.rs:5`
  - `crates/server/tests/fr_fr_session_004.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2026`
  - `docs/traceability/fr-session-004/fr-session-004-adr.md:1`
  - `docs/traceability/fr-session-004/fr-session-004-adr.md:6`

#### FR-SESSION-005

- Tests:
  - `crates/server/tests/fr_fr_session_005.rs:1`
  - `crates/server/tests/fr_fr_session_005.rs:5`
  - `crates/server/tests/fr_fr_session_005.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2028`
  - `docs/traceability/fr-session-005/fr-session-005-adr.md:1`
  - `docs/traceability/fr-session-005/fr-session-005-adr.md:6`

#### FR-SESSION-006

- Tests:
  - `crates/server/tests/fr_fr_session_006.rs:1`
  - `crates/server/tests/fr_fr_session_006.rs:5`
  - `crates/server/tests/fr_fr_session_006.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2032`
  - `docs/traceability/fr-session-006/fr-session-006-adr.md:1`
  - `docs/traceability/fr-session-006/fr-session-006-adr.md:6`

#### FR-SESSION-007

- Tests:
  - `crates/server/tests/fr_fr_session_007.rs:1`
  - `crates/server/tests/fr_fr_session_007.rs:5`
  - `crates/server/tests/fr_fr_session_007.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2034`
  - `docs/traceability/fr-session-007/fr-session-007-adr.md:1`
  - `docs/traceability/fr-session-007/fr-session-007-adr.md:6`

#### FR-SESSION-008

- Tests:
  - `crates/server/tests/fr_fr_session_008.rs:1`
  - `crates/server/tests/fr_fr_session_008.rs:5`
  - `crates/server/tests/fr_fr_session_008.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2036`
  - `docs/traceability/fr-session-008/fr-session-008-adr.md:1`
  - `docs/traceability/fr-session-008/fr-session-008-adr.md:6`

#### FR-SESSION-009

- Tests:
  - `crates/server/tests/fr_fr_session_009.rs:1`
  - `crates/server/tests/fr_fr_session_009.rs:5`
  - `crates/server/tests/fr_fr_session_009.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2038`
  - `docs/traceability/fr-session-009/fr-session-009-adr.md:1`
  - `docs/traceability/fr-session-009/fr-session-009-adr.md:6`

#### FR-SESSION-010

- Tests:
  - `crates/server/tests/fr_fr_session_010.rs:1`
  - `crates/server/tests/fr_fr_session_010.rs:5`
  - `crates/server/tests/fr_fr_session_010.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2040`
  - `docs/traceability/fr-session-010/fr-session-010-adr.md:1`
  - `docs/traceability/fr-session-010/fr-session-010-adr.md:6`

#### FR-SESSION-011

- Tests:
  - `crates/server/tests/fr_fr_session_011.rs:1`
  - `crates/server/tests/fr_fr_session_011.rs:5`
  - `crates/server/tests/fr_fr_session_011.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2044`
  - `docs/traceability/fr-session-011/fr-session-011-adr.md:1`
  - `docs/traceability/fr-session-011/fr-session-011-adr.md:6`

#### FR-SESSION-012

- Tests:
  - `crates/server/tests/fr_fr_session_012.rs:1`
  - `crates/server/tests/fr_fr_session_012.rs:5`
  - `crates/server/tests/fr_fr_session_012.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2046`
  - `docs/traceability/fr-session-012/fr-session-012-adr.md:1`
  - `docs/traceability/fr-session-012/fr-session-012-adr.md:6`

#### FR-SESSION-013

- Tests:
  - `crates/server/tests/fr_fr_session_013.rs:1`
  - `crates/server/tests/fr_fr_session_013.rs:5`
  - `crates/server/tests/fr_fr_session_013.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2048`
  - `docs/traceability/fr-session-013/fr-session-013-adr.md:1`
  - `docs/traceability/fr-session-013/fr-session-013-adr.md:6`

#### FR-SESSION-014

- Tests:
  - `crates/server/tests/fr_civ_server_tests.rs:4`
  - `crates/server/tests/fr_civ_server_tests.rs:51`
  - `crates/server/tests/fr_fr_session_014.rs:1`
- Spec/trace:
  - `agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:62`
  - `agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:126`
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2050`

#### FR-SESSION-015

- Tests:
  - `crates/server/tests/fr_fr_session_015.rs:1`
  - `crates/server/tests/fr_fr_session_015.rs:5`
  - `crates/server/tests/fr_fr_session_015.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2052`
  - `docs/traceability/fr-session-015/fr-session-015-adr.md:1`
  - `docs/traceability/fr-session-015/fr-session-015-adr.md:6`

#### FR-SESSION-016

- Tests:
  - `crates/server/tests/fr_fr_session_016.rs:1`
  - `crates/server/tests/fr_fr_session_016.rs:5`
  - `crates/server/tests/fr_fr_session_016.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2056`
  - `docs/traceability/fr-session-016/fr-session-016-adr.md:1`
  - `docs/traceability/fr-session-016/fr-session-016-adr.md:6`

#### FR-SESSION-017

- Tests:
  - `crates/server/tests/fr_fr_session_017.rs:1`
  - `crates/server/tests/fr_fr_session_017.rs:5`
  - `crates/server/tests/fr_fr_session_017.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2058`
  - `docs/traceability/fr-session-017/fr-session-017-adr.md:1`
  - `docs/traceability/fr-session-017/fr-session-017-adr.md:6`

#### FR-SESSION-018

- Tests:
  - `crates/server/tests/fr_fr_session_018.rs:1`
  - `crates/server/tests/fr_fr_session_018.rs:5`
  - `crates/server/tests/fr_fr_session_018.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2060`
  - `docs/traceability/fr-session-018/fr-session-018-adr.md:1`
  - `docs/traceability/fr-session-018/fr-session-018-adr.md:6`

#### FR-SESSION-019

- Tests:
  - `crates/server/tests/fr_fr_session_019.rs:1`
  - `crates/server/tests/fr_fr_session_019.rs:5`
  - `crates/server/tests/fr_fr_session_019.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2062`
  - `docs/traceability/fr-session-019/fr-session-019-adr.md:1`
  - `docs/traceability/fr-session-019/fr-session-019-adr.md:6`

#### FR-SESSION-020

- Tests:
  - `crates/server/tests/fr_fr_session_020.rs:1`
  - `crates/server/tests/fr_fr_session_020.rs:5`
  - `crates/server/tests/fr_fr_session_020.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2064`
  - `docs/traceability/fr-session-020/fr-session-020-adr.md:1`
  - `docs/traceability/fr-session-020/fr-session-020-adr.md:6`

#### FR-SESSION-021

- Tests:
  - `crates/server/tests/fr_fr_session_021.rs:1`
  - `crates/server/tests/fr_fr_session_021.rs:5`
  - `crates/server/tests/fr_fr_session_021.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2068`
  - `docs/traceability/fr-session-021/fr-session-021-adr.md:1`
  - `docs/traceability/fr-session-021/fr-session-021-adr.md:6`

#### FR-SESSION-022

- Tests:
  - `crates/server/tests/fr_fr_session_022.rs:1`
  - `crates/server/tests/fr_fr_session_022.rs:5`
  - `crates/server/tests/fr_fr_session_022.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2070`
  - `docs/traceability/fr-session-022/fr-session-022-adr.md:1`
  - `docs/traceability/fr-session-022/fr-session-022-adr.md:6`

#### FR-SESSION-023

- Tests:
  - `crates/server/tests/fr_fr_session_023.rs:1`
  - `crates/server/tests/fr_fr_session_023.rs:5`
  - `crates/server/tests/fr_fr_session_023.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2072`
  - `docs/traceability/fr-session-023/fr-session-023-adr.md:1`
  - `docs/traceability/fr-session-023/fr-session-023-adr.md:6`

#### FR-SESSION-024

- Tests:
  - `crates/server/tests/fr_fr_session_024.rs:1`
  - `crates/server/tests/fr_fr_session_024.rs:5`
  - `crates/server/tests/fr_fr_session_024.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2074`
  - `docs/traceability/fr-session-024/fr-session-024-adr.md:1`
  - `docs/traceability/fr-session-024/fr-session-024-adr.md:6`

#### FR-SESSION-025

- Tests:
  - `crates/server/tests/fr_fr_session_025.rs:1`
  - `crates/server/tests/fr_fr_session_025.rs:5`
  - `crates/server/tests/fr_fr_session_025.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2076`
  - `docs/traceability/fr-session-025/fr-session-025-adr.md:1`
  - `docs/traceability/fr-session-025/fr-session-025-adr.md:6`

#### FR-SESSION-026

- Tests:
  - `crates/server/tests/fr_fr_session_026.rs:1`
  - `crates/server/tests/fr_fr_session_026.rs:5`
  - `crates/server/tests/fr_fr_session_026.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2080`
  - `docs/traceability/fr-session-026/fr-session-026-adr.md:1`
  - `docs/traceability/fr-session-026/fr-session-026-adr.md:6`

#### FR-SESSION-027

- Tests:
  - `crates/server/tests/fr_fr_session_027.rs:1`
  - `crates/server/tests/fr_fr_session_027.rs:5`
  - `crates/server/tests/fr_fr_session_027.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2082`
  - `docs/traceability/fr-session-027/fr-session-027-adr.md:1`
  - `docs/traceability/fr-session-027/fr-session-027-adr.md:6`

#### FR-SESSION-028

- Tests:
  - `crates/server/tests/fr_fr_session_028.rs:1`
  - `crates/server/tests/fr_fr_session_028.rs:5`
  - `crates/server/tests/fr_fr_session_028.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2084`
  - `docs/traceability/fr-session-028/fr-session-028-adr.md:1`
  - `docs/traceability/fr-session-028/fr-session-028-adr.md:6`

#### FR-SESSION-029

- Tests:
  - `crates/server/tests/fr_fr_session_029.rs:1`
  - `crates/server/tests/fr_fr_session_029.rs:5`
  - `crates/server/tests/fr_fr_session_029.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2086`
  - `docs/traceability/fr-session-029/fr-session-029-adr.md:1`
  - `docs/traceability/fr-session-029/fr-session-029-adr.md:6`

#### FR-SESSION-030

- Tests:
  - `crates/server/tests/fr_fr_session_030.rs:1`
  - `crates/server/tests/fr_fr_session_030.rs:5`
  - `crates/server/tests/fr_fr_session_030.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2090`
  - `docs/traceability/fr-session-030/fr-session-030-adr.md:1`
  - `docs/traceability/fr-session-030/fr-session-030-adr.md:6`

#### FR-SESSION-031

- Tests:
  - `crates/server/tests/fr_fr_session_031.rs:1`
  - `crates/server/tests/fr_fr_session_031.rs:5`
  - `crates/server/tests/fr_fr_session_031.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2092`
  - `docs/traceability/fr-session-031/fr-session-031-adr.md:1`
  - `docs/traceability/fr-session-031/fr-session-031-adr.md:6`

#### FR-SESSION-032

- Tests:
  - `crates/server/tests/fr_fr_session_032.rs:1`
  - `crates/server/tests/fr_fr_session_032.rs:5`
  - `crates/server/tests/fr_fr_session_032.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2094`
  - `docs/traceability/fr-session-032/fr-session-032-adr.md:1`
  - `docs/traceability/fr-session-032/fr-session-032-adr.md:6`

#### FR-SESSION-033

- Tests:
  - `crates/server/tests/fr_fr_session_033.rs:1`
  - `crates/server/tests/fr_fr_session_033.rs:5`
  - `crates/server/tests/fr_fr_session_033.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2096`
  - `docs/traceability/fr-session-033/fr-session-033-adr.md:1`
  - `docs/traceability/fr-session-033/fr-session-033-adr.md:6`


### Epic `FR-SOC-INS` — 7 IDs

#### FR-SOC-INS-001

- Tests:
  - `crates/engine/tests/fr_fr_soc_ins_001.rs:1`
  - `crates/engine/tests/fr_fr_soc_ins_001.rs:5`
  - `crates/engine/tests/fr_fr_soc_ins_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1692`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1998`
  - `docs/traceability/fr-soc-ins-001/fr-soc-ins-001-adr.md:1`

#### FR-SOC-INS-002

- Tests:
  - `crates/engine/tests/fr_fr_soc_ins_002.rs:1`
  - `crates/engine/tests/fr_fr_soc_ins_002.rs:5`
  - `crates/engine/tests/fr_fr_soc_ins_002.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1708`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1999`
  - `docs/traceability/fr-soc-ins-002/fr-soc-ins-002-adr.md:1`

#### FR-SOC-INS-003

- Tests:
  - `crates/engine/tests/fr_fr_soc_ins_003.rs:1`
  - `crates/engine/tests/fr_fr_soc_ins_003.rs:5`
  - `crates/engine/tests/fr_fr_soc_ins_003.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1719`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2000`
  - `docs/traceability/fr-soc-ins-003/fr-soc-ins-003-adr.md:1`

#### FR-SOC-INS-004

- Tests:
  - `crates/engine/tests/fr_fr_soc_ins_004.rs:1`
  - `crates/engine/tests/fr_fr_soc_ins_004.rs:5`
  - `crates/engine/tests/fr_fr_soc_ins_004.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1733`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2001`
  - `docs/traceability/fr-soc-ins-004/fr-soc-ins-004-adr.md:1`

#### FR-SOC-INS-005

- Tests:
  - `crates/engine/tests/fr_fr_soc_ins_005.rs:1`
  - `crates/engine/tests/fr_fr_soc_ins_005.rs:5`
  - `crates/engine/tests/fr_fr_soc_ins_005.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1752`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2002`
  - `docs/traceability/fr-soc-ins-005/fr-soc-ins-005-adr.md:1`

#### FR-SOC-INS-006

- Tests:
  - `crates/engine/tests/fr_fr_soc_ins_006.rs:1`
  - `crates/engine/tests/fr_fr_soc_ins_006.rs:5`
  - `crates/engine/tests/fr_fr_soc_ins_006.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4432`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4575`
  - `docs/traceability/fr-soc-ins-006/fr-soc-ins-006-adr.md:1`

#### FR-SOC-INS-007

- Tests:
  - `crates/engine/tests/fr_fr_soc_ins_007.rs:1`
  - `crates/engine/tests/fr_fr_soc_ins_007.rs:5`
  - `crates/engine/tests/fr_fr_soc_ins_007.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4453`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4576`
  - `docs/traceability/fr-soc-ins-007/fr-soc-ins-007-adr.md:1`


### Epic `NFR-CIV-DET` — 4 IDs

#### NFR-CIV-DET-001

- Tests:
  - `crates/engine/tests/fr_engine_hash_lod_perf_tests.rs:4`
  - `crates/engine/tests/fr_engine_hash_lod_perf_tests.rs:80`
  - `crates/engine/tests/fr_engine_hash_lod_perf_tests.rs:83`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:95`
  - `docs/guides/voxel-emergent-vision-and-migration.md:183`
  - `docs/reference/non-functional-requirements.md:130`

#### NFR-CIV-DET-002

- Tests:
  - `crates/engine/tests/fr_engine_metrics_replay_tests.rs:148`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:185`
  - `docs/reference/non-functional-requirements.md:144`
  - `docs/reference/non-functional-requirements.md:427`

#### NFR-CIV-DET-003

- Tests:
  - `crates/engine/tests/fr_engine_metrics_replay_tests.rs:158`
  - `crates/engine/tests/fr_nfr_civ_det_003.rs:1`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:50`
  - `docs/guides/voxel-emergent-vision-and-migration.md:95`
  - `docs/guides/voxel-emergent-vision-and-migration.md:169`

#### NFR-CIV-DET-004

- Tests:
  - `crates/engine/tests/fr_nfr_civ_det_004.rs:1`
- Spec/trace:
  - `docs/reference/non-functional-requirements.md:172`
  - `docs/reference/non-functional-requirements.md:561`
  - `docs/reference/non-functional-requirements.md:598`

