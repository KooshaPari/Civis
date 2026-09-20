# P1 agent-F

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


## Your slice: 104 IDs in 15 epics


### Epic `FR-CIV-ACTOR` — 2 IDs

#### FR-CIV-ACTOR-001

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:142`
  - `crates/build/tests/fr_matrix_batch12.rs:145`
- Spec/trace:
  - `agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:24`
  - `agileplus-specs/civ-005-climate-disasters-seasons/spec.md:39`
  - `agileplus-specs/civ-006-deep-combat/spec.md:40`

#### FR-CIV-ACTOR-002

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:173`
  - `crates/build/tests/fr_matrix_batch12.rs:176`
- Spec/trace:
  - `agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:25`
  - `docs/reference/agileplus-artifacts-index.md:73`
  - `docs/reference/agileplus-artifacts-index.md:269`


### Epic `FR-CIV-BRUSH` — 13 IDs

#### FR-CIV-BRUSH-01

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_01.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_01.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_01.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:514`
  - `docs/traceability/fr-civ-brush-01/fr-civ-brush-01-adr.md:1`
  - `docs/traceability/fr-civ-brush-01/fr-civ-brush-01-adr.md:6`

#### FR-CIV-BRUSH-02

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_02.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_02.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_02.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:515`
  - `docs/traceability/fr-civ-brush-02/fr-civ-brush-02-adr.md:1`
  - `docs/traceability/fr-civ-brush-02/fr-civ-brush-02-adr.md:6`

#### FR-CIV-BRUSH-03

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_03.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_03.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_03.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:516`
  - `docs/traceability/fr-civ-brush-03/fr-civ-brush-03-adr.md:1`
  - `docs/traceability/fr-civ-brush-03/fr-civ-brush-03-adr.md:6`

#### FR-CIV-BRUSH-04

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_04.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_04.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_04.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:517`
  - `docs/traceability/fr-civ-brush-04/fr-civ-brush-04-adr.md:1`
  - `docs/traceability/fr-civ-brush-04/fr-civ-brush-04-adr.md:6`

#### FR-CIV-BRUSH-05

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_05.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_05.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_05.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:518`
  - `docs/traceability/fr-civ-brush-05/fr-civ-brush-05-adr.md:1`
  - `docs/traceability/fr-civ-brush-05/fr-civ-brush-05-adr.md:6`

#### FR-CIV-BRUSH-06

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_06.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_06.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_06.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:519`
  - `docs/traceability/fr-civ-brush-06/fr-civ-brush-06-adr.md:1`
  - `docs/traceability/fr-civ-brush-06/fr-civ-brush-06-adr.md:6`

#### FR-CIV-BRUSH-07

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_07.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_07.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_07.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:520`
  - `docs/traceability/fr-civ-brush-07/fr-civ-brush-07-adr.md:1`
  - `docs/traceability/fr-civ-brush-07/fr-civ-brush-07-adr.md:6`

#### FR-CIV-BRUSH-08

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_08.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_08.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_08.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:521`
  - `docs/traceability/fr-civ-brush-08/fr-civ-brush-08-adr.md:1`
  - `docs/traceability/fr-civ-brush-08/fr-civ-brush-08-adr.md:6`

#### FR-CIV-BRUSH-09

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_09.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_09.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_09.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:522`
  - `docs/traceability/fr-civ-brush-09/fr-civ-brush-09-adr.md:1`
  - `docs/traceability/fr-civ-brush-09/fr-civ-brush-09-adr.md:6`

#### FR-CIV-BRUSH-10

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_10.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_10.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_10.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:523`
  - `docs/traceability/fr-civ-brush-10/fr-civ-brush-10-adr.md:1`
  - `docs/traceability/fr-civ-brush-10/fr-civ-brush-10-adr.md:6`

#### FR-CIV-BRUSH-11

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_11.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_11.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_11.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:524`
  - `docs/traceability/fr-civ-brush-11/fr-civ-brush-11-adr.md:1`
  - `docs/traceability/fr-civ-brush-11/fr-civ-brush-11-adr.md:6`

#### FR-CIV-BRUSH-12

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_12.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_12.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_12.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:525`
  - `docs/traceability/fr-civ-brush-12/fr-civ-brush-12-adr.md:1`
  - `docs/traceability/fr-civ-brush-12/fr-civ-brush-12-adr.md:6`

#### FR-CIV-BRUSH-13

- Tests:
  - `crates/engine/tests/fr_fr_civ_brush_13.rs:1`
  - `crates/engine/tests/fr_fr_civ_brush_13.rs:4`
  - `crates/engine/tests/fr_fr_civ_brush_13.rs:10`
