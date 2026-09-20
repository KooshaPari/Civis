# P1 agent-D

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


## Your slice: 90 IDs in 16 epics


### Epic `FR-CIV-3D` — 15 IDs

#### FR-CIV-3D-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_001.rs:6`
  - `crates/engine/tests/fr_fr_civ_3d_001.rs:15`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1888`
  - `docs/traceability/fr-civ-3d-001/fr-civ-3d-001-adr.md:1`
  - `docs/traceability/fr-civ-3d-001/fr-civ-3d-001-adr.md:6`

#### FR-CIV-3D-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_002.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1896`
  - `docs/traceability/fr-civ-3d-002/fr-civ-3d-002-adr.md:1`
  - `docs/traceability/fr-civ-3d-002/fr-civ-3d-002-adr.md:6`

#### FR-CIV-3D-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_003.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_003.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1904`
  - `docs/traceability/fr-civ-3d-003/fr-civ-3d-003-adr.md:1`
  - `docs/traceability/fr-civ-3d-003/fr-civ-3d-003-adr.md:6`

#### FR-CIV-3D-004

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_004.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_004.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1912`
  - `docs/traceability/fr-civ-3d-004/fr-civ-3d-004-adr.md:1`
  - `docs/traceability/fr-civ-3d-004/fr-civ-3d-004-adr.md:6`

#### FR-CIV-3D-005

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_005.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_005.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1920`
  - `docs/traceability/fr-civ-3d-005/fr-civ-3d-005-adr.md:1`
  - `docs/traceability/fr-civ-3d-005/fr-civ-3d-005-adr.md:6`

#### FR-CIV-3D-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_006.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_006.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1928`
  - `docs/traceability/fr-civ-3d-006/fr-civ-3d-006-adr.md:1`
  - `docs/traceability/fr-civ-3d-006/fr-civ-3d-006-adr.md:6`

#### FR-CIV-3D-007

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_007.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_007.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1936`
  - `docs/traceability/fr-civ-3d-007/fr-civ-3d-007-adr.md:1`
  - `docs/traceability/fr-civ-3d-007/fr-civ-3d-007-adr.md:6`

#### FR-CIV-3D-008

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_008.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_008.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1944`
  - `docs/traceability/fr-civ-3d-008/fr-civ-3d-008-adr.md:1`
  - `docs/traceability/fr-civ-3d-008/fr-civ-3d-008-adr.md:6`

#### FR-CIV-3D-009

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_009.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_009.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1952`
  - `docs/traceability/fr-civ-3d-009/fr-civ-3d-009-adr.md:1`
  - `docs/traceability/fr-civ-3d-009/fr-civ-3d-009-adr.md:6`

#### FR-CIV-3D-010

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_010.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_010.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1960`
  - `docs/traceability/fr-civ-3d-010/fr-civ-3d-010-adr.md:1`
  - `docs/traceability/fr-civ-3d-010/fr-civ-3d-010-adr.md:6`

#### FR-CIV-3D-011

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_011.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_011.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1968`
  - `docs/traceability/fr-civ-3d-011/fr-civ-3d-011-adr.md:1`
  - `docs/traceability/fr-civ-3d-011/fr-civ-3d-011-adr.md:6`

#### FR-CIV-3D-012

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_012.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_012.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1976`
  - `docs/traceability/fr-civ-3d-012/fr-civ-3d-012-adr.md:1`
  - `docs/traceability/fr-civ-3d-012/fr-civ-3d-012-adr.md:6`

#### FR-CIV-3D-013

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_013.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_013.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1984`
  - `docs/traceability/fr-civ-3d-013/fr-civ-3d-013-adr.md:1`
  - `docs/traceability/fr-civ-3d-013/fr-civ-3d-013-adr.md:6`

#### FR-CIV-3D-014

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_014.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_014.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1992`
  - `docs/traceability/fr-civ-3d-014/fr-civ-3d-014-adr.md:1`
  - `docs/traceability/fr-civ-3d-014/fr-civ-3d-014-adr.md:6`

