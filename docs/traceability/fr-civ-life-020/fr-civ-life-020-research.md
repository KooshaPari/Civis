# Research: FR-CIV-LIFE-020 -- Life simulation and needs

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-LIFE-020
> Epic: FR-CIV-LIFE

## Research Question

What is the best approach to implement Life simulation and needs within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-LIFE epic and is expected to be implemented in `crates/species/src/`.

### Existing Code References
- `crates/engine/src/engine.rs:445`
- `crates/engine/src/engine.rs:2163`

### Test References
- `crates/economy/src/stocks.rs:285`
- `crates/economy/src/stocks.rs:452`
- `crates/economy/tests/fr_matrix_batch7.rs:8`
- `crates/economy/tests/fr_matrix_batch7.rs:46`
- `crates/economy/tests/fr_matrix_batch7.rs:49`
- `crates/economy/tests/fr_matrix_batch7.rs:61`
- `crates/economy/tests/fr_matrix_batch7.rs:77`
- `crates/economy/tests/fr_matrix_batch7.rs:106`

## Findings

### Codebase Analysis
- The `crates/species/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/species/src/`
2. Add integration tests in `crates/species/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/species/` crate documentation
