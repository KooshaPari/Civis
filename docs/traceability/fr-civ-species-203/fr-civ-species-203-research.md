# Research: FR-CIV-SPECIES-203 -- Species definitions

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-SPECIES-203
> Epic: FR-CIV-SPECIES

## Research Question

What is the best approach to implement Species definitions within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-SPECIES epic and is expected to be implemented in `crates/species/src/`.

### Existing Code References
- `docs/design/species-sentience.md:101`

### Test References
> _No test coverage yet._

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
