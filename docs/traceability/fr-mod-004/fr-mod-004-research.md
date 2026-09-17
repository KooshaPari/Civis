# Research: FR-MOD-004 -- Modding

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-MOD-004
> Epic: FR-MOD

## Research Question

What is the best approach to implement Modding within the Civis simulation engine?

## Background

This FR belongs to the FR-MOD epic and is expected to be implemented in `crates/mod-host/src/`.

### Existing Code References
- `crates/engine/src/engine.rs:1227`
- `crates/engine/src/replay.rs:51`
- `crates/engine/src/replay.rs:63`
- `crates/engine/src/replay.rs:82`
- `crates/engine/src/replay.rs:273`
- `crates/engine/src/replay.rs:285`
- `crates/mod-host/src/lib.rs:204`
- `crates/mod-host/src/lib.rs:217`

### Test References
- `crates/engine/src/scenario.rs:347`
- `crates/engine/tests/fr_matrix_batch1.rs:16`
- `crates/engine/tests/fr_matrix_batch1.rs:263`
- `crates/engine/tests/fr_matrix_batch1.rs:264`
- `crates/engine/tests/fr_matrix_batch1.rs:265`
- `crates/engine/tests/fr_matrix_batch1.rs:303`
- `crates/engine/tests/fr_matrix_batch3.rs:120`
- `crates/engine/tests/fr_matrix_batch3.rs:121`

## Findings

### Codebase Analysis
- The `crates/mod-host/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/mod-host/src/`
2. Add integration tests in `crates/mod-host/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/mod-host/` crate documentation
