# Intent: FR-CIV-FEST-001 — Festival & celebration system

> Date: 2026-09-20
> FR: FR-CIV-FEST-001
> Epic: FR-CIV-FEST

## What This FR Captures

The full festival engine in `crates/engine/src/festivals.rs`. A
`FestivalEngine` is evaluated each tick against settlement state;
when conditions are met (resource surplus, recent victory, belief
cohesion, cultural output, trade surplus, or unrest emergency)
and per-settlement cooldowns have elapsed, a festival spawns with
a `FestivalType` (`Harvest`, `Victory`, `Religious`, `Cultural`,
`Market`, `Emergency`). Active festivals tick down in duration;
`apply_effects` returns the `FestivalEffects` deltas that feed
the economy, unrest, and social subsystems.

## User Intent

Settlements need visible moments of celebration — harvest feasts,
victory parades, religious holidays, market days — to break up
the grind and to provide narrative texture. Festivals are the
gameplay-visible reward for hitting prosperity or morale
thresholds; they also impose a short productivity cost so
they're not a pure bonus.

## Acceptance Signal

- `FestivalType` enum has 6 variants; default is `Harvest`.
- `FestivalEngine::tick` evaluates each settlement and emits
  festivals subject to cooldowns.
- Active festivals decrement duration each tick.
- `apply_effects` produces `FestivalEffects` with happiness /
  unrest / economy deltas.
- All math is deterministic.

## Implementing Code

- `crates/engine/src/festivals.rs:1` — module header w/ FR ref
  and design summary.

## Test Coverage

> Module-level unit tests live in `crates/engine/src/festivals.rs`
> and integration coverage is via the engine test suite.

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-fest-001-intent.md` |
| Implementing crate | `crates/engine/src/` |