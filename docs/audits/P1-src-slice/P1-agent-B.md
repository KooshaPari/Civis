# P1 agent-B

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


## Your slice: 60 IDs in 16 epics


### Epic `FR-CIV` — 9 IDs

#### FR-CIV-0001

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:106`
  - `crates/build/tests/fr_matrix_batch12.rs:109`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:212`
  - `docs/guides/GIT_WORKTREE_GUIDE.md:151`
  - `PLAN.md:16`

#### FR-CIV-0104-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_0104_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_0104_001.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1454`
  - `docs/traceability/fr-civ-0104-001/fr-civ-0104-001-adr.md:1`
  - `docs/traceability/fr-civ-0104-001/fr-civ-0104-001-adr.md:6`

#### FR-CIV-0104-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_0104_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_0104_002.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1459`
  - `docs/traceability/fr-civ-0104-002/fr-civ-0104-002-adr.md:1`
  - `docs/traceability/fr-civ-0104-002/fr-civ-0104-002-adr.md:6`

#### FR-CIV-0104-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_0104_003.rs:1`
  - `crates/engine/tests/fr_fr_civ_0104_003.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1464`
  - `docs/traceability/fr-civ-0104-003/fr-civ-0104-003-adr.md:1`
  - `docs/traceability/fr-civ-0104-003/fr-civ-0104-003-adr.md:6`

#### FR-CIV-0104-005

- Tests:
  - `crates/engine/tests/fr_fr_civ_0104_005.rs:1`
  - `crates/engine/tests/fr_fr_civ_0104_005.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1474`
  - `docs/traceability/fr-civ-0104-005/fr-civ-0104-005-adr.md:1`
  - `docs/traceability/fr-civ-0104-005/fr-civ-0104-005-adr.md:6`

#### FR-CIV-0104-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_0104_006.rs:1`
  - `crates/engine/tests/fr_fr_civ_0104_006.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1479`
  - `docs/traceability/fr-civ-0104-006/fr-civ-0104-006-adr.md:1`
  - `docs/traceability/fr-civ-0104-006/fr-civ-0104-006-adr.md:6`

#### FR-CIV-0104-007

- Tests:
  - `crates/engine/tests/fr_fr_civ_0104_007.rs:1`
  - `crates/engine/tests/fr_fr_civ_0104_007.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1484`
  - `docs/traceability/fr-civ-0104-007/fr-civ-0104-007-adr.md:1`
  - `docs/traceability/fr-civ-0104-007/fr-civ-0104-007-adr.md:6`

#### FR-CIV-0104-009

- Tests:
  - `crates/engine/tests/fr_fr_civ_0104_009.rs:1`
  - `crates/engine/tests/fr_fr_civ_0104_009.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1494`
  - `docs/traceability/fr-civ-0104-009/fr-civ-0104-009-adr.md:1`
  - `docs/traceability/fr-civ-0104-009/fr-civ-0104-009-adr.md:6`

#### FR-CIV-0104-010

- Tests:
  - `crates/engine/tests/fr_fr_civ_0104_010.rs:1`
  - `crates/engine/tests/fr_fr_civ_0104_010.rs:6`
- Spec/trace:
  - `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1499`
  - `docs/traceability/fr-civ-0104-010/fr-civ-0104-010-adr.md:1`
  - `docs/traceability/fr-civ-0104-010/fr-civ-0104-010-adr.md:6`


### Epic `FR-CIV-ARCH` — 1 IDs

#### FR-CIV-ARCH-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_arch_006.rs:1`
  - `crates/engine/tests/fr_fr_civ_arch_006.rs:9`
  - `crates/engine/tests/fr_fr_civ_arch_006.rs:18`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-arch-006/fr-civ-arch-006-adr.md:1`
  - `docs/traceability/fr-civ-arch-006/fr-civ-arch-006-adr.md:6`


### Epic `FR-CIV-CORE` — 19 IDs

#### FR-CIV-CORE-001

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:723`
  - `crates/build/tests/fr_matrix_batch12.rs:726`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:212`
  - `docs/AGILE_WORKSTREAM.md:372`
  - `docs/AGILE_WORKSTREAM.md:444`

#### FR-CIV-CORE-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_002.rs:7`
  - `crates/engine/tests/fr_fr_civ_core_002.rs:14`
