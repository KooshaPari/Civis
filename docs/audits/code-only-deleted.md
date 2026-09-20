# Code-Only FR/NFR IDs Deleted

This log records phantom or duplicated code-only IDs that were deleted
without writing a spec doc, so future audits don't waste cycles reopening
the same question.

## 2026-09-19 — agent-F P3 fan-out

**Source of verdict:** `docs/audits/P3-cleanup-slice/P3-agent-F.md`,
"CODE-ONLY-no-spec" batch.

### Generic-prefix orphans with no source and no substance

The 17 IDs below had **no implementation, no documentation, and no test
coverage** in the workspace. The audit inventory lists each as
`in_code: []`, `in_specs: []`, `in_traceability: []`, with only auto-generated
`in_stub_tests` entries that assert `ws.tick == 0` on a default `WorldState`.
They are not real requirements — they are placeholder rows from an earlier
generic-prefix skeleton and have no substance to spec.

Per the slice bias (`Default to Option B for FR-NFR-P/NFR-C/NFR-R/NFR-S etc.
that look like generic categories without substance.`), all 17 stub test
files were deleted with `git rm`.

| ID | Test file deleted |
|---|---|
| FR-NFR-CIV-AI-001 | `crates/engine/tests/fr_nfr_civ_ai_001.rs` |
| FR-NFR-CIV-AI-002 | `crates/engine/tests/fr_nfr_civ_ai_002.rs` |
| FR-NFR-CIV-AI-003 | `crates/engine/tests/fr_nfr_civ_ai_003.rs` |
| FR-NFR-CIV-MAINT-001 | `crates/engine/tests/fr_nfr_civ_maint_001.rs` |
| FR-NFR-CIV-MAINT-002 | `crates/engine/tests/fr_nfr_civ_maint_002.rs` |
| FR-NFR-CIV-MAINT-003 | `crates/engine/tests/fr_nfr_civ_maint_003.rs` |
| FR-NFR-CIV-MAINT-004 | `crates/engine/tests/fr_nfr_civ_maint_004.rs` |
| FR-NFR-CIV-MAINT-005 | `crates/engine/tests/fr_nfr_civ_maint_005.rs` |
| FR-NFR-CIV-MAINT-006 | `crates/engine/tests/fr_nfr_civ_maint_006.rs` |
| FR-NFR-P-01 | `crates/engine/tests/fr_nfr_p_01.rs` |
| FR-NFR-P-02 | `crates/engine/tests/fr_nfr_p_02.rs` |
| FR-NFR-P-03 | `crates/engine/tests/fr_nfr_p_03.rs` |
| FR-NFR-P-04 | `crates/engine/tests/fr_nfr_p_04.rs` |
| FR-NFR-P-05 | `crates/engine/tests/fr_nfr_p_05.rs` |
| FR-NFR-P-06 | `crates/engine/tests/fr_nfr_p_06.rs` |
| FR-NFR-P-07 | `crates/engine/tests/fr_nfr_p_07.rs` |
| FR-NFR-P-08 | `crates/engine/tests/fr_nfr_p_08.rs` |

**Real NFR coverage lives elsewhere.** The functional NFR surface is owned by
[`docs/traceability/fr-nfr-matrix.md`](../../traceability/fr-nfr-matrix.md),
which traces the canonical `NFR-CIV-PERF-*` / `NFR-CIV-DET-*` /
`NFR-CIV-SCALE-*` / `NFR-CIV-REL-*` / `NFR-CIV-SEC-*` / `NFR-CIV-ACC-*` /
`NFR-CIV-PORT-*` / `NFR-CIV-MAINT-*` IDs against real source paths and
acceptance contracts. The deleted `FR-NFR-*` IDs above are an unrelated,
unowned generic-prefix set with no semantic overlap.
