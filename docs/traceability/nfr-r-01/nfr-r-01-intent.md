# Intent: FR-NFR-R-01 -- Reliability placeholder (deprecated)

> Date: 2026-09-20
> FR: FR-NFR-R-01
> Epic: FR-NFR-R
> Status: DEPRECATED — orphan stub deleted

## User Intent

Originally a generic reliability NFR placeholder (auto-generated).
No specific code or test coverage beyond the consolidated
`docs/reference/non-functional-requirements.md` and the
`NFR-CIV-*` per-domain contracts.

### What This FR Achieves

Marks the slot so downstream tooling has a non-empty spec entry.
The pre-existing `nfr-r-01-spec.md` template stub is preserved as
historical context but is no longer the source of truth.

### Product Context

The engine test that exercised this slot was a degenerate
`assert!(ws.tick == 0)` stub (`crates/engine/tests/fr_nfr_r_01.rs`)
and has been removed. Per the P3 cleanup bias, generic FR-NFR-R-*
prefixes without substance are deleted rather than documented.

## Acceptance Signal

- `cargo build --workspace --tests` succeeds without the orphan
  stub file.
- Real reliability coverage lives in the per-domain NFR-CIV-*
  specs.

## Traceability

| Artifact | Path |
|----------|------|
| Stub (removed) | `crates/engine/tests/fr_nfr_r_01.rs` |
| Real contract | `docs/reference/non-functional-requirements.md` |
