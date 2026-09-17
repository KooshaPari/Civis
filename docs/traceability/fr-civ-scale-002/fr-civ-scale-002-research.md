# Research: FR-CIV-SCALE-002 -- Scale and performance

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-SCALE-002
> Epic: FR-CIV-SCALE

## Research Question

What is the best approach to implement Scale and performance within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-SCALE epic and is expected to be implemented in `crates/engine/src/`.

### Existing Code References
- `crates/voxel/src/scale_budget.rs:11`
- `crates/voxel/src/scale_budget.rs:277`
- `crates/voxel/src/scale_budget.rs:284`
- `crates/voxel/src/scale_budget.rs:300`
- `crates/voxel/src/scale_budget.rs:305`
- `crates/voxel/src/scale_budget.rs:306`
- `crates/voxel/src/scale_budget.rs:317`
- `crates/voxel/src/scale_budget.rs:367`

### Test References
- `crates/voxel/src/scale_budget.rs:970`
- `crates/voxel/src/scale_budget.rs:972`
- `crates/voxel/src/scale_budget.rs:985`
- `crates/voxel/src/scale_budget.rs:1006`
- `crates/voxel/src/scale_budget.rs:1025`

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
