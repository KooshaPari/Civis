# Code-Only-Deleted Log

Tracks orphaned code (test files or modules) removed during P3 cleanup because
they had no spec, were auto-generated stubs, and added no signal beyond
`WorldState::default().tick == 0`.

## Entries

| Date | ID | File removed | Reason |
|------|----|--------------|--------|
| 2026-09-19 | FR-NFR-CIV-LEGENDS-CONFIG-04 | `crates/engine/tests/fr_nfr_civ_legends_config_04.rs` | Auto-generated stub; `WorldState::default().tick == 0` adds no signal beyond the type-defaulting test. |
| 2026-09-19 | FR-NFR-CIV-REL-001 | `crates/engine/tests/fr_nfr_civ_rel_001.rs` | Auto-generated stub; same generic default-tick assertion. |
| 2026-09-19 | FR-NFR-CIV-REL-002 | `crates/engine/tests/fr_nfr_civ_rel_002.rs` | Auto-generated stub; same generic default-tick assertion. |
| 2026-09-19 | FR-NFR-CIV-REL-003 | `crates/engine/tests/fr_nfr_civ_rel_003.rs` | Auto-generated stub; same generic default-tick assertion. |
| 2026-09-19 | FR-NFR-SCALE-02 | `crates/engine/tests/fr_nfr_scale_02.rs` | Auto-generated stub; same generic default-tick assertion. |


---

# Appended from next-P3-C

# Code-Only Orphans — Deleted (P3 agent-C, 2026-09-19)

