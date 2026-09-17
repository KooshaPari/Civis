# Research: FR-CIV-BUILD-010 -- Building tiers and construction

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-BUILD-010
> Epic: FR-CIV-BUILD

## Research Question

What is the best approach to implement Building tiers and construction within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-BUILD epic and is expected to be implemented in `crates/physics-substrate/src/`.

### Existing Code References
- `docs/design/emergent-systems-spec.md:97`
- `docs/development-guide/fr-3d-additions.md:31`

### Test References
- `crates/build/src/lib.rs:604`
- `crates/build/src/lib.rs:743`
- `crates/build/tests/fr_matrix_batch12.rs:529`
- `crates/build/tests/fr_matrix_batch12.rs:532`

## Findings

### Codebase Analysis
- The `crates/physics-substrate/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/physics-substrate/src/`
2. Add integration tests in `crates/physics-substrate/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/physics-substrate/` crate documentation
