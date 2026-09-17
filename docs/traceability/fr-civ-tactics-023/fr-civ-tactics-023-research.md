# Research: FR-CIV-TACTICS-023 -- Tactics and strategy

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-TACTICS-023
> Epic: FR-CIV-TACTICS

## Research Question

What is the best approach to implement Tactics and strategy within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-TACTICS epic and is expected to be implemented in `crates/tactics/src/`.

### Existing Code References
- `crates/tactics/src/doctrine_fitness.rs:1`
- `docs/design/warfare.md:122`
- `docs/development-guide/p-w1-kickoff.md:26`

### Test References
- `crates/tactics/src/lib.rs:450`
- `crates/tactics/tests/fr_matrix_batch2.rs:179`
- `crates/tactics/tests/fr_matrix_batch2.rs:182`
- `crates/tactics/tests/fr_matrix_batch2.rs:183`
- `crates/tactics/tests/fr_matrix_batch5.rs:13`
- `crates/tactics/tests/fr_matrix_batch5.rs:188`
- `crates/tactics/tests/fr_matrix_batch5.rs:189`

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
