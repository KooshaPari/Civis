# Research: FR-CIV-CA-008 -- Combat and military

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-CA-008
> Epic: FR-CIV-CA

## Research Question

What is the best approach to implement Combat and military within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-CA epic and is expected to be implemented in `crates/ai/src/`.

### Existing Code References
- `crates/voxel/src/fluid_ca.rs:33`
- `crates/voxel/src/fluid_ca.rs:60`
- `crates/voxel/src/fluid_ca.rs:85`
- `crates/voxel/src/fluid_ca.rs:248`
- `crates/voxel/src/fluid_ca.rs:257`
- `crates/voxel/src/fluid_ca.rs:265`
- `crates/voxel/src/fluid_ca.rs:694`
- `crates/voxel/src/fluid_ca.rs:1076`

### Test References
- `crates/voxel/src/fluid_ca.rs:2000`
- `crates/voxel/src/fluid_ca.rs:2005`
- `crates/voxel/src/fluid_ca.rs:2032`
- `crates/voxel/src/fluid_ca.rs:2033`

## Findings

### Codebase Analysis
- The `crates/ai/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/ai/src/`
2. Add integration tests in `crates/ai/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/ai/` crate documentation