#### FR-CIV-3D-015

- Tests:
  - `crates/engine/tests/fr_fr_civ_3d_015.rs:1`
  - `crates/engine/tests/fr_fr_civ_3d_015.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:2000`
  - `docs/traceability/fr-civ-3d-015/fr-civ-3d-015-adr.md:1`
  - `docs/traceability/fr-civ-3d-015/fr-civ-3d-015-adr.md:6`


### Epic `FR-CIV-ASSET` — 8 IDs

#### FR-CIV-ASSET-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_render_001.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:80`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2425`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2427`

#### FR-CIV-ASSET-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_render_002.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2447`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3207`
  - `docs/traceability/fr-civ-asset-003/fr-civ-asset-003-adr.md:1`

#### FR-CIV-ASSET-004

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_render_003.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2457`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3208`
  - `docs/traceability/fr-civ-asset-004/fr-civ-asset-004-adr.md:1`

#### FR-CIV-ASSET-005

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_nation_001.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2467`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3209`
  - `docs/traceability/fr-civ-asset-005/fr-civ-asset-005-adr.md:1`

#### FR-CIV-ASSET-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_render_004.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2477`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3210`
  - `docs/traceability/fr-civ-asset-006/fr-civ-asset-006-adr.md:1`

#### FR-CIV-ASSET-007

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_render_005.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2487`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3211`
  - `docs/traceability/fr-civ-asset-007/fr-civ-asset-007-adr.md:1`

#### FR-CIV-ASSET-016

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_nation_002.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2579`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3220`
  - `docs/traceability/fr-civ-asset-016/fr-civ-asset-016-adr.md:1`

#### FR-CIV-ASSET-018

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_zoom_001.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2599`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3222`
  - `docs/traceability/fr-civ-asset-018/fr-civ-asset-018-adr.md:1`


### Epic `FR-CIV-CULT` — 2 IDs

#### FR-CIV-CULT-001

- Tests:
  - `crates/agents/tests/fr_civ_cult_tests.rs:1`
  - `crates/agents/tests/fr_civ_cult_tests.rs:6`
  - `crates/agents/tests/fr_civ_cult_tests.rs:22`
- Spec/trace:
  - `agileplus-specs/civ-009-culture-diffusion/plan.md:5`
  - `agileplus-specs/civ-009-culture-diffusion/spec.md:24`
  - `agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:82`

#### FR-CIV-CULT-003

- Tests:
  - `crates/agents/tests/fr_civ_cult_tests.rs:1`
  - `crates/agents/tests/fr_civ_cult_tests.rs:8`
  - `crates/agents/tests/fr_civ_cult_tests.rs:224`
- Spec/trace:
  - `agileplus-specs/civ-009-culture-diffusion/plan.md:18`
  - `agileplus-specs/civ-009-culture-diffusion/spec.md:26`
  - `docs/reference/agileplus-artifacts-index.md:169`


### Epic `FR-CIV-FOG` — 5 IDs

#### FR-CIV-FOG-001

- Tests:
  - `crates/tactics/tests/fr_fr_civ_fog_001.rs:1`
  - `crates/tactics/tests/fr_fr_civ_fog_001.rs:6`
  - `crates/tactics/tests/fr_fr_civ_fog_001.rs:14`
- Spec/trace:
  - `agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:47`
  - `docs/traceability/fr-civ-fog-001/fr-civ-fog-001-adr.md:1`
  - `docs/traceability/fr-civ-fog-001/fr-civ-fog-001-adr.md:6`

#### FR-CIV-FOG-002

- Tests:
  - `crates/tactics/tests/fr_fr_civ_fog_002.rs:1`
  - `crates/tactics/tests/fr_fr_civ_fog_002.rs:6`
  - `crates/tactics/tests/fr_fr_civ_fog_002.rs:14`
- Spec/trace:
  - `agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:51`
  - `docs/traceability/fr-civ-fog-002/fr-civ-fog-002-adr.md:1`
  - `docs/traceability/fr-civ-fog-002/fr-civ-fog-002-adr.md:6`

