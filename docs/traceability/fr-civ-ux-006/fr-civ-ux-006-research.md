# Research: FR-CIV-UX-006 -- User experience

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-UX-006
> Epic: FR-CIV-UX

## Research Question

What is the best approach to implement User experience within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-UX epic and is expected to be implemented in `crates/hud/src/`.

### Existing Code References
- `clients/godot-ref/rust/src/ux.rs:52`
- `clients/unreal-show/Intermediate/Build/Win64/UnrealEditor/Inc/CivShow/UHT/CivProtocolClient.gen.cpp:427`
- `clients/unreal-show/Intermediate/Build/Win64/UnrealEditor/Inc/CivShow/UHT/CivProtocolClient.gen.cpp:431`
- `clients/unreal-show/Source/CivShow/CivProtocolClient.h:32`
- `crates/engine/src/spawn.rs:1`
- `crates/server/src/jsonrpc.rs:58`
- `crates/server/src/jsonrpc.rs:868`
- `crates/server/src/jsonrpc.rs:1381`

### Test References
- `crates/server/tests/ws_smoke.rs:1715`
- `crates/server/tests/ws_smoke.rs:1716`

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
