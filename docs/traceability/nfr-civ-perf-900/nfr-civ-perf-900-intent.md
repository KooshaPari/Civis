# Intent: FR-NFR-CIV-PERF-900 -- Performance placeholder (suite 900)

> Date: 2026-09-20
> FR: FR-NFR-CIV-PERF-900
> Epic: FR-NFR-CIV-PERF
> Status: NO-OP PLACEHOLDER

## User Intent

Generic performance placeholder in the suite-900 series (reserved
for forward-looking / not-yet-implemented perf contracts). No
specific code or test coverage beyond the consolidated perf NFR
contract in `docs/reference/non-functional-requirements.md`.

### What This FR Achieves

Marks the slot so downstream tooling has a non-empty spec entry;
the real per-domain perf contracts live in NFR-CIV-LEGENDS-PERF-01,
NFR-CIV-SCALE-001..004, NFR-CIV-PORT-001..003.

### Product Context

Auto-generated NFR placeholder; the engine tests that exercised
this slot were degenerate `assert!(ws.tick == 0)` stubs and have
been removed.

## Acceptance Signal

- Coverage is asserted by the real perf specs above.
- `cargo build --workspace --tests` succeeds.

## Traceability

| Artifact | Path |
|----------|------|
| Stub (removed) | `crates/engine/tests/fr_nfr_civ_perf_900.rs` |
| Real contract | `docs/reference/non-functional-requirements.md` |