- Spec/trace:
  - `docs/design/brush-tool-system.md:526`
  - `docs/traceability/fr-civ-brush-13/fr-civ-brush-13-adr.md:1`
  - `docs/traceability/fr-civ-brush-13/fr-civ-brush-13-adr.md:6`


### Epic `FR-CIV-DIPLO-002-SHADOW` — 1 IDs

#### FR-CIV-DIPLO-002-SHADOW

- Tests:
  - `crates/diplomacy/tests/fr_civ_diplo_tests.rs:3`
  - `crates/diplomacy/tests/fr_civ_diplo_tests.rs:8`
  - `crates/diplomacy/tests/fr_civ_diplo_tests.rs:11`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:215`
  - `PLAN.md:209`
  - `PLAN.md:210`


### Epic `FR-CIV-GODTOOL` — 5 IDs

#### FR-CIV-GODTOOL-910

- Tests:
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:2`
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:173`
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:176`
- Spec/trace:
  - `docs/agileplus/epics/civ-w1-voxel-render.md:9`
  - `docs/agileplus/epics/civ-w1-voxel-render.md:19`
  - `docs/agileplus/README.md:20`

#### FR-CIV-GODTOOL-911

- Tests:
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:429`
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:432`
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:473`
- Spec/trace:
  - `docs/agileplus/epics/civ-w1-voxel-render.md:10`
  - `docs/agileplus/epics/civ-w1-voxel-render.md:20`
  - `docs/agileplus/README.md:20`

#### FR-CIV-GODTOOL-912

- Tests:
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:612`
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:615`
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:654`
- Spec/trace:
  - `docs/agileplus/epics/civ-w1-voxel-render.md:11`
  - `docs/agileplus/epics/civ-w1-voxel-render.md:21`
  - `docs/agileplus/README.md:20`

#### FR-CIV-GODTOOL-920

- Tests:
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:763`
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:766`
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:802`
- Spec/trace:
  - `docs/agileplus/epics/civ-w1-voxel-render.md:12`
  - `docs/agileplus/epics/civ-w1-voxel-render.md:22`
  - `docs/agileplus/epics/civ-w6-ui.md:10`

#### FR-CIV-GODTOOL-921

- Tests:
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:886`
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:889`
  - `crates/engine/tests/fr_civ_godtool_cluster.rs:940`
- Spec/trace:
  - `docs/agileplus/epics/civ-w1-voxel-render.md:13`
  - `docs/agileplus/epics/civ-w1-voxel-render.md:23`
  - `docs/agileplus/epics/civ-w6-ui.md:11`


### Epic `FR-CIV-LEGENDS-INSPECT` — 1 IDs

#### FR-CIV-LEGENDS-INSPECT-08

- Tests:
  - `crates/legends/tests/fr_fr_civ_legends_inspect_08.rs:1`
  - `crates/legends/tests/fr_fr_civ_legends_inspect_08.rs:10`
  - `crates/legends/tests/fr_fr_civ_legends_inspect_08.rs:16`
- Spec/trace:
  - `docs/design/legends-engine.md:443`
  - `docs/traceability/fr-civ-legends-inspect-08/fr-civ-legends-inspect-08-adr.md:1`
  - `docs/traceability/fr-civ-legends-inspect-08/fr-civ-legends-inspect-08-adr.md:6`


### Epic `FR-CIV-LLM` — 6 IDs

#### FR-CIV-LLM-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_llm_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_llm_001.rs:5`
  - `crates/engine/tests/fr_fr_civ_llm_001.rs:9`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-llm-001/fr-civ-llm-001-adr.md:1`
  - `docs/traceability/fr-civ-llm-001/fr-civ-llm-001-adr.md:6`

#### FR-CIV-LLM-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_llm_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_llm_002.rs:5`
  - `crates/engine/tests/fr_fr_civ_llm_002.rs:9`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-llm-002/fr-civ-llm-002-adr.md:1`
  - `docs/traceability/fr-civ-llm-002/fr-civ-llm-002-adr.md:6`

#### FR-CIV-LLM-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_llm_003.rs:1`
  - `crates/engine/tests/fr_fr_civ_llm_003.rs:5`
  - `crates/engine/tests/fr_fr_civ_llm_003.rs:9`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-llm-003/fr-civ-llm-003-adr.md:1`
  - `docs/traceability/fr-civ-llm-003/fr-civ-llm-003-adr.md:6`

#### FR-CIV-LLM-004

- Tests:
  - `crates/engine/tests/fr_fr_civ_llm_004.rs:1`
  - `crates/engine/tests/fr_fr_civ_llm_004.rs:5`
  - `crates/engine/tests/fr_fr_civ_llm_004.rs:9`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-llm-004/fr-civ-llm-004-adr.md:1`
  - `docs/traceability/fr-civ-llm-004/fr-civ-llm-004-adr.md:6`

