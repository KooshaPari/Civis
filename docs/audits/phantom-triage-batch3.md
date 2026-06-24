# Phantom-ID Triage — Batch 3

Source: `docs/audits/fr-matrix.json` (1181 IDs), filtered to `CODE-ONLY-no-spec` (786 rows),
sorted by code-ref count. Skipping IDs already covered in batch 1 and batch 2,
this report records the **next 75 IDs** (positions 251–325) and writes batch-3 verdicts.

## Summary
- Real requirement (covered): **74**
- Real requirement (uncovered, stub-ready): **0**
- STALE-ID: **1**
- RENAME: **0**

## Per-row verdicts

| # | FR ID | Verdict | Coverage | Evidence (matrix code-ref, first match) |
|---|-------|---------|----------|----------------------------------------|
| 1 | `FR-SAVE-011` | REAL-REQUIREMENT | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2810` |
| 2 | `FR-SAVE-012` | REAL-REQUIREMENT | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2811` |
| 3 | `FR-SAVE-013` | REAL-REQUIREMENT | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2812` |
| 4 | `FR-SAVE-014` | REAL-REQUIREMENT | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2813` |
| 5 | `FR-SAVE-015` | REAL-REQUIREMENT | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2814` |
| 6 | `FR-SAVE-016` | REAL-REQUIREMENT | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2815` |
| 7 | `FR-SAVE-017` | REAL-REQUIREMENT | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2816` |
| 8 | `FR-SAVE-018` | REAL-REQUIREMENT | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2817` |
| 9 | `FR-SAVE-019` | REAL-REQUIREMENT | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2818` |
| 10 | `FR-SAVE-021` | REAL-REQUIREMENT | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2820` |
| 11 | `FR-SAVE-022` | REAL-REQUIREMENT | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2821` |
| 12 | `FR-SAVE-023` | REAL-REQUIREMENT | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2822` |
| 13 | `FR-SOC-CIV-001` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4326` |
| 14 | `FR-SOC-CIV-002` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4355` |
| 15 | `FR-SOC-COH-001` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1533` |
| 16 | `FR-SOC-COH-002` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1546` |
| 17 | `FR-SOC-COH-003` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1559` |
| 18 | `FR-SOC-COH-004` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1571` |
| 19 | `FR-SOC-DET-002` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1516` |
| 20 | `FR-SOC-FAC-001` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4277` |
| 21 | `FR-SOC-FAC-002` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4300` |
| 22 | `FR-SOC-HLT-001` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1643` |
| 23 | `FR-SOC-HLT-002` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1657` |
| 24 | `FR-SOC-HLT-003` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1667` |
| 25 | `FR-SOC-HLT-004` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1675` |
| 26 | `FR-SOC-HLT-005` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4412` |
| 27 | `FR-SOC-IDE-001` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1588` |
| 28 | `FR-SOC-IDE-002` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1601` |
| 29 | `FR-SOC-IDE-003` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1612` |
| 30 | `FR-SOC-IDE-004` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1624` |
| 31 | `FR-SOC-IDE-005` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4373` |
| 32 | `FR-SOC-IDE-006` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4390` |
| 33 | `FR-SOC-INS-001` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1692` |
| 34 | `FR-SOC-INS-002` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1708` |
| 35 | `FR-SOC-INS-003` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1719` |
| 36 | `FR-SOC-INS-004` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1733` |
| 37 | `FR-SOC-INS-005` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1752` |
| 38 | `FR-SOC-INS-006` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4432` |
| 39 | `FR-SOC-INS-007` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4453` |
| 40 | `FR-SOC-INT-001` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1770` |
| 41 | `FR-SOC-INT-002` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1778` |
| 42 | `FR-SOC-INT-003` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1786` |
| 43 | `FR-SOC-INT-004` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1797` |
| 44 | `FR-SOC-INTG-001` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1812` |
| 45 | `FR-SOC-INTG-002` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1822` |
| 46 | `FR-SOC-INTG-003` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1830` |
| 47 | `FR-SOC-INTG-004` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4478` |
| 48 | `FR-SOC-INTG-005` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4495` |
| 49 | `FR-SOC-INTG-006` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4517` |
| 50 | `FR-SOC-INTG-007` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4537` |
| 51 | `NFR-CIV-LEGENDS-CONFIG-04` | REAL-REQUIREMENT | yes | `crates/legends/src/config.rs:1` |
| 52 | `NFR-CIV-LEGENDS-SCALE-02` | REAL-REQUIREMENT | yes | `crates/legends/src/lib.rs:17` |
| 53 | `NFR-CIV-PERF-008` | REAL-REQUIREMENT | yes | `docs/guides/voxel-emergent-vision-and-migration.md:171` |
| 54 | `FR-CIV-0104-001` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1452` |
| 55 | `FR-CIV-0104-002` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1457` |
| 56 | `FR-CIV-0104-003` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1462` |
| 57 | `FR-CIV-0104-004` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1467` |
| 58 | `FR-CIV-0104-005` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1472` |
| 59 | `FR-CIV-0104-006` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1477` |
| 60 | `FR-CIV-0104-007` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1482` |
| 61 | `FR-CIV-0104-008` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1487` |
| 62 | `FR-CIV-0104-009` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1492` |
| 63 | `FR-CIV-0104-010` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1497` |
| 64 | `FR-CIV-3D` | STALE-ID | n/a | `scripts/fr-coverage/run-fr-coverage.sh:7` |
| 65 | `FR-CIV-3D-001` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1888` |
| 66 | `FR-CIV-3D-002` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1896` |
| 67 | `FR-CIV-3D-003` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1904` |
| 68 | `FR-CIV-3D-004` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1912` |
| 69 | `FR-CIV-3D-005` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1920` |
| 70 | `FR-CIV-3D-006` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1928` |
| 71 | `FR-CIV-3D-007` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1936` |
| 72 | `FR-CIV-3D-008` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1944` |
| 73 | `FR-CIV-3D-009` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1952` |
| 74 | `FR-CIV-3D-010` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1960` |
| 75 | `FR-CIV-3D-011` | REAL-REQUIREMENT | yes | `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1968` |

## Stubs added to agileplus-specs/civ-021-recovered-requirements/spec.md

No new real-requirement stubs were added in batch 3 because all non-stale IDs already
trace to documented locations in `docs/specs`, `docs/design`, or guides.

## RENAME mappings

No RENAME mappings were identified in batch 3.

## Notes

- `FR-CIV-3D` is marked `STALE-ID` because its sole matrix code-ref is a
  traceability helper script line (`scripts/fr-coverage/run-fr-coverage.sh:7`) and has no
  dedicated requirement spec/home in the current repository tree.
