# Intent: FR-CIV-DIPLO-003-04 -- Every recorded flow is logged

> Date: 2026-09-20
> FR: FR-CIV-DIPLO-003-04
> Epic: FR-CIV-DIPLO

## User Intent

`drain_events` must return one `FlowRecorded` event for every
flow ever supplied to `record_flow` — no flow is silently dropped.

### What This FR Achieves

Establishes the logging completeness contract for the
shadow-network event buffer, which is the audit trail for replay.

### Product Context

Replay tools rely on this buffer to reproduce diplomacy outcomes;
silently dropping a flow would corrupt the determinism guarantee.

## Acceptance Signal

- Unit test `every_flow_is_logged` in
  `crates/diplomacy/src/shadow_networks.rs:594` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/diplomacy/src/shadow_networks.rs:594` |
| Code + test | `crates/diplomacy/src/shadow_networks.rs:596` |
| Implementing crate | `crates/diplomacy/src/` |
