# P2 agent-D

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


## Your slice: 25 IDs in 7 epics


### Epic `FR-CIV-ASSET-MANI` — 2 IDs

#### FR-CIV-ASSET-MANI-001

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3212`
  - `docs/traceability/fr-civ-asset-mani-001/fr-civ-asset-mani-001-adr.md:1`
  - `docs/traceability/fr-civ-asset-mani-001/fr-civ-asset-mani-001-adr.md:6`

#### FR-CIV-ASSET-MANI-002

- Spec/trace:
  - `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3213`
  - `docs/traceability/fr-civ-asset-mani-002/fr-civ-asset-mani-002-adr.md:1`
  - `docs/traceability/fr-civ-asset-mani-002/fr-civ-asset-mani-002-adr.md:6`


### Epic `FR-CIV-GEO` — 10 IDs

#### FR-CIV-GEO-001

- Spec/trace:
  - `docs/reference/FR_TRACKER.md:22`
  - `docs/reports/STATUS_REPORT.md:95`
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2024`

#### FR-CIV-GEO-002

- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2025`
  - `docs/traceability/fr-civ-geo-002/fr-civ-geo-002-adr.md:1`
  - `docs/traceability/fr-civ-geo-002/fr-civ-geo-002-adr.md:6`

#### FR-CIV-GEO-003

- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2026`
  - `docs/traceability/fr-civ-geo-003/fr-civ-geo-003-adr.md:1`
  - `docs/traceability/fr-civ-geo-003/fr-civ-geo-003-adr.md:6`

#### FR-CIV-GEO-004

- Spec/trace:
  - `docs/reports/STATUS_REPORT.md:96`
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2027`
  - `docs/traceability/fr-civ-geo-004/fr-civ-geo-004-adr.md:1`

#### FR-CIV-GEO-005

- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2028`
  - `docs/traceability/fr-civ-geo-005/fr-civ-geo-005-adr.md:1`
  - `docs/traceability/fr-civ-geo-005/fr-civ-geo-005-adr.md:6`

#### FR-CIV-GEO-006

- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2029`
  - `docs/traceability/fr-civ-geo-006/fr-civ-geo-006-adr.md:1`
  - `docs/traceability/fr-civ-geo-006/fr-civ-geo-006-adr.md:6`

#### FR-CIV-GEO-007

- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2030`
  - `docs/traceability/fr-civ-geo-007/fr-civ-geo-007-adr.md:1`
  - `docs/traceability/fr-civ-geo-007/fr-civ-geo-007-adr.md:6`

#### FR-CIV-GEO-008

- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2031`
  - `docs/traceability/fr-civ-geo-008/fr-civ-geo-008-adr.md:1`
  - `docs/traceability/fr-civ-geo-008/fr-civ-geo-008-adr.md:6`

#### FR-CIV-GEO-009

- Spec/trace:
  - `docs/specs/CIV-0300-rts-ui-ux-spec.md:2032`
  - `docs/traceability/fr-civ-geo-009/fr-civ-geo-009-adr.md:1`
  - `docs/traceability/fr-civ-geo-009/fr-civ-geo-009-adr.md:6`

#### FR-CIV-GEO-010

- Spec/trace:
  - `docs/specs/CIV-0101-two-zoom-lod-v1.md:1580`
  - `docs/specs/CIV-0101-two-zoom-lod-v1.md:1582`
  - `docs/specs/CIV-0101-two-zoom-lod-v1.md:1584`


### Epic `FR-CIV-LIFE` — 1 IDs

#### FR-CIV-LIFE-004

- Spec/trace:
  - `docs/design/civ-003-emergent-lifecycle.md:101`


### Epic `FR-CIV-SOCIAL-001-INSTITUTIONS` — 1 IDs

#### FR-CIV-SOCIAL-001-INSTITUTIONS

- Spec/trace:
  - `agileplus-specs/civ-021-recovered-requirements/spec.md:225`
  - `PLAN.md:147`
  - `PLAN.md:148`


### Epic `FR-CIV-WEB` — 4 IDs

#### FR-CIV-WEB-000

- Spec/trace:
  - `docs/development-guide/fr-web-spectator.md:3`
  - `docs/development-guide/fr-web-spectator.md:29`
  - `docs/development-guide/pr-296-body.md:20`

#### FR-CIV-WEB-001

- Spec/trace:
  - `docs/development-guide/fr-web-spectator.md:30`
  - `docs/traceability/fr-civ-web-001/fr-civ-web-001-adr.md:1`
  - `docs/traceability/fr-civ-web-001/fr-civ-web-001-adr.md:6`

#### FR-CIV-WEB-004

- Spec/trace:
  - `docs/development-guide/fr-web-spectator.md:33`
  - `docs/traceability/fr-civ-web-004/fr-civ-web-004-adr.md:1`
  - `docs/traceability/fr-civ-web-004/fr-civ-web-004-adr.md:6`

#### FR-CIV-WEB-005

- Spec/trace:
  - `docs/development-guide/fr-web-spectator.md:34`
  - `docs/traceability/fr-civ-web-005/fr-civ-web-005-adr.md:1`
  - `docs/traceability/fr-civ-web-005/fr-civ-web-005-adr.md:6`


### Epic `NFR-CIV-DEV-HYGIENE` — 1 IDs

#### NFR-CIV-DEV-HYGIENE-001

- Spec/trace:
  - `docs/ops/history-purge-plan.md:4`
  - `docs/traceability/index.md:1169`
  - `docs/traceability/nfr-civ-dev-hygiene-001/nfr-civ-dev-hygiene-001-spec.md:1`


### Epic `NFR-CIV-SCALE` — 6 IDs

#### NFR-CIV-SCALE-003

- Spec/trace:
  - `docs/reference/non-functional-requirements.md:220`
  - `docs/reference/non-functional-requirements.md:564`
  - `docs/reference/non-functional-requirements.md:602`

#### NFR-CIV-SCALE-004

- Spec/trace:
  - `docs/guides/voxel-emergent-vision-and-migration.md:172`
  - `docs/traceability/index.md:1201`
  - `docs/traceability/nfr-civ-scale-004/nfr-civ-scale-004-adr.md:1`

#### NFR-CIV-SCALE-900

- Spec/trace:
  - `docs/agileplus/epics/civ-w5-scale.md:9`
  - `docs/agileplus/epics/civ-w5-scale.md:22`
  - `docs/agileplus/README.md:24`

#### NFR-CIV-SCALE-902

- Spec/trace:
  - `docs/agileplus/epics/civ-w5-scale.md:11`
  - `docs/agileplus/epics/civ-w5-scale.md:24`
  - `docs/agileplus/README.md:24`

#### NFR-CIV-SCALE-910

- Spec/trace:
  - `docs/agileplus/epics/civ-w5-scale.md:12`
  - `docs/agileplus/epics/civ-w5-scale.md:25`
  - `docs/agileplus/README.md:24`

#### NFR-CIV-SCALE-920

- Spec/trace:
  - `docs/agileplus/epics/civ-w5-scale.md:13`
  - `docs/agileplus/epics/civ-w5-scale.md:26`
  - `docs/agileplus/README.md:24`

