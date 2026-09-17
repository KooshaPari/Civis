# Research: FR-CIV-SCALE-001 -- Scale and performance

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-SCALE-001
> Epic: FR-CIV-SCALE

## Research Question

What is the best approach to implement Scale and performance within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-SCALE epic and is expected to be implemented in `crates/engine/src/`.

### Existing Code References
- `crates/voxel/src/scale_budget.rs:1`
- `crates/voxel/src/scale_budget.rs:9`
- `crates/voxel/src/scale_budget.rs:57`
- `crates/voxel/src/scale_budget.rs:62`
- `crates/voxel/src/scale_budget.rs:82`
- `crates/voxel/src/scale_budget.rs:84`
- `crates/voxel/src/scale_budget.rs:209`
- `crates/voxel/src/scale_budget.rs:210`

### Test References
- `crates/voxel/src/scale_budget.rs:895`
- `crates/voxel/src/scale_budget.rs:907`
- `crates/voxel/src/scale_budget.rs:909`
- `crates/voxel/src/scale_budget.rs:924`
- `crates/voxel/src/scale_budget.rs:936`
- `crates/voxel/src/scale_budget.rs:951`

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
