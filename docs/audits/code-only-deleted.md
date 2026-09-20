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
