# Intent: FR-CIV-CONTENT-001 -- Biome yield factor for food production

> Date: 2026-09-19
> FR: FR-CIV-CONTENT-001
> Epic: FR-CIV-CONTENT

## User Intent

The product owner requires Biome yield factor for food production as part of the FR-CIV-CONTENT epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that Biome yield factor for food production is properly specified, implemented, and testable within the simulation engine.

`crates/engine/src/emergence_coupling.rs::biome_yield_factor(biome)`
maps each `BiomeKind` (Rainforest, Wetland, Grassland, ..., Alpine,
Shrubland, Steppe) to a fixed-point food multiplier in the range
`[0.1, 1.5]`. Fertile biomes (Rainforest 1.3x, Wetland 1.2x,
Grassland 1.2x) yield more food; barren biomes (Desert 0.5x, Tundra
0.45x, Ocean 0.2x, Glacier 0.1x) yield less. Callers multiply
per-farm food output by the returned factor to obtain terrain-aware
production.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with a
fixed-point math kernel. FR-CIV-CONTENT-001 contributes to the
downward-causation policy set (`emergence_coupling.rs`) by binding
terrain properties to food output, so fertile settlements can
realistically grow without breaking determinism. The function is
`#[allow(dead_code)]` and reserved for future simulation integration
once `phase_production` reads the biome layer.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `crates/engine/` compiles and passes all checks
- [x] Pure function: no RNG / wall-clock input
- [x] Output clamped to `[0.1, 1.5]`
- [x] Mapping table covers every `BiomeKind` variant

### How We Know This FR Is Satisfied

1. `cargo build -p civ-engine` succeeds
2. All `BiomeKind` arms return a `Fixed` with magnitude in `[0.1, 1.5]`
3. Future-fan-in: when `phase_production` adopts the factor, output is
   deterministic per `(biome, faction, tick)` tuple

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-content-001-intent.md` |
| Source | `crates/engine/src/emergence_coupling.rs:471` |

<!-- Covers: FR-CIV-CONTENT-001 -->