- Spec/trace:
  - `docs/AGILE_WORKSTREAM.md:445`
  - `docs/AGILE_WORKSTREAM.md:455`
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:1368`

#### FR-CIV-CORE-003

- Tests:
  - `crates/engine/tests/fr_core_cluster.rs:1`
  - `crates/engine/tests/fr_core_cluster.rs:7`
  - `crates/engine/tests/fr_core_cluster.rs:41`
- Spec/trace:
  - `docs/AGILE_WORKSTREAM.md:446`
  - `docs/AGILE_WORKSTREAM.md:455`
  - `docs/reference/CODE_ENTITY_MAP.md:17`

#### FR-CIV-CORE-004

- Tests:
  - `crates/engine/tests/fr_core_cluster.rs:25`
  - `crates/engine/tests/fr_fr_civ_core_004.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_004.rs:5`
- Spec/trace:
  - `docs/reference/CODE_ENTITY_MAP.md:9`
  - `docs/reference/FR_TRACKER.md:49`
  - `docs/specs/CIV-0001-core-simulation-loop.md:882`

#### FR-CIV-CORE-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_006.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_006.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0001-core-simulation-loop.md:892`
  - `docs/traceability/fr-civ-core-006/fr-civ-core-006-adr.md:1`
  - `docs/traceability/fr-civ-core-006/fr-civ-core-006-adr.md:6`

#### FR-CIV-CORE-007

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_007.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_007.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0001-core-simulation-loop.md:897`
  - `docs/traceability/fr-civ-core-007/fr-civ-core-007-adr.md:1`
  - `docs/traceability/fr-civ-core-007/fr-civ-core-007-adr.md:6`

#### FR-CIV-CORE-008

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_008.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_008.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0001-core-simulation-loop.md:902`
  - `docs/traceability/fr-civ-core-008/fr-civ-core-008-adr.md:1`
  - `docs/traceability/fr-civ-core-008/fr-civ-core-008-adr.md:6`

#### FR-CIV-CORE-009

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_009.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_009.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0001-core-simulation-loop.md:907`
  - `docs/traceability/fr-civ-core-009/fr-civ-core-009-adr.md:1`
  - `docs/traceability/fr-civ-core-009/fr-civ-core-009-adr.md:6`

#### FR-CIV-CORE-010

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_010.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_010.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0001-core-simulation-loop.md:912`
  - `docs/traceability/fr-civ-core-010/fr-civ-core-010-adr.md:1`
  - `docs/traceability/fr-civ-core-010/fr-civ-core-010-adr.md:6`

#### FR-CIV-CORE-011

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_011.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_011.rs:5`
- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:1369`
  - `docs/specs/CIV-0001-core-simulation-loop.md:917`
  - `docs/traceability/fr-civ-core-011/fr-civ-core-011-adr.md:1`

#### FR-CIV-CORE-012

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_012.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_012.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0001-core-simulation-loop.md:922`
  - `docs/traceability/fr-civ-core-012/fr-civ-core-012-adr.md:1`
  - `docs/traceability/fr-civ-core-012/fr-civ-core-012-adr.md:6`

#### FR-CIV-CORE-013

- Tests:
  - `crates/engine/tests/fr_core_cluster.rs:12`
  - `crates/engine/tests/fr_core_cluster.rs:250`
  - `crates/engine/tests/fr_core_cluster.rs:253`
- Spec/trace:
  - `docs/AGILE_WORKSTREAM.md:447`
  - `docs/AGILE_WORKSTREAM.md:455`
  - `docs/specs/CIV-0001-core-simulation-loop.md:927`

#### FR-CIV-CORE-014

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_014.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_014.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0001-core-simulation-loop.md:932`
  - `docs/traceability/fr-civ-core-014/fr-civ-core-014-adr.md:1`
  - `docs/traceability/fr-civ-core-014/fr-civ-core-014-adr.md:6`

