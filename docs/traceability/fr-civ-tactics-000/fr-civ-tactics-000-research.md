# Research: FR-CIV-TACTICS-000 -- Tactics and strategy

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-TACTICS-000
> Epic: FR-CIV-TACTICS

## Research Question

What is the best approach to implement Tactics and strategy within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-TACTICS epic and is expected to be implemented in `crates/tactics/src/`.

### Existing Code References
- `docs/development-guide/fr-3d-additions.md:85`
- `docs/development-guide/p-w1-kickoff.md:20`

### Test References
- `crates/tactics/src/lib.rs:208`
- `crates/tactics/tests/fr_matrix_batch2.rs:36`
- `crates/tactics/tests/fr_matrix_batch2.rs:39`
- `crates/tactics/tests/fr_matrix_batch2.rs:40`
- `crates/tactics/tests/fr_matrix_batch5.rs:7`
- `crates/tactics/tests/fr_matrix_batch5.rs:76`
- `crates/tactics/tests/fr_matrix_batch5.rs:77`

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
