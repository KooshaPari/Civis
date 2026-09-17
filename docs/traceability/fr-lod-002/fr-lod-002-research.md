# Research: FR-LOD-002 -- Level of detail

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-LOD-002
> Epic: FR-LOD

## Research Question

What is the best approach to implement Level of detail within the Civis simulation engine?

## Background

This FR belongs to the FR-LOD epic and is expected to be implemented in `crates/engine/src/`.

### Existing Code References
- `crates/engine/src/lod.rs:63`

### Test References
- `crates/engine/src/lod.rs:102`
- `crates/engine/tests/fr_matrix_batch1.rs:13`
- `crates/engine/tests/fr_matrix_batch1.rs:60`
- `crates/engine/tests/fr_matrix_batch1.rs:61`
- `crates/engine/tests/fr_matrix_batch1.rs:62`
- `crates/engine/tests/fr_matrix_batch3.rs:27`
- `crates/engine/tests/fr_matrix_batch3.rs:28`

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
