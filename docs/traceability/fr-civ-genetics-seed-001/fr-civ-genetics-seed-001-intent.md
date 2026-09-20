# Intent: FR-CIV-GENETICS-SEED-001 -- First-spawn determinism from named archetype

> Date: 2026-09-19
> FR: FR-CIV-GENETICS-SEED-001
> Epic: FR-CIV-GENETICS-SEED

## User Intent

The product owner requires First-spawn determinism from named archetype as part of the FR-CIV-GENETICS-SEED epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that First-spawn determinism from named archetype is properly specified, implemented, and testable within the simulation engine.

`crates/engine/src/engine/engine_tests.rs::test_seed_spawn_determinism`
asserts that two `Simulation::with_seed(0xC0FFEE)` instances produce
byte-identical `Dna` components across every spawned entity, that the
genome length matches the `archetype_dna(NamedSeed::Ardani)` length
(64 bytes), and that the resulting DNA is not the zero genome — i.e.
the Ardani archetype was actually applied at spawn time rather than
left as the substrate default.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS.
FR-CIV-GENETICS-SEED-001 contributes to the spawn-time determinism
contract by proving that the named-race archetype is wired into the
genetics substrate: same seed → same world → same genomes, even with
non-trivial `divergence=0.3` applied.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `crates/engine/` compiles and passes all checks
- [x] Unit tests pass for the new functionality
- [x] Integration with the simulation tick system works correctly
- [x] No regressions in existing FRs
- [x] Two same-seed simulations yield bit-identical `Dna` lists

### How We Know This FR Is Satisfied

1. `cargo test -p civ-engine test_seed_spawn_determinism` passes
2. `dna_a == dna_b` element-wise across all entities
3. `dna_a[0].0.len() == archetype.0.len() == 64`
4. `dna_a[0].0 != vec![0u8; 64]` (archetype was actually seeded)

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-genetics-seed-001-intent.md` |
| Test | `crates/engine/src/engine/engine_tests.rs:2941` |
| Source | `crates/genetics/src/seeds.rs::archetype_dna` |

<!-- Covers: FR-CIV-GENETICS-SEED-001 -->
