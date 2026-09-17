# Research: FR-CIV-TACTICS-033 -- Tactics and strategy

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-TACTICS-033
> Epic: FR-CIV-TACTICS

## Research Question

What is the best approach to implement Tactics and strategy within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-TACTICS epic and is expected to be implemented in `crates/tactics/src/`.

### Existing Code References
- `crates/tactics/src/pathfinding.rs:1`
- `docs/development-guide/p-w1-kickoff.md:32`

### Test References
- `crates/tactics/src/lib.rs:353`
- `crates/tactics/tests/fr_matrix_batch2.rs:290`
- `crates/tactics/tests/fr_matrix_batch2.rs:293`
- `crates/tactics/tests/fr_matrix_batch2.rs:294`
- `crates/tactics/tests/fr_matrix_batch5.rs:15`
- `crates/tactics/tests/fr_matrix_batch5.rs:223`
- `crates/tactics/tests/fr_matrix_batch5.rs:224`

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
