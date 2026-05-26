# Batch Texture Processing Commands

## Quick Reference

### Generate all 24 faction-color building textures
```bash
cd packs/warfare-starwars
python3 generate_textures.py
```

### Verify generation
```bash
# Check count
ls assets/textures/buildings/*_albedo.png | wc -l  # Should output: 24

# Check file sizes
ls -lh assets/textures/buildings/ | tail -24
```

### ImageMagick Alternative (if needed)

For recoloring with ImageMagick instead of Python:

```bash
# Example: Recolor single texture to Republic colors (white + blue)
convert sci-fi-rts/scifiEnvironment_01.png \
  -modulate 100,50 \
  -colorspace HSL \
  -normalize \
  -colorspace RGB \
  assets/textures/buildings/rep_house_clone_quarters_albedo.png

# Example: Batch recolor all sci-fi-rts textures to CIS colors
for src in sci-fi-rts/PNG/Default\ size/Environment/*.png; do
  convert "$src" \
    -modulate 85,60 \
    -colorspace HSL \
    -recolor "0.8 0 0  0 0.8 0  0 0 0.8" \
    assets/textures/buildings/cis_$(basename "$src")
done
```

## Python Script Usage

### Basic usage (generate all)
```bash
python3 generate_textures.py
```

### Programmatic usage (in Python)
```python
from generate_textures import recolor_image, FACTION_COLORS, KENNEY_SOURCES, BUILDINGS

# Get faction colors
rep_colors = FACTION_COLORS["rep"]
cis_colors = FACTION_COLORS["cis"]

# Recolor single texture
kenney_path = KENNEY_SOURCES["house"]
img = recolor_image(
    kenney_path,
    rep_colors["body_rgb"],
    rep_colors["accent_rgb"],
    output_size=512
)
img.save("rep_house.png")

# Batch recolor all houses
for building_type in ["house", "farm", "stone"]:
    kenney_path = KENNEY_SOURCES[building_type]
    for faction_code, faction_data in FACTION_COLORS.items():
        img = recolor_image(
            kenney_path,
            faction_data["body_rgb"],
            faction_data["accent_rgb"],
            output_size=512
        )
        img.save(f"{faction_code}_{building_type}.png")
```

## File Manifests

### Generated Republic Textures (12)
```
rep_house_clone_quarters_albedo.png        (52 KB)
rep_farm_hydroponic_albedo.png             (35 KB)
rep_granary_synthesizer_albedo.png         (51 KB)
rep_hospital_medbay_albedo.png             (42 KB)
rep_forester_extraction_post_albedo.png    (22 KB)
rep_stone_durasteel_refinery_albedo.png    (24 KB)
rep_iron_deep_core_rig_albedo.png          (18 KB)
rep_iron_tibanna_extractor_albedo.png      (18 KB)
rep_soul_crystal_excavator_albedo.png      (18 KB)
rep_builder_engineer_corps_albedo.png      (40 KB)
rep_guild_engineering_lab_albedo.png       (46 KB)
rep_gate_blast_gate_albedo.png             (1.9 KB - placeholder)
```

### Generated CIS Textures (12)
```
cis_house_droid_pod_albedo.png             (54 KB)
cis_farm_fuel_harvester_albedo.png         (35 KB)
cis_granary_power_depot_albedo.png         (55 KB)
cis_hospital_repair_station_albedo.png     (43 KB)
cis_forester_raw_extractor_albedo.png      (22 KB)
cis_stone_scrap_works_albedo.png           (24 KB)
cis_iron_ore_plant_albedo.png              (20 KB)
cis_iron_endless_extractor_albedo.png      (20 KB)
cis_soul_dark_energy_tap_albedo.png        (18 KB)
cis_builder_droid_bay_albedo.png           (41 KB)
cis_guild_techno_workshop_albedo.png       (47 KB)
cis_gate_security_barrier_albedo.png       (1.9 KB - placeholder)
```

## Troubleshooting

### "Kenney source not found" warning
If the script generates placeholders instead of recolored textures:

1. Verify Kenney pack is downloaded:
   ```bash
   ls packs/warfare-starwars/assets/source/kenney/sci-fi-rts/PNG/
   ```

2. Check texture paths match in `generate_textures.py`:
   ```python
   KENNEY_SOURCES = {
       "house": r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\sci-fi-rts\PNG\Default size\Environment\scifiEnvironment_01.png",
   }
   ```

3. Test single texture load:
   ```python
   from PIL import Image
   img = Image.open(r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\sci-fi-rts\PNG\Default size\Environment\scifiEnvironment_01.png")
   print(img.size)  # Should output (256, 256) or similar
   ```

### Slow generation
- Pixel-by-pixel HSL conversion is intentionally precise but slow (1 min for 24 textures)
- For faster batch processing, consider NumPy-based vectorization (see future improvements)

### Memory issues
- Each texture is loaded one at a time; max memory ~20 MB
- If generating 1000+ textures, consider streaming to disk between conversions

## Integration with Asset Pipeline

Generated textures are automatically registered in the manifest:

```yaml
# packs/warfare-starwars/assets/manifest.yaml
- id: rep_house_clone_quarters_texture
  type: Texture
  file: textures/buildings/rep_house_clone_quarters_albedo.png
```

No manual registration needed; the ContentLoader picks them up automatically.

## Performance Metrics

| Metric | Value |
|--------|-------|
| Generation time (all 24) | ~60 seconds |
| Time per texture | ~2.5 seconds |
| Output file size (avg) | 30 KB |
| Output resolution | 512x512 |
| Color precision | 8-bit RGB |

## References

- **Source script**: `packs/warfare-starwars/generate_textures.py`
- **Output directory**: `packs/warfare-starwars/assets/textures/buildings/`
- **Kenney assets**: `packs/warfare-starwars/assets/source/kenney/`
- **Documentation**: `packs/warfare-starwars/TEXTURE_GENERATION.md`
