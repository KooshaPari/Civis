# Research: FR-ECON-002 -- Economics

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-ECON-002
> Epic: FR-ECON

## Research Question

What is the best approach to implement Economics within the Civis simulation engine?

## Background

This FR belongs to the FR-ECON epic and is expected to be implemented in `crates/economy/src/`.

### Existing Code References
- `docs/reference/agileplus-artifacts-index.md:57`
- `docs/reference/agileplus-artifacts-index.md:261`

### Test References
- `crates/economy/src/allocator.rs:765`
- `crates/economy/src/allocator.rs:816`
- `crates/economy/src/lib.rs:381`
- `crates/economy/tests/fr_matrix_batch7.rs:10`
- `crates/economy/tests/fr_matrix_batch7.rs:180`
- `crates/economy/tests/fr_matrix_batch7.rs:183`
- `crates/economy/tests/fr_matrix_batch7.rs:223`

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
