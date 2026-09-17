# Research: FR-CIV-SCALE-004 -- Scale and performance

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-SCALE-004
> Epic: FR-CIV-SCALE

## Research Question

What is the best approach to implement Scale and performance within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-SCALE epic and is expected to be implemented in `crates/engine/src/`.

### Existing Code References
- `crates/voxel/src/scale_budget.rs:18`
- `crates/voxel/src/scale_budget.rs:667`
- `crates/voxel/src/scale_budget.rs:798`
- `crates/voxel/src/scale_budget.rs:799`
- `docs/design/streaming-window.md:150`

### Test References
- `crates/voxel/src/scale_budget.rs:1113`
- `crates/voxel/src/scale_budget.rs:1115`
- `crates/voxel/src/scale_budget.rs:1127`
- `crates/voxel/src/scale_budget.rs:1156`
- `crates/voxel/src/scale_budget.rs:1174`
- `crates/voxel/src/scale_budget.rs:1201`

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
