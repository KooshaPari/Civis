# Research: FR-CIV-INSPECT-901 -- Inspection tools

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-INSPECT-901
> Epic: FR-CIV-INSPECT

## Research Question

What is the best approach to implement Inspection tools within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-INSPECT epic and is expected to be implemented in `crates/hud/src/`.

### Existing Code References
- `docs/agileplus/epics/civ-w4-perception.md:18`
- `docs/agileplus/epics/civ-w4-perception.md:36`
- `docs/agileplus/README.md:23`
- `docs/specs/requirements/FR-CIV-INSPECT.md:12`
- `docs/specs/requirements/FR-CIV-PSYCHE.md:12`

### Test References
> _No test coverage yet._

## Findings

### Codebase Analysis
- The `crates/hud/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/hud/src/`
2. Add integration tests in `crates/hud/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/hud/` crate documentation
