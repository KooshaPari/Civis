# Phantom-ID Triage — Batch 7 (2026-06-11)

**Source:** `docs/audits/fr-matrix.json` (1181 IDs) — filtered to **CODE-ONLY-no-spec** (786 rows), sorted by reference count (`len(code_refs) + len(test_refs)`)

Skipped: rows already covered in `docs/audits/phantom-triage-batch1.md` through
`docs/audits/phantom-triage-batch6.md` (~550 rows).

**Scope for this batch:** next **75** rows, **positions 551–625** (starting at `FR-CIV-VEHICLE-030`, ending at `FR-SAVE-011`).

**Verdict taxonomy**
- **REAL** — matrix row maps to an implemented feature/capability with an existing or discoverable spec/home.
- **REAL +stub** — implemented behavior exists but no existing spec anchor; add one-line stub to `agileplus-specs/civ-021-recovered-requirements/spec.md`.
- **STALE** — trace/work-log-only row with no dedicated implementation.
- **RENAME** — maps to an existing differently named spec'd ID.

## Summary

- **REAL:** **71**
- **REAL +stub:** **0**
- **STALE:** **2**
- **RENAME:** **2**

## Per-row verdicts

| # | FR ID | Verdict | Evidence (file:line) |
|---|-------|---------|----------------------|
| 1 | `FR-CIV-VEHICLE-030` | **REAL** | `docs/design/vehicles-logistics.md:222` |
| 2 | `FR-CIV-VEHICLE-040` | **REAL** | `docs/design/vehicles-logistics.md:277` |
| 3 | `FR-CIV-VEHICLE-041` | **REAL** | `docs/design/vehicles-logistics.md:280` |
| 4 | `FR-CIV-VEHICLE-042` | **REAL** | `docs/design/vehicles-logistics.md:282` |
| 5 | `FR-CIV-VEHICLE-043` | **REAL** | `docs/design/vehicles-logistics.md:284` |
| 6 | `FR-CIV-VEHICLE-044` | **REAL** | `docs/design/vehicles-logistics.md:286` |
| 7 | `FR-CIV-VEHICLE-045` | **REAL** | `docs/design/vehicles-logistics.md:288` |
| 8 | `FR-CIV-VEHICLE-046` | **REAL** | `docs/design/vehicles-logistics.md:290` |
| 9 | `FR-CIV-VEHICLE-047` | **REAL** | `docs/design/vehicles-logistics.md:292` |
| 10 | `FR-CIV-VEHICLE-050` | **REAL** | `docs/design/vehicles-logistics.md:309` |
| 11 | `FR-CIV-VEHICLE-060` | **REAL** | `docs/design/vehicles-logistics.md:342` |
| 12 | `FR-CIV-VOXEL-005` | **REAL** | `docs/traceability/civis-tracelinks.md:1`, `crates/voxel/src/lib.rs:249`, `docs/worklogs/2026-05-22-civis-3d-kickoff.md:73` |
| 13 | `FR-CIV-VOXEL-006` | **REAL** | `crates/engine/src/engine.rs:2779` |
| 14 | `FR-CIV-VOXEL-007` | **REAL** | `crates/engine/src/engine.rs:2828` |
| 15 | `FR-CIV-VOXEL-020` | **REAL** | `clients/bevy-ref/src/bin/standalone.rs:189`, `crates/voxel/src/stream.rs:32` |
| 16 | `FR-CIV-VOXEL-021` | **REAL** | `crates/voxel/src/worldgen.rs:540`, `docs/guides/voxel-emergent-vision-and-migration.md:124` |
| 17 | `FR-CIV-VOXEL-022` | **REAL** | `docs/guides/voxel-emergent-vision-and-migration.md:125`, `docs/guides/voxel-emergent-vision-and-migration.md:183` |
| 18 | `FR-CIV-VOXEL-023` | **REAL** | `docs/guides/voxel-emergent-vision-and-migration.md:126` |
| 19 | `FR-CIV-VOXEL-024` | **REAL** | `docs/guides/voxel-emergent-vision-and-migration.md:127` |
| 20 | `FR-CIV-VOXEL-025` | **REAL** | `docs/guides/voxel-emergent-vision-and-migration.md:128` |
| 21 | `FR-CIV-VOXEL-030` | **REAL** | `docs/guides/voxel-emergent-vision-and-migration.md:129` |
| 22 | `FR-CIV-VOXEL-031` | **REAL** | `docs/guides/voxel-emergent-vision-and-migration.md:130` |
| 23 | `FR-CIV-VOXEL-032` | **REAL** | `docs/guides/voxel-emergent-vision-and-migration.md:131` |
| 24 | `FR-CIV-WAR-001-UNITS` | **RENAME** | `PLAN.md:203`, `PLAN.md:204` |
| 25 | `FR-CIV-WAR-002-COMBAT` | **RENAME** | `PLAN.md:205`, `PLAN.md:206` |
| 26 | `FR-CIV-WAR-010` | **REAL** | `docs/design/warfare.md:77`, `docs/design/warfare.md:191` |
| 27 | `FR-CIV-WAR-011` | **REAL** | `docs/design/warfare.md:83`, `docs/design/warfare.md:192` |
| 28 | `FR-CIV-WAR-012` | **REAL** | `docs/design/warfare.md:86`, `docs/design/warfare.md:193` |
| 29 | `FR-CIV-WAR-013` | **REAL** | `docs/design/warfare.md:89`, `docs/design/warfare.md:194` |
| 30 | `FR-CIV-WAR-020` | **REAL** | `docs/design/warfare.md:106`, `docs/design/warfare.md:195` |
| 31 | `FR-CIV-WAR-021` | **REAL** | `docs/design/warfare.md:111`, `docs/design/warfare.md:196` |
| 32 | `FR-CIV-WAR-022` | **REAL** | `docs/design/warfare.md:114`, `docs/design/warfare.md:197` |
| 33 | `FR-CIV-WAR-030` | **REAL** | `docs/design/warfare.md:124`, `docs/design/warfare.md:198` |
| 34 | `FR-CIV-WAR-040` | **REAL** | `docs/design/warfare.md:144`, `docs/design/warfare.md:199` |
| 35 | `FR-CIV-WAR-041` | **REAL** | `docs/design/warfare.md:147`, `docs/design/warfare.md:200` |
| 36 | `FR-CIV-WAR-042` | **REAL** | `docs/design/warfare.md:150`, `docs/design/warfare.md:201` |
| 37 | `FR-CIV-WEB-000` | **REAL** | `docs/development-guide/fr-web-spectator.md:3`, `docs/development-guide/fr-web-spectator.md:29` |
| 38 | `FR-CIV-WEB-001` | **REAL** | `docs/development-guide/fr-web-spectator.md:30` |
| 39 | `FR-CIV-WEB-002` | **REAL** | `docs/development-guide/fr-web-spectator.md:31`, `web/dashboard/src/lib/civisServer.ts:1` |
| 40 | `FR-CIV-WEB-003` | **REAL** | `crates/engine/src/spectator.rs:1`, `docs/development-guide/fr-web-spectator.md:32` |
| 41 | `FR-CIV-WEB-004` | **REAL** | `docs/development-guide/fr-web-spectator.md:33` |
| 42 | `FR-CIV-WEB-005` | **REAL** | `docs/development-guide/fr-web-spectator.md:34` |
| 43 | `FR-CIV-WEB-006` | **REAL** | `docs/development-guide/fr-web-spectator.md:35`, `web/dashboard/src/lib/frame3d.ts:1` |
| 44 | `FR-CIV-WEB-007` | **REAL** | `docs/development-guide/fr-web-spectator.md:36`, `web/dashboard/src/babylon_scene.tsx:15` |
| 45 | `FR-CIV-WEB-008` | **REAL** | `docs/development-guide/fr-web-spectator.md:37`, `web/dashboard/src/lib/authoring.ts:95` |
| 46 | `FR-DET-001` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:439` |
| 47 | `FR-DET-002` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:290`, `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:440` |
| 48 | `FR-DET-003` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:441` |
| 49 | `FR-DET-004` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:442` |
| 50 | `FR-DET-005` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:443` |
| 51 | `FR-DET-006` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:342`, `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:444` |
| 52 | `FR-DET-007` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:445`, `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:451` |
| 53 | `FR-ECO-001` | **REAL** | `docs/specs/CIV-0100-economy-v1.md:1651` |
| 54 | `FR-ECO-002` | **REAL** | `docs/specs/CIV-0100-economy-v1.md:1662` |
| 55 | `FR-ECO-003` | **REAL** | `docs/specs/CIV-0100-economy-v1.md:1682` |
| 56 | `FR-ECO-004` | **REAL** | `docs/specs/CIV-0100-economy-v1.md:1696` |
| 57 | `FR-ECO-005` | **REAL** | `docs/specs/CIV-0100-economy-v1.md:1719` |
| 58 | `FR-ECO-006` | **REAL** | `docs/specs/CIV-0100-economy-v1.md:1739` |
| 59 | `FR-ECO-007` | **REAL** | `docs/specs/CIV-0100-economy-v1.md:1766` |
| 60 | `FR-ECO-008` | **REAL** | `docs/specs/CIV-0100-economy-v1.md:1792` |
| 61 | `FR-ECO-009` | **REAL** | `docs/specs/CIV-0100-economy-v1.md:1821` |
| 62 | `FR-ECO-010` | **REAL** | `docs/specs/CIV-0100-economy-v1.md:1845` |
| 63 | `FR-GUARD-001` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1260` |
| 64 | `FR-GUARD-002` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1337` |
| 65 | `FR-INT-001` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1739` |
| 66 | `FR-MET-001` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1174`, `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1203` |
| 67 | `FR-PHENO-VOXEL-CUBIC-001` | **STALE** | `docs/worklogs/2026-05-22-civis-3d-kickoff.md:71` |
| 68 | `FR-PHENO-VOXEL-WORLD-001` | **STALE** | `docs/worklogs/2026-05-22-civis-3d-kickoff.md:70` |
| 69 | `FR-REP-001` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:489`, `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:532` |
| 70 | `FR-SAVE-006` | **REAL** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2805`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2943` |
| 71 | `FR-SAVE-007` | **REAL** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2806`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2943` |
| 72 | `FR-SAVE-008` | **REAL** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2807`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2949` |
| 73 | `FR-SAVE-009` | **REAL** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2808` |
| 74 | `FR-SAVE-010` | **REAL** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2809`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2955` |
| 75 | `FR-SAVE-011` | **REAL** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2810`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2963` |
