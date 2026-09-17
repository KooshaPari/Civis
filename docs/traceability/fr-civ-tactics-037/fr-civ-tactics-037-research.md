# Research: FR-CIV-TACTICS-037 -- Tactics and strategy

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-TACTICS-037
> Epic: FR-CIV-TACTICS

## Research Question

What is the best approach to implement Tactics and strategy within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-TACTICS epic and is expected to be implemented in `crates/tactics/src/`.

### Existing Code References
- `crates/tactics/src/pathfinding.rs:81`
- `crates/tactics/src/pathfinding.rs:84`
- `crates/tactics/src/pathfinding.rs:103`
- `docs/development-guide/p-w1-kickoff.md:37`

### Test References
- `crates/tactics/src/pathfinding.rs:241`
- `crates/tactics/tests/fr_matrix_batch5.rs:16`
- `crates/tactics/tests/fr_matrix_batch5.rs:234`
- `crates/tactics/tests/fr_matrix_batch5.rs:235`

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