#### FR-CIV-LLM-005

- Tests:
  - `crates/engine/tests/fr_fr_civ_llm_005.rs:1`
  - `crates/engine/tests/fr_fr_civ_llm_005.rs:5`
  - `crates/engine/tests/fr_fr_civ_llm_005.rs:9`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-llm-005/fr-civ-llm-005-adr.md:1`
  - `docs/traceability/fr-civ-llm-005/fr-civ-llm-005-adr.md:6`

#### FR-CIV-LLM-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_llm_006.rs:1`
  - `crates/engine/tests/fr_fr_civ_llm_006.rs:5`
  - `crates/engine/tests/fr_fr_civ_llm_006.rs:9`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-llm-006/fr-civ-llm-006-adr.md:1`
  - `docs/traceability/fr-civ-llm-006/fr-civ-llm-006-adr.md:6`


### Epic `FR-CIV-PERF` — 19 IDs

#### FR-CIV-PERF-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_002.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_002.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1926`
  - `docs/traceability/fr-civ-perf-002/fr-civ-perf-002-adr.md:1`
  - `docs/traceability/fr-civ-perf-002/fr-civ-perf-002-adr.md:6`

#### FR-CIV-PERF-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_003.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_003.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_003.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1931`
  - `docs/traceability/fr-civ-perf-003/fr-civ-perf-003-adr.md:1`
  - `docs/traceability/fr-civ-perf-003/fr-civ-perf-003-adr.md:6`

#### FR-CIV-PERF-004

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_004.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_004.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_004.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1936`
  - `docs/traceability/fr-civ-perf-004/fr-civ-perf-004-adr.md:1`
  - `docs/traceability/fr-civ-perf-004/fr-civ-perf-004-adr.md:6`

#### FR-CIV-PERF-005

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_005.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_005.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_005.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1941`
  - `docs/traceability/fr-civ-perf-005/fr-civ-perf-005-adr.md:1`
  - `docs/traceability/fr-civ-perf-005/fr-civ-perf-005-adr.md:6`

#### FR-CIV-PERF-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_006.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_006.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_006.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1946`
  - `docs/traceability/fr-civ-perf-006/fr-civ-perf-006-adr.md:1`
  - `docs/traceability/fr-civ-perf-006/fr-civ-perf-006-adr.md:6`

#### FR-CIV-PERF-007

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_007.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_007.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_007.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1951`
  - `docs/traceability/fr-civ-perf-007/fr-civ-perf-007-adr.md:1`
  - `docs/traceability/fr-civ-perf-007/fr-civ-perf-007-adr.md:6`

#### FR-CIV-PERF-008

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_008.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_008.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_008.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1956`
  - `docs/traceability/fr-civ-perf-008/fr-civ-perf-008-adr.md:1`
  - `docs/traceability/fr-civ-perf-008/fr-civ-perf-008-adr.md:6`

#### FR-CIV-PERF-009

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_009.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_009.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_009.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1961`
  - `docs/traceability/fr-civ-perf-009/fr-civ-perf-009-adr.md:1`
  - `docs/traceability/fr-civ-perf-009/fr-civ-perf-009-adr.md:6`

#### FR-CIV-PERF-010

- Tests:
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:1176`
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:1184`
  - `crates/engine/tests/fr_fr_civ_perf_010.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1966`
  - `docs/traceability/fr-civ-perf-010/fr-civ-perf-010-adr.md:1`
  - `docs/traceability/fr-civ-perf-010/fr-civ-perf-010-adr.md:6`

#### FR-CIV-PERF-011

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_011.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_011.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_011.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1971`
  - `docs/traceability/fr-civ-perf-011/fr-civ-perf-011-adr.md:1`
  - `docs/traceability/fr-civ-perf-011/fr-civ-perf-011-adr.md:6`

#### FR-CIV-PERF-012

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_012.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_012.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_012.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1976`
  - `docs/traceability/fr-civ-perf-012/fr-civ-perf-012-adr.md:1`
  - `docs/traceability/fr-civ-perf-012/fr-civ-perf-012-adr.md:6`

#### FR-CIV-PERF-013

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_013.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_013.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_013.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1981`
  - `docs/traceability/fr-civ-perf-013/fr-civ-perf-013-adr.md:1`
  - `docs/traceability/fr-civ-perf-013/fr-civ-perf-013-adr.md:6`

