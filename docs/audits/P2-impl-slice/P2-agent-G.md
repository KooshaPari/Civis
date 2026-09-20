# P2 agent-G

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


### Epic `FR-CIV-BEVY` — 8 IDs

#### FR-CIV-BEVY-013

- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:125`
  - `docs/traceability/fr-3d-matrix.md:178`
  - `docs/traceability/full-traceability-matrix.md:307`

#### FR-CIV-BEVY-014

- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:82`
  - `docs/traceability/fr-3d-matrix.md:179`
  - `docs/traceability/full-traceability-matrix.md:308`

#### FR-CIV-BEVY-015

- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:127`
  - `docs/traceability/fr-3d-matrix.md:180`
  - `docs/traceability/full-traceability-matrix.md:309`

#### FR-CIV-BEVY-017

- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:129`
  - `docs/traceability/fr-3d-matrix.md:182`
  - `docs/traceability/full-traceability-matrix.md:311`

#### FR-CIV-BEVY-018

- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:130`
  - `docs/traceability/fr-3d-matrix.md:183`
  - `docs/traceability/full-traceability-matrix.md:312`

#### FR-CIV-BEVY-019

- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:131`
  - `docs/traceability/fr-3d-matrix.md:184`
  - `docs/traceability/full-traceability-matrix.md:313`

#### FR-CIV-BEVY-020

- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:132`
  - `docs/traceability/fr-3d-matrix.md:185`
  - `docs/traceability/full-traceability-matrix.md:314`

#### FR-CIV-BEVY-021

- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:83`
  - `docs/development-guide/p-w1-kickoff.md:133`
  - `docs/traceability/fr-3d-matrix.md:186`


### Epic `FR-CIV-L10N` — 4 IDs

#### FR-CIV-L10N-010

- Spec/trace:
  - `docs/adr/ADR-021-accessibility-and-l10n-strategy.md:49`

#### FR-CIV-L10N-020

- Spec/trace:
  - `docs/adr/ADR-021-accessibility-and-l10n-strategy.md:50`

#### FR-CIV-L10N-030

- Spec/trace:
  - `docs/adr/ADR-021-accessibility-and-l10n-strategy.md:51`

#### FR-CIV-L10N-040

- Spec/trace:
  - `docs/adr/ADR-021-accessibility-and-l10n-strategy.md:52`


### Epic `FR-CIV-REL` — 1 IDs

#### FR-CIV-REL-004

- Spec/trace:
  - `docs/traceability/fr-emergence-matrix.md:79`


### Epic `FR-CIV-TACTICS` — 1 IDs

#### FR-CIV-TACTICS-032

- Spec/trace:
  - `docs/development-guide/p-w1-kickoff.md:31`
  - `docs/traceability/fr-3d-matrix.md:119`
  - `docs/traceability/full-traceability-matrix.md:227`


### Epic `NFR-C` — 7 IDs

#### NFR-C-01

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2041`
  - `docs/traceability/index.md:1151`
  - `docs/traceability/nfr-c-01/nfr-c-01-spec.md:1`

#### NFR-C-02

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2042`
  - `docs/traceability/index.md:1152`
  - `docs/traceability/nfr-c-02/nfr-c-02-spec.md:1`

#### NFR-C-03

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2043`
  - `docs/traceability/index.md:1153`
  - `docs/traceability/nfr-c-03/nfr-c-03-spec.md:1`

#### NFR-C-04

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2044`
  - `docs/traceability/index.md:1154`
  - `docs/traceability/nfr-c-04/nfr-c-04-spec.md:1`

#### NFR-C-05

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2045`
  - `docs/traceability/index.md:1155`
  - `docs/traceability/nfr-c-05/nfr-c-05-spec.md:1`

#### NFR-C-06

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2046`
  - `docs/traceability/index.md:1156`
  - `docs/traceability/nfr-c-06/nfr-c-06-spec.md:1`

#### NFR-C-07

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2047`
  - `docs/traceability/index.md:1157`
  - `docs/traceability/nfr-c-07/nfr-c-07-spec.md:1`


### Epic `NFR-CIV-MAINT` — 6 IDs

#### NFR-CIV-MAINT-001

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:260`
  - `docs/reference/non-functional-requirements.md:461`
  - `docs/reference/non-functional-requirements.md:499`

#### NFR-CIV-MAINT-002

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:475`
  - `docs/reference/non-functional-requirements.md:541`
  - `docs/reference/non-functional-requirements.md:581`

#### NFR-CIV-MAINT-003

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:489`
  - `docs/reference/non-functional-requirements.md:582`
  - `docs/traceability/fr-nfr-matrix.md:105`

#### NFR-CIV-MAINT-004

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:503`
  - `docs/reference/non-functional-requirements.md:583`
  - `docs/reference/non-functional-requirements.md:607`

#### NFR-CIV-MAINT-005

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:517`
  - `docs/reference/non-functional-requirements.md:584`
  - `docs/reference/non-functional-requirements.md:608`

#### NFR-CIV-MAINT-006

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:531`
  - `docs/reference/non-functional-requirements.md:585`
  - `docs/traceability/fr-nfr-matrix.md:108`


### Epic `NFR-P` — 8 IDs

#### NFR-P-01

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2053`
  - `docs/traceability/index.md:1217`
  - `docs/traceability/nfr-p-01/nfr-p-01-spec.md:1`

#### NFR-P-02

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2054`
  - `docs/traceability/index.md:1218`
  - `docs/traceability/nfr-p-02/nfr-p-02-spec.md:1`

#### NFR-P-03

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2055`
  - `docs/traceability/index.md:1219`
  - `docs/traceability/nfr-p-03/nfr-p-03-spec.md:1`

#### NFR-P-04

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2056`
  - `docs/traceability/index.md:1220`
  - `docs/traceability/nfr-p-04/nfr-p-04-spec.md:1`

#### NFR-P-05

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2057`
  - `docs/traceability/index.md:1221`
  - `docs/traceability/nfr-p-05/nfr-p-05-spec.md:1`

#### NFR-P-06

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2058`
  - `docs/traceability/index.md:1222`
  - `docs/traceability/nfr-p-06/nfr-p-06-spec.md:1`

#### NFR-P-07

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2059`
  - `docs/traceability/index.md:1223`
  - `docs/traceability/nfr-p-07/nfr-p-07-spec.md:1`

#### NFR-P-08

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2060`
  - `docs/traceability/index.md:1224`
  - `docs/traceability/nfr-p-08/nfr-p-08-spec.md:1`