#### FR-CIV-FOG-003

- Tests:
  - `crates/tactics/tests/fr_fr_civ_fog_003.rs:1`
  - `crates/tactics/tests/fr_fr_civ_fog_003.rs:6`
  - `crates/tactics/tests/fr_fr_civ_fog_003.rs:15`
- Spec/trace:
  - `agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:54`
  - `docs/traceability/fr-civ-fog-003/fr-civ-fog-003-adr.md:1`
  - `docs/traceability/fr-civ-fog-003/fr-civ-fog-003-adr.md:6`

#### FR-CIV-FOG-004

- Tests:
  - `crates/tactics/tests/fr_fr_civ_fog_004.rs:1`
  - `crates/tactics/tests/fr_fr_civ_fog_004.rs:6`
  - `crates/tactics/tests/fr_fr_civ_fog_004.rs:14`
- Spec/trace:
  - `agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:57`
  - `docs/traceability/fr-civ-fog-004/fr-civ-fog-004-adr.md:1`
  - `docs/traceability/fr-civ-fog-004/fr-civ-fog-004-adr.md:6`

#### FR-CIV-FOG-005

- Tests:
  - `crates/tactics/tests/fr_fr_civ_fog_005.rs:1`
  - `crates/tactics/tests/fr_fr_civ_fog_005.rs:6`
  - `crates/tactics/tests/fr_fr_civ_fog_005.rs:15`
- Spec/trace:
  - `agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:61`
  - `docs/traceability/fr-civ-fog-005/fr-civ-fog-005-adr.md:1`
  - `docs/traceability/fr-civ-fog-005/fr-civ-fog-005-adr.md:6`


### Epic `FR-CIV-LEGENDS-CAUSAL` — 1 IDs

#### FR-CIV-LEGENDS-CAUSAL-06

- Tests:
  - `crates/legends/tests/fr_fr_civ_legends_causal_06.rs:1`
  - `crates/legends/tests/fr_fr_civ_legends_causal_06.rs:10`
  - `crates/legends/tests/fr_fr_civ_legends_causal_06.rs:23`
- Spec/trace:
  - `docs/design/legends-engine.md:441`
  - `docs/traceability/fr-civ-legends-causal-06/fr-civ-legends-causal-06-adr.md:1`
  - `docs/traceability/fr-civ-legends-causal-06/fr-civ-legends-causal-06-adr.md:6`


### Epic `FR-CIV-LEGENDS-RESOLVE` — 1 IDs

#### FR-CIV-LEGENDS-RESOLVE-04

- Tests:
  - `crates/legends/tests/fr_fr_civ_legends_resolve_04.rs:1`
  - `crates/legends/tests/fr_fr_civ_legends_resolve_04.rs:10`
  - `crates/legends/tests/fr_fr_civ_legends_resolve_04.rs:26`
- Spec/trace:
  - `docs/design/legends-engine.md:439`
  - `docs/traceability/fr-civ-legends-resolve-04/fr-civ-legends-resolve-04-adr.md:1`
  - `docs/traceability/fr-civ-legends-resolve-04/fr-civ-legends-resolve-04-adr.md:6`


### Epic `FR-CIV-MOD` — 20 IDs

#### FR-CIV-MOD-000

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_000.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_000.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_000.rs:17`
- Spec/trace:
  - `docs/design/modding-platform.md:26`
  - `docs/design/modding-platform.md:89`
  - `docs/design/modding-platform.md:164`

#### FR-CIV-MOD-002

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_002.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_002.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_002.rs:21`
- Spec/trace:
  - `docs/design/modding-platform.md:28`
  - `docs/design/modding-platform.md:178`
  - `docs/specs/CIV-0700-modding-api-spec.md:2364`

