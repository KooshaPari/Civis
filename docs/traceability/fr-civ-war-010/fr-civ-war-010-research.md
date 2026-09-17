# Research: FR-CIV-WAR-010 -- War and conflict

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-WAR-010
> Epic: FR-CIV-WAR

## Research Question

What is the best approach to implement War and conflict within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-WAR epic and is expected to be implemented in `crates/tactics/src/`.

### Existing Code References
- `docs/design/warfare.md:77`
- `docs/design/warfare.md:191`

### Test References
- `crates/tactics/src/lib.rs:391`
- `crates/tactics/src/lib.rs:417`
- `crates/tactics/src/war_bridge.rs:460`

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
