# Intent: FR-CIV-DIPLO-003-006 -- process_tick flow+enforcement integration

> Date: 2026-09-20
> FR: FR-CIV-DIPLO-003-006
> Epic: FR-CIV-DIPLO

## User Intent

`ShadowNetworkState::process_tick(flows, enforcement, tick)` must
apply all incoming flows, then enforce the supplied list, returning
the combined event stream in a single call.

### What This FR Achieves

Provides the single entry point used by the diplomacy tick phase;
guarantees that flows are recorded before enforcement runs so that
`LegitimacyModifier` reflects the post-flow state.

### Product Context

This is the integration surface for the FR-CIV-DIPLO-003 sub-series
and is the only shadow-network hook the simulation engine calls.

## Acceptance Signal

- Unit test `process_tick_records_flows_and_applies_enforcement` in
  `crates/diplomacy/src/shadow_networks.rs:658` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/diplomacy/src/shadow_networks.rs:658` |
| Implementing crate | `crates/diplomacy/src/` |
