# Research: FR-MET-001 -- Metrics

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-MET-001
> Epic: FR-MET

## Research Question

What is the best approach to implement Metrics within the Civis simulation engine?

## Background

This FR belongs to the FR-MET epic and is expected to be implemented in `crates/observability/src/`.

### Existing Code References
- `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1174`
- `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1203`

### Test References
> _No test coverage yet._

## Findings

### Codebase Analysis
- The `crates/observability/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/observability/src/`
2. Add integration tests in `crates/observability/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/observability/` crate documentation
