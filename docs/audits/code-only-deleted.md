# Code-only orphan deletions — agent-G (P3)

Date: 2026-09-20
Agent: agent-G
Phase: P3 — CODE-ONLY-no-spec → COVERED or remove

This file records every orphan code artifact deleted during agent-G's
P3 pass so that later audit runs do not waste cycles rediscovering
the same orphans.

## Deleted orphan stub tests

All files below were auto-generated `assert!(ws.tick == 0)`
"upgraded from stub to real assertions" tests with no real
coverage. They contributed nothing to determinism / perf / reliability
verification and were the only references for the listed orphan IDs.

| ID | Deleted file |
|----|--------------|
| FR-NFR-CIV-DET-001 | `crates/engine/tests/fr_nfr_civ_det_001.rs` |
| FR-NFR-CIV-DET-002 | `crates/engine/tests/fr_nfr_civ_det_002.rs` |
| FR-NFR-CIV-PERF-002 | `crates/engine/tests/fr_nfr_civ_perf_002.rs` |
| FR-NFR-CIV-PERF-003 | `crates/engine/tests/fr_nfr_civ_perf_003.rs` |
| FR-NFR-CIV-PERF-004 | `crates/engine/tests/fr_nfr_civ_perf_004.rs` |
| FR-NFR-CIV-PERF-005 | `crates/engine/tests/fr_nfr_civ_perf_005.rs` |
| FR-NFR-CIV-PERF-006 | `crates/engine/tests/fr_nfr_civ_perf_006.rs` |
| FR-NFR-CIV-PERF-007 | `crates/engine/tests/fr_nfr_civ_perf_007.rs` |
| FR-NFR-CIV-PERF-008 | `crates/engine/tests/fr_nfr_civ_perf_008.rs` |
| FR-NFR-CIV-PERF-900 | `crates/engine/tests/fr_nfr_civ_perf_900.rs` |
| FR-NFR-CIV-PERF-901 | `crates/engine/tests/fr_nfr_civ_perf_901.rs` |
| FR-NFR-CIV-PERF-902 | `crates/engine/tests/fr_nfr_civ_perf_902.rs` |
| FR-NFR-R-01 | `crates/engine/tests/fr_nfr_r_01.rs` |
| FR-NFR-R-02 | `crates/engine/tests/fr_nfr_r_02.rs` |
| FR-NFR-R-03 | `crates/engine/tests/fr_nfr_r_03.rs` |
| FR-NFR-R-04 | `crates/engine/tests/fr_nfr_r_04.rs` |
| FR-NFR-R-05 | `crates/engine/tests/fr_nfr_r_05.rs` |
| FR-NFR-R-06 | `crates/engine/tests/fr_nfr_r_06.rs` |

## Rationale

Per the P3 cleanup bias in `docs/audits/P3-cleanup-slice/_plan-P3-summary.md`:

> Default to Option A. Most FR-NFR-prefixed orphans are intentional.
> Default to Option B for FR-NFR-P/NFR-C/NFR-R/NFR-S etc. that look
> like generic categories without substance.

The 18 deletions above are all generic NFR placeholders whose only
"implementation" was a degenerate stub test asserting that
`WorldState::default().tick == 0`. They are not load-bearing for any
real coverage contract. Per-ID no-op intent docs remain in
`docs/traceability/<id>/<id>-intent.md` so the audit slot is no
longer orphan.
