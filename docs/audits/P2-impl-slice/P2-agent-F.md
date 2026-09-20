# P2 agent-F

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


## Your slice: 87 IDs in 7 epics


### Epic `FR-CIV-AUDIO` — 4 IDs

#### FR-CIV-AUDIO-009

- Spec/trace:
  - `docs/design/audio-direction.md:302`
  - `docs/traceability/fr-civ-audio-009/fr-civ-audio-009-adr.md:1`
  - `docs/traceability/fr-civ-audio-009/fr-civ-audio-009-adr.md:6`

#### FR-CIV-AUDIO-010

- Spec/trace:
  - `docs/design/audio-direction.md:303`
  - `docs/traceability/fr-civ-audio-010/fr-civ-audio-010-adr.md:1`
  - `docs/traceability/fr-civ-audio-010/fr-civ-audio-010-adr.md:6`

#### FR-CIV-AUDIO-011

- Spec/trace:
  - `docs/design/audio-direction.md:304`
  - `docs/traceability/fr-civ-audio-011/fr-civ-audio-011-adr.md:1`
  - `docs/traceability/fr-civ-audio-011/fr-civ-audio-011-adr.md:6`

#### FR-CIV-AUDIO-012

- Spec/trace:
  - `docs/design/audio-direction.md:305`
  - `docs/traceability/fr-civ-audio-012/fr-civ-audio-012-adr.md:1`
  - `docs/traceability/fr-civ-audio-012/fr-civ-audio-012-adr.md:6`


### Epic `FR-CIV-GODOT-UX` — 1 IDs

#### FR-CIV-GODOT-UX-000

- Spec/trace:
  - `docs/development-guide/fr-godot-attach.md:13`
  - `docs/traceability/fr-civ-godot-ux-000/fr-civ-godot-ux-000-adr.md:1`
  - `docs/traceability/fr-civ-godot-ux-000/fr-civ-godot-ux-000-adr.md:6`


### Epic `FR-CIV-PSYCHE` — 17 IDs

#### FR-CIV-PSYCHE-004

- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-psyche-004/fr-civ-psyche-004-adr.md:1`
  - `docs/traceability/fr-civ-psyche-004/fr-civ-psyche-004-adr.md:6`

#### FR-CIV-PSYCHE-007

- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-psyche-007/fr-civ-psyche-007-adr.md:1`
  - `docs/traceability/fr-civ-psyche-007/fr-civ-psyche-007-adr.md:6`

#### FR-CIV-PSYCHE-008

- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-psyche-008/fr-civ-psyche-008-adr.md:1`
  - `docs/traceability/fr-civ-psyche-008/fr-civ-psyche-008-adr.md:6`

#### FR-CIV-PSYCHE-010

- Spec/trace:
  - `docs/design/psyche-social.md:142`
  - `docs/design/psyche-social.md:255`
  - `docs/design/psyche-social.md:277`

#### FR-CIV-PSYCHE-011

- Spec/trace:
  - `docs/design/psyche-social.md:191`
  - `docs/design/psyche-social.md:259`
  - `docs/design/psyche-social.md:278`

#### FR-CIV-PSYCHE-020

- Spec/trace:
  - `docs/design/civ-culture-emergent.md:7`
  - `docs/design/psyche-social.md:153`
  - `docs/design/psyche-social.md:256`

#### FR-CIV-PSYCHE-021

- Spec/trace:
  - `docs/design/psyche-social.md:209`
  - `docs/design/psyche-social.md:280`
  - `docs/traceability/fr-civ-psyche-021/fr-civ-psyche-021-adr.md:1`

#### FR-CIV-PSYCHE-024

- Spec/trace:
  - `docs/design/psyche-social.md:233`
  - `docs/design/psyche-social.md:264`
  - `docs/design/psyche-social.md:281`

#### FR-CIV-PSYCHE-030

- Spec/trace:
  - `docs/design/psyche-social.md:162`
  - `docs/design/psyche-social.md:257`
  - `docs/design/psyche-social.md:258`

#### FR-CIV-PSYCHE-031

- Spec/trace:
  - `docs/design/psyche-social.md:174`
  - `docs/design/psyche-social.md:283`
  - `docs/traceability/fr-civ-psyche-031/fr-civ-psyche-031-adr.md:1`

#### FR-CIV-PSYCHE-032

- Spec/trace:
  - `docs/design/psyche-social.md:203`
  - `docs/design/psyche-social.md:260`
  - `docs/design/psyche-social.md:284`

#### FR-CIV-PSYCHE-033

- Spec/trace:
  - `docs/design/psyche-social.md:206`
  - `docs/design/psyche-social.md:261`
  - `docs/design/psyche-social.md:285`

#### FR-CIV-PSYCHE-034

- Spec/trace:
  - `docs/design/psyche-social.md:241`
  - `docs/design/psyche-social.md:286`
  - `docs/traceability/fr-civ-psyche-034/fr-civ-psyche-034-adr.md:1`

#### FR-CIV-PSYCHE-035

- Spec/trace:
  - `docs/design/psyche-social.md:245`
  - `docs/design/psyche-social.md:287`
  - `docs/traceability/fr-civ-psyche-035/fr-civ-psyche-035-adr.md:1`

#### FR-CIV-PSYCHE-036

- Spec/trace:
  - `docs/design/psyche-social.md:246`
  - `docs/design/psyche-social.md:288`
  - `docs/traceability/fr-civ-psyche-036/fr-civ-psyche-036-adr.md:1`

#### FR-CIV-PSYCHE-037

- Spec/trace:
  - `docs/design/psyche-social.md:247`
  - `docs/design/psyche-social.md:289`
  - `docs/traceability/fr-civ-psyche-037/fr-civ-psyche-037-adr.md:1`

#### FR-CIV-PSYCHE-040

- Spec/trace:
  - `docs/design/psyche-social.md:6`
  - `docs/design/psyche-social.md:290`
  - `docs/traceability/fr-civ-psyche-040/fr-civ-psyche-040-adr.md:1`


### Epic `FR-CIV-SPECIES` — 36 IDs

#### FR-CIV-SPECIES-012

- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-species-012/fr-civ-species-012-adr.md:1`
  - `docs/traceability/fr-civ-species-012/fr-civ-species-012-adr.md:6`

