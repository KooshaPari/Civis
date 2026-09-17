# Research: FR-CIV-0104-008 -- Core civilisation simulation

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-0104-008
> Epic: FR-CIV

## Research Question

What is the best approach to implement Core civilisation simulation within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV epic and is expected to be implemented in `crates/engine/src/`.

### Existing Code References
- `docs/specs/CIV-0104-minimal-constraint-set-theorem.md:1487`

### Test References
> _No test coverage yet._

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