#### FR-CIV-MOD-003

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_003.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_003.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_003.rs:19`
- Spec/trace:
  - `docs/design/modding-platform.md:29`
  - `docs/design/modding-platform.md:192`
  - `docs/specs/CIV-0700-modding-api-spec.md:2372`

#### FR-CIV-MOD-004

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_004.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_004.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_004.rs:19`
- Spec/trace:
  - `docs/design/modding-platform.md:30`
  - `docs/design/modding-platform.md:203`
  - `docs/specs/CIV-0700-modding-api-spec.md:2380`

#### FR-CIV-MOD-005

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_005.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_005.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_005.rs:17`
- Spec/trace:
  - `docs/design/modding-platform.md:31`
  - `docs/design/modding-platform.md:214`
  - `docs/specs/CIV-0700-modding-api-spec.md:2388`

#### FR-CIV-MOD-006

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_006.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_006.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_006.rs:20`
- Spec/trace:
  - `docs/design/modding-platform.md:32`
  - `docs/design/modding-platform.md:226`
  - `docs/specs/CIV-0700-modding-api-spec.md:2396`

#### FR-CIV-MOD-007

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_007.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_007.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_007.rs:15`
- Spec/trace:
  - `docs/design/modding-platform.md:33`
  - `docs/design/modding-platform.md:234`
  - `docs/specs/CIV-0700-modding-api-spec.md:2404`

#### FR-CIV-MOD-008

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_008.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_008.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_008.rs:14`
- Spec/trace:
  - `docs/design/modding-platform.md:34`
  - `docs/design/modding-platform.md:249`
  - `docs/specs/CIV-0700-modding-api-spec.md:2412`

#### FR-CIV-MOD-009

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_009.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_009.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_009.rs:14`
- Spec/trace:
  - `docs/design/modding-platform.md:35`
  - `docs/design/modding-platform.md:73`
  - `docs/design/modding-platform.md:260`

#### FR-CIV-MOD-010

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_010.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_010.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_010.rs:17`
- Spec/trace:
  - `docs/design/modding-platform.md:36`
  - `docs/design/modding-platform.md:287`
  - `docs/specs/CIV-0700-modding-api-spec.md:2428`

#### FR-CIV-MOD-011

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_011.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_011.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_011.rs:16`
- Spec/trace:
  - `docs/design/modding-platform.md:37`
  - `docs/design/modding-platform.md:120`
  - `docs/design/modding-platform.md:299`

#### FR-CIV-MOD-012

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_012.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_012.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_012.rs:17`
- Spec/trace:
  - `docs/design/modding-platform.md:38`
  - `docs/design/modding-platform.md:307`
  - `docs/specs/CIV-0700-modding-api-spec.md:2444`

#### FR-CIV-MOD-013

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_013.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_013.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_013.rs:15`
- Spec/trace:
  - `docs/design/modding-platform.md:39`
  - `docs/design/modding-platform.md:320`
  - `docs/specs/CIV-0700-modding-api-spec.md:2452`

#### FR-CIV-MOD-014

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_014.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_014.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_014.rs:17`
- Spec/trace:
  - `docs/design/modding-platform.md:40`
  - `docs/design/modding-platform.md:355`
  - `docs/specs/CIV-0700-modding-api-spec.md:2460`

#### FR-CIV-MOD-015

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_015.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_015.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_015.rs:19`
- Spec/trace:
  - `docs/design/modding-platform.md:41`
  - `docs/design/modding-platform.md:375`
  - `docs/specs/CIV-0700-modding-api-spec.md:2468`

#### FR-CIV-MOD-016

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_016.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_016.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_016.rs:18`
- Spec/trace:
  - `docs/design/modding-platform.md:42`
  - `docs/design/modding-platform.md:398`
  - `docs/traceability/fr-civ-mod-016/fr-civ-mod-016-adr.md:1`

#### FR-CIV-MOD-017

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_017.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_017.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_017.rs:20`
- Spec/trace:
  - `docs/design/modding-platform.md:43`
  - `docs/design/modding-platform.md:419`
  - `docs/traceability/fr-civ-mod-017/fr-civ-mod-017-adr.md:1`

#### FR-CIV-MOD-018

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_018.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_018.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_018.rs:30`
- Spec/trace:
  - `docs/design/modding-platform.md:44`
  - `docs/design/modding-platform.md:457`
  - `docs/traceability/fr-civ-mod-018/fr-civ-mod-018-adr.md:1`

