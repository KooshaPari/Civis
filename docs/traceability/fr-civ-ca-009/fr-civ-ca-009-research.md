# Research: FR-CIV-CA-009 -- Combat and military

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-CA-009
> Epic: FR-CIV-CA

## Research Question

What is the best approach to implement Combat and military within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-CA epic and is expected to be implemented in `crates/ai/src/`.

### Existing Code References
- `crates/engine/src/engine.rs:417`
- `crates/engine/src/engine.rs:1449`
- `crates/engine/src/engine.rs:1501`
- `crates/voxel/src/fluid_ca.rs:124`
- `crates/voxel/src/fluid_ca.rs:141`

### Test References
- `crates/engine/src/engine.rs:3343`
- `crates/engine/src/engine.rs:3346`
- `crates/engine/src/engine.rs:3356`
- `crates/voxel/src/fluid_ca.rs:2105`
- `crates/voxel/src/fluid_ca.rs:2110`

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
