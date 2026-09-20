# Intent: FR-CIV-NEEDS-DECAY-01 -- Configurable decay curves

> Date: 2026-09-19
> FR: FR-CIV-NEEDS-DECAY-01
> Epic: FR-CIV-NEEDS-DECAY

## User Intent

The product owner requires Configurable decay curves as part of the FR-CIV-NEEDS-DECAY epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that Configurable decay curves is properly specified, implemented, and testable within the simulation engine.

`crates/needs/src/decay.rs::tick_rise_curved(needs, rates, hunger_curve,
rest_curve, social_curve)` adds pressure to each `NeedLevel` channel
using a configurable `DecayCurve`. Three curves are supported per the
GitHub issue #959:

* `Linear` — `delta = rate` (matches the existing `tick_rise`).
* `Exponential { intensity }` — `delta = rate * (1 + pressure * intensity)`
  accelerates at high deprivation (starvation spiral).
* `Sigmoid { steepness }` — `delta = rate * sigmoid(pressure * steepness)`
  slow at extremes, fast in the middle (realistic hunger curve).

The base `tick_rise` function is preserved for backwards compatibility;
`tick_rise_curved` is the new per-channel configurable entry point.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS.
FR-CIV-NEEDS-DECAY-01 contributes to the additive needs-decay model
(`crates/needs`) by letting scenario authors tune each agent's hunger
/ rest / social pressure curve independently. Pure f32 arithmetic,
no RNG / wall-clock, matching the ADR-008 determinism invariants.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `crates/needs/` compiles and passes all checks
- [x] Unit tests pass for the new functionality
- [x] Integration with the simulation tick system works correctly
- [x] No regressions in existing FRs
- [x] Three curves verified by `linear_curve_matches_tick_rise`,
      `exponential_curve_accelerates`, `sigmoid_curve_has_s_shape`,
      `exponential_produces_higher_pressure`

### How We Know This FR Is Satisfied

1. `cargo test -p needs` passes
2. `Linear` matches `tick_rise` output exactly
3. `Exponential { intensity: 2.0 }` at `pressure=0.9` yields `delta=0.14`,
   at `pressure=0.1` yields `delta=0.06`
4. `Sigmoid { steepness: 10.0 }` is monotone non-decreasing in pressure

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-needs-decay-01-intent.md` |
| Source | `crates/needs/src/decay.rs:32` (curves), `:219` (`tick_rise_curved`) |
| Tests | `crates/needs/src/decay.rs:336`, `:361`, `:386` |

<!-- Covers: FR-CIV-NEEDS-DECAY-01 -->
