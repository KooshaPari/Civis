# Civis Bevy PBR Texture Assets

All textures must be **CC0 / public domain**. See `docs/guides/asset-sources.md` for
the canonical source URLs and download instructions. Textures are **not committed**
to git (see `.gitignore`); pull them with the asset-fetch script (TBD) or download
manually per the source guide.

## Expected layout per biome

Each biome directory should contain (Phase 2 set):

```
<biome>/
  albedo.ktx2          # base_color_texture (sRGB)
  normal.ktx2          # normal_map (linear, GL/+Y up)
  orm.ktx2             # packed Occlusion(R) Roughness(G) Metallic(B)
  height.ktx2          # optional, for parallax / splat blend (Phase 3+)
  LICENSE.txt          # CC0 declaration + author + source URL
```

Phase 1 minimum: `albedo.ktx2` + `normal.ktx2` only.

## Biomes

- `grass_field/`   — temperate grass plains
- `sand_beach/`    — coastal sand / dunes
- `rock_cliff/`    — exposed cliff rock (tri-planar candidate)
- `snow_pure/`     — clean alpine snow
- `forest_floor/`  — leaf litter / moss
- `dirt_ground/`   — bare earth / packed dirt

## Format notes

- KTX2 + BasisU (UASTC for normal, ETC1S for albedo/ORM) keeps GPU memory low.
- Bevy 0.18 supports KTX2 natively via `bevy_image/ktx2`.
- 1024x1024 is the default; 2048 only for hero close-ups.
