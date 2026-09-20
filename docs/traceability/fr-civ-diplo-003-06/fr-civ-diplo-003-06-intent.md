# Intent: FR-CIV-DIPLO-003-06 -- reset_tick preserves legitimacy

> Date: 2026-09-20
> FR: FR-CIV-DIPLO-003-06
> Epic: FR-CIV-DIPLO

## User Intent

`ShadowNetworkState::reset_tick` must zero the per-tick counters
(`active_pair_count`, `total_leakage`, per-polity
`enforcement_intensity`) while preserving the cumulative
`legitimacy_modifier`.

### What This FR Achieves

Distinguishes per-tick ephemeral state from cumulative
state-modifiers; legacy hits to legitimacy persist across ticks
while pair counters do not.

### Product Context

Replay and persistence both rely on this boundary: cumulative
legitimacy is part of the world archive, but per-tick pair counts
are tick-scoped scratch state.

## Acceptance Signal

- Unit test `process_tick_records_flows_and_applies_enforcement`
  in `crates/diplomacy/src/shadow_networks.rs:658` exercises the
  path; the per-state reset assertion is enforced via
  `reset_tick_clears_counters_preserves_legitimacy`.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/diplomacy/src/shadow_networks.rs:658` |
| Implementing crate | `crates/diplomacy/src/` |