#### FR-CIV-SPECIES-013

- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-species-013/fr-civ-species-013-adr.md:1`
  - `docs/traceability/fr-civ-species-013/fr-civ-species-013-adr.md:6`

#### FR-CIV-SPECIES-014

- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-species-014/fr-civ-species-014-adr.md:1`
  - `docs/traceability/fr-civ-species-014/fr-civ-species-014-adr.md:6`

#### FR-CIV-SPECIES-015

- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-species-015/fr-civ-species-015-adr.md:1`
  - `docs/traceability/fr-civ-species-015/fr-civ-species-015-adr.md:6`

#### FR-CIV-SPECIES-016

- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-species-016/fr-civ-species-016-adr.md:1`
  - `docs/traceability/fr-civ-species-016/fr-civ-species-016-adr.md:6`

#### FR-CIV-SPECIES-017

- Spec/trace:
  - `FUNCTIONAL_REQUIREMENTS.md`
  - `docs/traceability/fr-civ-species-017/fr-civ-species-017-adr.md:1`
  - `docs/traceability/fr-civ-species-017/fr-civ-species-017-adr.md:6`

#### FR-CIV-SPECIES-100

- Spec/trace:
  - `docs/design/species-sentience.md:73`
  - `docs/traceability/fr-civ-species-100/fr-civ-species-100-adr.md:1`
  - `docs/traceability/fr-civ-species-100/fr-civ-species-100-adr.md:6`

#### FR-CIV-SPECIES-101

- Spec/trace:
  - `docs/design/species-sentience.md:74`
  - `docs/traceability/fr-civ-species-101/fr-civ-species-101-adr.md:1`
  - `docs/traceability/fr-civ-species-101/fr-civ-species-101-adr.md:6`

#### FR-CIV-SPECIES-102

- Spec/trace:
  - `docs/design/species-sentience.md:75`
  - `docs/traceability/fr-civ-species-102/fr-civ-species-102-adr.md:1`
  - `docs/traceability/fr-civ-species-102/fr-civ-species-102-adr.md:6`

#### FR-CIV-SPECIES-103

- Spec/trace:
  - `docs/design/species-sentience.md:76`
  - `docs/traceability/fr-civ-species-103/fr-civ-species-103-adr.md:1`
  - `docs/traceability/fr-civ-species-103/fr-civ-species-103-adr.md:6`

#### FR-CIV-SPECIES-104

- Spec/trace:
  - `docs/design/species-sentience.md:77`
  - `docs/design/species-sentience.md:101`
  - `docs/traceability/fr-civ-species-104/fr-civ-species-104-adr.md:1`

#### FR-CIV-SPECIES-105

