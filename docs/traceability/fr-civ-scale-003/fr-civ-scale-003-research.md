# Research: FR-CIV-SCALE-003 -- Scale and performance

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-SCALE-003
> Epic: FR-CIV-SCALE

## Research Question

What is the best approach to implement Scale and performance within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-SCALE epic and is expected to be implemented in `crates/engine/src/`.

### Existing Code References
- `crates/voxel/src/scale_budget.rs:14`
- `crates/voxel/src/scale_budget.rs:404`
- `crates/voxel/src/scale_budget.rs:494`
- `crates/voxel/src/scale_budget.rs:495`
- `docs/design/streaming-window.md:225`

### Test References
- `crates/voxel/src/scale_budget.rs:1033`
- `crates/voxel/src/scale_budget.rs:1035`
- `crates/voxel/src/scale_budget.rs:1058`
- `crates/voxel/src/scale_budget.rs:1077`
- `crates/voxel/src/scale_budget.rs:1095`

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
