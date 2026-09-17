# Research: FR-CIV-TACTICS-035 -- Tactics and strategy

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-TACTICS-035
> Epic: FR-CIV-TACTICS

## Research Question

What is the best approach to implement Tactics and strategy within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-TACTICS epic and is expected to be implemented in `crates/tactics/src/`.

### Existing Code References
- `crates/engine/src/engine.rs:434`
- `crates/tactics/src/military_phase.rs:1`
- `docs/development-guide/p-w1-kickoff.md:34`

### Test References
- `crates/engine/src/engine.rs:3136`
- `crates/engine/src/scenario.rs:317`

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
