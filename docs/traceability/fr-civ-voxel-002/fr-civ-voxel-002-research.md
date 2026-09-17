# Research: FR-CIV-VOXEL-002 -- Voxel rendering

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-VOXEL-002
> Epic: FR-CIV-VOXEL

## Research Question

What is the best approach to implement Voxel rendering within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-VOXEL epic and is expected to be implemented in `crates/voxel/src/`.

### Existing Code References
- `crates/engine/src/engine.rs:478`
- `crates/engine/src/engine.rs:1308`
- `crates/engine/src/engine.rs:1329`
- `crates/engine/src/engine.rs:1365`
- `docs/development-guide/fr-3d-additions.md:18`
- `docs/guides/voxel-emergent-vision-and-migration.md:183`

### Test References
- `crates/engine/src/engine.rs:2561`
- `crates/engine/src/engine.rs:2605`
- `crates/voxel/src/lib.rs:100`
- `crates/voxel/src/lib.rs:198`
- `crates/voxel/src/lib.rs:199`

## Findings

### Codebase Analysis
- The `crates/voxel/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/voxel/src/`
2. Add integration tests in `crates/voxel/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/voxel/` crate documentation
