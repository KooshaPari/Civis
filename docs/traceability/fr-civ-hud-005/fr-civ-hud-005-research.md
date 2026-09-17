# Research: FR-CIV-HUD-005 -- HUD overlay

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-HUD-005
> Epic: FR-CIV-HUD

## Research Question

What is the best approach to implement HUD overlay within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-HUD epic and is expected to be implemented in `crates/hud/src/`.

### Existing Code References
- `crates/voxel/src/hud.rs:11`
- `crates/voxel/src/hud.rs:636`
- `docs/reference/agileplus-artifacts-index.md:201`
- `docs/reference/agileplus-artifacts-index.md:304`

### Test References
- `crates/voxel/src/hud.rs:900`
- `crates/voxel/src/hud.rs:902`
- `crates/voxel/src/hud.rs:919`

## Findings

### Codebase Analysis
- The `crates/hud/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/hud/src/`
2. Add integration tests in `crates/hud/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/hud/` crate documentation
