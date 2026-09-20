# P1 agent-E

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


## Your slice: 34 IDs in 15 epics


### Epic `FR-CIV-ACT` — 4 IDs

#### FR-CIV-ACT-001

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:119`
  - `crates/build/tests/fr_matrix_batch12.rs:122`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:60`
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:213`
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:225`

#### FR-CIV-ACT-003

- Tests:
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:14`
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:210`
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:213`
- Spec/trace:
  - `docs/reference/REFERENCE_GAME_ANALYSIS.md:511`
  - `docs/traceability/fr-civ-act-003/fr-civ-act-003-adr.md:1`
  - `docs/traceability/fr-civ-act-003/fr-civ-act-003-adr.md:6`

#### FR-CIV-ACT-004

- Tests:
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:16`
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:348`
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:351`
- Spec/trace:
  - `docs/reference/REFERENCE_GAME_ANALYSIS.md:183`
  - `docs/traceability/fr-civ-act-004/fr-civ-act-004-adr.md:1`
  - `docs/traceability/fr-civ-act-004/fr-civ-act-004-adr.md:6`

#### FR-CIV-ACT-005

- Tests:
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:18`
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:429`
  - `crates/engine/tests/fr_civ_act_arch_cluster.rs:432`
- Spec/trace:
  - `docs/reports/STATUS_REPORT.md:98`
  - `docs/traceability/fr-civ-act-005/fr-civ-act-005-adr.md:1`
  - `docs/traceability/fr-civ-act-005/fr-civ-act-005-adr.md:6`


### Epic `FR-CIV-BIO` — 3 IDs

#### FR-CIV-BIO-001

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:388`
  - `crates/build/tests/fr_matrix_batch12.rs:391`
- Spec/trace:
  - `agileplus-specs/civ-008-genetics-species/plan.md:5`
  - `agileplus-specs/civ-008-genetics-species/spec.md:24`
  - `docs/guides/voxel-emergent-vision-and-migration.md:33`

#### FR-CIV-BIO-002

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:417`
  - `crates/build/tests/fr_matrix_batch12.rs:420`
- Spec/trace:
  - `agileplus-specs/civ-008-genetics-species/plan.md:11`
  - `agileplus-specs/civ-008-genetics-species/spec.md:25`
  - `docs/reference/agileplus-artifacts-index.md:153`

#### FR-CIV-BIO-003

- Tests:
  - `crates/build/tests/fr_matrix_batch12.rs:449`
  - `crates/build/tests/fr_matrix_batch12.rs:452`
- Spec/trace:
  - `agileplus-specs/civ-008-genetics-species/plan.md:18`
  - `agileplus-specs/civ-008-genetics-species/spec.md:26`
  - `docs/reference/agileplus-artifacts-index.md:153`


### Epic `FR-CIV-DET` — 1 IDs

#### FR-CIV-DET-001

- Tests:
  - `crates/engine/tests/fr_fr_civ_det_001.rs:1`
  - `crates/engine/tests/fr_fr_civ_det_001.rs:9`
  - `crates/engine/tests/fr_fr_civ_det_001.rs:24`
- Spec/trace:
  - `agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:36`
  - `agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:53`
  - `agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:73`


### Epic `FR-CIV-GODOT-ATTACH` — 1 IDs

#### FR-CIV-GODOT-ATTACH-000

- Tests:
  - `clients/godot-ref/rust/tests/fr_godot_attach_tests.rs:1`
  - `clients/godot-ref/rust/tests/fr_godot_attach_tests.rs:6`
  - `clients/godot-ref/rust/tests/fr_godot_attach_tests.rs:23`
- Spec/trace:
  - `docs/development-guide/fr-godot-attach.md:8`
  - `docs/development-guide/fr-p-u1-roadmap.md:12`
  - `docs/traceability/fr-3d-matrix.md:242`


### Epic `FR-CIV-LEGENDS-GAP` — 1 IDs

#### FR-CIV-LEGENDS-GAP-12

- Tests:
  - `crates/legends/tests/fr_fr_civ_legends_gap_12.rs:1`
  - `crates/legends/tests/fr_fr_civ_legends_gap_12.rs:10`
  - `crates/legends/tests/fr_fr_civ_legends_gap_12.rs:19`
- Spec/trace:
  - `docs/design/legends-engine.md:447`
  - `docs/traceability/fr-civ-legends-gap-12/fr-civ-legends-gap-12-adr.md:1`
  - `docs/traceability/fr-civ-legends-gap-12/fr-civ-legends-gap-12-adr.md:6`


### Epic `FR-CIV-LEGENDS-SIG` — 1 IDs

#### FR-CIV-LEGENDS-SIG-05

- Tests:
  - `crates/legends/tests/fr_fr_civ_legends_sig_05.rs:1`
  - `crates/legends/tests/fr_fr_civ_legends_sig_05.rs:11`
  - `crates/legends/tests/fr_fr_civ_legends_sig_05.rs:19`
