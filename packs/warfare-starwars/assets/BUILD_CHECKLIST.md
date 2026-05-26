# Star Wars Building Assets — Build Checklist

**Purpose**: Track the assembly of 24 vanilla DINO building reskins (Republic + CIS factions) from Kenney source models.

**Art Style**: Low-poly TABS aesthetic (300-700 tris per building)

**Status**: Asset pipeline scaffolding complete; texture generation and FBX assembly starting.

---

## Summary

| Metric | Value |
|--------|-------|
| Total buildings | 24 (12 Republic + 12 CIS) |
| Kenney source kits | 2 (space-kit, modular-space-kit) |
| Status: todo | 22 |
| Status: in-progress | 0 |
| Status: done | 0 |
| Estimated remaining hours | 60-72 |

---

## Key Folders

| Path | Purpose |
|------|---------|
| `assets/source/kenney/` | Kenney source assets (PNG, FBX, obj, textures) |
| `assets/textures/buildings/` | **Output**: faction-color texture variants |
| `assets/meshes/buildings/` | **Output**: compiled FBX models (game-ready) |
| `assets/crosswalk_*.yaml` | Asset swap manifest (maps vanilla → custom) |

---

## Build Process (General Pipeline)

Each building follows this pipeline:

1. **Identify source Kenney model(s)** from `space-kit` or `modular-space-kit`
2. **Create texture variant**:
   - Take Kenney source texture (usually grayscale or generic)
   - Color-shift to faction palette (Republic: white+blue; CIS: grey+orange)
   - Export as PNG to `assets/textures/buildings/`
3. **Assemble in Blender**:
   - Import source FBX from Kenney
   - Apply faction texture variant
   - Add faction-specific details (decals, glows, trim)
   - Optimize poly count (target: 300-600 tris)
   - Ensure TABS-style proportions
4. **Export as FBX** to `assets/meshes/buildings/`
5. **Validate**:
   - Check tri count < 700
   - Verify texture assignment
   - Check import in game (no missing materials, correct scale)
   - Mark as done in checklist

---

## Texture Palette Reference

### Republic (Galactic Republic)

| Element | Hex | Description |
|---------|-----|-------------|
| Primary armor | `#F5F5F5` | Near-white durasteel |
| Accent stripe | `#1A3A6B` | Deep navy blue |
| Trim/detail | `#DDDDDD` to `#EEEEEE` | Light off-white |
| Medical red | `#CC2222` | Insignia only |
| Gold accent | `#FFD700` | Heavy/elite details |

**Palette approach**: Start with Kenney texture, multiply by Republic blue, overlay white highlights.

### CIS (Confederacy of Independent Systems)

| Element | Hex | Description |
|---------|-----|-------------|
| Primary hull | `#444444` to `#555555` | Medium-dark grey |
| Accent orange | `#B35A00` | Rust/hazard orange |
| Recesses | `#2A2A2A` | Shadow darks |
| Energy glow | `#FF6600` | Orange heat/warning |
| Red alert | `#CC2222` | Sentry/danger |

**Palette approach**: Start with Kenney texture, desaturate to grey, overlay orange stripes and rust tones.

---

## Building Checklist

### RESIDENTIAL / UTILITY STRUCTURES

#### 1. House → Clone Quarters Pod (Rep) / Droid Storage Pod (CIS)

| Aspect | Rep | CIS |
|--------|-----|-----|
| Vanilla building | house | house |
| Custom ID | rep_house_clone_quarters | cis_house_droid_pod |
| Display name | Clone Quarters Pod | Droid Storage Pod |
| Source Kenney | space-kit/structure.fbx | space-kit/structure.fbx |
| Texture base | Generic grey | Generic grey |
| Effort (hours) | 2 | 2 |
| **Status** | **todo** | **todo** |
| Tri budget | 300-400 | 300-400 |
| Notes | Circular habitat unit; white+blue | Stacked droid rack; grey+orange |

