# P2 agent-A

# Phase: SPEC-ONLY → IMPL-NO-TEST or COVERED

Each ID has spec/trace references but no source code. You have two options:

## Option A: Write minimal source
1. Read the spec at the listed `spec_refs` paths.
2. If the spec is well-defined and small (≤1 function, ≤20 lines), write
   a minimal source implementation that satisfies it.
3. Add the function to the appropriate `crates/<crate>/src/<file>.rs`.
4. Add `// FR-XYZ` comment above the function.
5. Optionally write a minimal test in `crates/<crate>/tests/fr_<id>.rs`.

## Option B: Defer or delete
1. If the spec is large / unclear / depends on systems we don't have
   yet, **skip the ID** — leave it as SPEC-ONLY for a future agent.
2. Add a comment to `docs/audits/spec-only-deferred.md` listing
   deferred IDs with a brief reason.

## Bias
- Prefer Option A for FRs with `spec_refs` pointing to small ADR/plan
  files (< 100 lines).
- Prefer skip for FRs whose spec requires infrastructure not yet built.


## Your slice: 54 IDs in 8 epics


### Epic `FR-CIV` — 2 IDs

#### FR-CIV-0104-011

- Spec/trace:
  - `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1504`

#### FR-CIV-0700

- Spec/trace:
  - `docs/design/civ-actor-assets-fix.md:322`


### Epic `FR-CIV-ECON-002-JOULE` — 1 IDs

#### FR-CIV-ECON-002-JOULE

- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:70`
  - `docs/guides/COPILOT_L3_AGENTS.md:92`
  - `docs/guides/COPILOT_L3_AGENTS.md:93`


### Epic `FR-CIV-LEGENDS-CONFIG` — 1 IDs

#### FR-CIV-LEGENDS-CONFIG-04

- Spec/trace:
  - `docs/traceability/fr-emergence-matrix.md:259`


### Epic `FR-CIV-RESEARCH-004-REPLAY` — 1 IDs

#### FR-CIV-RESEARCH-004-REPLAY

- Spec/trace:
  - `PLAN.md:239`
  - `docs/traceability/fr-civ-research-004-replay/fr-civ-research-004-replay-adr.md:1`
  - `docs/traceability/fr-civ-research-004-replay/fr-civ-research-004-replay-adr.md:6`


### Epic `FR-CIV-TECH` — 21 IDs

#### FR-CIV-TECH-001

- Spec/trace:
  - `docs/design/tech-engineering.md:225`
  - `docs/traceability/fr-civ-tech-001/fr-civ-tech-001-adr.md:1`
  - `docs/traceability/fr-civ-tech-001/fr-civ-tech-001-adr.md:6`

#### FR-CIV-TECH-002

- Spec/trace:
  - `docs/design/tech-engineering.md:226`
  - `docs/traceability/fr-civ-tech-002/fr-civ-tech-002-adr.md:1`
  - `docs/traceability/fr-civ-tech-002/fr-civ-tech-002-adr.md:6`

#### FR-CIV-TECH-003

- Spec/trace:
  - `docs/design/tech-engineering.md:227`
  - `docs/traceability/fr-civ-tech-003/fr-civ-tech-003-adr.md:1`
  - `docs/traceability/fr-civ-tech-003/fr-civ-tech-003-adr.md:6`

#### FR-CIV-TECH-004

- Spec/trace:
  - `docs/design/tech-engineering.md:228`
  - `docs/traceability/fr-civ-tech-004/fr-civ-tech-004-adr.md:1`
  - `docs/traceability/fr-civ-tech-004/fr-civ-tech-004-adr.md:6`

#### FR-CIV-TECH-005

- Spec/trace:
  - `docs/design/tech-engineering.md:229`
  - `docs/traceability/fr-civ-tech-005/fr-civ-tech-005-adr.md:1`
  - `docs/traceability/fr-civ-tech-005/fr-civ-tech-005-adr.md:6`

#### FR-CIV-TECH-006

- Spec/trace:
  - `docs/design/tech-engineering.md:230`
  - `docs/traceability/fr-civ-tech-006/fr-civ-tech-006-adr.md:1`
  - `docs/traceability/fr-civ-tech-006/fr-civ-tech-006-adr.md:6`

#### FR-CIV-TECH-007

- Spec/trace:
  - `docs/design/tech-engineering.md:231`
  - `docs/traceability/fr-civ-tech-007/fr-civ-tech-007-adr.md:1`
  - `docs/traceability/fr-civ-tech-007/fr-civ-tech-007-adr.md:6`

#### FR-CIV-TECH-008

- Spec/trace:
  - `docs/design/tech-engineering.md:232`
  - `docs/traceability/fr-civ-tech-008/fr-civ-tech-008-adr.md:1`
  - `docs/traceability/fr-civ-tech-008/fr-civ-tech-008-adr.md:6`

#### FR-CIV-TECH-009

- Spec/trace:
  - `docs/design/tech-engineering.md:233`
  - `docs/traceability/fr-civ-tech-009/fr-civ-tech-009-adr.md:1`
  - `docs/traceability/fr-civ-tech-009/fr-civ-tech-009-adr.md:6`

#### FR-CIV-TECH-010

- Spec/trace:
  - `docs/design/tech-engineering.md:234`
  - `docs/traceability/fr-civ-tech-010/fr-civ-tech-010-adr.md:1`
  - `docs/traceability/fr-civ-tech-010/fr-civ-tech-010-adr.md:6`

#### FR-CIV-TECH-011

- Spec/trace:
  - `docs/design/tech-engineering.md:235`
  - `docs/traceability/fr-civ-tech-011/fr-civ-tech-011-adr.md:1`
  - `docs/traceability/fr-civ-tech-011/fr-civ-tech-011-adr.md:6`

#### FR-CIV-TECH-012

- Spec/trace:
  - `docs/design/tech-engineering.md:236`
  - `docs/traceability/fr-civ-tech-012/fr-civ-tech-012-adr.md:1`
  - `docs/traceability/fr-civ-tech-012/fr-civ-tech-012-adr.md:6`

#### FR-CIV-TECH-013

- Spec/trace:
  - `docs/design/tech-engineering.md:237`
  - `docs/traceability/fr-civ-tech-013/fr-civ-tech-013-adr.md:1`
  - `docs/traceability/fr-civ-tech-013/fr-civ-tech-013-adr.md:6`

#### FR-CIV-TECH-014

- Spec/trace:
  - `docs/design/tech-engineering.md:238`
  - `docs/traceability/fr-civ-tech-014/fr-civ-tech-014-adr.md:1`
  - `docs/traceability/fr-civ-tech-014/fr-civ-tech-014-adr.md:6`

#### FR-CIV-TECH-015

- Spec/trace:
  - `docs/design/tech-engineering.md:239`
  - `docs/traceability/fr-civ-tech-015/fr-civ-tech-015-adr.md:1`
  - `docs/traceability/fr-civ-tech-015/fr-civ-tech-015-adr.md:6`

#### FR-CIV-TECH-016

- Spec/trace:
  - `docs/design/tech-engineering.md:240`
  - `docs/traceability/fr-civ-tech-016/fr-civ-tech-016-adr.md:1`
  - `docs/traceability/fr-civ-tech-016/fr-civ-tech-016-adr.md:6`

#### FR-CIV-TECH-017

- Spec/trace:
  - `docs/design/tech-engineering.md:241`
  - `docs/traceability/fr-civ-tech-017/fr-civ-tech-017-adr.md:1`
  - `docs/traceability/fr-civ-tech-017/fr-civ-tech-017-adr.md:6`

#### FR-CIV-TECH-018

- Spec/trace:
  - `docs/design/tech-engineering.md:242`
  - `docs/traceability/fr-civ-tech-018/fr-civ-tech-018-adr.md:1`
  - `docs/traceability/fr-civ-tech-018/fr-civ-tech-018-adr.md:6`

#### FR-CIV-TECH-019

- Spec/trace:
  - `docs/design/tech-engineering.md:243`
  - `docs/traceability/fr-civ-tech-019/fr-civ-tech-019-adr.md:1`
  - `docs/traceability/fr-civ-tech-019/fr-civ-tech-019-adr.md:6`

#### FR-CIV-TECH-020

- Spec/trace:
  - `docs/design/tech-engineering.md:244`
  - `docs/traceability/fr-civ-tech-020/fr-civ-tech-020-adr.md:1`
  - `docs/traceability/fr-civ-tech-020/fr-civ-tech-020-adr.md:6`

#### FR-CIV-TECH-021

- Spec/trace:
  - `docs/design/tech-engineering.md:245`
  - `docs/traceability/fr-civ-tech-021/fr-civ-tech-021-adr.md:1`
  - `docs/traceability/fr-civ-tech-021/fr-civ-tech-021-adr.md:6`


### Epic `NFR-CIV` — 13 IDs

#### NFR-CIV-001

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:52`