#### FR-CIV-CORE-015

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_015.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_015.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0001-core-simulation-loop.md:937`
  - `docs/traceability/fr-civ-core-015/fr-civ-core-015-adr.md:1`
  - `docs/traceability/fr-civ-core-015/fr-civ-core-015-adr.md:6`

#### FR-CIV-CORE-016

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_016.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_016.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0001-core-simulation-loop.md:942`
  - `docs/traceability/fr-civ-core-016/fr-civ-core-016-adr.md:1`
  - `docs/traceability/fr-civ-core-016/fr-civ-core-016-adr.md:6`

#### FR-CIV-CORE-017

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_017.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_017.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0001-core-simulation-loop.md:947`
  - `docs/traceability/fr-civ-core-017/fr-civ-core-017-adr.md:1`
  - `docs/traceability/fr-civ-core-017/fr-civ-core-017-adr.md:6`

#### FR-CIV-CORE-018

- Tests:
  - `crates/engine/tests/fr_fr_civ_core_018.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_018.rs:5`
- Spec/trace:
  - `docs/specs/CIV-0001-core-simulation-loop.md:952`
  - `docs/traceability/fr-civ-core-018/fr-civ-core-018-adr.md:1`
  - `docs/traceability/fr-civ-core-018/fr-civ-core-018-adr.md:6`

#### FR-CIV-CORE-019

- Tests:
  - `crates/engine/tests/fr_core_cluster.rs:26`
  - `crates/engine/tests/fr_fr_civ_core_019.rs:1`
  - `crates/engine/tests/fr_fr_civ_core_019.rs:5`
- Spec/trace:
  - `docs/AGILE_WORKSTREAM.md:196`
  - `docs/AGILE_WORKSTREAM.md:246`
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2103`

#### FR-CIV-CORE-020

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:738`
  - `crates/build/tests/fr_matrix_batch12.rs:741`
- Spec/trace:
  - `docs/specs/CIV-0001-core-simulation-loop.md:962`
  - `PRD.md:263`
  - `docs/traceability/fr-civ-core-020/fr-civ-core-020-adr.md:1`


### Epic `FR-CIV-EMERG` — 2 IDs

#### FR-CIV-EMERG-004

- Tests:
  - `crates/civ-emergence-metrics/tests/fr_fr_civ_emerg_004.rs:1`
- Spec/trace:
  - `agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:51`
  - `docs/traceability/fr-civ-emerg-004/fr-civ-emerg-004-adr.md:1`
  - `docs/traceability/fr-civ-emerg-004/fr-civ-emerg-004-adr.md:6`

#### FR-CIV-EMERG-005

- Tests:
  - `crates/civ-emergence-metrics/tests/fr_fr_civ_emerg_005.rs:1`
- Spec/trace:
  - `agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:55`
  - `docs/traceability/fr-civ-emerg-005/fr-civ-emerg-005-adr.md:1`
  - `docs/traceability/fr-civ-emerg-005/fr-civ-emerg-005-adr.md:6`


### Epic `FR-CIV-LANG` — 5 IDs

#### FR-CIV-LANG-004

- Tests:
  - `crates/engine/tests/fr_fr_civ_lang_004.rs:1`
  - `crates/engine/tests/fr_fr_civ_lang_004.rs:8`
  - `crates/engine/tests/fr_fr_civ_lang_004.rs:16`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-lang-004/fr-civ-lang-004-adr.md:1`
  - `docs/traceability/fr-civ-lang-004/fr-civ-lang-004-adr.md:6`

#### FR-CIV-LANG-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_lang_006.rs:1`
  - `crates/engine/tests/fr_fr_civ_lang_006.rs:8`
  - `crates/engine/tests/fr_fr_civ_lang_006.rs:22`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-lang-006/fr-civ-lang-006-adr.md:1`
  - `docs/traceability/fr-civ-lang-006/fr-civ-lang-006-adr.md:6`

#### FR-CIV-LANG-007

- Tests:
  - `crates/engine/tests/fr_fr_civ_lang_007.rs:1`
  - `crates/engine/tests/fr_fr_civ_lang_007.rs:8`
  - `crates/engine/tests/fr_fr_civ_lang_007.rs:17`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-lang-007/fr-civ-lang-007-adr.md:1`
  - `docs/traceability/fr-civ-lang-007/fr-civ-lang-007-adr.md:6`