- Spec/trace:
  - `docs/design/legends-engine.md:440`
  - `docs/traceability/fr-civ-legends-sig-05/fr-civ-legends-sig-05-adr.md:1`
  - `docs/traceability/fr-civ-legends-sig-05/fr-civ-legends-sig-05-adr.md:6`


### Epic `FR-CIV-NOTIFY` — 5 IDs

#### FR-CIV-NOTIFY-901

- Tests:
  - `crates/engine/tests/fr_civ_notify_cluster.rs:12`
  - `crates/engine/tests/fr_civ_notify_cluster.rs:91`
  - `crates/engine/tests/fr_civ_notify_cluster.rs:94`
- Spec/trace:
  - `docs/agileplus/epics/civ-w6-ui.md:13`
  - `docs/agileplus/epics/civ-w6-ui.md:26`
  - `docs/agileplus/README.md:25`

#### FR-CIV-NOTIFY-910

- Tests:
  - `crates/engine/tests/fr_civ_notify_cluster.rs:18`
  - `crates/engine/tests/fr_civ_notify_cluster.rs:309`
  - `crates/engine/tests/fr_civ_notify_cluster.rs:363`
- Spec/trace:
  - `docs/agileplus/epics/civ-w6-ui.md:14`
  - `docs/agileplus/epics/civ-w6-ui.md:27`
  - `docs/agileplus/README.md:25`

#### FR-CIV-NOTIFY-911

- Tests:
  - `crates/engine/tests/fr_civ_notify_cluster.rs:23`
  - `crates/engine/tests/fr_civ_notify_cluster.rs:551`
  - `crates/engine/tests/fr_civ_notify_cluster.rs:554`
- Spec/trace:
  - `docs/agileplus/epics/civ-w6-ui.md:15`
  - `docs/agileplus/epics/civ-w6-ui.md:27`
  - `docs/agileplus/README.md:25`

#### FR-CIV-NOTIFY-920

- Tests:
  - `crates/engine/tests/fr_civ_notify_cluster.rs:28`
  - `crates/engine/tests/fr_civ_notify_cluster.rs:655`
  - `crates/engine/tests/fr_civ_notify_cluster.rs:683`
- Spec/trace:
  - `docs/agileplus/epics/civ-w6-ui.md:16`
  - `docs/agileplus/epics/civ-w6-ui.md:28`
  - `docs/agileplus/README.md:25`

#### FR-CIV-NOTIFY-921

- Tests:
  - `crates/engine/tests/fr_civ_notify_cluster.rs:33`
  - `crates/engine/tests/fr_civ_notify_cluster.rs:901`
  - `crates/engine/tests/fr_civ_notify_cluster.rs:904`
- Spec/trace:
  - `docs/agileplus/epics/civ-w6-ui.md:17`
  - `docs/agileplus/epics/civ-w6-ui.md:29`
  - `docs/agileplus/README.md:25`


### Epic `FR-CIV-PSYCHE` — 4 IDs

#### FR-CIV-PSYCHE-002

- Tests:
  - `crates/agents/tests/fr_civ_psyche_tests.rs:29`
  - `crates/agents/tests/fr_civ_psyche_tests.rs:32`
  - `crates/agents/tests/fr_civ_psyche_tests.rs:40`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/design/psyche-social.md:273`
  - `docs/traceability/fr-civ-psyche-002/fr-civ-psyche-002-adr.md:1`

#### FR-CIV-PSYCHE-003

- Tests:
  - `crates/agents/tests/fr_civ_psyche_tests.rs:52`
  - `crates/agents/tests/fr_civ_psyche_tests.rs:55`
  - `crates/agents/tests/fr_civ_social_tests.rs:3`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/design/psyche-social.md:134`
  - `docs/design/psyche-social.md:274`

#### FR-CIV-PSYCHE-005

- Tests:
  - `crates/agents/tests/fr_civ_psyche_tests.rs:66`
  - `crates/agents/tests/fr_civ_psyche_tests.rs:69`
  - `crates/agents/tests/fr_civ_social_tests.rs:4`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/design/psyche-social.md:224`
  - `docs/design/psyche-social.md:263`

#### FR-CIV-PSYCHE-006

- Tests:
  - `crates/agents/tests/fr_civ_psyche_tests.rs:3`
  - `crates/agents/tests/fr_civ_psyche_tests.rs:84`
  - `crates/agents/tests/fr_civ_psyche_tests.rs:87`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/design/psyche-social.md:225`
  - `docs/design/psyche-social.md:276`


### Epic `FR-CIV-ROAD` — 5 IDs

#### FR-CIV-ROAD-901

- Tests:
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:1`
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:17`
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:49`
- Spec/trace:
  - `docs/agileplus/epics/civ-w3-infrastructure.md:10`
  - `docs/agileplus/epics/civ-w3-infrastructure.md:21`
  - `docs/agileplus/README.md:22`

#### FR-CIV-ROAD-902

- Tests:
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:15`
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:232`
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:235`
- Spec/trace:
  - `docs/agileplus/epics/civ-w3-infrastructure.md:11`
  - `docs/agileplus/epics/civ-w3-infrastructure.md:22`
  - `docs/agileplus/README.md:22`

