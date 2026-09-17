# Research: FR-CIV-TACTICS-021 -- Tactics and strategy

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-TACTICS-021
> Epic: FR-CIV-TACTICS

## Research Question

What is the best approach to implement Tactics and strategy within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-TACTICS epic and is expected to be implemented in `crates/tactics/src/`.

### Existing Code References
- `crates/tactics/src/formation.rs:1`
- `docs/development-guide/fr-3d-additions.md:90`
- `docs/development-guide/p-w1-kickoff.md:24`

### Test References
- `crates/tactics/src/formation.rs:205`
- `crates/tactics/src/lib.rs:303`
- `crates/tactics/tests/fr_matrix_batch2.rs:126`
- `crates/tactics/tests/fr_matrix_batch2.rs:129`
- `crates/tactics/tests/fr_matrix_batch2.rs:130`
- `crates/tactics/tests/fr_matrix_batch5.rs:11`
- `crates/tactics/tests/fr_matrix_batch5.rs:162`
- `crates/tactics/tests/fr_matrix_batch5.rs:163`

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