#### FR-CIV-LANG-008

- Tests:
  - `crates/engine/tests/fr_fr_civ_lang_008.rs:1`
  - `crates/engine/tests/fr_fr_civ_lang_008.rs:10`
  - `crates/engine/tests/fr_fr_civ_lang_008.rs:24`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-lang-008/fr-civ-lang-008-adr.md:1`
  - `docs/traceability/fr-civ-lang-008/fr-civ-lang-008-adr.md:6`

#### FR-CIV-LANG-010

- Tests:
  - `crates/engine/tests/fr_fr_civ_lang_010.rs:1`
  - `crates/engine/tests/fr_fr_civ_lang_010.rs:11`
  - `crates/engine/tests/fr_fr_civ_lang_010.rs:21`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-lang-010/fr-civ-lang-010-adr.md:1`
  - `docs/traceability/fr-civ-lang-010/fr-civ-lang-010-adr.md:6`


### Epic `FR-CIV-LEGENDS-PRESIM` — 1 IDs

#### FR-CIV-LEGENDS-PRESIM-10

- Tests:
  - `crates/legends/tests/fr_fr_civ_legends_presim_10.rs:1`
  - `crates/legends/tests/fr_fr_civ_legends_presim_10.rs:10`
  - `crates/legends/tests/fr_fr_civ_legends_presim_10.rs:20`
- Spec/trace:
  - `docs/design/legends-engine.md:445`
  - `docs/traceability/fr-civ-legends-presim-10/fr-civ-legends-presim-10-adr.md:1`
  - `docs/traceability/fr-civ-legends-presim-10/fr-civ-legends-presim-10-adr.md:6`


### Epic `FR-CIV-METRICS` — 1 IDs

#### FR-CIV-METRICS-001

- Tests:
  - `crates/engine/tests/fr_engine_metrics_replay_tests.rs:3`
  - `crates/engine/tests/fr_engine_metrics_replay_tests.rs:69`
  - `crates/engine/tests/fr_engine_metrics_replay_tests.rs:72`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:216`
  - `PLAN.md:151`
  - `PLAN.md:152`


### Epic `FR-CIV-PERF-WEB` — 1 IDs

#### FR-CIV-PERF-WEB-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_perf_web_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_perf_web_001.rs:5`
  - `crates/engine/tests/fr_fr_civ_perf_web_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3219`
  - `docs/traceability/fr-civ-perf-web-001/fr-civ-perf-web-001-adr.md:1`
  - `docs/traceability/fr-civ-perf-web-001/fr-civ-perf-web-001-adr.md:6`


### Epic `FR-CIV-RESEARCH-001-SCENARIO` — 1 IDs

#### FR-CIV-RESEARCH-001-SCENARIO

- Tests:
  - `crates/research/tests/fr_civ_research_tests.rs:3`
  - `crates/research/tests/fr_civ_research_tests.rs:7`
  - `crates/research/tests/fr_civ_research_tests.rs:50`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:218`
  - `PLAN.md:233`
  - `PLAN.md:234`


### Epic `FR-CIV-RTS-ZOOM` — 1 IDs

#### FR-CIV-RTS-ZOOM-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_rts_zoom_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_rts_zoom_001.rs:4`
  - `crates/engine/tests/fr_fr_civ_rts_zoom_001.rs:11`
- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3222`
  - `docs/traceability/fr-civ-rts-zoom-001/fr-civ-rts-zoom-001-adr.md:1`
  - `docs/traceability/fr-civ-rts-zoom-001/fr-civ-rts-zoom-001-adr.md:6`


### Epic `FR-CIV-VERIFY` — 10 IDs

#### FR-CIV-VERIFY-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_verify_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_verify_001.rs:7`
- Spec/trace:
  - `agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:34`
  - `agileplus-specs/civ-017-civis-mcp-server/spec.md:70`
  - `docs/traceability/fr-civ-verify-001/fr-civ-verify-001-adr.md:1`

