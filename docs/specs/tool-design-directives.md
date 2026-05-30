# Tool Design Directives (binding — UI, Infrastructure, Material/Terraform Leads)

User directives (2026-05-29), to be honored by the UI/UX, Infrastructure, and Voxel/Material Leads and reflected in the feature-matrix + FR/NFR.

## Terraform tooling = SUPERSET of Cities Skylines + WorldBox
The terraform tool category must combine the strengths of BOTH, not pick one:
- **From Cities Skylines (CS1/CS2):** precise, ergonomic terrain editing — adjustable **brush size + strength + falloff/softness**, modes for **raise / lower / level (to a picked height) / smooth / slope (line between two points) / shift**, height readout, undo, and a clean radial/slider control surface. Surveyor-grade control.
- **From WorldBox:** playful, immediate **god-brush** feel — instant chunky brushes that **add/remove land, raise mountains, dig oceans/lakes/rivers, drop biomes**, with satisfying scale and zero friction. Sandbox spontaneity.
- **Superset result:** one Terraform category exposing BOTH the precise CS-style controls AND the fast WorldBox god-brushes (e.g. precise-mode toggle vs god-mode toggle), sharing brush params. Operates on the VOXEL substrate (moves/edits material columns), respecting gravity/fluid CA after edits.

## Material editing = closely after WorldBox + the researched games
The Material category (paint voxel materials: water, sand, dirt, stone, lava, gas, ice, ore, …) should feel **closely like WorldBox material/element placement** (instant brush paint of a material that then obeys the fluid/gravity CA — water flows, sand piles, lava burns/cools) plus the best ideas surfaced in the reference-game research (e.g. Noita/Powder-Toy element interactions, CS overlays). Brush shares the terraform brush-param surface (size/strength/falloff). Painted materials immediately participate in the CA so the world reacts.

## Material taxonomy must be RICH — between WorldBox and The Powder Toy
The current ~8-12 material set is too thin. Expand toward a **Powder-Toy-class element variety** (dozens), while keeping WorldBox's instant-brush feel and our Phase classification (Liquid/Powder/Gas/Solid/Empty) + the laws/energy constraints. Each element has properties (density, flow, angle-of-repose, temperature, flammability, conductivity, reactions) driving CA + interactions. Target families (illustrative, not exhaustive — let the Research Lead + laws DB finalize):
- **Liquids:** water, salt water, lava/magma, oil, acid, blood, mud, molten metal, liquid nitrogen/coolant.
- **Powders:** sand, dirt/soil, gravel, clay, ash, snow, gunpowder, salt, seeds, dust.
- **Solids:** stone, granite, dirt-packed, ice, wood, metal/iron ore, coal, glass, crystal, brick, bone.
- **Gases:** air, steam, smoke, methane, oxygen, toxic gas, CO2.
- **Energetic/reactive:** fire, ember, plasma, electricity/spark, radioactive.
- **Bio/special:** plant/moss, mold, organic sludge (abiogenesis-relevant).
Elements INTERACT per the laws/energy DB: lava + water → stone + steam; fire + oil → combustion; acid dissolves; water + cold → ice; gunpowder ignites; electricity conducts through metal/water. This is the Powder-Toy/Noita interaction depth on the voxel-fluid CA. Material brush = WorldBox immediacy + Powder-Toy element/reaction richness. (Substrate work: extend crates/voxel material.rs taxonomy + add a reactions table; Voxel/Material Lead owns it, validated against crates/laws.)

## Shared brush model
Terraform + Material + (where sensible) other paint-style tools share ONE brush controller: size, strength, falloff, shape, continuous-paint vs single-stamp, world-bounds clamp, egui pointer-capture guard. Implement once, reuse across categories.

These are go-to references alongside the Research Lead's broader teardown; material behavior tracks WorldBox most closely. See ../guides/emergence-charter.md.
