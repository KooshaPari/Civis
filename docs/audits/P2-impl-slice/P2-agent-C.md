# P2 agent-C

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


## Your slice: 22 IDs in 7 epics


### Epic `FR-CIV-ASSET` — 12 IDs

#### FR-CIV-ASSET-002

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2437`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3206`
  - `docs/traceability/fr-civ-asset-002/fr-civ-asset-002-adr.md:1`

#### FR-CIV-ASSET-008

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2497`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3212`
  - `docs/traceability/fr-civ-asset-008/fr-civ-asset-008-adr.md:1`

#### FR-CIV-ASSET-009

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2507`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3213`
  - `docs/traceability/fr-civ-asset-009/fr-civ-asset-009-adr.md:1`

#### FR-CIV-ASSET-010

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:80`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2425`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2517`

#### FR-CIV-ASSET-011

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:81`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2527`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2529`

#### FR-CIV-ASSET-012

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2539`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2917`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3216`

#### FR-CIV-ASSET-013

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2549`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3217`
  - `docs/traceability/fr-civ-asset-013/fr-civ-asset-013-adr.md:1`

#### FR-CIV-ASSET-014

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2559`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3218`
  - `docs/traceability/fr-civ-asset-014/fr-civ-asset-014-adr.md:1`

#### FR-CIV-ASSET-015

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:1714`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2569`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3219`

#### FR-CIV-ASSET-017

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2589`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3221`
  - `docs/traceability/fr-civ-asset-017/fr-civ-asset-017-adr.md:1`

#### FR-CIV-ASSET-019

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2609`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3223`
  - `docs/traceability/fr-civ-asset-019/fr-civ-asset-019-adr.md:1`

#### FR-CIV-ASSET-020

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:81`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2527`
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2619`


### Epic `FR-CIV-EMERGENCE-RELIGION` — 2 IDs

#### FR-CIV-EMERGENCE-RELIGION-1

- Spec/trace:
  - `docs/design/RELIGION_EMERGENCE.md:243`

#### FR-CIV-EMERGENCE-RELIGION-2

- Spec/trace:
  - `docs/design/RELIGION_EMERGENCE.md:456`


### Epic `FR-CIV-LEGENDS-SCALE` — 1 IDs

#### FR-CIV-LEGENDS-SCALE-02

- Spec/trace:
  - `docs/traceability/fr-emergence-matrix.md:261`


### Epic `FR-CIV-SOCIAL` — 2 IDs

#### FR-CIV-SOCIAL-001

- Spec/trace:
  - `agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:26`
  - `agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:40`
  - `agileplus-specs/civ-007-diplomacy-laws-government/spec.md:42`

#### FR-CIV-SOCIAL-002

- Spec/trace:
  - `agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:27`
  - `agileplus-specs/civ-009-culture-diffusion/spec.md:37`
  - `docs/reference/agileplus-artifacts-index.md:73`


### Epic `FR-CIV-WAR` — 1 IDs

#### FR-CIV-WAR-020

- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:228`
  - `docs/design/warfare.md:106`
  - `docs/design/warfare.md:195`


### Epic `NFR-CIV-AI` — 1 IDs

#### NFR-CIV-AI-002

- Spec/trace:
  - `docs/design/civ-ai-crate.md:49`
  - `docs/traceability/index.md:1163`
  - `docs/traceability/nfr-civ-ai-002/nfr-civ-ai-002-research.md:1`


### Epic `NFR-CIV-REL` — 3 IDs

#### NFR-CIV-REL-001

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:236`
  - `docs/reference/non-functional-requirements.md:565`
  - `docs/reference/non-functional-requirements.md:595`

#### NFR-CIV-REL-002

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:250`
  - `docs/reference/non-functional-requirements.md:332`
  - `docs/reference/non-functional-requirements.md:566`

#### NFR-CIV-REL-003

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:264`
  - `docs/reference/non-functional-requirements.md:567`
  - `docs/reference/non-functional-requirements.md:595`