#### FR-CIV-VERIFY-002

- Tests:
  - `crates/engine/tests/fr_fr_civ_verify_002.rs:1`
  - `crates/engine/tests/fr_fr_civ_verify_002.rs:7`
- Spec/trace:
  - `agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:38`
  - `docs/traceability/fr-civ-verify-002/fr-civ-verify-002-adr.md:1`
  - `docs/traceability/fr-civ-verify-002/fr-civ-verify-002-adr.md:6`

#### FR-CIV-VERIFY-003

- Tests:
  - `crates/engine/tests/fr_fr_civ_verify_003.rs:1`
  - `crates/engine/tests/fr_fr_civ_verify_003.rs:7`
- Spec/trace:
  - `agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:41`
  - `docs/traceability/fr-civ-verify-003/fr-civ-verify-003-adr.md:1`
  - `docs/traceability/fr-civ-verify-003/fr-civ-verify-003-adr.md:6`

#### FR-CIV-VERIFY-004

- Tests:
  - `crates/engine/tests/fr_fr_civ_verify_004.rs:1`
  - `crates/engine/tests/fr_fr_civ_verify_004.rs:7`
- Spec/trace:
  - `agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:44`
  - `docs/traceability/fr-civ-verify-004/fr-civ-verify-004-adr.md:1`
  - `docs/traceability/fr-civ-verify-004/fr-civ-verify-004-adr.md:6`

#### FR-CIV-VERIFY-005

- Tests:
  - `crates/engine/tests/fr_fr_civ_verify_005.rs:1`
  - `crates/engine/tests/fr_fr_civ_verify_005.rs:7`
- Spec/trace:
  - `agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:47`
  - `docs/traceability/fr-civ-verify-005/fr-civ-verify-005-adr.md:1`
  - `docs/traceability/fr-civ-verify-005/fr-civ-verify-005-adr.md:6`

#### FR-CIV-VERIFY-006

- Tests:
  - `crates/engine/tests/fr_fr_civ_verify_006.rs:1`
  - `crates/engine/tests/fr_fr_civ_verify_006.rs:7`
- Spec/trace:
  - `agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:50`
  - `docs/traceability/fr-civ-verify-006/fr-civ-verify-006-adr.md:1`
  - `docs/traceability/fr-civ-verify-006/fr-civ-verify-006-adr.md:6`

#### FR-CIV-VERIFY-007

- Tests:
  - `crates/engine/tests/fr_fr_civ_verify_007.rs:1`
  - `crates/engine/tests/fr_fr_civ_verify_007.rs:7`
- Spec/trace:
  - `agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:54`
  - `agileplus-specs/civ-018-verify-harness-extension/spec.md:31`
  - `agileplus-specs/civ-018-verify-harness-extension/spec.md:42`

#### FR-CIV-VERIFY-008

- Tests:
  - `crates/engine/tests/fr_fr_civ_verify_008.rs:1`
  - `crates/engine/tests/fr_fr_civ_verify_008.rs:7`
- Spec/trace:
  - `agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:61`
  - `agileplus-specs/civ-018-verify-harness-extension/spec.md:30`
  - `agileplus-specs/civ-018-verify-harness-extension/spec.md:46`

#### FR-CIV-VERIFY-009

- Tests:
  - `crates/engine/tests/fr_fr_civ_verify_009.rs:1`
  - `crates/engine/tests/fr_fr_civ_verify_009.rs:7`
- Spec/trace:
  - `agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:65`
  - `agileplus-specs/civ-018-verify-harness-extension/spec.md:50`
  - `docs/traceability/fr-civ-verify-009/fr-civ-verify-009-adr.md:1`

#### FR-CIV-VERIFY-010

- Tests:
  - `crates/engine/tests/fr_fr_civ_verify_010.rs:1`
  - `crates/engine/tests/fr_fr_civ_verify_010.rs:7`
