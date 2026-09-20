# Intent: FR-CIV-DIPLO-003-05 -- Symmetric pair handling

> Date: 2026-09-20
> FR: FR-CIV-DIPLO-003-05
> Epic: FR-CIV-DIPLO

## User Intent

Flows from `a -> b` and from `b -> a` must aggregate under the same
canonical `Pair` (always the lexicographically smaller `PolityId`
first), and both directions must contribute to the same
`flow_count`, `total_leakage`, and `flows_by_type` map.

### What This FR Achieves

Prevents ledger duplication when flow direction flips; ensures
aggregate queries return the true bilateral leakage regardless of
which side initiated the flow.

### Product Context

Pairs with FR-CIV-DIPLO-003-01 (record + update) and
FR-CIV-DIPLO-003-04 (logging) to make the ledger direction-agnostic.

## Acceptance Signal

- Unit test `bidirectional_flows_share_pair_aggregate` in
  `crates/diplomacy/src/shadow_networks.rs:620` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/diplomacy/src/shadow_networks.rs:620` |
| Code + test | `crates/diplomacy/src/shadow_networks.rs:622` |
| Implementing crate | `crates/diplomacy/src/` |
