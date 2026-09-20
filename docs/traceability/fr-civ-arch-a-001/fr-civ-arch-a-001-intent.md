# Intent: FR-CIV-ARCH-A-001 — Facade style varies by culture

> FR: FR-CIV-ARCH-A-001
> Epic: FR-CIV-ARCH
> // Covers: FR-CIV-ARCH-A-001

## Requirement

The emergent facade style resolved for an architectural emergence key must
vary by culture id. Different cultures produce different facade style names
even when era, wealth, and biome are held constant.

## Source

`crates/build/src/tiers.rs:405` — test `fr_arch_a001_style_varies_by_culture`
exercises `facade_for_emergence(EmergentStyleKey::new(culture, era, wealth, biome), ...)`
with two cultures (`0` and `2`) at the same era/biome and asserts
`style_a.name != style_b.name`.

The implementing function is `facade_for_emergence` in
`crates/build/src/tiers.rs`, which composes `facade_for_vector` and
`apply_biome_facade_bias`.

## Acceptance

- `cargo test -p civ-build fr_arch_a001_style_varies_by_culture` passes.
- Culture-id permutation of an emergence key resolves to a different style
  name; era/wealth/biome permutations are held constant in this test.
