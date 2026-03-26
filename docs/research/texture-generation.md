# Faction-Color Building Texture Generation

## Overview

This document describes the generation of faction-color texture variants for all 24 vanilla DINO building reskins in the Star Wars pack. Two faction color schemes are applied to representative Kenney base textures via HSL (Hue, Saturation, Lightness) recoloring.

## Faction Color Schemes

### Republic (white + blue accent)
- **Body Color**: `#F5F5F5` (white)
- **Accent Color**: `#1A3A6B` (Republic blue)

### CIS (dark grey + rust-orange accent)
- **Body Color**: `#444444` (dark grey)
- **Accent Color**: `#B35A00` (rust-orange)

## Kenney Base Textures

Each building type maps to a representative Kenney source texture pack:

| Building Type | Kenney Source | Texture File | Pack | Notes |
|---------------|---------------|--------------|------|-------|
| house | sci-fi-rts | scifiEnvironment_01.png | sci-fi-rts | White modular structure |
| farm | sci-fi-rts | scifiEnvironment_04.png | sci-fi-rts | Terrain/vegetation |
| granary | sci-fi-rts | scifiEnvironment_07.png | sci-fi-rts | Storage structure |
| hospital | sci-fi-rts | scifiEnvironment_10.png | sci-fi-rts | Facility building |
| forester | sci-fi-rts | scifiEnvironment_13.png | sci-fi-rts | Extraction post |
| stone | sci-fi-rts | scifiEnvironment_16.png | sci-fi-rts | Mining/industrial |
| iron | modular-space-kit | variation-a.png | modular-space-kit | Metallic variant A |
| soul | modular-space-kit | variation-b.png | modular-space-kit | Metallic variant B |
| builder | sci-fi-rts | scifiEnvironment_19.png | sci-fi-rts | Construction facility |
| guild | sci-fi-rts | scifiEnvironment_20.png | sci-fi-rts | Advanced research |
| gate | modular-space-kit | colormap.png | modular-space-kit | Barrier/gate (placeholder fallback) |

## Building Variants Generated

### Republic (12 buildings)
1. **rep_house_clone_quarters_albedo.png** - Clone living quarters
2. **rep_farm_hydroponic_albedo.png** - Hydroponics farm
3. **rep_granary_synthesizer_albedo.png** - Food synthesizer
4. **rep_hospital_medbay_albedo.png** - Medical bay
5. **rep_forester_extraction_post_albedo.png** - Resource extraction
6. **rep_stone_durasteel_refinery_albedo.png** - Durasteel refinery
7. **rep_iron_deep_core_rig_albedo.png** - Deep core mining
8. **rep_iron_tibanna_extractor_albedo.png** - Tibanna gas extraction
9. **rep_soul_crystal_excavator_albedo.png** - Crystal excavation
10. **rep_builder_engineer_corps_albedo.png** - Engineering corps
11. **rep_guild_engineering_lab_albedo.png** - Engineering laboratory
12. **rep_gate_blast_gate_albedo.png** - Blast gate (placeholder)

### CIS (12 buildings)
1. **cis_house_droid_pod_albedo.png** - Droid habitat pod
2. **cis_farm_fuel_harvester_albedo.png** - Fuel harvester
3. **cis_granary_power_depot_albedo.png** - Power depot
4. **cis_hospital_repair_station_albedo.png** - Droid repair station
5. **cis_forester_raw_extractor_albedo.png** - Raw material extraction
6. **cis_stone_scrap_works_albedo.png** - Scrap recycling works
7. **cis_iron_ore_plant_albedo.png** - Ore processing plant
8. **cis_iron_endless_extractor_albedo.png** - Endless resource extractor
9. **cis_soul_dark_energy_tap_albedo.png** - Dark energy tap
10. **cis_builder_droid_bay_albedo.png** - Droid construction bay
11. **cis_guild_techno_workshop_albedo.png** - Technical workshop
12. **cis_gate_security_barrier_albedo.png** - Security barrier

## Technical Implementation

### Texture Recoloring Algorithm

The recoloring process uses HSL (Hue, Saturation, Lightness) color space conversion:

1. **Load base texture** from Kenney pack
2. **Resize to 512x512** (standard building texture size)
3. **For each pixel**:
   - Convert RGB → HLS color space
   - Determine if pixel is "dark" (accent) or "light" (body) based on lightness threshold (0.5)
   - Apply target faction hue and saturation while preserving original luminosity
   - Convert back to RGB
4. **Save as PNG** with lossless compression

**Lightness threshold rationale**: Pixels with L &lt; 0.5 are treated as accent/shadow areas (mapped to faction accent color), while L ≥ 0.5 are treated as body/highlight areas (mapped to faction body color). This preserves material definition and surface detail while shifting the overall color scheme.

### Batch Processing

All 24 textures are generated via a single Python script:

```bash
python3 packs/warfare-starwars/generate_textures.py
```

**Script location**: `/packs/warfare-starwars/generate_textures.py`

**Output directory**: `/packs/warfare-starwars/assets/textures/buildings/`

### Dependencies

- **Python 3.6+**
- **PIL/Pillow** (Python Imaging Library)
  ```bash
  pip install Pillow
  ```

## Color Mapping Examples

### Republic (White + Blue)
```
Original dark pixels (L < 0.5)  → Republic blue (#1A3A6B)
Original light pixels (L ≥ 0.5) → Republic white (#F5F5F5)
```

### CIS (Dark Grey + Rust-Orange)
```
Original dark pixels (L < 0.5)  → CIS rust-orange (#B35A00)
Original light pixels (L ≥ 0.5) → CIS dark grey (#444444)
```

## Regenerating Textures

To regenerate all 24 textures from scratch:

```bash
# From project root
cd packs/warfare-starwars

# Run the generation script
python3 generate_textures.py

# Verify output
ls -lh assets/textures/buildings/ | wc -l  # Should show 24 textures
```

To regenerate a single building type:

```python
from generate_textures import recolor_image, FACTION_COLORS, KENNEY_SOURCES

# Example: regenerate Republic house texture
kenney_path = KENNEY_SOURCES["house"]
rep_colors = FACTION_COLORS["rep"]
img = recolor_image(
    kenney_path,
    rep_colors["body_rgb"],
    rep_colors["accent_rgb"],
    output_size=512
)
img.save("assets/textures/buildings/rep_house_clone_quarters_albedo.png")
```

## Fallback Behavior

If a Kenney source texture cannot be located, the script generates a simple procedural placeholder:
- Solid faction body color (512x512)
- Vertical accent stripe (1/4 width) on right edge

**Current fallback**: Gate textures (both rep and cis) use placeholder due to missing `colormap.png` at the expected path. These should be replaced with actual Kenney textures once the path is verified.

## Asset Pipeline Integration

These textures are loaded via the standard DINOForge asset pipeline:

```yaml
# In packs/warfare-starwars/assets/manifest.yaml
assets:
  - id: rep_house_clone_quarters_texture
    type: Texture
    file: textures/buildings/rep_house_clone_quarters_albedo.png
    license: CC0
```

The ContentLoader automatically registers these as albedo maps for the corresponding building meshes.

## Performance Notes

- **Generation time**: ~1 minute for all 24 textures (pixel-by-pixel HSL conversion)
- **File sizes**: 18-55 KB per texture (PNG with LZ compression)
- **Memory usage**: ~20 MB peak (one 512x512 image at a time)

## Future Improvements

1. **Roughness/Metallic maps**: Generate additional PBR texture channels (normal maps, metallic, roughness) for physically-based rendering
2. **Texture atlasing**: Combine faction variants into shared texture atlases to reduce draw calls
3. **Material variations**: Support multiple material finishes (matte, metallic, weathered) per faction
4. **Optimization**: Use vectorized NumPy operations instead of pixel-by-pixel loops for faster generation

## References

- **Kenney.nl texture packs**: https://kenney.nl/assets?q=space (CC0 license)
- **PIL/Pillow documentation**: https://pillow.readthedocs.io/
- **HSL color space**: https://en.wikipedia.org/wiki/HSL_and_HSV
- **DINO Asset Pipeline**: See `src/SDK/Assets/`