#### FR-CIV-PERF-014

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_014.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_014.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_014.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1986`
  - `docs/traceability/fr-civ-perf-014/fr-civ-perf-014-adr.md:1`
  - `docs/traceability/fr-civ-perf-014/fr-civ-perf-014-adr.md:6`

#### FR-CIV-PERF-015

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_015.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_015.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_015.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1991`
  - `docs/traceability/fr-civ-perf-015/fr-civ-perf-015-adr.md:1`
  - `docs/traceability/fr-civ-perf-015/fr-civ-perf-015-adr.md:6`

#### FR-CIV-PERF-016

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_016.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_016.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_016.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:1996`
  - `docs/traceability/fr-civ-perf-016/fr-civ-perf-016-adr.md:1`
  - `docs/traceability/fr-civ-perf-016/fr-civ-perf-016-adr.md:6`

#### FR-CIV-PERF-017

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_017.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_017.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_017.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:2001`
  - `docs/traceability/fr-civ-perf-017/fr-civ-perf-017-adr.md:1`
  - `docs/traceability/fr-civ-perf-017/fr-civ-perf-017-adr.md:6`

#### FR-CIV-PERF-018

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_018.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_018.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_018.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:2006`
  - `docs/traceability/fr-civ-perf-018/fr-civ-perf-018-adr.md:1`
  - `docs/traceability/fr-civ-perf-018/fr-civ-perf-018-adr.md:6`

#### FR-CIV-PERF-019

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_019.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_019.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_019.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:2011`
  - `docs/traceability/fr-civ-perf-019/fr-civ-perf-019-adr.md:1`
  - `docs/traceability/fr-civ-perf-019/fr-civ-perf-019-adr.md:6`

#### FR-CIV-PERF-020

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_020.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_020.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_020.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0500-performance-optimization-spec.md:2016`
  - `docs/traceability/fr-civ-perf-020/fr-civ-perf-020-adr.md:1`
  - `docs/traceability/fr-civ-perf-020/fr-civ-perf-020-adr.md:6`


### Epic `FR-CIV-QOL` — 14 IDs

#### FR-CIV-QOL-100

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_100.rs:1`
  - `crates/engine/tests/fr_fr_civ_qol_100.rs:6`
- Spec/trace:
  - `docs/design/onboarding-qol.md:37`
  - `docs/traceability/fr-civ-qol-100/fr-civ-qol-100-adr.md:1`
  - `docs/traceability/fr-civ-qol-100/fr-civ-qol-100-adr.md:6`

#### FR-CIV-QOL-110

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_110.rs:1`
  - `crates/engine/tests/fr_fr_civ_qol_110.rs:6`
- Spec/trace:
  - `docs/design/onboarding-qol.md:74`
  - `docs/traceability/fr-civ-qol-110/fr-civ-qol-110-adr.md:1`
  - `docs/traceability/fr-civ-qol-110/fr-civ-qol-110-adr.md:6`

#### FR-CIV-QOL-120

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_120.rs:1`
  - `crates/engine/tests/fr_fr_civ_qol_120.rs:6`
- Spec/trace:
  - `docs/design/onboarding-qol.md:90`
  - `docs/traceability/fr-civ-qol-120/fr-civ-qol-120-adr.md:1`
  - `docs/traceability/fr-civ-qol-120/fr-civ-qol-120-adr.md:6`

#### FR-CIV-QOL-130

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_130.rs:1`
  - `crates/engine/tests/fr_fr_civ_qol_130.rs:6`
- Spec/trace:
  - `docs/design/onboarding-qol.md:104`
  - `docs/traceability/fr-civ-qol-130/fr-civ-qol-130-adr.md:1`
  - `docs/traceability/fr-civ-qol-130/fr-civ-qol-130-adr.md:6`

#### FR-CIV-QOL-140

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_140.rs:1`
  - `crates/engine/tests/fr_fr_civ_qol_140.rs:7`
- Spec/trace:
  - `docs/design/onboarding-qol.md:120`
  - `docs/traceability/fr-civ-qol-140/fr-civ-qol-140-adr.md:1`
  - `docs/traceability/fr-civ-qol-140/fr-civ-qol-140-adr.md:6`

#### FR-CIV-QOL-150

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_150.rs:1`
- Spec/trace:
  - `docs/design/onboarding-qol.md:136`
  - `docs/traceability/fr-civ-qol-150/fr-civ-qol-150-adr.md:1`
  - `docs/traceability/fr-civ-qol-150/fr-civ-qol-150-adr.md:6`

#### FR-CIV-QOL-160

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_160.rs:1`
- Spec/trace:
  - `docs/design/onboarding-qol.md:146`
  - `docs/traceability/fr-civ-qol-160/fr-civ-qol-160-adr.md:1`
  - `docs/traceability/fr-civ-qol-160/fr-civ-qol-160-adr.md:6`

