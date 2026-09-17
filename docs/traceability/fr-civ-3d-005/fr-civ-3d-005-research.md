# Research: FR-CIV-3D-005 -- Core civilisation simulation

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-3D-005
> Epic: FR-CIV

## Research Question

What is the best approach to implement Core civilisation simulation within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV epic and is expected to be implemented in `crates/protocol-3d/src/`.

### Existing Code References
- `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1920`

### Test References
> _No test coverage yet._

## Findings

### Codebase Analysis
- The `crates/protocol-3d/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/protocol-3d/src/`
2. Add integration tests in `crates/protocol-3d/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/protocol-3d/` crate documentation
