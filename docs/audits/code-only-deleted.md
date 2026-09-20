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