#### FR-CIV-QOL-170

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_170.rs:1`
- Spec/trace:
  - `docs/design/onboarding-qol.md:162`
  - `docs/traceability/fr-civ-qol-170/fr-civ-qol-170-adr.md:1`
  - `docs/traceability/fr-civ-qol-170/fr-civ-qol-170-adr.md:6`

#### FR-CIV-QOL-180

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_180.rs:1`
  - `crates/engine/tests/fr_fr_civ_qol_180.rs:7`
- Spec/trace:
  - `docs/design/onboarding-qol.md:172`
  - `docs/traceability/fr-civ-qol-180/fr-civ-qol-180-adr.md:1`
  - `docs/traceability/fr-civ-qol-180/fr-civ-qol-180-adr.md:6`

#### FR-CIV-QOL-190

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_190.rs:1`
- Spec/trace:
  - `docs/design/onboarding-qol.md:191`
  - `docs/traceability/fr-civ-qol-190/fr-civ-qol-190-adr.md:1`
  - `docs/traceability/fr-civ-qol-190/fr-civ-qol-190-adr.md:6`

#### FR-CIV-QOL-200

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_200.rs:1`
- Spec/trace:
  - `docs/design/onboarding-qol.md:205`
  - `docs/traceability/fr-civ-qol-200/fr-civ-qol-200-adr.md:1`
  - `docs/traceability/fr-civ-qol-200/fr-civ-qol-200-adr.md:6`

#### FR-CIV-QOL-210

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_210.rs:1`
- Spec/trace:
  - `docs/design/onboarding-qol.md:222`
  - `docs/traceability/fr-civ-qol-210/fr-civ-qol-210-adr.md:1`
  - `docs/traceability/fr-civ-qol-210/fr-civ-qol-210-adr.md:6`

#### FR-CIV-QOL-220

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_220.rs:1`
- Spec/trace:
  - `docs/design/onboarding-qol.md:240`
  - `docs/traceability/fr-civ-qol-220/fr-civ-qol-220-adr.md:1`
  - `docs/traceability/fr-civ-qol-220/fr-civ-qol-220-adr.md:6`

#### FR-CIV-QOL-230

- Tests:
  - `crates/engine/tests/fr_fr_civ_qol_230.rs:1`
  - `crates/engine/tests/fr_fr_civ_qol_230.rs:7`
- Spec/trace:
  - `docs/design/onboarding-qol.md:249`
  - `docs/traceability/fr-civ-qol-230/fr-civ-qol-230-adr.md:1`
  - `docs/traceability/fr-civ-qol-230/fr-civ-qol-230-adr.md:6`


### Epic `FR-CIV-RTS` — 15 IDs

#### FR-CIV-RTS-001

- Tests:
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:7`
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:128`
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:131`
- Spec/trace:
  - `docs/reference/FR_TRACKER.md:16`
  - `docs/reports/STATUS_REPORT.md:93`
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:1313`

#### FR-CIV-RTS-002

- Tests:
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:8`
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:370`
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:373`
- Spec/trace:
  - `docs/reports/STATUS_REPORT.md:94`
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:1314`
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:1315`

#### FR-CIV-RTS-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_003.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_003.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:1116`
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:1318`
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:1344`

#### FR-CIV-RTS-004

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_004.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_004.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:1143`
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:1317`
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2007`

#### FR-CIV-RTS-005

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_005.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_005.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2008`
  - `docs/traceability/fr-civ-rts-005/fr-civ-rts-005-adr.md:1`
  - `docs/traceability/fr-civ-rts-005/fr-civ-rts-005-adr.md:6`

#### FR-CIV-RTS-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_006.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_006.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2009`
  - `docs/traceability/fr-civ-rts-006/fr-civ-rts-006-adr.md:1`
  - `docs/traceability/fr-civ-rts-006/fr-civ-rts-006-adr.md:6`

#### FR-CIV-RTS-007

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_007.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_007.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2010`
  - `docs/traceability/fr-civ-rts-007/fr-civ-rts-007-adr.md:1`
  - `docs/traceability/fr-civ-rts-007/fr-civ-rts-007-adr.md:6`

#### FR-CIV-RTS-008

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_008.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_008.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2011`
  - `docs/traceability/fr-civ-rts-008/fr-civ-rts-008-adr.md:1`
  - `docs/traceability/fr-civ-rts-008/fr-civ-rts-008-adr.md:6`