These FR IDs appeared in the coverage matrix as `CODE-ONLY-no-spec`
but had no real implementation, only auto-generated stub test files
and stub spec templates. Per the P3 bias ("Default to Option B for
FR-NFR-P/NFR-C/NFR-R/NFR-S generic categories without substance"),
the stub test files and stub spec dirs have been deleted.

| ID | Epic | What was deleted |
|----|------|------------------|
| FR-NFR-CIV-LEGENDS-LOUD-03 | FR-NFR-CIV-LEGENDS-LOUD | `crates/engine/tests/fr_nfr_civ_legends_loud_03.rs` + `docs/traceability/nfr-civ-legends-loud-03/` |
| FR-NFR-CIV-SCALE-001 | FR-NFR-CIV-SCALE | `crates/engine/tests/fr_nfr_civ_scale_001.rs` + `docs/traceability/nfr-civ-scale-001/` |
| FR-NFR-CIV-SCALE-002 | FR-NFR-CIV-SCALE | `crates/engine/tests/fr_nfr_civ_scale_002.rs` + `docs/traceability/nfr-civ-scale-002/` |
| FR-NFR-CIV-SCALE-003 | FR-NFR-CIV-SCALE | `crates/engine/tests/fr_nfr_civ_scale_003.rs` + `docs/traceability/nfr-civ-scale-003/` |
| FR-NFR-CIV-SCALE-004 | FR-NFR-CIV-SCALE | `crates/engine/tests/fr_nfr_civ_scale_004.rs` + `docs/traceability/nfr-civ-scale-004/` |
| FR-NFR-CIV-SCALE-900 | FR-NFR-CIV-SCALE | `crates/engine/tests/fr_nfr_civ_scale_900.rs` + `docs/traceability/nfr-civ-scale-900/` |
| FR-NFR-CIV-SCALE-901 | FR-NFR-CIV-SCALE | `crates/engine/tests/fr_nfr_civ_scale_901.rs` + `docs/traceability/nfr-civ-scale-901/` |
| FR-NFR-CIV-SCALE-902 | FR-NFR-CIV-SCALE | `crates/engine/tests/fr_nfr_civ_scale_902.rs` + `docs/traceability/nfr-civ-scale-902/` |
| FR-NFR-CIV-SCALE-910 | FR-NFR-CIV-SCALE | `crates/engine/tests/fr_nfr_civ_scale_910.rs` + `docs/traceability/nfr-civ-scale-910/` |
| FR-NFR-CIV-SCALE-920 | FR-NFR-CIV-SCALE | `crates/engine/tests/fr_nfr_civ_scale_920.rs` + `docs/traceability/nfr-civ-scale-920/` |

All deleted files were:

* Stub tests: `let ws = civ_engine::WorldState::default(); assert!(ws.tick == 0);`
  with `//! Stub: TDD-red — replace with real FR assertions` markers.
* Stub specs: auto-generated `SPEC-TEMPLATE` files dated 2026-09-16
  with all sections blank / comment-only.

None of these tests referenced any production source code (no source
code exists for these IDs). Removing them does not break any
production path. The general `docs/reference/non-functional-requirements.md`
still captures the high-level NFR categories.


---

# Appended from next-P3-D

# CODE-ONLY orphans deleted — agent-D (P3 cleanup pass)

These orphan test stubs were auto-generated, contained no real source
code, and pointed at FR-IDs that have no substantive implementation. They
were deleted under slice-D Option B per the instructions ("Default to
Option B for FR-NFR-P/NFR-C/NFR-R/NFR-S etc. that look like generic
categories without substance").

## Deleted files

### FR-NFR-C-01..07 (7 stubs) — generic compliance category shells

| File | Test body content |
|------|-------------------|
| `crates/engine/tests/fr_nfr_c_01.rs` | `civ_engine::WorldState::default(); assert!(ws.tick == 0);` |
| `crates/engine/tests/fr_nfr_c_02.rs` | same stub body, different ID |
| `crates/engine/tests/fr_nfr_c_03.rs` | same stub body, different ID |
| `crates/engine/tests/fr_nfr_c_04.rs` | same stub body, different ID |
| `crates/engine/tests/fr_nfr_c_05.rs` | same stub body, different ID |
| `crates/engine/tests/fr_nfr_c_06.rs` | same stub body, different ID |
| `crates/engine/tests/fr_nfr_c_07.rs` | same stub body, different ID |

### FR-NFR-CIV-SEC-001..004 (4 stubs) — security-compliance shells

| File | Test body content |
|------|-------------------|
| `crates/engine/tests/fr_nfr_civ_sec_001.rs` | `WorldState::default(); tick==0` |
| `crates/engine/tests/fr_nfr_civ_sec_002.rs` | same stub body |
| `crates/engine/tests/fr_nfr_civ_sec_003.rs` | same stub body |
| `crates/engine/tests/fr_nfr_civ_sec_004.rs` | same stub body |

### FR-NFR-CIV-LEGENDS-PERF-01 (1 stub)

| File | Test body content |
|------|-------------------|
| `crates/engine/tests/fr_nfr_civ_legends_perf_01.rs` | `WorldState::default(); tick==0` |

## Rationale

Each deleted file contained only the boilerplate `verify_*_basic()` test
that asserts `WorldState::default().tick == 0` — a "test" that proves
nothing about the actual FR. No `spec_refs`, no real source. Continuing
to maintain them just adds noise to `cargo test --workspace` output.

## Sister IDs to investigate later (NOT in scope for this slice)

- `NFR-C-*` compliance style IDs in `docs/traceability/nfr-c-*` already
  have ADR/Plan/Research/Spec files; the `FR-NFR-C-0X` IDs were unrelated
  generic shells and not the same lineage.
- `FR-NFR-CIV-SEC-*` exist as audit table rows only. If a real compliance
  surface lands, write the spec under `docs/traceability/fr-nfr-civ-sec-*/`
  rather than recreating the stub tests.

## Reference

- Slice file: `docs/audits/P3-cleanup-slice/P3-agent-D.md`
- Build/test verification: ran `cargo build --workspace --tests` —
  see final summary commit message for outcome.


---

# Appended from next-P3-E

# Code-only deleted IDs (P3 cleanup)

This file tracks FR/NFR IDs whose only "code" was a stub test file in
`crates/engine/tests/fr_<id_lower>.rs` that never had a corresponding
real implementation or spec. Each entry records the deletion so that
future audits don't keep re-flagging the same orphan.

The stub tests used the boilerplate pattern:

```rust
#[test]
fn verify_<id>_basic() {
    let ws = civ_engine::WorldState::default();
    assert!(ws.tick == 0);
}
```

This pattern adds zero behavioral coverage and was kept only because the
matrix audit treated the IDs as "code-only". Per the P3 slice bias rule
("Default to Option B for FR-NFR-* generic prefixes without substance"),
we delete the stub and note it here.

## agent-E batch (2026-09-20)

| ID | Stub file | Notes |
|----|-----------|-------|
| FR-NFR-CIV-ACC-001 | crates/engine/tests/fr_nfr_civ_acc_001.rs | Accessibility generic placeholder. |
| FR-NFR-CIV-ACC-002 | crates/engine/tests/fr_nfr_civ_acc_002.rs | Accessibility generic placeholder. |
| FR-NFR-CIV-ACC-003 | crates/engine/tests/fr_nfr_civ_acc_003.rs | Accessibility generic placeholder. |
| FR-NFR-CIV-ACC-004 | crates/engine/tests/fr_nfr_civ_acc_004.rs | Accessibility generic placeholder. |
| FR-NFR-CIV-LEGENDS-SCALE-02 | crates/engine/tests/fr_nfr_civ_legends_scale_02.rs | Legends scale generic placeholder. |
| FR-NFR-O-01 | crates/engine/tests/fr_nfr_o_01.rs | "Other" generic placeholder. |
| FR-NFR-O-02 | crates/engine/tests/fr_nfr_o_02.rs | "Other" generic placeholder. |
| FR-NFR-O-03 | crates/engine/tests/fr_nfr_o_03.rs | "Other" generic placeholder. |
| FR-NFR-O-04 | crates/engine/tests/fr_nfr_o_04.rs | "Other" generic placeholder. |
| FR-NFR-O-05 | crates/engine/tests/fr_nfr_o_05.rs | "Other" generic placeholder. |
| FR-NFR-O-06 | crates/engine/tests/fr_nfr_o_06.rs | "Other" generic placeholder. |

If any of these IDs ever get real FR backing, recreate the test file
in the same path with a real assertion, and remove the row above.


---

# Appended from next-P3-F

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


---

# Appended from next-P3-G

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
