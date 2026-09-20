# Intent: FR-CIV-DIPLO-003-01 -- Shadow flow record + pair aggregate update

> Date: 2026-09-20
> FR: FR-CIV-DIPLO-003-01
> Epic: FR-CIV-DIPLO

## User Intent

A single `ShadowFlow` recorded via
`ShadowNetworkState::record_flow` must update the canonical
`(source, destination)` pair aggregate, increment the system-wide
`total_leakage`, and produce exactly one `FlowRecorded` event.

### What This FR Achieves

Defines the base write path for shadow-network leakage: every flow
is attributed to a pair and tracked by type.

### Product Context

Pairs with FR-CIV-DIPLO-003-04 (logging completeness) and
FR-CIV-DIPLO-003-05 (symmetric pair handling) to make the
shadow-network ledger auditable.

## Acceptance Signal

- Unit tests `record_flow_updates_pair_aggregate_and_total_leakage`
  and `multiple_flows_accumulate_by_type` in
  `crates/diplomacy/src/shadow_networks.rs:403` pass.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/diplomacy/src/shadow_networks.rs:403` |
| Code + test | `crates/diplomacy/src/shadow_networks.rs:405` |
| Code + test | `crates/diplomacy/src/shadow_networks.rs:433` |
| Implementing crate | `crates/diplomacy/src/` |