#### FR-CIV-RTS-009

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_009.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_009.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2012`
  - `docs/traceability/fr-civ-rts-009/fr-civ-rts-009-adr.md:1`
  - `docs/traceability/fr-civ-rts-009/fr-civ-rts-009-adr.md:6`

#### FR-CIV-RTS-010

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_010.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_010.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2013`
  - `docs/traceability/fr-civ-rts-010/fr-civ-rts-010-adr.md:1`
  - `docs/traceability/fr-civ-rts-010/fr-civ-rts-010-adr.md:6`

#### FR-CIV-RTS-011

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_011.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_011.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2014`
  - `docs/traceability/fr-civ-rts-011/fr-civ-rts-011-adr.md:1`
  - `docs/traceability/fr-civ-rts-011/fr-civ-rts-011-adr.md:6`

#### FR-CIV-RTS-012

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_012.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_012.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2015`
  - `docs/specs/CIV-0400-ai-npc-behavior-spec.md:2532`
  - `docs/traceability/fr-civ-rts-012/fr-civ-rts-012-adr.md:1`

#### FR-CIV-RTS-013

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_013.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_013.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2016`
  - `docs/specs/CIV-0400-ai-npc-behavior-spec.md:2531`
  - `docs/traceability/fr-civ-rts-013/fr-civ-rts-013-adr.md:1`

#### FR-CIV-RTS-014

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_014.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_014.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2017`
  - `docs/specs/CIV-0400-ai-npc-behavior-spec.md:14`
  - `docs/specs/CIV-0400-ai-npc-behavior-spec.md:2514`

#### FR-CIV-RTS-015

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_015.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_015.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2018`
  - `docs/specs/CIV-0400-ai-npc-behavior-spec.md:2533`
  - `docs/traceability/fr-civ-rts-015/fr-civ-rts-015-adr.md:1`


### Epic `FR-CIV-TACTICS` — 15 IDs

#### FR-CIV-TACTICS-051

- Tests:
  - `crates/tactics/tests/fr_civ_tactics_tests.rs:18`
  - `crates/tactics/tests/fr_civ_tactics_tests.rs:26`
  - `crates/tactics/tests/fr_fr_civ_tactics_051.rs:1`
- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:53`
  - `docs/traceability/fr-3d-matrix.md:141`
  - `docs/traceability/full-traceability-matrix.md:246`

#### FR-CIV-TACTICS-058

- Tests:
  - `crates/mod-host/tests/fr_matrix_batch10.rs:12`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:48`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:49`
- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:61`
  - `docs/traceability/fr-3d-matrix.md:149`
  - `docs/traceability/full-traceability-matrix.md:253`

#### FR-CIV-TACTICS-060

- Tests:
  - `crates/mod-host/tests/fr_matrix_batch10.rs:13`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:94`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:95`
- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:63`
  - `docs/traceability/fr-3d-matrix.md:151`
  - `docs/traceability/full-traceability-matrix.md:255`

#### FR-CIV-TACTICS-062

- Tests:
  - `crates/mod-host/tests/fr_matrix_batch10.rs:13`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:115`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:116`
- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:64`
  - `docs/traceability/fr-3d-matrix.md:152`
  - `docs/traceability/full-traceability-matrix.md:257`

#### FR-CIV-TACTICS-064

- Tests:
  - `crates/mod-host/tests/fr_matrix_batch10.rs:13`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:141`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:142`
- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:66`
  - `docs/traceability/fr-3d-matrix.md:154`
  - `docs/traceability/full-traceability-matrix.md:259`

#### FR-CIV-TACTICS-065

- Tests:
  - `crates/tactics/tests/fr_civ_tactics_tests.rs:33`
  - `crates/tactics/tests/fr_fr_civ_tactics_065.rs:1`
  - `crates/tactics/tests/fr_fr_civ_tactics_065.rs:6`
- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:67`
  - `docs/traceability/fr-3d-matrix.md:155`
  - `docs/traceability/full-traceability-matrix.md:260`

#### FR-CIV-TACTICS-067

- Tests:
  - `crates/mod-host/tests/fr_matrix_batch10.rs:14`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:171`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:172`
- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:69`
  - `docs/traceability/fr-3d-matrix.md:157`
  - `docs/traceability/full-traceability-matrix.md:262`

#### FR-CIV-TACTICS-069

- Tests:
  - `crates/mod-host/tests/fr_matrix_batch10.rs:14`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:195`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:196`
- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:71`
  - `docs/traceability/fr-3d-matrix.md:159`
  - `docs/traceability/full-traceability-matrix.md:264`

#### FR-CIV-TACTICS-070

- Tests:
  - `crates/mod-host/tests/fr_matrix_batch10.rs:14`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:225`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:226`
- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:72`
  - `docs/traceability/fr-3d-matrix.md:160`
  - `docs/traceability/full-traceability-matrix.md:265`

#### FR-CIV-TACTICS-072

- Tests:
  - `crates/mod-host/tests/fr_matrix_batch10.rs:15`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:248`
  - `crates/mod-host/tests/fr_matrix_batch10.rs:249`
- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:74`
  - `docs/traceability/fr-3d-matrix.md:162`
  - `docs/traceability/full-traceability-matrix.md:267`

#### FR-CIV-TACTICS-073

- Tests:
  - `crates/tactics/tests/fr_fr_civ_tactics_073.rs:1`
  - `crates/tactics/tests/fr_fr_civ_tactics_073.rs:6`
  - `crates/tactics/tests/fr_fr_civ_tactics_073.rs:16`
- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:75`
  - `docs/traceability/fr-3d-matrix.md:163`
  - `docs/traceability/full-traceability-matrix.md:268`

#### FR-CIV-TACTICS-076

- Tests:
  - `crates/tactics/tests/fr_civ_tactics_tests.rs:41`
  - `crates/tactics/tests/fr_civ_tactics_tests.rs:49`
  - `crates/tactics/tests/fr_fr_civ_tactics_076.rs:1`
- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:78`
  - `docs/traceability/fr-3d-matrix.md:166`
  - `docs/traceability/full-traceability-matrix.md:271`

#### FR-CIV-TACTICS-100

- Tests:
  - `crates/tactics/tests/fr_fr_civ_tactics_100.rs:1`
  - `crates/tactics/tests/fr_fr_civ_tactics_100.rs:6`
  - `crates/tactics/tests/fr_fr_civ_tactics_100.rs:14`
- Spec/trace:
  - `agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:36`
  - `docs/traceability/fr-civ-tactics-100/fr-civ-tactics-100-adr.md:1`
  - `docs/traceability/fr-civ-tactics-100/fr-civ-tactics-100-adr.md:6`

#### FR-CIV-TACTICS-101

- Tests:
  - `crates/tactics/tests/fr_fr_civ_tactics_101.rs:1`
  - `crates/tactics/tests/fr_fr_civ_tactics_101.rs:6`
  - `crates/tactics/tests/fr_fr_civ_tactics_101.rs:14`
- Spec/trace:
  - `agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:40`
  - `docs/traceability/fr-civ-tactics-101/fr-civ-tactics-101-adr.md:1`
  - `docs/traceability/fr-civ-tactics-101/fr-civ-tactics-101-adr.md:6`

#### FR-CIV-TACTICS-102

- Tests:
  - `crates/tactics/tests/fr_fr_civ_tactics_102.rs:1`
  - `crates/tactics/tests/fr_fr_civ_tactics_102.rs:6`
  - `crates/tactics/tests/fr_fr_civ_tactics_102.rs:14`
- Spec/trace:
  - `agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:43`
  - `docs/traceability/fr-civ-tactics-102/fr-civ-tactics-102-adr.md:1`
  - `docs/traceability/fr-civ-tactics-102/fr-civ-tactics-102-adr.md:6`


### Epic `FR-CORE` — 2 IDs

#### FR-CORE-003

- Tests:
  - `crates/engine/tests/fr_fr_core_003.rs:1`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `agileplus-specs/civ-001-core-simulation-engine/spec.md:26`
  - `agileplus-specs/civ-002-economy-joule-system/spec.md:42`

#### FR-CORE-008

- Tests:
  - `crates/engine/tests/fr_fr_core_008.rs:1`
- Spec/trace:
  - `docs/adr/ADR-022-runtime-representation-deviations.md:20`
  - `docs/adr/ADR-022-runtime-representation-deviations.md:46`
  - `docs/adr/ADR-022-runtime-representation-deviations.md:70`


### Epic `FR-MET` — 1 IDs

#### FR-MET-001

- Tests:
  - `crates/engine/tests/fr_fr_met_001.rs:1`
  - `crates/engine/tests/fr_fr_met_001.rs:8`
  - `crates/engine/tests/fr_fr_met_001.rs:15`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1174`
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1203`
  - `docs/traceability/fr-met-001/fr-met-001-adr.md:1`


### Epic `FR-REPLAY` — 1 IDs

#### FR-REPLAY-002