**Assembly notes**:
- Use `space-kit/structure.fbx` as base
- Rep: apply white primary, blue accent stripe on side, add clone emblem decal
- CIS: apply dark grey, orange rust stripes, add droid silhouette details
- Keep low dome roof visible

#### 2. Farm → Hydroponic Farm Array (Rep) / Fuel Cell Harvester (CIS)

| Aspect | Rep | CIS |
|--------|-----|-----|
| Vanilla building | farm | farm |
| Custom ID | rep_farm_hydroponic | cis_farm_fuel_harvester |
| Display name | Hydroponic Farm Array | Fuel Cell Harvester |
| Source Kenney | space-kit/platform_large.fbx | space-kit/structure_detailed.fbx |
| Texture base | Generic | Generic |
| Effort (hours) | 3 | 3 |
| **Status** | **todo** | **todo** |
| Tri budget | 400-500 | 400-500 |
| Notes | Tiered bays with glow; transparent walls | Ground bore with red liquid glow |

**Assembly notes**:
- Rep: Use platform + transparent panel overlays; blue-green emissive glow for grow-lights
- CIS: Use bore/extractor base; add red conduit glow, rotating bore head animation prep

---

#### 3. Granary → Nutrient Synthesizer (Rep) / Power Cell Depot (CIS)

| Aspect | Rep | CIS |
|--------|-----|-----|
| Vanilla building | granary | granary |
| Custom ID | rep_granary_synthesizer | cis_granary_power_depot |
| Display name | Nutrient Synthesizer | Power Cell Depot |
| Source Kenney | space-kit/structure.fbx | space-kit/structure_closed.fbx |
| Texture base | Generic | Generic |
| Effort (hours) | 2 | 2 |
| **Status** | **todo** | **todo** |
| Tri budget | 300-400 | 300-400 |
| Notes | Tall cylindrical silo | Stacked battery cells |

**Assembly notes**:
- Rep: Tall cylinder, white finish, side hatch, blue panel accent, data display decal
- CIS: Stacked rectangular cells, dark grey, hazard stripe textures, red LED indicators

---

#### 4. Hospital → Clone Medical Bay (Rep) / Droid Repair Station (CIS)

| Aspect | Rep | CIS |
|--------|-----|-----|
| Vanilla building | hospital | hospital |
| Custom ID | rep_hospital_medbay | cis_hospital_repair_station |
| Display name | Clone Medical Bay | Droid Repair Station |
| Source Kenney | space-kit/structure.fbx | space-kit/structure_detailed.fbx |
| Texture base | Generic | Generic |
| Effort (hours) | 2.5 | 2.5 |
| **Status** | **todo** | **todo** |
| Tri budget | 300-400 | 350-450 |
| Notes | Prefab white walls, clean interior | Open-frame repair cradle, arms |

**Assembly notes**:
- Rep: Compact prefab; white walls, interior suggestion, NO red cross (per lore), blue trim
- CIS: Open frame, dangling articulated maintenance arm, tool racks, droid parts visible

---

### RESOURCE / EXTRACTION STRUCTURES

#### 5. Forester House → Resource Extraction Post (Rep) / Raw Material Extractor (CIS)

| Aspect | Rep | CIS |
|--------|-----|-----|
| Vanilla building | forester_house | forester_house |
| Custom ID | rep_forester_extraction_post | cis_forester_raw_extractor |
| Display name | Resource Extraction Post | Raw Material Extractor |
| Source Kenney | space-kit/structure.fbx | space-kit/structure.fbx + custom spider legs |
| Texture base | Generic | Generic |
| Effort (hours) | 2 | 3 |
| **Status** | **todo** | **todo** |
| Tri budget | 250-300 | 300-350 |
| Notes | Small outpost, antenna dish | Spider-leg anchors, rotating claw |

