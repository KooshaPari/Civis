# Research: FR-SAVE-002 -- Save system

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-SAVE-002
> Epic: FR-SAVE

## Research Question

What is the best approach to implement Save system within the Civis simulation engine?

## Background

This FR belongs to the FR-SAVE epic and is expected to be implemented in `crates/save-db/src/`.

### Existing Code References
- `crates/engine/src/replay.rs:297`
- `docs/specs/CIV-1000-save-load-persistence-spec.md:2801`

### Test References
- `crates/engine/src/save.rs:278`
- `crates/engine/tests/fr_matrix_batch1.rs:17`
- `crates/engine/tests/fr_matrix_batch1.rs:322`
- `crates/engine/tests/fr_matrix_batch1.rs:323`
- `crates/engine/tests/fr_matrix_batch1.rs:324`
- `crates/engine/tests/fr_matrix_batch3.rs:151`
- `crates/engine/tests/fr_matrix_batch3.rs:152`

## Findings

### Codebase Analysis
- The `crates/save-db/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/save-db/src/`
2. Add integration tests in `crates/save-db/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/save-db/` crate documentation
