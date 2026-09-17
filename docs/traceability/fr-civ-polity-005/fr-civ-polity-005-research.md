# Research: FR-CIV-POLITY-005 -- Polity system

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-POLITY-005
> Epic: FR-CIV-POLITY

## Research Question

What is the best approach to implement Polity system within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-POLITY epic and is expected to be implemented in `crates/diplomacy/src/`.

### Existing Code References
- `docs/design/polities-markets.md:82`

### Test References
> _No test coverage yet._

## Findings

### Codebase Analysis
- The `crates/diplomacy/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/diplomacy/src/`
2. Add integration tests in `crates/diplomacy/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/diplomacy/` crate documentation