**Assembly notes**:
- Rep: Small footprint, sensor array top, resource arm, antenna, Republic branding
- CIS: Add custom spider-leg attachment kit (4 anchor legs); rotating claw head; minimal branding

---

#### 6. Stone Cutter → Durasteel Refinery (Rep) / Scrap Metal Works (CIS)

| Aspect | Rep | CIS |
|--------|-----|-----|
| Vanilla building | stone_cutter | stone_cutter |
| Custom ID | rep_stone_durasteel_refinery | cis_stone_scrap_works |
| Display name | Durasteel Refinery | Scrap Metal Works |
| Source Kenney | space-kit/structure_detailed.fbx | space-kit/structure.fbx |
| Texture base | Generic | Generic |
| Effort (hours) | 3 | 3 |
| **Status** | **todo** | **todo** |
| Tri budget | 400-500 | 400-500 |
| Notes | Industrial smelter, chimneys, glow | Crusher with gears, shrapnel effect |

**Assembly notes**:
- Rep: Multiple chimneys, molten-glow windows (blue emission), conveyor input, white/grey industrial finish
- CIS: Exposed gears, jagged mesh panels, debris pile effect, orange/grey palette, VFX spark emitter point prep

---

#### 7. Iron Mine → Tibanna Gas Extractor (Rep) / Ore Processing Plant (CIS)

| Aspect | Rep | CIS |
|--------|-----|-----|
| Vanilla building | iron_mine | iron_mine |
| Custom ID | rep_iron_tibanna_extractor | cis_iron_ore_plant |
| Display name | Tibanna Gas Extractor | Ore Processing Plant |
| Source Kenney | space-kit/tower-A equivalent | space-kit/tower-A equivalent |
| Texture base | Generic | Generic |
| Effort (hours) | 3 | 3 |
| **Status** | **todo** | **todo** |
| Tri budget | 400-500 | 400-500 |
| Notes | Derrick-style tall tower | Compact tower with conveyor |

**Assembly notes**:
- Rep: Tall derrick frame, pressurized tanks, gas conduit tubing, Republic insignia, white/grey finish
- CIS: Compact ore conveyor belt, grinder housing, rust-orange paint, Techno Union branding (orange/grey)

---

#### 8. Infinite Iron Mine → Deep-Core Tibanna Rig (Rep) / Endless Ore Extractor (CIS)

| Aspect | Rep | CIS |
|--------|-----|-----|
| Vanilla building | infinite_iron_mine | infinite_iron_mine |
| Custom ID | rep_iron_deep_core_rig | cis_iron_endless_extractor |
| Display name | Deep-Core Tibanna Rig | Endless Ore Extractor |
| Source Kenney | space-kit/tower-A x2 (mirrored) | space-kit/tower-A x2 (mirrored) |
| Texture base | Generic | Generic |
| Effort (hours) | 1.5 | 1.5 |
| **Status** | **todo** | **todo** |
| Tri budget | 500-600 | 500-600 |
| Notes | Twin-derrick version | Dual-bore heavy extraction |

**Assembly notes**:
- **BOTH**: Duplicate + mirror tower-A variant, add flare stack piece between derricks
- Rep: Twin derricks, larger footprint, flare exhaust stack, white/grey, Republic emblem
- CIS: Dual-bore, Techno Union insignia, rust tones, orange accent stripes

---

#### 9. Soul Mine → Force Crystal Excavator (Rep) / Dark Side Energy Tap (CIS)

| Aspect | Rep | CIS |
|--------|-----|-----|
| Vanilla building | soul_mine | soul_mine |
| Custom ID | rep_soul_crystal_excavator | cis_soul_dark_energy_tap |
| Display name | Force Crystal Excavator | Dark Side Energy Tap |
| Source Kenney | space-kit/tower-B | space-kit/tower-B |
| Texture base | Generic | Generic |
| Effort (hours) | 4 | 4 |
| **Status** | **todo** | **todo** |
| Tri budget | 500-600 | 500-600 |
| Notes | Crystal mining cage, purple/blue glow | Dark lattice, red/purple glow |
| **Complexity** | **HIGH** | **HIGH** |

