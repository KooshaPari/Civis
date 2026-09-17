# Research: FR-REPLAY-001 -- Replay

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-REPLAY-001
> Epic: FR-REPLAY

## Research Question

What is the best approach to implement Replay within the Civis simulation engine?

## Background

This FR belongs to the FR-REPLAY epic and is expected to be implemented in `crates/engine/src/`.

### Existing Code References
- `crates/engine/src/engine.rs:1268`
- `crates/engine/src/replay_format.rs:1`
- `docs/reference/agileplus-artifacts-index.md:239`
- `docs/reference/agileplus-artifacts-index.md:311`

### Test References
- `crates/engine/src/engine.rs:3206`
- `crates/engine/tests/fr_matrix_batch1.rs:15`
- `crates/engine/tests/fr_matrix_batch1.rs:191`
- `crates/engine/tests/fr_matrix_batch1.rs:192`
- `crates/engine/tests/fr_matrix_batch1.rs:193`
- `crates/engine/tests/fr_matrix_batch3.rs:87`
- `crates/engine/tests/fr_matrix_batch3.rs:88`

## Findings

### Codebase Analysis
- The `crates/engine/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/engine/src/`
2. Add integration tests in `crates/engine/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/engine/` crate documentation
