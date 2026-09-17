# Research: FR-CIV-TACTICS-024 -- Tactics and strategy

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-TACTICS-024
> Epic: FR-CIV-TACTICS

## Research Question

What is the best approach to implement Tactics and strategy within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-TACTICS epic and is expected to be implemented in `crates/tactics/src/`.

### Existing Code References
- `crates/engine/src/engine.rs:411`
- `crates/engine/src/engine.rs:2245`
- `crates/tactics/src/war_bridge.rs:29`
- `docs/development-guide/p-w1-kickoff.md:27`

### Test References
- `crates/engine/src/engine.rs:3278`
- `crates/tactics/src/war_bridge.rs:351`
- `crates/tactics/tests/fr_matrix_batch2.rs:201`
- `crates/tactics/tests/fr_matrix_batch2.rs:204`
- `crates/tactics/tests/fr_matrix_batch2.rs:205`
- `crates/tactics/tests/fr_matrix_batch5.rs:14`
- `crates/tactics/tests/fr_matrix_batch5.rs:208`
- `crates/tactics/tests/fr_matrix_batch5.rs:209`

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
