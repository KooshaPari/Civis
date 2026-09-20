# Intent: FR-CIV-PLANET-050 — Whittaker biome classifier

> FR: FR-CIV-PLANET-050
> Epic: FR-CIV-PLANET
> // Covers: FR-CIV-PLANET-050

## Requirement

`classify_biome(latitude, temperature, precipitation)` maps a per-cell
climate triple onto a `BiomeKind` using the Whittaker classification.
Specifically, the planet crate commits to these label edges:

- hot + semi-arid → `Shrubland`
- cold + dry → `Steppe`
- high elevation + cold (non-glacial) → `Alpine`
- hot + wet shore → `Mangrove`
- hot + very wet → `Rainforest`
- cold + wet → `Taiga`

## Source

`crates/planet/src/geology.rs:651..665` — the test block exercises
each label edge:

- `classify_shrubland_hot_semiarid` (line 651)
- `classify_steppe_cold_dry` (line 658)
- `classify_alpine_high_cold` (line 665)

(`classify_mangrove_coastal_hot_wet`, `classify_rainforest_hot_wet`,
`classify_taiga_cold_wet` extend the same edge family.)

## Acceptance

- `cargo test -p civ-planet classify_shrubland_hot_semiarid`
- `cargo test -p civ-planet classify_steppe_cold_dry`
- `cargo test -p civ-planet classify_alpine_high_cold`
all pass.
