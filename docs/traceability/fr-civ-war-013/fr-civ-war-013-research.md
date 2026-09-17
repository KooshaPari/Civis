# Research: FR-CIV-WAR-013 -- War and conflict

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-WAR-013
> Epic: FR-CIV-WAR

## Research Question

What is the best approach to implement War and conflict within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-WAR epic and is expected to be implemented in `crates/tactics/src/`.

### Existing Code References
- `docs/design/warfare.md:89`
- `docs/design/warfare.md:194`

### Test References
> _No test coverage yet._

## Findings

### Codebase Analysis
- The `crates/tactics/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/tactics/src/`
2. Add integration tests in `crates/tactics/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/tactics/` crate documentation
