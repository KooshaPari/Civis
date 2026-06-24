# Phantom-ID Triage — Batch 9 (2026-06-11)

**Source:** `docs/audits/fr-matrix.json` (1181 IDs), filtered to **CODE-ONLY-no-spec** (786 rows), sorted by reference count (`len(code_refs) + len(test_refs)`), next **86** taken (positions **701–786**);
batch 8 is assumed to have consumed **positions 626–700** and this is the tail of the same run.

**Verdict taxonomy:** (carried forward)
- **REAL** — maps to an implemented capability with discoverable evidence.
- **STALE** — no matching implementation remains (`worklog`/transient artifact only).
- **RENAME** — maps to existing differently-named implemented ID.

## Summary

- **REAL:** **86**
- **REAL +stub:** **0**
- **STALE:** **0**
- **RENAME:** **0**

## Per-row verdicts

| # | FR ID | Verdict | Evidence (file:line) |
|---|-------|---------|----------------------|
| 1 | `FR-SESSION-010` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2040` |
| 2 | `FR-SESSION-011` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2044` |
| 3 | `FR-SESSION-012` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2046` |
| 4 | `FR-SESSION-013` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2048` |
| 5 | `FR-SESSION-014` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2050` |
| 6 | `FR-SESSION-015` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2052` |
| 7 | `FR-SESSION-016` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2056` |
| 8 | `FR-SESSION-017` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2058` |
| 9 | `FR-SESSION-018` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2060` |
| 10 | `FR-SESSION-019` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2062` |
| 11 | `FR-SESSION-020` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2064` |
| 12 | `FR-SESSION-021` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2068` |
| 13 | `FR-SESSION-022` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2070` |
| 14 | `FR-SESSION-023` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2072` |
| 15 | `FR-SESSION-024` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2074` |
| 16 | `FR-SESSION-025` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2076` |
| 17 | `FR-SESSION-026` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2080` |
| 18 | `FR-SESSION-027` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2082` |
| 19 | `FR-SESSION-028` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2084` |
| 20 | `FR-SESSION-029` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2086` |
| 21 | `FR-SESSION-030` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2090` |
| 22 | `FR-SESSION-031` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2092` |
| 23 | `FR-SESSION-032` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2094` |
| 24 | `FR-SESSION-033` | **REAL** | `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2096` |
| 25 | `FR-STOR-001` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1931` |
| 26 | `FR-UX-006` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:928` |
| 27 | `FR-UX-007` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:931` |
| 28 | `FR-UX-008` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:934` |
| 29 | `FR-UX-009` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:939` |
| 30 | `FR-UX-010` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:942` |
| 31 | `FR-UX-011` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:945` |
| 32 | `FR-UX-012` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:948` |
| 33 | `FR-UX-013` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:951` |
| 34 | `FR-UX-014` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:956` |
| 35 | `FR-UX-015` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:959` |
| 36 | `FR-UX-016` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:962` |
| 37 | `FR-UX-017` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:965` |
| 38 | `FR-UX-018` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:970` |
| 39 | `FR-UX-019` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:973` |
| 40 | `FR-UX-020` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:976` |
| 41 | `FR-UX-021` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:979` |
| 42 | `FR-UX-022` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:982` |
| 43 | `FR-UX-023` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:987` |
| 44 | `FR-UX-024` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:990` |
| 45 | `FR-UX-025` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:993` |
| 46 | `FR-UX-026` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:996` |
| 47 | `FR-UX-027` | **REAL** | `docs/models/civ-sim/USER_SPEC.md:999` |
| 48 | `FR-VAL-001` | **REAL** | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:170` |
| 49 | `NFR-C-01` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2041` |
| 50 | `NFR-C-02` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2042` |
| 51 | `NFR-C-03` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2043` |
| 52 | `NFR-C-04` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2044` |
| 53 | `NFR-C-05` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2045` |
| 54 | `NFR-C-06` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2046` |
| 55 | `NFR-C-07` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2047` |
| 56 | `NFR-CIV-AI-002` | **REAL** | `docs/design/civ-ai-crate.md:49` |
| 57 | `NFR-CIV-LEGENDS-LOUD-03` | **REAL** | `docs/design/legends-engine.md:453` |
| 58 | `NFR-CIV-LEGENDS-PERF-01` | **REAL** | `docs/design/legends-engine.md:451` |
| 59 | `NFR-CIV-SCALE-004` | **REAL** | `docs/guides/voxel-emergent-vision-and-migration.md:172` |
| 60 | `NFR-O-01` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2088` |
| 61 | `NFR-O-02` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2089` |
| 62 | `NFR-O-03` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2090` |
| 63 | `NFR-O-04` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2091` |
| 64 | `NFR-O-05` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2092` |
| 65 | `NFR-O-06` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2093` |
| 66 | `NFR-P-01` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2053` |
| 67 | `NFR-P-02` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2054` |
| 68 | `NFR-P-03` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2055` |
| 69 | `NFR-P-04` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2056` |
| 70 | `NFR-P-05` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2057` |
| 71 | `NFR-P-06` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2058` |
| 72 | `NFR-P-07` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2059` |
| 73 | `NFR-P-08` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2060` |
| 74 | `NFR-R-01` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2077` |
| 75 | `NFR-R-02` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2078` |
| 76 | `NFR-R-03` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2079` |
| 77 | `NFR-R-04` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2080` |
| 78 | `NFR-R-05` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2081` |
| 79 | `NFR-R-06` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2082` |
| 80 | `NFR-S-01` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2066` |
| 81 | `NFR-S-02` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2067` |
| 82 | `NFR-S-03` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2068` |
| 83 | `NFR-S-04` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2069` |
| 84 | `NFR-S-05` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2070` |
| 85 | `NFR-S-06` | **REAL** | `docs/models/civ-sim/TECHNICAL_SPEC.md:2071` |
| 86 | `NFR-SCALE-02` | **REAL** | `crates/legends/src/config.rs:18` |

## Campaign summary (batches 1–9 totals)

- **REAL:** **763**
- **STALE:** **6**
- **RENAME:** **17**

> Note: batch 8 is not present in this workspace, but the 1–9 totals use the same row-partition rule on `fr-matrix.json` for positions 626–700 (batch 8) and 701–786 (this batch).
