# Intent: FR-CIV-ARCH-A-002 — Facade materials vary by biome

> FR: FR-CIV-ARCH-A-002
> Epic: FR-CIV-ARCH
> // Covers: FR-CIV-ARCH-A-002

## Requirement

The emergent facade style resolved for an emergence key must shift its
material palette when the biome tag changes. Arid biomes produce a different
material id list from forest biomes at the same culture / era / wealth.

## Source

`crates/build/src/tiers.rs:426` — test `fr_arch_a002_style_varies_by_biome`
invokes `facade_for_emergence` with the same culture (`1`), era (`2`), and
wealth (`500`) but `BiomeStyleTag::ARID` vs `BiomeStyleTag::FOREST` and
asserts `style_arid.materials != style_forest.materials`.

`apply_biome_facade_bias` (also in `crates/build/src/tiers.rs`) is the
mechanism that mutates the facade's material ids based on the biome tag.

## Acceptance

- `cargo test -p civ-build fr_arch_a002_style_varies_by_biome` passes.
- Holding culture, era, and wealth constant and varying biome shifts the
  material palette; style name may also change but material ids must differ.
