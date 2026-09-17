# Research: FR-CIV-ACTOR-002 -- Actor and citizen lifecycle

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-ACTOR-002
> Epic: FR-CIV-ACTOR

## Research Question

What is the best approach to implement Actor and citizen lifecycle within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-ACTOR epic and is expected to be implemented in `crates/species/src/`.

### Existing Code References
- `docs/reference/agileplus-artifacts-index.md:73`
- `docs/reference/agileplus-artifacts-index.md:269`

### Test References
- `crates/build/tests/fr_matrix_batch12.rs:166`
- `crates/build/tests/fr_matrix_batch12.rs:169`

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