#### FR-CIV-ROAD-910

- Tests:
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:323`
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:326`
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:411`
- Spec/trace:
  - `docs/agileplus/epics/civ-w3-infrastructure.md:12`
  - `docs/agileplus/epics/civ-w3-infrastructure.md:23`
  - `docs/agileplus/README.md:22`

#### FR-CIV-ROAD-920

- Tests:
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:552`
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:555`
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:609`
- Spec/trace:
  - `docs/agileplus/epics/civ-w3-infrastructure.md:13`
  - `docs/agileplus/epics/civ-w3-infrastructure.md:24`
  - `docs/agileplus/README.md:22`

#### FR-CIV-ROAD-921

- Tests:
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:24`
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:695`
  - `crates/civ-traffic/tests/fr_civ_road_cluster.rs:698`
- Spec/trace:
  - `docs/agileplus/epics/civ-w3-infrastructure.md:14`
  - `docs/agileplus/epics/civ-w3-infrastructure.md:25`
  - `docs/agileplus/README.md:22`


### Epic `FR-CIV-SERVER-002-PROTO` — 1 IDs

#### FR-CIV-SERVER-002-PROTO

- Tests:
  - `crates/server/tests/fr_civ_server_tests.rs:4`
  - `crates/server/tests/fr_civ_server_tests.rs:38`
  - `crates/server/tests/fr_fr_civ_server_002_proto.rs:1`
- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:223`
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:224`
  - `PLAN.md:176`


### Epic `FR-CLIENT` — 3 IDs

#### FR-CLIENT-001

- Tests:
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:9`
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:595`
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:598`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `agileplus-specs/civ-011-bevy-primary-client/plan.md:5`
  - `agileplus-specs/civ-011-bevy-primary-client/plan.md:12`

#### FR-CLIENT-002

- Tests:
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:10`
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:682`
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:685`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/reference/agileplus-artifacts-index.md:335`
  - `docs/reference/non-functional-requirements.md:362`

#### FR-CLIENT-003

- Tests:
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:11`
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:740`
  - `crates/engine/tests/fr_civ_rts_client_perf_cluster.rs:743`
- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `agileplus-specs/civ-010-multi-client-protocol/plan.md:25`
  - `agileplus-specs/civ-010-multi-client-protocol/spec.md:30`


### Epic `FR-INT` — 1 IDs

#### FR-INT-001

- Tests:
  - `crates/engine/tests/fr_fr_int_001.rs:1`
  - `crates/engine/tests/fr_fr_int_001.rs:8`
  - `crates/engine/tests/fr_fr_int_001.rs:19`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1739`
  - `docs/traceability/fr-int-001/fr-int-001-adr.md:1`
  - `docs/traceability/fr-int-001/fr-int-001-adr.md:6`


### Epic `FR-REP` — 1 IDs

#### FR-REP-001

- Tests:
  - `crates/engine/tests/fr_fr_rep_001.rs:1`
  - `crates/engine/tests/fr_fr_rep_001.rs:13`
  - `crates/engine/tests/fr_fr_rep_001.rs:22`
- Spec/trace:
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:489`
  - `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:532`
  - `docs/traceability/fr-rep-001/fr-rep-001-adr.md:1`


### Epic `FR-SOC-FAC` — 2 IDs

#### FR-SOC-FAC-001

- Tests:
  - `crates/engine/tests/fr_fr_soc_fac_001.rs:1`
  - `crates/engine/tests/fr_fr_soc_fac_001.rs:5`
  - `crates/engine/tests/fr_fr_soc_fac_001.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4277`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4568`
  - `docs/traceability/fr-soc-fac-001/fr-soc-fac-001-adr.md:1`

#### FR-SOC-FAC-002

- Tests:
  - `crates/engine/tests/fr_fr_soc_fac_002.rs:1`
  - `crates/engine/tests/fr_fr_soc_fac_002.rs:5`
  - `crates/engine/tests/fr_fr_soc_fac_002.rs:9`
- Spec/trace:
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4300`
  - `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4569`
  - `docs/traceability/fr-soc-fac-002/fr-soc-fac-002-adr.md:1`


### Epic `FR-TEST` — 1 IDs

#### FR-TEST-001

- Tests:
  - `crates/engine/tests/fr_fr_test_001.rs:1`
  - `crates/engine/tests/fr_fr_test_001.rs:5`
  - `crates/engine/tests/fr_fr_test_001.rs:9`
- Spec/trace:
  - `docs/FR_DETAILED.md:341`
  - `docs/traceability/fr-test-001/fr-test-001-adr.md:1`
  - `docs/traceability/fr-test-001/fr-test-001-adr.md:6`