#### FR-CIV-MOD-019

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_019.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_019.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_019.rs:21`
- Spec/trace:
  - `docs/design/modding-platform.md:45`
  - `docs/design/modding-platform.md:486`
  - `docs/traceability/fr-civ-mod-019/fr-civ-mod-019-adr.md:1`

#### FR-CIV-MOD-020

- Tests:
  - `crates/mod-host/tests/fr_fr_civ_mod_020.rs:1`
  - `crates/mod-host/tests/fr_fr_civ_mod_020.rs:8`
  - `crates/mod-host/tests/fr_fr_civ_mod_020.rs:14`
- Spec/trace:
  - `docs/design/modding-platform.md:46`
  - `docs/design/modding-platform.md:500`
  - `docs/traceability/fr-civ-mod-020/fr-civ-mod-020-adr.md:1`


### Epic `FR-CIV-PROTO` — 14 IDs

#### FR-CIV-PROTO-002

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_002.rs:1`
- Spec/trace:
  - `docs/AGILE_WORKSTREAM.md:266`
  - `docs/AGILE_WORKSTREAM.md:296`
  - `docs/AGILE_WORKSTREAM.md:301`

#### FR-CIV-PROTO-003

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_003.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1134`
  - `docs/traceability/fr-civ-proto-003/fr-civ-proto-003-adr.md:1`
  - `docs/traceability/fr-civ-proto-003/fr-civ-proto-003-adr.md:6`

#### FR-CIV-PROTO-004

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_004.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1139`
  - `docs/traceability/fr-civ-proto-004/fr-civ-proto-004-adr.md:1`
  - `docs/traceability/fr-civ-proto-004/fr-civ-proto-004-adr.md:6`

#### FR-CIV-PROTO-005

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_005.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1144`
  - `docs/traceability/fr-civ-proto-005/fr-civ-proto-005-adr.md:1`
  - `docs/traceability/fr-civ-proto-005/fr-civ-proto-005-adr.md:6`

#### FR-CIV-PROTO-006

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_006.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1149`
  - `docs/traceability/fr-civ-proto-006/fr-civ-proto-006-adr.md:1`
  - `docs/traceability/fr-civ-proto-006/fr-civ-proto-006-adr.md:6`

#### FR-CIV-PROTO-007

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_007.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1154`
  - `docs/traceability/fr-civ-proto-007/fr-civ-proto-007-adr.md:1`
  - `docs/traceability/fr-civ-proto-007/fr-civ-proto-007-adr.md:6`

#### FR-CIV-PROTO-008

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_008.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1159`
  - `docs/traceability/fr-civ-proto-008/fr-civ-proto-008-adr.md:1`
  - `docs/traceability/fr-civ-proto-008/fr-civ-proto-008-adr.md:6`

#### FR-CIV-PROTO-009

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_009.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1164`
  - `docs/traceability/fr-civ-proto-009/fr-civ-proto-009-adr.md:1`
  - `docs/traceability/fr-civ-proto-009/fr-civ-proto-009-adr.md:6`

#### FR-CIV-PROTO-010

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_010.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1169`
  - `docs/traceability/fr-civ-proto-010/fr-civ-proto-010-adr.md:1`
  - `docs/traceability/fr-civ-proto-010/fr-civ-proto-010-adr.md:6`

#### FR-CIV-PROTO-011

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_011.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1174`
  - `docs/traceability/fr-civ-proto-011/fr-civ-proto-011-adr.md:1`
  - `docs/traceability/fr-civ-proto-011/fr-civ-proto-011-adr.md:6`

#### FR-CIV-PROTO-012

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_012.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1179`
  - `docs/traceability/fr-civ-proto-012/fr-civ-proto-012-adr.md:1`
  - `docs/traceability/fr-civ-proto-012/fr-civ-proto-012-adr.md:6`