**Assembly notes**:
- Rep: Angular cage frame, Kyber crystal emission glow (purple/blue), custom Blender emission shader
- CIS: Black lattice frame, red/purple resonance glow, alien rune engravings, custom dark emission shader
- **Both**: Requires material customization; plan shader work early

---

### MILITARY / PRODUCTION STRUCTURES

#### 10. Builder House → Republic Engineer Corps (Rep) / Construction Droid Bay (CIS)

| Aspect | Rep | CIS |
|--------|-----|-----|
| Vanilla building | builder_house | builder_house |
| Custom ID | rep_builder_engineer_corps | cis_builder_droid_bay |
| Display name | Republic Engineer Corps | Construction Droid Bay |
| Source Kenney | space-kit/structure.fbx | space-kit/structure.fbx |
| Texture base | Generic | Generic |
| Effort (hours) | 2 | 2 |
| **Status** | **todo** | **todo** |
| Tri budget | 300-400 | 300-400 |
| Notes | Mobile depot, crane arm | Droid-operated assembly depot |

**Assembly notes**:
- Rep: Mobile engineering depot, crane arm, tool racks, supply crates, Republic emblem, white/blue
- CIS: Lifting arm, storage racks with B1 silhouettes, CIS grey/orange finish

---

#### 11. Engineer Guild → Advanced Engineering Lab (Rep) / Techno Union Workshop (CIS)

| Aspect | Rep | CIS |
|--------|-----|-----|
| Vanilla building | engineer_guild | engineer_guild |
| Custom ID | rep_guild_engineering_lab | cis_guild_techno_workshop |
| Display name | Advanced Engineering Lab | Techno Union Workshop |
| Source Kenney | space-kit/structure_detailed.fbx | Custom asymmetric layout |
| Texture base | Generic | Generic |
| Effort (hours) | 3.5 | 4.5 |
| **Status** | **todo** | **todo** |
| Tri budget | 500-600 | 600-700 |
| Notes | Symmetrical wings, holodisplay dome | Asymmetric organic-tech hybrid |
| **Complexity** | **MEDIUM** | **HIGH** |

**Assembly notes**:
- Rep: Large footprint, symmetrical wings, holographic display dome (blue emission), white/blue finish
- CIS: Asymmetric wing layout (manual Blender rearrangement), organic-tech hybrid panel language, Techno Union logo, alien architecture sensibility (grey/orange)

---

#### 12. Gate → Republic Security Gate (Rep) / CIS Security Barrier (CIS)

| Aspect | Rep | CIS |
|--------|-----|-----|
| Vanilla building | gate | gate |
| Custom ID | rep_gate_security_gate | cis_gate_security_barrier |
| Display name | Republic Security Gate | CIS Security Barrier |
| Source Kenney | space-kit/gate or modular-space-kit/gate | space-kit/gate or modular-space-kit/gate |
| Texture base | Generic | Generic |
| Effort (hours) | 2 | 2 |
| **Status** | **todo** | **todo** |
| Tri budget | 250-350 | 250-350 |
| Notes | Clean gate with panels | Armored barrier with sentry alcoves |

**Assembly notes**:
- Rep: Sliding gate frame, Republic insignia panels, white/blue finish, clean lines
- CIS: Armored sliding gate, sentry droid alcoves on flanking pillars, dark grey barrier, red energy field indicator stripe

---

## Artifact Targets

### Republic (12 buildings)