- Tests:
  - `crates/engine/tests/fr_engine_metrics_replay_tests.rs:4`
  - `crates/engine/tests/fr_engine_metrics_replay_tests.rs:101`
  - `crates/engine/tests/fr_engine_metrics_replay_tests.rs:104`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `agileplus-specs/civ-013-research-api/plan.md:11`
  - `agileplus-specs/civ-013-research-api/spec.md:30`


### Epic `FR-SOC-HLT` — 5 IDs

#### FR-SOC-HLT-001

- Tests:
  - `crates/engine/tests/fr_fr_soc_hlt_001.rs:1`
  - `crates/engine/tests/fr_fr_soc_hlt_001.rs:5`
  - `crates/engine/tests/fr_fr_soc_hlt_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1643`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1994`
  - `docs/traceability/fr-soc-hlt-001/fr-soc-hlt-001-adr.md:1`

#### FR-SOC-HLT-002

- Tests:
  - `crates/engine/tests/fr_fr_soc_hlt_002.rs:1`
  - `crates/engine/tests/fr_fr_soc_hlt_002.rs:5`
  - `crates/engine/tests/fr_fr_soc_hlt_002.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1657`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1995`
  - `docs/traceability/fr-soc-hlt-002/fr-soc-hlt-002-adr.md:1`

#### FR-SOC-HLT-003

- Tests:
  - `crates/engine/tests/fr_fr_soc_hlt_003.rs:1`
  - `crates/engine/tests/fr_fr_soc_hlt_003.rs:5`
  - `crates/engine/tests/fr_fr_soc_hlt_003.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1667`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1996`
  - `docs/traceability/fr-soc-hlt-003/fr-soc-hlt-003-adr.md:1`

#### FR-SOC-HLT-004

- Tests:
  - `crates/engine/tests/fr_fr_soc_hlt_004.rs:1`
  - `crates/engine/tests/fr_fr_soc_hlt_004.rs:5`
  - `crates/engine/tests/fr_fr_soc_hlt_004.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1675`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1997`
  - `docs/traceability/fr-soc-hlt-004/fr-soc-hlt-004-adr.md:1`

#### FR-SOC-HLT-005

- Tests:
  - `crates/engine/tests/fr_fr_soc_hlt_005.rs:1`
  - `crates/engine/tests/fr_fr_soc_hlt_005.rs:5`
  - `crates/engine/tests/fr_fr_soc_hlt_005.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4412`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4574`
  - `docs/traceability/fr-soc-hlt-005/fr-soc-hlt-005-adr.md:1`


### Epic `FR-THRY` — 4 IDs

#### FR-THRY-001

- Tests:
  - `crates/engine/tests/fr_fr_thry_001.rs:1`
  - `crates/engine/tests/fr_fr_thry_001.rs:3`
  - `crates/engine/tests/fr_fr_thry_001.rs:16`
- Spec/trace:
  - `docs/traceability/TRACEABILITY_MATRIX.md:118`
  - `docs/traceability/fr-thry-001/fr-thry-001-adr.md:1`
  - `docs/traceability/fr-thry-001/fr-thry-001-adr.md:6`

#### FR-THRY-002

- Tests:
  - `crates/engine/tests/fr_fr_thry_002.rs:1`
  - `crates/engine/tests/fr_fr_thry_002.rs:3`
  - `crates/engine/tests/fr_fr_thry_002.rs:15`
- Spec/trace:
  - `docs/traceability/TRACEABILITY_MATRIX.md:119`
  - `docs/traceability/fr-thry-002/fr-thry-002-adr.md:1`
  - `docs/traceability/fr-thry-002/fr-thry-002-adr.md:6`

#### FR-THRY-003

- Tests:
  - `crates/engine/tests/fr_fr_thry_003.rs:1`
  - `crates/engine/tests/fr_fr_thry_003.rs:3`
  - `crates/engine/tests/fr_fr_thry_003.rs:15`
- Spec/trace:
  - `docs/traceability/TRACEABILITY_MATRIX.md:120`
  - `docs/traceability/fr-thry-003/fr-thry-003-adr.md:1`
  - `docs/traceability/fr-thry-003/fr-thry-003-adr.md:6`

#### FR-THRY-004

- Tests:
  - `crates/engine/tests/fr_fr_thry_004.rs:1`
  - `crates/engine/tests/fr_fr_thry_004.rs:3`
  - `crates/engine/tests/fr_fr_thry_004.rs:16`
- Spec/trace:
  - `docs/traceability/TRACEABILITY_MATRIX.md:121`
  - `docs/traceability/fr-thry-004/fr-thry-004-adr.md:1`
  - `docs/traceability/fr-thry-004/fr-thry-004-adr.md:6`