#### FR-CIV-PROTO-013

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_013.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1184`
  - `docs/traceability/fr-civ-proto-013/fr-civ-proto-013-adr.md:1`
  - `docs/traceability/fr-civ-proto-013/fr-civ-proto-013-adr.md:6`

#### FR-CIV-PROTO-014

- Tests:
  - `crates/protocol-3d/tests/fr_fr_civ_proto_014.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1189`
  - `docs/traceability/fr-civ-proto-014/fr-civ-proto-014-adr.md:1`
  - `docs/traceability/fr-civ-proto-014/fr-civ-proto-014-adr.md:6`

#### FR-CIV-PROTO-015

- Tests:
  - `crates/protocol-3d/tests/fr_civ_proto_tests.rs:3`
  - `crates/protocol-3d/tests/fr_civ_proto_tests.rs:17`
  - `crates/protocol-3d/tests/fr_fr_civ_proto_015.rs:1`
- Spec/trace:
  - `docs/specs/CIV-0200-client-protocol.md:1194`
  - `PRD.md:307`
  - `docs/traceability/fr-civ-proto-015/fr-civ-proto-015-adr.md:1`


### Epic `FR-CIV-RESEARCH-003-EXPORT` — 1 IDs

#### FR-CIV-RESEARCH-003-EXPORT

- Tests:
  - `crates/research/tests/fr_civ_research_tests.rs:3`
  - `crates/research/tests/fr_civ_research_tests.rs:34`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:220`
  - `PLAN.md:237`
  - `PLAN.md:238`


### Epic `FR-CIV-SERVER-001-WS` — 1 IDs

#### FR-CIV-SERVER-001-WS

- Tests:
  - `crates/server/tests/fr_civ_server_tests.rs:3`
  - `crates/server/tests/fr_civ_server_tests.rs:18`
  - `crates/server/tests/fr_fr_civ_server_001_ws.rs:1`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:221`
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:222`
  - `PLAN.md:174`


### Epic `FR-CIV-WAR` — 9 IDs

#### FR-CIV-WAR-011

- Tests:
  - `crates/tactics/tests/fr_fr_civ_war_011.rs:1`
  - `crates/tactics/tests/fr_fr_civ_war_011.rs:6`
  - `crates/tactics/tests/fr_fr_civ_war_011.rs:14`
- Spec/trace:
  - `docs/design/warfare.md:83`
  - `docs/design/warfare.md:192`
  - `docs/traceability/fr-civ-war-011/fr-civ-war-011-adr.md:1`

#### FR-CIV-WAR-012

- Tests:
  - `crates/tactics/tests/fr_fr_civ_war_012.rs:1`
  - `crates/tactics/tests/fr_fr_civ_war_012.rs:6`
  - `crates/tactics/tests/fr_fr_civ_war_012.rs:14`
- Spec/trace:
  - `docs/design/warfare.md:86`
  - `docs/design/warfare.md:193`
  - `docs/traceability/fr-civ-war-012/fr-civ-war-012-adr.md:1`

#### FR-CIV-WAR-013

- Tests:
  - `crates/tactics/tests/fr_fr_civ_war_013.rs:1`
  - `crates/tactics/tests/fr_fr_civ_war_013.rs:6`
  - `crates/tactics/tests/fr_fr_civ_war_013.rs:15`
- Spec/trace:
  - `docs/design/warfare.md:89`
  - `docs/design/warfare.md:194`
  - `docs/traceability/fr-civ-war-013/fr-civ-war-013-adr.md:1`

#### FR-CIV-WAR-021

- Tests:
  - `crates/tactics/tests/fr_fr_civ_war_021.rs:1`
  - `crates/tactics/tests/fr_fr_civ_war_021.rs:6`
  - `crates/tactics/tests/fr_fr_civ_war_021.rs:15`