| # | Building | FBX Output | Texture Output | Status |
|---|----------|-----------|----------------|--------|
| 1 | rep_house_clone_quarters | `meshes/buildings/rep_house_clone_quarters.fbx` | `textures/buildings/rep_house_clone_quarters_albedo.png` | todo |
| 2 | rep_farm_hydroponic | `meshes/buildings/rep_farm_hydroponic.fbx` | `textures/buildings/rep_farm_hydroponic_albedo.png` | todo |
| 3 | rep_granary_synthesizer | `meshes/buildings/rep_granary_synthesizer.fbx` | `textures/buildings/rep_granary_synthesizer_albedo.png` | todo |
| 4 | rep_hospital_medbay | `meshes/buildings/rep_hospital_medbay.fbx` | `textures/buildings/rep_hospital_medbay_albedo.png` | todo |
| 5 | rep_forester_extraction_post | `meshes/buildings/rep_forester_extraction_post.fbx` | `textures/buildings/rep_forester_extraction_post_albedo.png` | todo |
| 6 | rep_stone_durasteel_refinery | `meshes/buildings/rep_stone_durasteel_refinery.fbx` | `textures/buildings/rep_stone_durasteel_refinery_albedo.png` | todo |
| 7 | rep_iron_tibanna_extractor | `meshes/buildings/rep_iron_tibanna_extractor.fbx` | `textures/buildings/rep_iron_tibanna_extractor_albedo.png` | todo |
| 8 | rep_iron_deep_core_rig | `meshes/buildings/rep_iron_deep_core_rig.fbx` | `textures/buildings/rep_iron_deep_core_rig_albedo.png` | todo |
| 9 | rep_soul_crystal_excavator | `meshes/buildings/rep_soul_crystal_excavator.fbx` | `textures/buildings/rep_soul_crystal_excavator_albedo.png` | todo |
| 10 | rep_builder_engineer_corps | `meshes/buildings/rep_builder_engineer_corps.fbx` | `textures/buildings/rep_builder_engineer_corps_albedo.png` | todo |
| 11 | rep_guild_engineering_lab | `meshes/buildings/rep_guild_engineering_lab.fbx` | `textures/buildings/rep_guild_engineering_lab_albedo.png` | todo |
| 12 | rep_gate_security_gate | `meshes/buildings/rep_gate_security_gate.fbx` | `textures/buildings/rep_gate_security_gate_albedo.png` | todo |

### CIS (12 buildings)

| # | Building | FBX Output | Texture Output | Status |
|---|----------|-----------|----------------|--------|
| 1 | cis_house_droid_pod | `meshes/buildings/cis_house_droid_pod.fbx` | `textures/buildings/cis_house_droid_pod_albedo.png` | todo |
| 2 | cis_farm_fuel_harvester | `meshes/buildings/cis_farm_fuel_harvester.fbx` | `textures/buildings/cis_farm_fuel_harvester_albedo.png` | todo |
| 3 | cis_granary_power_depot | `meshes/buildings/cis_granary_power_depot.fbx` | `textures/buildings/cis_granary_power_depot_albedo.png` | todo |
| 4 | cis_hospital_repair_station | `meshes/buildings/cis_hospital_repair_station.fbx` | `textures/buildings/cis_hospital_repair_station_albedo.png` | todo |
| 5 | cis_forester_raw_extractor | `meshes/buildings/cis_forester_raw_extractor.fbx` | `textures/buildings/cis_forester_raw_extractor_albedo.png` | todo |
| 6 | cis_stone_scrap_works | `meshes/buildings/cis_stone_scrap_works.fbx` | `textures/buildings/cis_stone_scrap_works_albedo.png` | todo |
| 7 | cis_iron_ore_plant | `meshes/buildings/cis_iron_ore_plant.fbx` | `textures/buildings/cis_iron_ore_plant_albedo.png` | todo |
| 8 | cis_iron_endless_extractor | `meshes/buildings/cis_iron_endless_extractor.fbx` | `textures/buildings/cis_iron_endless_extractor_albedo.png` | todo |
| 9 | cis_soul_dark_energy_tap | `meshes/buildings/cis_soul_dark_energy_tap.fbx` | `textures/buildings/cis_soul_dark_energy_tap_albedo.png` | todo |
| 10 | cis_builder_droid_bay | `meshes/buildings/cis_builder_droid_bay.fbx` | `textures/buildings/cis_builder_droid_bay_albedo.png` | todo |
| 11 | cis_guild_techno_workshop | `meshes/buildings/cis_guild_techno_workshop.fbx` | `textures/buildings/cis_guild_techno_workshop_albedo.png` | todo |
| 12 | cis_gate_security_barrier | `meshes/buildings/cis_gate_security_barrier.fbx` | `textures/buildings/cis_gate_security_barrier_albedo.png` | todo |

