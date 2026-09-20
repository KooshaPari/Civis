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