- Spec/trace:
  - `agileplus-specs/civ-016-devx-verify-harness-and-worktree-hygiene/spec.md:69`
  - `agileplus-specs/civ-018-verify-harness-extension/spec.md:55`
  - `agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:66`


### Epic `FR-DOC` — 1 IDs

#### FR-DOC-001

- Tests:
  - `crates/engine/tests/fr_fr_doc_001.rs:1`
  - `crates/engine/tests/fr_fr_doc_001.rs:8`
  - `crates/engine/tests/fr_fr_doc_001.rs:16`
- Spec/trace:
  - `docs/FR_DETAILED.md:358`
  - `docs/traceability/fr-doc-001/fr-doc-001-adr.md:1`
  - `docs/traceability/fr-doc-001/fr-doc-001-adr.md:6`


### Epic `FR-PERF` — 1 IDs

#### FR-PERF-001

- Tests:
  - `crates/engine/tests/fr_fr_perf_001.rs:1`
- Spec/trace:
  - `docs/FR_DETAILED.md:296`
  - `docs/traceability/TRACEABILITY_MATRIX.md:287`
  - `docs/traceability/fr-perf-001/fr-perf-001-adr.md:1`


### Epic `FR-SOC-CIV` — 2 IDs

#### FR-SOC-CIV-001

- Tests:
  - `crates/engine/tests/fr_fr_soc_civ_001.rs:1`
  - `crates/engine/tests/fr_fr_soc_civ_001.rs:5`
  - `crates/engine/tests/fr_fr_soc_civ_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4326`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4570`
  - `docs/traceability/fr-soc-civ-001/fr-soc-civ-001-adr.md:1`

#### FR-SOC-CIV-002

- Tests:
  - `crates/engine/tests/fr_fr_soc_civ_002.rs:1`
  - `crates/engine/tests/fr_fr_soc_civ_002.rs:5`
  - `crates/engine/tests/fr_fr_soc_civ_002.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4355`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4571`
  - `docs/traceability/fr-soc-civ-002/fr-soc-civ-002-adr.md:1`


### Epic `FR-SOC-INT` — 4 IDs

#### FR-SOC-INT-001

- Tests:
  - `crates/engine/tests/fr_fr_soc_int_001.rs:1`
  - `crates/engine/tests/fr_fr_soc_int_001.rs:5`
  - `crates/engine/tests/fr_fr_soc_int_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1770`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2003`
  - `docs/traceability/fr-soc-int-001/fr-soc-int-001-adr.md:1`

#### FR-SOC-INT-002

- Tests:
  - `crates/engine/tests/fr_fr_soc_int_002.rs:1`
  - `crates/engine/tests/fr_fr_soc_int_002.rs:5`
  - `crates/engine/tests/fr_fr_soc_int_002.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1778`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2004`
  - `docs/traceability/fr-soc-int-002/fr-soc-int-002-adr.md:1`

#### FR-SOC-INT-003

- Tests:
  - `crates/engine/tests/fr_fr_soc_int_003.rs:1`
  - `crates/engine/tests/fr_fr_soc_int_003.rs:5`
  - `crates/engine/tests/fr_fr_soc_int_003.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1786`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2005`
  - `docs/traceability/fr-soc-int-003/fr-soc-int-003-adr.md:1`

#### FR-SOC-INT-004

- Tests:
  - `crates/engine/tests/fr_fr_soc_int_004.rs:1`
  - `crates/engine/tests/fr_fr_soc_int_004.rs:5`
  - `crates/engine/tests/fr_fr_soc_int_004.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1797`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2006`
  - `docs/traceability/fr-soc-int-004/fr-soc-int-004-adr.md:1`


### Epic `NFR-CIV-PERF` — 1 IDs

#### NFR-CIV-PERF-002

- Tests:
  - `crates/engine/tests/fr_engine_hash_lod_perf_tests.rs:127`
- Spec/trace:
  - `docs/reference/non-functional-requirements.md:41`
  - `docs/reference/non-functional-requirements.md:441`
  - `docs/reference/non-functional-requirements.md:552`