---

## Batch Assembly Workflow (Python/Blender)

### Prerequisites

- Blender 3.6+ installed
- Python 3.9+ with Pillow (for texture color-shifting)
- Kenney source files available in `assets/source/kenney/`

### Phase 1: Texture Generation (Python)

A Python script that generates faction-color texture variants:

```python
#!/usr/bin/env python3
"""
generate_faction_textures.py
Batch-generate Republic and CIS faction-color texture variants from Kenney source.
"""

import os
from PIL import Image, ImageOps, ImageEnhance
import json

# Palette definitions
REPUBLIC_PALETTE = {
    "primary": (245, 245, 245),      # #F5F5F5 white
    "accent": (26, 58, 107),         # #1A3A6B navy blue
    "light": (238, 238, 238),        # #EEEEEE off-white
    "gold": (255, 215, 0),           # #FFD700 gold
}

CIS_PALETTE = {
    "primary": (68, 68, 68),         # #444444 grey
    "accent": (179, 90, 0),          # #B35A00 rust orange
    "dark": (42, 42, 42),            # #2A2A2A shadow
    "energy": (255, 102, 0),         # #FF6600 heat orange
}

def colorize_grayscale(image_path, faction_palette, output_path):
    """Convert grayscale Kenney texture to faction colors."""
    img = Image.open(image_path).convert('RGB')

    # For now: simple multiply blend of grayscale by faction colors
    # Advanced: use Hue-Saturation shift for better results

    # Grayscale multiply approach
    primary = faction_palette["primary"]
    accent = faction_palette.get("accent", (200, 200, 200))

    # Split into normalized layers
    img_array = ImageOps.grayscale(img)

    # Create output: blend primary and accent based on value
    output = Image.new('RGB', img.size)
    pixels = output.load()

    for y in range(img.size[1]):
        for x in range(img.size[0]):
            val = img_array.getpixel((x, y)) / 255.0
            # Darker areas get accent color, lighter get primary
            if val > 0.5:
                r = int(primary[0] * val)
                g = int(primary[1] * val)
                b = int(primary[2] * val)
            else:
                r = int(accent[0] * val)
                g = int(accent[1] * val)
                b = int(accent[2] * val)
            pixels[x, y] = (r, g, b)

    output.save(output_path)
    print(f"Generated: {output_path}")

def batch_generate_textures(source_dir, output_dir):
    """Generate all building textures."""
    buildings = [
        # (vanilla_name, rep_id, cis_id, kenney_source)
        ("house", "rep_house_clone_quarters", "cis_house_droid_pod", ""),
        ("farm", "rep_farm_hydroponic", "cis_farm_fuel_harvester", ""),
        # ... add all 12 buildings
    ]

    for vanilla, rep_id, cis_id, source in buildings:
        # TODO: Generate textures
        pass

if __name__ == "__main__":
    SOURCE_DIR = "assets/source/kenney/space-kit"
    OUTPUT_DIR = "assets/textures/buildings"
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    # Placeholder
    print("Texture generation framework ready; source Kenney files needed")
```

### Phase 2: Blender Batch Assembly (Blender Python API)

