# Intent: FR-CIV-COHESION-001 — Cohesion phase

> Date: 2026-09-20
> FR: FR-CIV-COHESION-001
> Epic: FR-CIV-COHESION

## What This FR Captures

The cohesion phase of the social/settlement pipeline —
`Engine::phase_cohesion` — which runs once per tick to compute each
settlement's social-fabric snapshot. The phase aggregates per-actor
fabric scores derived from kinship, trust, hardship, and
institutions, then emits `CohesionEvent`s (`Strengthened`,
`Weakened`, `Fragmented`) whenever an actor's fabric moves
appreciably between ticks. Each settlement's snapshot is stored in
`last_tick_cohesion_snapshots` and consumed by later phases.

## User Intent

Players should see a settlement's social cohesion shift over time —
sometimes growing stronger through shared kin/trust/institutions,
sometimes fragmenting when hardship or trust collapse pushes fabric
below -50. The cohesion phase is the source of those signals.

## Acceptance Signal

- `phase_cohesion` runs deterministically each tick given identical
  state.
- `last_tick_cohesion_events` is populated with one event per
  actor whose |delta| > 5 or whose fabric dropped below -50.
- `last_tick_cohesion_snapshots` is populated with one snapshot per
  occupied settlement.

## Implementing Code

- `crates/engine/src/engine/social_settlement_phases.rs:272` —
  `phase_cohesion` definition with FR header.

## Test Coverage

- Tested implicitly via engine integration tests and the
  cohesion event-log API; no dedicated `fr_cohesion_001_*` test in
  the engine test suite today.

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-cohesion-001-intent.md` |
| Implementing crate | `crates/engine/src/engine/` |