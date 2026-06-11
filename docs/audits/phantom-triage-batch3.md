# Phantom-ID Triage — Batch 3 (2026-06-11)

**Source:** `docs/audits/fr-matrix.json` (1181 IDs) — filtered to **CODE-ONLY-no-spec** (786 rows),
sorted by reference count (`len(code_refs) + len(test_refs)`), next **75** taken.

**Covered window:** positions **251–325** (starting at `FR-SAVE-011`, ending at `FR-CIV-3D-011`).

## Verdict taxonomy
- **REAL-REQUIREMENT** — matrix row references a concrete spec/code capability.
  - `COVERED` when the FR ID is already named in an existing spec artifact.
  - `UNCOVERED` when no existing spec exists and a one-line stub should be appended to
    `agileplus-specs/civ-021-recovered-requirements/spec.md`.
- **STALE-ID** — matrix row is a trace artifact / runner shim with no dedicated
  implementation-level requirement.
- **RENAME** — matrix ID maps onto an existing ID under naming drift.

## Summary

- **REAL-REQUIREMENT:** `74`
- **STALE-ID:** `1`
- **RENAME:** `0`
- **NEW stubs appended:** `0`

## Per-row verdicts

| #  | FR ID | Verdict | Evidence (file:line) |
|----|-------|---------|----------------------|
| 1  | `FR-SAVE-011` | **REAL-REQUIREMENT** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2810`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2963` |
| 2  | `FR-SAVE-012` | **REAL-REQUIREMENT** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2811`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2963` |
| 3  | `FR-SAVE-013` | **REAL-REQUIREMENT** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2812`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2969` |
| 4  | `FR-SAVE-014` | **REAL-REQUIREMENT** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2813`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2969` |
| 5  | `FR-SAVE-015` | **REAL-REQUIREMENT** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2814`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2969` |
| 6  | `FR-SAVE-016` | **REAL-REQUIREMENT** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2815`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2977` |
| 7  | `FR-SAVE-017` | **REAL-REQUIREMENT** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2816`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2977` |
| 8  | `FR-SAVE-018` | **REAL-REQUIREMENT** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2817`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2977` |
| 9  | `FR-SAVE-019` | **REAL-REQUIREMENT** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2818`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2977` |
| 10 | `FR-SAVE-021` | **REAL-REQUIREMENT** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2820`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2987` |
| 11 | `FR-SAVE-022` | **REAL-REQUIREMENT** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2821`, `docs/specs/CIV-1000-save-load-persistence-spec.md:2993` |
| 12 | `FR-SAVE-023` | **REAL-REQUIREMENT** | `docs/specs/CIV-1000-save-load-persistence-spec.md:2822`, `docs/specs/CIV-1000-save-load-persistence-spec.md:3001` |
| 13 | `FR-SOC-CIV-001` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4326`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4570` |
| 14 | `FR-SOC-CIV-002` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4355`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4571` |
| 15 | `FR-SOC-COH-001` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1533`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1986` |
| 16 | `FR-SOC-COH-002` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1546`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1987` |
| 17 | `FR-SOC-COH-003` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1559`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1988` |
| 18 | `FR-SOC-COH-004` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1571`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1989` |
| 19 | `FR-SOC-DET-002` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1516`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1985` |
| 20 | `FR-SOC-FAC-001` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4277`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4568` |
| 21 | `FR-SOC-FAC-002` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4300`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4569` |
| 22 | `FR-SOC-HLT-001` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1643`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1994` |
| 23 | `FR-SOC-HLT-002` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1657`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1995` |
| 24 | `FR-SOC-HLT-003` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1667`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1996` |
| 25 | `FR-SOC-HLT-004` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1675`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1997` |
| 26 | `FR-SOC-HLT-005` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4412`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4574` |
| 27 | `FR-SOC-IDE-001` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1588`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1990` |
| 28 | `FR-SOC-IDE-002` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1601`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1991` |
| 29 | `FR-SOC-IDE-003` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1612`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1992` |
| 30 | `FR-SOC-IDE-004` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1624`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1993` |
| 31 | `FR-SOC-IDE-005` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4373`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4572` |
| 32 | `FR-SOC-IDE-006` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4390`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4573` |
| 33 | `FR-SOC-INS-001` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1692`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1998` |
| 34 | `FR-SOC-INS-002` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1708`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1999` |
| 35 | `FR-SOC-INS-003` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1719`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2000` |
| 36 | `FR-SOC-INS-004` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1733`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2001` |
| 37 | `FR-SOC-INS-005` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1752`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2002` |
| 38 | `FR-SOC-INS-006` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4432`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4575` |
| 39 | `FR-SOC-INS-007` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4453`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4576` |
| 40 | `FR-SOC-INT-001` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1770`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2003` |
| 41 | `FR-SOC-INT-002` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1778`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2004` |
| 42 | `FR-SOC-INT-003` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1786`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2005` |
| 43 | `FR-SOC-INT-004` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1797`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2006` |
| 44 | `FR-SOC-INTG-001` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1812`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2007` |
| 45 | `FR-SOC-INTG-002` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1822`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2008` |
| 46 | `FR-SOC-INTG-003` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1830`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:2009` |
| 47 | `FR-SOC-INTG-004` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4478`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4577` |
| 48 | `FR-SOC-INTG-005` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4495`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4578` |
| 49 | `FR-SOC-INTG-006` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4517`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4579` |
| 50 | `FR-SOC-INTG-007` | **REAL-REQUIREMENT** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4537`, `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4580` |
| 51 | `NFR-CIV-LEGENDS-CONFIG-04` | **REAL-REQUIREMENT** | `crates/legends/src/config.rs:1`, `docs/design/legends-engine.md:454` |
| 52 | `NFR-CIV-LEGENDS-SCALE-02` | **REAL-REQUIREMENT** | `crates/legends/src/lib.rs:17`, `docs/design/legends-engine.md:452` |
| 53 | `NFR-CIV-PERF-008` | **REAL-REQUIREMENT** | `docs/guides/voxel-emergent-vision-and-migration.md:171`, `docs/guides/voxel-emergent-vision-and-migration.md:191` |
| 54 | `FR-CIV-0104-001` | **REAL-REQUIREMENT** | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1452` |
| 55 | `FR-CIV-0104-002` | **REAL-REQUIREMENT** | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1457` |
| 56 | `FR-CIV-0104-003` | **REAL-REQUIREMENT** | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1462` |
| 57 | `FR-CIV-0104-004` | **REAL-REQUIREMENT** | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1467` |
| 58 | `FR-CIV-0104-005` | **REAL-REQUIREMENT** | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1472` |
| 59 | `FR-CIV-0104-006` | **REAL-REQUIREMENT** | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1477` |
| 60 | `FR-CIV-0104-007` | **REAL-REQUIREMENT** | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1482` |
| 61 | `FR-CIV-0104-008` | **REAL-REQUIREMENT** | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1487` |
| 62 | `FR-CIV-0104-009` | **REAL-REQUIREMENT** | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1492` |
| 63 | `FR-CIV-0104-010` | **REAL-REQUIREMENT** | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1497` |
| 64 | `FR-CIV-3D` | **STALE-ID** | `scripts/fr-coverage/run-fr-coverage.sh:7` |
| 65 | `FR-CIV-3D-001` | **REAL-REQUIREMENT** | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1888` |
| 66 | `FR-CIV-3D-002` | **REAL-REQUIREMENT** | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1896` |
| 67 | `FR-CIV-3D-003` | **REAL-REQUIREMENT** | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1904` |
| 68 | `FR-CIV-3D-004` | **REAL-REQUIREMENT** | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1912` |
| 69 | `FR-CIV-3D-005` | **REAL-REQUIREMENT** | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1920` |
| 70 | `FR-CIV-3D-006` | **REAL-REQUIREMENT** | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1928` |
| 71 | `FR-CIV-3D-007` | **REAL-REQUIREMENT** | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1936` |
| 72 | `FR-CIV-3D-008` | **REAL-REQUIREMENT** | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1944` |
| 73 | `FR-CIV-3D-009` | **REAL-REQUIREMENT** | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1952` |
| 74 | `FR-CIV-3D-010` | **REAL-REQUIREMENT** | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1960` |
| 75 | `FR-CIV-3D-011` | **REAL-REQUIREMENT** | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1968` |

## Spec stubs to append

**0** new stubs in this batch (`all reviewed `REAL-REQUIREMENT` rows already map to an existing
spec/home artifact and are `COVERED`).

## RENAME mappings

No RENAME candidates in this batch.

## Notes

- The next scope for batch 4 should continue at the next sorted row after
  `FR-CIV-3D-011` (same `CODE-ONLY-no-spec` window, ref count = `1`).
- `FR-CIV-3D` is treated as `STALE-ID` because its code reference is only the coverage
  runner shim and does not point to a distinct implementation spec body.