```python
#!/usr/bin/env python3
"""
blender_assemble_buildings.py
Blender batch assembly script. Run with: blender --python blender_assemble_buildings.py
"""

import bpy
from pathlib import Path

class BuildingAssembler:
    def __init__(self, source_kenney_dir, output_meshes_dir, output_textures_dir):
        self.source_kenney = Path(source_kenney_dir)
        self.output_meshes = Path(output_meshes_dir)
        self.output_textures = Path(output_textures_dir)

    def assemble_building(self, building_id, source_fbx, faction, palette):
        """
        Assemble a single building:
        1. Import source FBX
        2. Apply faction texture
        3. Add faction-specific details
        4. Optimize poly count
        5. Export as FBX
        """
        # Clear scene
        bpy.ops.object.select_all(action='SELECT')
        bpy.ops.object.delete()

        # Import source FBX
        fbx_path = self.source_kenney / "Models" / "FBX format" / source_fbx
        bpy.ops.import_scene.fbx(filepath=str(fbx_path))

        # Get imported object
        obj = bpy.context.selected_objects[0]

        # Apply faction texture
        texture_name = f"{building_id}_albedo.png"
        texture_path = self.output_textures / texture_name

        # TODO: Load texture and apply to material

        # Export
        output_fbx = self.output_meshes / f"{building_id}.fbx"
        bpy.ops.export_scene.fbx(filepath=str(output_fbx), use_selection=True)

        print(f"Assembled: {output_fbx}")

# Assembly manifest
BUILDINGS = [
    # (building_id, source_fbx, faction, palette_key)
    ("rep_house_clone_quarters", "space-kit/structure.fbx", "rep", "REPUBLIC"),
    ("cis_house_droid_pod", "space-kit/structure.fbx", "cis", "CIS"),
    # ... add all 24 buildings
]

if __name__ == "__main__":
    assembler = BuildingAssembler(
        source_kenney_dir="assets/source/kenney",
        output_meshes_dir="assets/meshes/buildings",
        output_textures_dir="assets/textures/buildings"
    )

    for building_id, source_fbx, faction, palette in BUILDINGS:
        assembler.assemble_building(building_id, source_fbx, faction, palette)
```

---

## Next Steps (Priority Order)

### Immediate (Week 1)

1. **[IN PROGRESS]** Create texture color-shift Python script
2. **[TODO]** Generate 24 faction-color texture variants (PNG)
3. **[TODO]** Import 2 pilot buildings into Blender:
   - `rep_house_clone_quarters` (small, simple)
   - `cis_house_droid_pod` (small, simple)
4. **[TODO]** Manually assemble, texture, and export as FBX
5. **[TODO]** Validate in game (scale, materials, tri count)

### Short-term (Week 2-3)

6. Refine Blender assembly script based on pilot learnings
7. Batch-assemble remaining 22 buildings
8. Optimize poly counts (current target: 300-600 tris)
9. Add faction-specific detail passes:
   - Republic: emblems, panel lines, glows
   - CIS: rust/hazard overlays, mechanical details
10. Validate all 24 in game

### Medium-term (Week 4+)

11. Create custom shader variants for high-detail buildings (soul_mine, guild)
12. Add animated details (rotating parts, glowing conduits, VFX emitter points)
13. Package and deploy to pack loader

---

## Key Constraints

- **Do NOT modify Kenney source files** — only read from `source/kenney/`
- All work must be **repeatable** — document every step
- Output FBX must be **game-ready** (correct scale, materials, tri count)
- Textures must be **faction-consistent** (use provided hex palettes)

---

## References

- **Kenney assets**: `assets/source/kenney/` (CC0 license)
- **Faction manifests**: `crosswalk_republic_vanilla.yaml`, `crosswalk_cis_vanilla.yaml`
- **Art style**: `ASSET_PIPELINE.md` (low-poly TABS aesthetic)
- **Color palettes**: See "Texture Palette Reference" section above

---

**Last updated**: 2026-03-12
**Pipeline status**: Scaffolding complete; texture generation starting