- Spec/trace:
  - `docs/design/warfare.md:111`
  - `docs/design/warfare.md:196`
  - `docs/traceability/fr-civ-war-021/fr-civ-war-021-adr.md:1`

#### FR-CIV-WAR-022

- Tests:
  - `crates/tactics/tests/fr_fr_civ_war_022.rs:1`
  - `crates/tactics/tests/fr_fr_civ_war_022.rs:6`
  - `crates/tactics/tests/fr_fr_civ_war_022.rs:16`
- Spec/trace:
  - `docs/design/warfare.md:114`
  - `docs/design/warfare.md:197`
  - `docs/traceability/fr-civ-war-022/fr-civ-war-022-adr.md:1`

#### FR-CIV-WAR-030

- Tests:
  - `crates/tactics/tests/fr_fr_civ_war_030.rs:1`
  - `crates/tactics/tests/fr_fr_civ_war_030.rs:6`
  - `crates/tactics/tests/fr_fr_civ_war_030.rs:15`
- Spec/trace:
  - `docs/design/warfare.md:124`
  - `docs/design/warfare.md:198`
  - `docs/traceability/fr-civ-war-030/fr-civ-war-030-adr.md:1`

#### FR-CIV-WAR-040

- Tests:
  - `crates/tactics/tests/fr_fr_civ_war_040.rs:1`
  - `crates/tactics/tests/fr_fr_civ_war_040.rs:6`
  - `crates/tactics/tests/fr_fr_civ_war_040.rs:15`
- Spec/trace:
  - `docs/design/warfare.md:144`
  - `docs/design/warfare.md:199`
  - `docs/traceability/fr-civ-war-040/fr-civ-war-040-adr.md:1`

#### FR-CIV-WAR-041

- Tests:
  - `crates/tactics/tests/fr_fr_civ_war_041.rs:1`
  - `crates/tactics/tests/fr_fr_civ_war_041.rs:6`
  - `crates/tactics/tests/fr_fr_civ_war_041.rs:15`
- Spec/trace:
  - `docs/design/warfare.md:147`
  - `docs/design/warfare.md:200`
  - `docs/traceability/fr-civ-war-041/fr-civ-war-041-adr.md:1`

#### FR-CIV-WAR-042

- Tests:
  - `crates/tactics/tests/fr_fr_civ_war_042.rs:1`
  - `crates/tactics/tests/fr_fr_civ_war_042.rs:6`
  - `crates/tactics/tests/fr_fr_civ_war_042.rs:15`
- Spec/trace:
  - `docs/design/warfare.md:150`
  - `docs/design/warfare.md:201`
  - `docs/traceability/fr-civ-war-042/fr-civ-war-042-adr.md:1`


### Epic `FR-GUARD` — 2 IDs

#### FR-GUARD-001

- Tests:
  - `crates/engine/tests/fr_fr_guard_001.rs:1`
  - `crates/engine/tests/fr_fr_guard_001.rs:8`
  - `crates/engine/tests/fr_fr_guard_001.rs:15`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1260`
  - `docs/traceability/fr-guard-001/fr-guard-001-adr.md:1`
  - `docs/traceability/fr-guard-001/fr-guard-001-adr.md:6`

#### FR-GUARD-002

- Tests:
  - `crates/engine/tests/fr_fr_guard_002.rs:1`
  - `crates/engine/tests/fr_fr_guard_002.rs:8`
  - `crates/engine/tests/fr_fr_guard_002.rs:16`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1337`
  - `docs/traceability/fr-guard-002/fr-guard-002-adr.md:1`
  - `docs/traceability/fr-guard-002/fr-guard-002-adr.md:6`


### Epic `FR-PROTO` — 5 IDs

#### FR-PROTO-001

- Tests:
  - `crates/server/tests/ws_smoke.rs:1687`
  - `crates/server/tests/ws_smoke.rs:1688`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `agileplus-specs/civ-010-multi-client-protocol/spec.md:25`
  - `agileplus-specs/civ-014-terrain-playable-hardening/spec.md:67`

#### FR-PROTO-002

