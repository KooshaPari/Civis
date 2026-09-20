# Intent: FR-CIV-GENETICS-SEED-003 -- Zero-divergence archetype clone contract

> Date: 2026-09-19
> FR: FR-CIV-GENETICS-SEED-003
> Epic: FR-CIV-GENETICS-SEED

## User Intent

The product owner requires Zero-divergence archetype clone contract as part of the FR-CIV-GENETICS-SEED epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that Zero-divergence archetype clone contract is properly specified, implemented, and testable within the simulation engine.

`crates/genetics/src/seeds.rs::seed_with_divergence(base, divergence, rng)`
defines the spawn-time helper that interpolates between a base
genome and a fully random genome. At `divergence = 0.0` the function
returns `base.clone()` byte-for-byte — no RNG bytes are consumed. At
`divergence = 1.0` every byte is randomised in `[0, 255]`. The
`test_zero_divergence_exact` engine test pins the lower endpoint.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS.
FR-CIV-GENETICS-SEED-003 contributes to the divergence dial contract
by making the lower bound (`divergence=0.0`) deterministic across
RNG streams. Scenario authors who want a fixed-archetype run can
lock the dial to 0 without worrying about RNG drift.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `crates/genetics/` compiles and passes all checks
- [x] Unit tests pass for the new functionality
- [x] Zero-divergence contract verified in `crates/genetics/src/seeds.rs` and
      `crates/engine/src/engine/engine_tests.rs::test_zero_divergence_exact`
- [x] No regressions in existing FRs

### How We Know This FR Is Satisfied

1. `cargo test -p civ-genetics test_zero_divergence_returns_archetype` passes
2. `seed_with_divergence(&archetype, 0.0, &mut rng) == archetype` byte-for-byte
3. No RNG bytes are consumed when divergence is 0 (verified by side test
   `divergence_dial_zero_means_no_drift_over_generations`)
4. The same property holds for `spawn_genome` / `spawn_genome_with_divergence`

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-genetics-seed-003-intent.md` |
| Test | `crates/engine/src/engine/engine_tests.rs:3037` |
| Source | `crates/genetics/src/seeds.rs::seed_with_divergence`, `spawn_genome` |

<!-- Covers: FR-CIV-GENETICS-SEED-003 -->
