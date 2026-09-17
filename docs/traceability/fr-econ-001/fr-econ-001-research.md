# Research: FR-ECON-001 -- Economics

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-ECON-001
> Epic: FR-ECON

## Research Question

What is the best approach to implement Economics within the Civis simulation engine?

## Background

This FR belongs to the FR-ECON epic and is expected to be implemented in `crates/economy/src/`.

### Existing Code References
- `crates/economy/src/lib.rs:106`
- `crates/engine/src/engine.rs:2011`
- `docs/reference/agileplus-artifacts-index.md:57`
- `docs/reference/agileplus-artifacts-index.md:89`
- `docs/reference/agileplus-artifacts-index.md:260`

### Test References
- `crates/economy/src/lib.rs:308`
- `crates/economy/tests/fr_matrix_batch7.rs:9`
- `crates/economy/tests/fr_matrix_batch7.rs:135`
- `crates/economy/tests/fr_matrix_batch7.rs:138`
- `crates/economy/tests/fr_matrix_batch7.rs:154`
- `crates/economy/tests/fr_matrix_batch7.rs:165`

## Findings

### Codebase Analysis
- The `crates/economy/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/economy/src/`
2. Add integration tests in `crates/economy/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/economy/` crate documentation
