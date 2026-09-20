# Intent: FR-NFR-CIV-DET-002 -- Determinism placeholder

> Date: 2026-09-20
> FR: FR-NFR-CIV-DET-002
> Epic: FR-NFR-CIV-DET
> Status: NO-OP PLACEHOLDER (consolidated under FR-CIV-CORE-DET)

## User Intent

Originally a generic determinism NFR placeholder. The determinism
contract for Civis is now expressed by `FR-CIV-CORE-DET-*` and the
3/4 NFRs in this epic; this ID has no specific code, no test, and
no additional requirement beyond the consolidated contract.

### What This FR Achieves

Marks the slot so downstream tooling has a non-empty spec entry;
the real coverage lives in FR-CIV-CORE-DET-001..003 and
FR-NFR-CIV-DET-003/004.

### Product Context

Auto-generated NFR placeholder; the engine tests that exercised
this slot (`crates/engine/tests/fr_nfr_civ_det_002.rs`) are
degenerate `assert!(ws.tick == 0)` stubs and have been removed.

## Acceptance Signal

- Coverage is asserted by the real DET-003/004 specs.
- `cargo build --workspace --tests` succeeds with no removed
  reference.

## Traceability

| Artifact | Path |
|----------|------|
| Stub (removed) | `crates/engine/tests/fr_nfr_civ_det_002.rs` |
| Real contract | `docs/traceability/fr-civ-core-det-*` |
| Real contract | `docs/traceability/nfr-civ-det-003`, `-004` |