- Spec/trace:
  - `docs/design/species-sentience.md:78`
  - `docs/traceability/fr-civ-species-105/fr-civ-species-105-adr.md:1`
  - `docs/traceability/fr-civ-species-105/fr-civ-species-105-adr.md:6`

#### FR-CIV-SPECIES-200

- Spec/trace:
  - `docs/design/species-sentience.md:98`
  - `docs/traceability/fr-civ-species-200/fr-civ-species-200-adr.md:1`
  - `docs/traceability/fr-civ-species-200/fr-civ-species-200-adr.md:6`

#### FR-CIV-SPECIES-201

- Spec/trace:
  - `docs/design/species-sentience.md:99`
  - `docs/design/species-sentience.md:206`
  - `docs/traceability/fr-civ-species-201/fr-civ-species-201-adr.md:1`

#### FR-CIV-SPECIES-202

- Spec/trace:
  - `docs/design/species-sentience.md:100`
  - `docs/traceability/fr-civ-species-202/fr-civ-species-202-adr.md:1`
  - `docs/traceability/fr-civ-species-202/fr-civ-species-202-adr.md:6`

#### FR-CIV-SPECIES-203

- Spec/trace:
  - `docs/design/species-sentience.md:101`
  - `docs/traceability/fr-civ-species-203/fr-civ-species-203-adr.md:1`
  - `docs/traceability/fr-civ-species-203/fr-civ-species-203-adr.md:6`

#### FR-CIV-SPECIES-204

- Spec/trace:
  - `docs/design/species-sentience.md:102`
  - `docs/traceability/fr-civ-species-204/fr-civ-species-204-adr.md:1`
  - `docs/traceability/fr-civ-species-204/fr-civ-species-204-adr.md:6`

#### FR-CIV-SPECIES-205

- Spec/trace:
  - `docs/design/species-sentience.md:103`
  - `docs/traceability/fr-civ-species-205/fr-civ-species-205-adr.md:1`
  - `docs/traceability/fr-civ-species-205/fr-civ-species-205-adr.md:6`

#### FR-CIV-SPECIES-300

- Spec/trace:
  - `docs/design/species-sentience.md:120`
  - `docs/traceability/fr-civ-species-300/fr-civ-species-300-adr.md:1`
  - `docs/traceability/fr-civ-species-300/fr-civ-species-300-adr.md:6`

#### FR-CIV-SPECIES-301

- Spec/trace:
  - `docs/design/species-sentience.md:121`
  - `docs/traceability/fr-civ-species-301/fr-civ-species-301-adr.md:1`
  - `docs/traceability/fr-civ-species-301/fr-civ-species-301-adr.md:6`

#### FR-CIV-SPECIES-302

- Spec/trace:
  - `docs/design/species-sentience.md:122`
  - `docs/design/species-sentience.md:192`
  - `docs/traceability/fr-civ-species-302/fr-civ-species-302-adr.md:1`

#### FR-CIV-SPECIES-303

- Spec/trace:
  - `docs/design/species-sentience.md:123`
  - `docs/traceability/fr-civ-species-303/fr-civ-species-303-adr.md:1`
  - `docs/traceability/fr-civ-species-303/fr-civ-species-303-adr.md:6`

#### FR-CIV-SPECIES-304

- Spec/trace:
  - `docs/design/species-sentience.md:124`
  - `docs/traceability/fr-civ-species-304/fr-civ-species-304-adr.md:1`
  - `docs/traceability/fr-civ-species-304/fr-civ-species-304-adr.md:6`

#### FR-CIV-SPECIES-400

- Spec/trace:
  - `docs/design/species-sentience.md:167`
  - `docs/traceability/fr-civ-species-400/fr-civ-species-400-adr.md:1`
  - `docs/traceability/fr-civ-species-400/fr-civ-species-400-adr.md:6`

#### FR-CIV-SPECIES-401

- Spec/trace:
  - `docs/design/species-sentience.md:168`
  - `docs/traceability/fr-civ-species-401/fr-civ-species-401-adr.md:1`
  - `docs/traceability/fr-civ-species-401/fr-civ-species-401-adr.md:6`

#### FR-CIV-SPECIES-402

- Spec/trace:
  - `docs/design/species-sentience.md:169`
  - `docs/traceability/fr-civ-species-402/fr-civ-species-402-adr.md:1`
  - `docs/traceability/fr-civ-species-402/fr-civ-species-402-adr.md:6`

#### FR-CIV-SPECIES-403