#### NFR-CIV-002

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:53`

#### NFR-CIV-003

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:54`

#### NFR-CIV-004

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:55`

#### NFR-CIV-005

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:56`

#### NFR-CIV-006

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:57`

#### NFR-CIV-007

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:58`

#### NFR-CIV-008

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:59`

#### NFR-CIV-009

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:60`

#### NFR-CIV-010

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:61`

#### NFR-CIV-011

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:62`

#### NFR-CIV-012

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:63`

#### NFR-CIV-013

- Spec/trace:
  - `docs/traceability/nfr-matrix.md:64`


### Epic `NFR-CIV-PERF` — 9 IDs

#### NFR-CIV-PERF-003

- Spec/trace:
  - `agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:69`
  - `agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:25`
  - `docs/design/civ-perf-dirty-incremental.md:9`

#### NFR-CIV-PERF-004

- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:189`
  - `docs/reference/non-functional-requirements.md:69`
  - `docs/reference/non-functional-requirements.md:193`

#### NFR-CIV-PERF-005

- Spec/trace:
  - `agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md:24`
  - `docs/design/civ-perf-dirty-incremental.md:10`
  - `docs/design/civ-perf-dirty-incremental.md:502`

#### NFR-CIV-PERF-006

- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:172`
  - `docs/reference/non-functional-requirements.md:100`
  - `docs/reference/non-functional-requirements.md:556`

#### NFR-CIV-PERF-007

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:114`
  - `docs/reference/non-functional-requirements.md:230`
  - `docs/reference/non-functional-requirements.md:557`

#### NFR-CIV-PERF-008

- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:171`
  - `docs/guides/voxel-emergent-vision-and-migration.md:191`
  - `docs/traceability/index.md:1187`

#### NFR-CIV-PERF-900

- Spec/trace:
  - `docs/agileplus/epics/civ-w5-scale.md:14`
  - `docs/agileplus/epics/civ-w5-scale.md:27`
  - `docs/agileplus/README.md:24`

#### NFR-CIV-PERF-901

- Spec/trace:
  - `docs/agileplus/epics/civ-w5-scale.md:15`
  - `docs/agileplus/epics/civ-w5-scale.md:27`
  - `docs/agileplus/README.md:24`

#### NFR-CIV-PERF-902

- Spec/trace:
  - `docs/agileplus/epics/civ-w5-scale.md:16`
  - `docs/agileplus/epics/civ-w5-scale.md:28`
  - `docs/agileplus/README.md:24`


### Epic `NFR-R` — 6 IDs

#### NFR-R-01

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2077`
  - `docs/traceability/index.md:1225`
  - `docs/traceability/nfr-r-01/nfr-r-01-spec.md:1`

#### NFR-R-02

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2078`
  - `docs/traceability/index.md:1226`
  - `docs/traceability/nfr-r-02/nfr-r-02-spec.md:1`

#### NFR-R-03

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2079`
  - `docs/traceability/index.md:1227`
  - `docs/traceability/nfr-r-03/nfr-r-03-spec.md:1`

#### NFR-R-04

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2080`
  - `docs/traceability/index.md:1228`
  - `docs/traceability/nfr-r-04/nfr-r-04-spec.md:1`

#### NFR-R-05

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2081`
  - `docs/traceability/index.md:1229`
  - `docs/traceability/nfr-r-05/nfr-r-05-spec.md:1`

#### NFR-R-06

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2082`
  - `docs/traceability/index.md:1230`
  - `docs/traceability/nfr-r-06/nfr-r-06-spec.md:1`

