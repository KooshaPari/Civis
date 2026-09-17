# Research: FR-CIV-0001 -- Core civilisation simulation

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-0001
> Epic: FR-CIV

## Research Question

What is the best approach to implement Core civilisation simulation within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV epic and is expected to be implemented in `crates/engine/src/`.

### Existing Code References
- `docs/guides/GIT_WORKTREE_GUIDE.md:151`
- `PLAN.md:16`

### Test References
- `crates/build/tests/fr_matrix_batch12.rs:102`
- `crates/build/tests/fr_matrix_batch12.rs:105`

## Findings

### Codebase Analysis
- The `crates/engine/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/engine/src/`
2. Add integration tests in `crates/engine/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/engine/` crate documentation