- Spec/trace:
  - `docs/design/species-sentience.md:170`
  - `docs/traceability/fr-civ-species-403/fr-civ-species-403-adr.md:1`
  - `docs/traceability/fr-civ-species-403/fr-civ-species-403-adr.md:6`

#### FR-CIV-SPECIES-404

- Spec/trace:
  - `docs/design/species-sentience.md:171`
  - `docs/traceability/fr-civ-species-404/fr-civ-species-404-adr.md:1`
  - `docs/traceability/fr-civ-species-404/fr-civ-species-404-adr.md:6`

#### FR-CIV-SPECIES-405

- Spec/trace:
  - `docs/design/species-sentience.md:172`
  - `docs/traceability/fr-civ-species-405/fr-civ-species-405-adr.md:1`
  - `docs/traceability/fr-civ-species-405/fr-civ-species-405-adr.md:6`

#### FR-CIV-SPECIES-406

- Spec/trace:
  - `docs/design/species-sentience.md:173`
  - `docs/design/species-sentience.md:216`
  - `docs/traceability/fr-civ-species-406/fr-civ-species-406-adr.md:1`

#### FR-CIV-SPECIES-500

- Spec/trace:
  - `docs/design/species-sentience.md:191`
  - `docs/traceability/fr-civ-species-500/fr-civ-species-500-adr.md:1`
  - `docs/traceability/fr-civ-species-500/fr-civ-species-500-adr.md:6`

#### FR-CIV-SPECIES-501

- Spec/trace:
  - `docs/design/species-sentience.md:192`
  - `docs/traceability/fr-civ-species-501/fr-civ-species-501-adr.md:1`
  - `docs/traceability/fr-civ-species-501/fr-civ-species-501-adr.md:6`

#### FR-CIV-SPECIES-502

- Spec/trace:
  - `docs/design/species-sentience.md:193`
  - `docs/traceability/fr-civ-species-502/fr-civ-species-502-adr.md:1`
  - `docs/traceability/fr-civ-species-502/fr-civ-species-502-adr.md:6`

#### FR-CIV-SPECIES-503

- Spec/trace:
  - `docs/design/species-sentience.md:194`
  - `docs/traceability/fr-civ-species-503/fr-civ-species-503-adr.md:1`
  - `docs/traceability/fr-civ-species-503/fr-civ-species-503-adr.md:6`

#### FR-CIV-SPECIES-504

- Spec/trace:
  - `docs/design/species-sentience.md:195`
  - `docs/traceability/fr-civ-species-504/fr-civ-species-504-adr.md:1`
  - `docs/traceability/fr-civ-species-504/fr-civ-species-504-adr.md:6`

#### FR-CIV-SPECIES-505

- Spec/trace:
  - `docs/design/species-sentience.md:196`
  - `docs/traceability/fr-civ-species-505/fr-civ-species-505-adr.md:1`
  - `docs/traceability/fr-civ-species-505/fr-civ-species-505-adr.md:6`


### Epic `FR-UX` — 22 IDs

#### FR-UX-006

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:928`
  - `docs/traceability/fr-ux-006/fr-ux-006-adr.md:1`
  - `docs/traceability/fr-ux-006/fr-ux-006-adr.md:6`

#### FR-UX-007

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:931`
  - `docs/traceability/fr-ux-007/fr-ux-007-adr.md:1`
  - `docs/traceability/fr-ux-007/fr-ux-007-adr.md:6`

#### FR-UX-008

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:934`
  - `docs/traceability/fr-ux-008/fr-ux-008-adr.md:1`
  - `docs/traceability/fr-ux-008/fr-ux-008-adr.md:6`

#### FR-UX-009

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:939`
  - `docs/traceability/fr-ux-009/fr-ux-009-adr.md:1`
  - `docs/traceability/fr-ux-009/fr-ux-009-adr.md:6`

#### FR-UX-010

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:942`
  - `docs/traceability/fr-ux-010/fr-ux-010-adr.md:1`
  - `docs/traceability/fr-ux-010/fr-ux-010-adr.md:6`

#### FR-UX-011

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:945`
  - `docs/traceability/fr-ux-011/fr-ux-011-adr.md:1`
  - `docs/traceability/fr-ux-011/fr-ux-011-adr.md:6`

#### FR-UX-012

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:948`
  - `docs/traceability/fr-ux-012/fr-ux-012-adr.md:1`
  - `docs/traceability/fr-ux-012/fr-ux-012-adr.md:6`

#### FR-UX-013

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:951`
  - `docs/traceability/fr-ux-013/fr-ux-013-adr.md:1`
  - `docs/traceability/fr-ux-013/fr-ux-013-adr.md:6`

