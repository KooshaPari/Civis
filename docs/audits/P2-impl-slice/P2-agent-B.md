# P2 agent-B

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


## Your slice: 36 IDs in 8 epics


### Epic `FR-CIV-ACCESS` — 2 IDs

#### FR-CIV-ACCESS-010

- Spec/trace:
  - `docs/adr/ADR-021-accessibility-and-l10n-strategy.md:47`

#### FR-CIV-ACCESS-020

- Spec/trace:
  - `docs/adr/ADR-021-accessibility-and-l10n-strategy.md:48`


### Epic `FR-CIV-EMERGENCE` — 15 IDs

#### FR-CIV-EMERGENCE-100

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:150`

#### FR-CIV-EMERGENCE-111

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:151`

#### FR-CIV-EMERGENCE-119

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:152`

#### FR-CIV-EMERGENCE-124

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:153`

#### FR-CIV-EMERGENCE-132

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:154`

#### FR-CIV-EMERGENCE-141

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:155`

#### FR-CIV-EMERGENCE-144

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:156`

#### FR-CIV-EMERGENCE-151

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:157`

#### FR-CIV-EMERGENCE-168

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:158`

#### FR-CIV-EMERGENCE-198

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:159`

#### FR-CIV-EMERGENCE-221

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:160`

#### FR-CIV-EMERGENCE-236

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:161`

#### FR-CIV-EMERGENCE-239

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:162`

#### FR-CIV-EMERGENCE-241

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:163`

#### FR-CIV-EMERGENCE-249

- Spec/trace:
  - `docs/traceability/emergent-systems-tracelinks.md:164`


### Epic `FR-CIV-LEGENDS-PERF` — 1 IDs

#### FR-CIV-LEGENDS-PERF-01

- Spec/trace:
  - `docs/traceability/fr-emergence-matrix.md:260`


### Epic `FR-CIV-SAVE` — 2 IDs

#### FR-CIV-SAVE-003

- Spec/trace:
  - `docs/traceability/civis-tracelinks.md:66`
  - `docs/traceability/fr-civ-save-003/fr-civ-save-003-adr.md:1`
  - `docs/traceability/fr-civ-save-003/fr-civ-save-003-adr.md:6`

#### FR-CIV-SAVE-004

- Spec/trace:
  - `docs/traceability/civis-tracelinks.md:67`
  - `docs/traceability/fr-civ-save-004/fr-civ-save-004-adr.md:1`
  - `docs/traceability/fr-civ-save-004/fr-civ-save-004-adr.md:6`


### Epic `FR-CIV-UI` — 3 IDs

#### FR-CIV-UI-001

- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:99`
  - `docs/guides/voxel-emergent-vision-and-migration.md:155`
  - `docs/guides/voxel-emergent-vision-and-migration.md:159`

#### FR-CIV-UI-002

- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:99`
  - `docs/guides/voxel-emergent-vision-and-migration.md:160`
  - `docs/traceability/fr-civ-ui-002/fr-civ-ui-002-adr.md:1`

#### FR-CIV-UI-003

- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:99`
  - `docs/guides/voxel-emergent-vision-and-migration.md:155`
  - `docs/guides/voxel-emergent-vision-and-migration.md:161`


### Epic `NFR-CIV-ACC` — 4 IDs

#### NFR-CIV-ACC-001

- Spec/trace:
  - `agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md:108`
  - `docs/guides/voxel-emergent-vision-and-migration.md:173`
  - `docs/reference/non-functional-requirements.md:352`

#### NFR-CIV-ACC-002

- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:99`
  - `docs/reference/non-functional-requirements.md:366`
  - `docs/reference/non-functional-requirements.md:574`

#### NFR-CIV-ACC-003

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:380`
  - `docs/reference/non-functional-requirements.md:575`
  - `docs/traceability/fr-nfr-matrix.md:84`

#### NFR-CIV-ACC-004

- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:99`
  - `docs/reference/non-functional-requirements.md:394`
  - `docs/reference/non-functional-requirements.md:576`


### Epic `NFR-CIV-PORT` — 3 IDs

#### NFR-CIV-PORT-001

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:51`
  - `docs/reference/non-functional-requirements.md:154`
  - `docs/reference/non-functional-requirements.md:410`

#### NFR-CIV-PORT-002

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:431`
  - `docs/reference/non-functional-requirements.md:578`
  - `docs/reference/non-functional-requirements.md:610`

#### NFR-CIV-PORT-003

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:445`
  - `docs/reference/non-functional-requirements.md:527`
  - `docs/reference/non-functional-requirements.md:579`


### Epic `NFR-S` — 6 IDs

#### NFR-S-01

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2066`
  - `docs/traceability/index.md:1231`
  - `docs/traceability/nfr-s-01/nfr-s-01-spec.md:1`

#### NFR-S-02

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2067`
  - `docs/traceability/index.md:1232`
  - `docs/traceability/nfr-s-02/nfr-s-02-spec.md:1`

#### NFR-S-03

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2068`
  - `docs/traceability/index.md:1233`
  - `docs/traceability/nfr-s-03/nfr-s-03-spec.md:1`

#### NFR-S-04

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2069`
  - `docs/traceability/index.md:1234`
  - `docs/traceability/nfr-s-04/nfr-s-04-spec.md:1`

#### NFR-S-05

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2070`
  - `docs/traceability/index.md:1235`
  - `docs/traceability/nfr-s-05/nfr-s-05-spec.md:1`

#### NFR-S-06

- Spec/trace:
  - `docs/models/civ-sim/TECHNICAL_SPEC.md:2071`
  - `docs/traceability/index.md:1236`
  - `docs/traceability/nfr-s-06/nfr-s-06-spec.md:1`

