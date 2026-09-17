# Research: FR-CORE-005 -- Core system

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CORE-005
> Epic: FR-CORE

## Research Question

What is the best approach to implement Core system within the Civis simulation engine?

## Background

This FR belongs to the FR-CORE epic and is expected to be implemented in `crates/engine/src/`.

### Existing Code References
- `crates/engine/src/hash_chain.rs:1`
- `docs/reference/agileplus-artifacts-index.md:41`
- `docs/reference/agileplus-artifacts-index.md:257`

### Test References
- `crates/engine/src/hash_chain.rs:212`
- `crates/engine/tests/fr_matrix_batch1.rs:14`
- `crates/engine/tests/fr_matrix_batch1.rs:126`
- `crates/engine/tests/fr_matrix_batch1.rs:127`
- `crates/engine/tests/fr_matrix_batch1.rs:128`
- `crates/engine/tests/fr_matrix_batch3.rs:54`
- `crates/engine/tests/fr_matrix_batch3.rs:55`

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
