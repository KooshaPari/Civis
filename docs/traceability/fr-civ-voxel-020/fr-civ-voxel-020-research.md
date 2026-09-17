# Research: FR-CIV-VOXEL-020 -- Voxel rendering

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-VOXEL-020
> Epic: FR-CIV-VOXEL

## Research Question

What is the best approach to implement Voxel rendering within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-VOXEL epic and is expected to be implemented in `crates/voxel/src/`.

### Existing Code References
- `clients/bevy-ref/src/bin/standalone.rs:194`
- `clients/bevy-ref/src/voxel_stream.rs:13`
- `crates/voxel/src/stream.rs:32`
- `docs/guides/voxel-emergent-vision-and-migration.md:94`
- `docs/guides/voxel-emergent-vision-and-migration.md:119`
- `docs/guides/voxel-emergent-vision-and-migration.md:123`

### Test References
- `clients/bevy-ref/src/voxel_stream.rs:352`
- `clients/bevy-ref/src/voxel_stream.rs:364`

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
