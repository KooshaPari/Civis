# Intent: FR-CIV-ARCH-A-003 — Facade style varies by era

> FR: FR-CIV-ARCH-A-003
> Epic: FR-CIV-ARCH
> // Covers: FR-CIV-ARCH-A-003

## Requirement

The emergent facade style resolved for an emergence key must progress with
era index. Higher era values yield later facade style names at the same
culture / biome / wealth.

## Source

`crates/build/src/tiers.rs:448` — test `fr_arch_a003_style_varies_by_era`
invokes `facade_for_emergence` with the same culture (`0`), biome
(`NEUTRAL`), and wealth (`500`) but era `0` vs era `5` and asserts
`style_early.name != style_late.name`.

Era resolution is performed by `facade_for_vector` inside `tiers.rs`,
which maps `EmergentStyleKey::era` into the canonical style naming.

## Acceptance

- `cargo test -p civ-build fr_arch_a003_style_varies_by_era` passes.
- Holding culture, biome, and wealth constant and increasing era index
  yields a different style name.
