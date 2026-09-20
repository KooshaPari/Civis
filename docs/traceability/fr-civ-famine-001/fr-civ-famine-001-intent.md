# Intent: FR-CIV-FAMINE-001 -- Food collapse cascade model

> Date: 2026-09-20
> FR: FR-CIV-FAMINE-001
> Epic: FR-CIV-FAMINE

## User Intent

When a settlement's `food_per_capita` drops below critical
thresholds, the simulation must apply a four-stage famine cascade
whose per-stage effects feed into the existing subsystems
(unrest, labor, migration, diplomacy).

### What This FR Achieves

Defines `FamineStage` (None / Hungry / Starving / Famine /
Collapse) and the per-stage `FamineEffects` (unrest_delta,
labor_multiplier, migration_pressure, trade_desperation,
death_risk). Each stage feeds:

- Unrest → faction_decisions (protests, regime change)
- Labor reduction → economy (lower production)
- Migration pressure → agents (emigration decisions)
- Trade desperation → diplomacy (forced trade agreements)

### Product Context

Civis is a Rust-based civilisation simulation. Famine is the
highest-leverage crisis mode because it touches every other
subsystem; this FR is the canonical entry point.

## Acceptance Signal

- `crates/engine/src/famine.rs` compiles and exposes the four
  stages and the `FamineEffects::for_stage(food_per_capita)`
  helper.
- Unit tests for stage thresholds live in `crates/engine/src/famine.rs`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/engine/src/famine.rs:1` |
| Implementing crate | `crates/engine/src/` |
