# P2 agent-E

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


## Your slice: 35 IDs in 7 epics


### Epic `FR-CIV-ASSET-QUAL` — 1 IDs

#### FR-CIV-ASSET-QUAL-001

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3224`
  - `docs/traceability/fr-civ-asset-qual-001/fr-civ-asset-qual-001-adr.md:1`
  - `docs/traceability/fr-civ-asset-qual-001/fr-civ-asset-qual-001-adr.md:6`


### Epic `FR-CIV-GODOT-ATTACH` — 4 IDs

#### FR-CIV-GODOT-ATTACH-001

- Spec/trace:
  - `docs/development-guide/fr-godot-attach.md:9`
  - `docs/traceability/fr-civ-godot-attach-001/fr-civ-godot-attach-001-adr.md:1`
  - `docs/traceability/fr-civ-godot-attach-001/fr-civ-godot-attach-001-adr.md:6`

#### FR-CIV-GODOT-ATTACH-002

- Spec/trace:
  - `docs/development-guide/fr-godot-attach.md:10`
  - `docs/traceability/fr-civ-godot-attach-002/fr-civ-godot-attach-002-adr.md:1`
  - `docs/traceability/fr-civ-godot-attach-002/fr-civ-godot-attach-002-adr.md:6`

#### FR-CIV-GODOT-ATTACH-003

- Spec/trace:
  - `docs/development-guide/fr-godot-attach.md:11`
  - `docs/traceability/fr-civ-godot-attach-003/fr-civ-godot-attach-003-adr.md:1`
  - `docs/traceability/fr-civ-godot-attach-003/fr-civ-godot-attach-003-adr.md:6`

#### FR-CIV-GODOT-ATTACH-004

- Spec/trace:
  - `docs/development-guide/fr-godot-attach.md:12`
  - `docs/traceability/fr-civ-godot-attach-004/fr-civ-godot-attach-004-adr.md:1`
  - `docs/traceability/fr-civ-godot-attach-004/fr-civ-godot-attach-004-adr.md:6`


### Epic `FR-CIV-MIGRATION` — 5 IDs

#### FR-CIV-MIGRATION-001

- Spec/trace:
  - `docs/traceability/fr-emergence-matrix.md:215`

#### FR-CIV-MIGRATION-002

- Spec/trace:
  - `docs/traceability/fr-emergence-matrix.md:216`

#### FR-CIV-MIGRATION-003

- Spec/trace:
  - `docs/traceability/fr-emergence-matrix.md:217`

#### FR-CIV-MIGRATION-004

- Spec/trace:
  - `docs/traceability/fr-emergence-matrix.md:218`

#### FR-CIV-MIGRATION-005

- Spec/trace:
  - `docs/traceability/fr-emergence-matrix.md:219`


### Epic `FR-CIV-SOCIAL-002-IDEOLOGY` — 1 IDs

#### FR-CIV-SOCIAL-002-IDEOLOGY

- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:226`
  - `PLAN.md:149`
  - `PLAN.md:150`


### Epic `FR-SAVE` — 19 IDs

#### FR-SAVE-006

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2805`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2943`
  - `docs/traceability/fr-save-006/fr-save-006-adr.md:1`

#### FR-SAVE-007

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2806`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2943`
  - `docs/traceability/fr-save-007/fr-save-007-adr.md:1`

#### FR-SAVE-008

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2807`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2949`
  - `docs/traceability/fr-save-008/fr-save-008-adr.md:1`

#### FR-SAVE-009

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2808`
  - `docs/traceability/fr-save-009/fr-save-009-adr.md:1`
  - `docs/traceability/fr-save-009/fr-save-009-adr.md:6`

#### FR-SAVE-011

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2810`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2963`
  - `docs/traceability/fr-save-011/fr-save-011-adr.md:1`

#### FR-SAVE-012

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2811`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2963`
  - `docs/traceability/fr-save-012/fr-save-012-adr.md:1`

#### FR-SAVE-013

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2812`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2969`
  - `docs/traceability/fr-save-013/fr-save-013-adr.md:1`

#### FR-SAVE-014

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2813`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2969`
  - `docs/traceability/fr-save-014/fr-save-014-adr.md:1`

#### FR-SAVE-015

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2814`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2969`
  - `docs/traceability/fr-save-015/fr-save-015-adr.md:1`

#### FR-SAVE-016

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2815`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2977`
  - `docs/traceability/fr-save-016/fr-save-016-adr.md:1`

#### FR-SAVE-017

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2816`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2977`
  - `docs/traceability/fr-save-017/fr-save-017-adr.md:1`

#### FR-SAVE-018

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2817`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2977`
  - `docs/traceability/fr-save-018/fr-save-018-adr.md:1`

#### FR-SAVE-019

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2818`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2977`
  - `docs/traceability/fr-save-019/fr-save-019-adr.md:1`

#### FR-SAVE-020

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2819`
  - `docs/traceability/fr-save-020/fr-save-020-adr.md:1`
  - `docs/traceability/fr-save-020/fr-save-020-adr.md:6`

#### FR-SAVE-021

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2820`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2987`
  - `docs/traceability/fr-save-021/fr-save-021-adr.md:1`

#### FR-SAVE-022

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2821`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2993`
  - `docs/traceability/fr-save-022/fr-save-022-adr.md:1`

#### FR-SAVE-023

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2822`
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:3001`
  - `docs/traceability/fr-save-023/fr-save-023-adr.md:1`

#### FR-SAVE-024

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2823`
  - `docs/traceability/fr-save-024/fr-save-024-adr.md:1`
  - `docs/traceability/fr-save-024/fr-save-024-adr.md:6`

#### FR-SAVE-025

- Spec/trace:
  - `docs/specs/CIV-1000-save-load-persistence-spec.md:2824`
  - `docs/traceability/fr-save-025/fr-save-025-adr.md:1`
  - `docs/traceability/fr-save-025/fr-save-025-adr.md:6`


### Epic `NFR-CIV-LEGENDS-LOUD` — 1 IDs

#### NFR-CIV-LEGENDS-LOUD-03

- Spec/trace:
  - `docs/design/legends-engine.md:453`
  - `docs/traceability/index.md:1171`
  - `docs/traceability/nfr-civ-legends-loud-03/nfr-civ-legends-loud-03-research.md:1`


### Epic `NFR-CIV-SEC` — 4 IDs

#### NFR-CIV-SEC-001

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:294`
  - `docs/reference/non-functional-requirements.md:569`
  - `docs/reference/non-functional-requirements.md:607`

#### NFR-CIV-SEC-002

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:308`
  - `docs/reference/non-functional-requirements.md:346`
  - `docs/reference/non-functional-requirements.md:570`

#### NFR-CIV-SEC-003

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:322`
  - `docs/reference/non-functional-requirements.md:571`
  - `docs/reference/non-functional-requirements.md:602`

#### NFR-CIV-SEC-004

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:336`
  - `docs/reference/non-functional-requirements.md:572`
  - `docs/reference/non-functional-requirements.md:611`

