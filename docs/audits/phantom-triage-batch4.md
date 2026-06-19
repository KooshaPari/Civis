# Phantom-ID Triage — Batch 4 (2026-06-11)

**Source:** `docs/audits/fr-matrix.json` (`CODE-ONLY-no-spec` rows), sorted by code-ref count, skipping IDs already covered in
[`phantom-triage-batch1.md`](phantom-triage-batch1.md),
[`phantom-triage-batch2.md`](phantom-triage-batch2.md),
[`phantom-triage-batch3.md`](phantom-triage-batch3.md) (~325 rows total),
taking the next **75** rows.

**Verdict taxonomy:**
- **REAL-REQUIREMENT** — row corresponds to a real requirement.
  - `covered` = implementation already mapped in an existing spec artifact (`docs/specs/...`)
  - `+stub-to-civ-021-spec` = real requirement with no existing FR spec artifact (staging stub candidate)
- **STALE-ID** — artifact/no dedicated implementation.
- **RENAME** — implementation maps to an existing differently named requirement ID.

## Summary

- **REAL-REQUIREMENT (covered):** 22
- **REAL-REQUIREMENT +stub-to-civ-021-spec:** 53
- **STALE-ID:** 0
- **RENAME:** 0

## Per-row verdicts

| # | FR ID | Verdict | Evidence (file:line) |
|---|-------|---------|----------------------|
| 1 | FR-CIV-3D-012 | **REAL-REQUIREMENT** | docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1976 |
| 2 | FR-CIV-3D-013 | **REAL-REQUIREMENT** | docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1984 |
| 3 | FR-CIV-3D-014 | **REAL-REQUIREMENT** | docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1992 |
| 4 | FR-CIV-3D-015 | **REAL-REQUIREMENT** | docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:2000 |
| 5 | FR-CIV-ACT-003 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/reference/REFERENCE_GAME_ANALYSIS.md:511 |
| 6 | FR-CIV-ACT-004 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/reference/REFERENCE_GAME_ANALYSIS.md:183 |
| 7 | FR-CIV-ACT-005 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/reports/STATUS_REPORT.md:98 |
| 8 | FR-CIV-AGENTS-002 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:756 |
| 9 | FR-CIV-AGENTS-003 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:772 |
| 10 | FR-CIV-AGENTS-011 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:820 |
| 11 | FR-CIV-AGENTS-020 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:836 |
| 12 | FR-CIV-AGENTS-021 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:888 |
| 13 | FR-CIV-AGENTS-022 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:902 |
| 14 | FR-CIV-AGENTS-023 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:1140 |
| 15 | FR-CIV-AGENTS-024 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:1161 |
| 16 | FR-CIV-AGENTS-025 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:1179 |
| 17 | FR-CIV-AGENTS-030 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:1044 |
| 18 | FR-CIV-AGENTS-031 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:1065 |
| 19 | FR-CIV-AGENTS-032 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:1076 |
| 20 | FR-CIV-AGENTS-033 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:1092 |
| 21 | FR-CIV-AGENTS-034 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/agents/src/lib.rs:1120 |
| 22 | FR-CIV-ARCH-NOSVG-001 | **REAL-REQUIREMENT** | docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3218 |
| 23 | FR-CIV-ASSET-MANI-001 | **REAL-REQUIREMENT** | docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3212 |
| 24 | FR-CIV-ASSET-MANI-002 | **REAL-REQUIREMENT** | docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3213 |
| 25 | FR-CIV-ASSET-QUAL-001 | **REAL-REQUIREMENT** | docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3224 |
| 26 | FR-CIV-AUDIO-009 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/audio-direction.md:302 |
| 27 | FR-CIV-AUDIO-010 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/audio-direction.md:303 |
| 28 | FR-CIV-AUDIO-011 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/audio-direction.md:304 |
| 29 | FR-CIV-AUDIO-012 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/audio-direction.md:305 |
| 30 | FR-CIV-BEVY-003 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | clients/bevy-ref/src/lib.rs:1300 |
| 31 | FR-CIV-BEVY-013 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/development-guide/p-w1-kickoff.md:124 |
| 32 | FR-CIV-BEVY-014 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/development-guide/p-w1-kickoff.md:82 |
| 33 | FR-CIV-BEVY-015 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/development-guide/p-w1-kickoff.md:126 |
| 34 | FR-CIV-BEVY-017 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/development-guide/p-w1-kickoff.md:128 |
| 35 | FR-CIV-BEVY-018 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/development-guide/p-w1-kickoff.md:129 |
| 36 | FR-CIV-BEVY-019 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/development-guide/p-w1-kickoff.md:130 |
| 37 | FR-CIV-BEVY-020 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/development-guide/p-w1-kickoff.md:131 |
| 38 | FR-CIV-BRUSH-01 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:514 |
| 39 | FR-CIV-BRUSH-02 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:515 |
| 40 | FR-CIV-BRUSH-03 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:516 |
| 41 | FR-CIV-BRUSH-04 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:517 |
| 42 | FR-CIV-BRUSH-05 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:518 |
| 43 | FR-CIV-BRUSH-06 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:519 |
| 44 | FR-CIV-BRUSH-07 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:520 |
| 45 | FR-CIV-BRUSH-08 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:521 |
| 46 | FR-CIV-BRUSH-09 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:522 |
| 47 | FR-CIV-BRUSH-10 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:523 |
| 48 | FR-CIV-BRUSH-11 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:524 |
| 49 | FR-CIV-BRUSH-12 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:525 |
| 50 | FR-CIV-BRUSH-13 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | docs/design/brush-tool-system.md:526 |
| 51 | FR-CIV-CORE-006 | **REAL-REQUIREMENT** | docs/specs/CIV-0001-core-simulation-loop.md:892 |
| 52 | FR-CIV-CORE-007 | **REAL-REQUIREMENT** | docs/specs/CIV-0001-core-simulation-loop.md:897 |
| 53 | FR-CIV-CORE-008 | **REAL-REQUIREMENT** | docs/specs/CIV-0001-core-simulation-loop.md:902 |
| 54 | FR-CIV-CORE-009 | **REAL-REQUIREMENT** | docs/specs/CIV-0001-core-simulation-loop.md:907 |
| 55 | FR-CIV-CORE-010 | **REAL-REQUIREMENT** | docs/specs/CIV-0001-core-simulation-loop.md:912 |
| 56 | FR-CIV-CORE-012 | **REAL-REQUIREMENT** | docs/specs/CIV-0001-core-simulation-loop.md:922 |
| 57 | FR-CIV-CORE-014 | **REAL-REQUIREMENT** | docs/specs/CIV-0001-core-simulation-loop.md:932 |
| 58 | FR-CIV-CORE-015 | **REAL-REQUIREMENT** | docs/specs/CIV-0001-core-simulation-loop.md:937 |
| 59 | FR-CIV-CORE-016 | **REAL-REQUIREMENT** | docs/specs/CIV-0001-core-simulation-loop.md:942 |
| 60 | FR-CIV-CORE-017 | **REAL-REQUIREMENT** | docs/specs/CIV-0001-core-simulation-loop.md:947 |
| 61 | FR-CIV-CORE-018 | **REAL-REQUIREMENT** | docs/specs/CIV-0001-core-simulation-loop.md:952 |
| 62 | FR-CIV-CORE-DET-001 | **REAL-REQUIREMENT** | docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3206 |
| 63 | FR-CIV-CORE-DET-002 | **REAL-REQUIREMENT** | docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3214 |
| 64 | FR-CIV-CORE-DET-003 | **REAL-REQUIREMENT** | docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3223 |
| 65 | FR-CIV-DIFFUSION-002 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/diffusion/src/lib.rs:107 |
| 66 | FR-CIV-DIFFUSION-004 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/diffusion/src/lib.rs:131 |
| 67 | FR-CIV-DIFFUSION-005 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/diffusion/src/lib.rs:139 |
| 68 | FR-CIV-DIFFUSION-006 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/diffusion/src/lib.rs:153 |
| 69 | FR-CIV-DIFFUSION-007 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/diffusion/src/lib.rs:170 |
| 70 | FR-CIV-DIFFUSION-008 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/diffusion/src/lib.rs:185 |
| 71 | FR-CIV-DIFFUSION-009 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/diffusion/src/lib.rs:207 |
| 72 | FR-CIV-DIFFUSION-010 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/diffusion/src/lib.rs:223 |
| 73 | FR-CIV-DIFFUSION-011 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/diffusion/src/lib.rs:253 |
| 74 | FR-CIV-DIFFUSION-012 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/diffusion/src/lib.rs:281 |
| 75 | FR-CIV-DIFFUSION-013 | **REAL-REQUIREMENT +stub-to-civ-021-spec** | crates/diffusion/src/lib.rs:291 |
