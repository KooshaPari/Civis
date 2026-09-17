# Research: FR-CIV-CORE-019 -- Core simulation engine

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-CORE-019
> Epic: FR-CIV-CORE

## Research Question

What is the best approach to implement Core simulation engine within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-CORE epic and is expected to be implemented in `crates/engine/src/`.

### Existing Code References
- `docs/AGILE_WORKSTREAM.md:196`
- `docs/AGILE_WORKSTREAM.md:246`
- `docs/models/civ-sim/TECHNICAL_SPEC.md:2103`
- `docs/specs/CIV-0001-core-simulation-loop.md:957`

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