#### FR-UX-014

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:956`
  - `docs/traceability/fr-ux-014/fr-ux-014-adr.md:1`
  - `docs/traceability/fr-ux-014/fr-ux-014-adr.md:6`

#### FR-UX-015

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:959`
  - `docs/traceability/fr-ux-015/fr-ux-015-adr.md:1`
  - `docs/traceability/fr-ux-015/fr-ux-015-adr.md:6`

#### FR-UX-016

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:962`
  - `docs/traceability/fr-ux-016/fr-ux-016-adr.md:1`
  - `docs/traceability/fr-ux-016/fr-ux-016-adr.md:6`

#### FR-UX-017

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:965`
  - `docs/traceability/fr-ux-017/fr-ux-017-adr.md:1`
  - `docs/traceability/fr-ux-017/fr-ux-017-adr.md:6`

#### FR-UX-018

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:970`
  - `docs/traceability/fr-ux-018/fr-ux-018-adr.md:1`
  - `docs/traceability/fr-ux-018/fr-ux-018-adr.md:6`

#### FR-UX-019

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:973`
  - `docs/traceability/fr-ux-019/fr-ux-019-adr.md:1`
  - `docs/traceability/fr-ux-019/fr-ux-019-adr.md:6`

#### FR-UX-020

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:976`
  - `docs/traceability/fr-ux-020/fr-ux-020-adr.md:1`
  - `docs/traceability/fr-ux-020/fr-ux-020-adr.md:6`

#### FR-UX-021

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:979`
  - `docs/traceability/fr-ux-021/fr-ux-021-adr.md:1`
  - `docs/traceability/fr-ux-021/fr-ux-021-adr.md:6`

#### FR-UX-022

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:982`
  - `docs/traceability/fr-ux-022/fr-ux-022-adr.md:1`
  - `docs/traceability/fr-ux-022/fr-ux-022-adr.md:6`

#### FR-UX-023

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:987`
  - `docs/traceability/fr-ux-023/fr-ux-023-adr.md:1`
  - `docs/traceability/fr-ux-023/fr-ux-023-adr.md:6`

#### FR-UX-024

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:990`
  - `docs/traceability/fr-ux-024/fr-ux-024-adr.md:1`
  - `docs/traceability/fr-ux-024/fr-ux-024-adr.md:6`

#### FR-UX-025

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:993`
  - `docs/traceability/fr-ux-025/fr-ux-025-adr.md:1`
  - `docs/traceability/fr-ux-025/fr-ux-025-adr.md:6`

#### FR-UX-026

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:996`
  - `docs/traceability/fr-ux-026/fr-ux-026-adr.md:1`
  - `docs/traceability/fr-ux-026/fr-ux-026-adr.md:6`

#### FR-UX-027

- Spec/trace:
  - `docs/models/civ-sim/USER_SPEC.md:999`
  - `docs/traceability/fr-ux-027/fr-ux-027-adr.md:1`
  - `docs/traceability/fr-ux-027/fr-ux-027-adr.md:6`


### Epic `NFR-CIV-LEGENDS-PERF` — 1 IDs

#### NFR-CIV-LEGENDS-PERF-01

- Spec/trace:
  - `docs/design/legends-engine.md:451`
  - `docs/traceability/index.md:1172`
  - `docs/traceability/nfr-civ-legends-perf-01/nfr-civ-legends-perf-01-research.md:1`


### Epic `NFR-O` — 6 IDs

#### NFR-O-01

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2088`
  - `docs/traceability/index.md:1211`
  - `docs/traceability/nfr-o-01/nfr-o-01-spec.md:1`

#### NFR-O-02

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2089`
  - `docs/traceability/index.md:1212`
  - `docs/traceability/nfr-o-02/nfr-o-02-spec.md:1`

#### NFR-O-03

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2090`
  - `docs/traceability/index.md:1213`
  - `docs/traceability/nfr-o-03/nfr-o-03-spec.md:1`

#### NFR-O-04

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2091`
  - `docs/traceability/index.md:1214`
  - `docs/traceability/nfr-o-04/nfr-o-04-spec.md:1`

#### NFR-O-05

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2092`
  - `docs/traceability/index.md:1215`
  - `docs/traceability/nfr-o-05/nfr-o-05-spec.md:1`

#### NFR-O-06

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2093`
  - `docs/traceability/index.md:1216`
  - `docs/traceability/nfr-o-06/nfr-o-06-spec.md:1`