- Tests:
  - `crates/engine/tests/fr_fr_proto_002.rs:1`
  - `crates/engine/tests/fr_fr_proto_002.rs:5`
  - `crates/engine/tests/fr_fr_proto_002.rs:9`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `agileplus-specs/civ-010-multi-client-protocol/spec.md:26`
  - `agileplus-specs/civ-015-tactics-fog-of-war-and-combat-pipeline/spec.md:81`

#### FR-PROTO-003

- Tests:
  - `crates/engine/tests/fr_fr_proto_003.rs:1`
  - `crates/engine/tests/fr_fr_proto_003.rs:5`
  - `crates/engine/tests/fr_fr_proto_003.rs:9`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `agileplus-specs/civ-010-multi-client-protocol/spec.md:27`
  - `agileplus-specs/civ-011-bevy-primary-client/spec.md:42`

#### FR-PROTO-004

- Tests:
  - `crates/engine/tests/fr_fr_proto_004.rs:1`
  - `crates/engine/tests/fr_fr_proto_004.rs:5`
  - `crates/engine/tests/fr_fr_proto_004.rs:9`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `agileplus-specs/civ-010-multi-client-protocol/spec.md:28`
  - `agileplus-specs/civ-011-bevy-primary-client/spec.md:43`

#### FR-PROTO-005

- Tests:
  - `crates/engine/tests/fr_fr_proto_005.rs:1`
  - `crates/engine/tests/fr_fr_proto_005.rs:5`
  - `crates/engine/tests/fr_fr_proto_005.rs:9`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `agileplus-specs/civ-010-multi-client-protocol/spec.md:29`
  - `docs/reference/agileplus-artifacts-index.md:185`


### Epic `FR-SOC-DET` — 2 IDs

#### FR-SOC-DET-001

- Tests:
  - `crates/engine/tests/fr_fr_soc_det_001.rs:1`
  - `crates/engine/tests/fr_fr_soc_det_001.rs:5`
  - `crates/engine/tests/fr_fr_soc_det_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1495`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1498`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1984`

#### FR-SOC-DET-002

- Tests:
  - `crates/engine/tests/fr_fr_soc_det_002.rs:1`
  - `crates/engine/tests/fr_fr_soc_det_002.rs:5`
  - `crates/engine/tests/fr_fr_soc_det_002.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1516`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1985`
  - `docs/traceability/fr-soc-det-002/fr-soc-det-002-adr.md:1`


### Epic `FR-STOR` — 1 IDs

#### FR-STOR-001

- Tests:
  - `crates/engine/tests/fr_fr_stor_001.rs:1`
  - `crates/engine/tests/fr_fr_stor_001.rs:5`
  - `crates/engine/tests/fr_fr_stor_001.rs:9`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1931`
  - `docs/traceability/fr-stor-001/fr-stor-001-adr.md:1`
  - `docs/traceability/fr-stor-001/fr-stor-001-adr.md:6`


### Epic `NFR-CIV-SCALE` — 3 IDs

#### NFR-CIV-SCALE-001

- Tests:
  - `crates/protocol-3d/tests/fr_perf_005_frame3d_timing.rs:85`
- Spec/trace:
  - `docs/reference/non-functional-requirements.md:82`
  - `docs/reference/non-functional-requirements.md:188`
  - `docs/reference/non-functional-requirements.md:562`

#### NFR-CIV-SCALE-002

- Tests:
  - `crates/voxel/tests/fr_civ_render_001_chunk_stream_radius.rs:6`
- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:96`
  - `docs/guides/voxel-emergent-vision-and-migration.md:152`
  - `docs/reference/non-functional-requirements.md:110`

#### NFR-CIV-SCALE-901

- Tests:
  - `crates/voxel/tests/fr_nfr_civ_scale_901.rs:1`
- Spec/trace:
  - `docs/agileplus/epics/civ-w5-scale.md:10`
  - `docs/agileplus/epics/civ-w5-scale.md:23`
  - `docs/agileplus/README.md:24`

