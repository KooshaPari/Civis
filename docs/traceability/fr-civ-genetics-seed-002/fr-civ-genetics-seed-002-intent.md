# Intent: FR-CIV-GENETICS-SEED-002 -- Three distinct archetypes in 128 spawns

> Date: 2026-09-19
> FR: FR-CIV-GENETICS-SEED-002
> Epic: FR-CIV-GENETICS-SEED

## User Intent

The product owner requires Three distinct archetypes in 128 spawns as part of the FR-CIV-GENETICS-SEED epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that Three distinct archetypes in 128 spawns is properly specified, implemented, and testable within the simulation engine.

`crates/engine/src/engine/engine_tests.rs::test_faction_archetype_variety`
asserts that `archetype_dna` returns three pairwise-distinct genomes
for `NamedSeed::Ardani`, `Velthari`, and `Grundak`, and that running
`Simulation::with_seed(1)` produces 128 civilians whose `Dna` set
contains at least 3 unique genomes — proving the round-robin
`named_seed_round_robin(spawn_index)` actually cycles through the
archetype list at spawn time.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS.
FR-CIV-GENETICS-SEED-002 contributes to the named-race diversity
contract by guaranteeing that the `% 3` rotation is exercised by the
default spawn path. Without three distinct archetypes, the
round-robin would collapse into a single race and the speciation
distance test (FR-CIV-EMERGENCE-004) would have nothing to compare.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `crates/engine/` compiles and passes all checks
- [x] Unit tests pass for the new functionality
- [x] Integration with the simulation tick system works correctly
- [x] No regressions in existing FRs
- [x] Three named archetypes are pairwise distinct
- [x] 128-spawn simulation yields ≥ 3 unique `Dna` values

### How We Know This FR Is Satisfied

1. `cargo test -p civ-engine test_faction_archetype_variety` passes
2. `archetype_dna(Ardani) != archetype_dna(Velthari)`
3. `archetype_dna(Ardani) != archetype_dna(Grundak)`
4. `archetype_dna(Velthari) != archetype_dna(Grundak)`
5. `unique_count >= 3` across all 128 spawned DNA components

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-genetics-seed-002-intent.md` |
| Test | `crates/engine/src/engine/engine_tests.rs:2991` |
| Source | `crates/genetics/src/seeds.rs::archetype_dna`, `named_seed_round_robin` |

<!-- Covers: FR-CIV-GENETICS-SEED-002 -->
