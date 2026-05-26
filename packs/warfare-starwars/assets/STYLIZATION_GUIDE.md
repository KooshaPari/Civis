# Star Wars Clone Wars → DINO Stylization Guide

Converting fan-created Star Wars Clone Wars models to DINO's low-poly, readable battlefield aesthetic.

---

## 1. Visual Style Philosophy

The target aesthetic blends **TABS (Totally Accurate Battle Simulator)** sensibilities with **Clone Wars Star Wars** IP:

- **Silhouette > Detail**: From 100 units away on a battlefield, unit shape must be instantly recognizable as Republic Clone, CIS Droid, or Stormtrooper without color
- **Low-poly with personality**: 800–3,000 polys per unit (vs 4,000–60,000 on Sketchfab)
- **Flat color + minimal texture**: Single diffuse color per material, no detailed bump maps or specular layers
- **Exaggerated proportions**: Slightly larger helmets, broader shoulders, more prominent weapons—readability over realism
- **Faction visual language**:
  - **Republic**: Clean, rounded silhouettes, white base + bright blue accents, technologically polished
  - **CIS**: Angular, utilitarian, tan/brown palette, mechanical/industrial feel

### Design Intent
Units should feel **toy-like and readable** rather than film-accurate. Distant players should instantly identify unit type and faction by outline alone.

---

## 2. Color Palettes

### Republic Faction
| Role | Primary | Secondary | Accent | Detail |
|------|---------|-----------|--------|--------|
| **Clone Trooper** | `#FFFFFF` (white) | `#1a4d7a` (republic blue) | `#d4af37` (gold) | `#4a7ba7` (light blue) |
| **Officer** | `#FFFFFF` (white) | `#2d5fa3` (darker blue) | `#ffd700` (bright gold) | `#1a4d7a` (deep blue) |
| **Heavy Weapons** | `#FFFFFF` (white) | `#1a4d7a` (republic blue) | `#c0c0c0` (silver) | `#666666` (charcoal) |

### CIS Faction
| Role | Primary | Secondary | Accent | Detail |
|------|---------|-----------|--------|--------|
| **B1 Battle Droid** | `#c9a84c` (tan/gold) | `#8b6914` (brown) | `#555555` (gray) | `#333333` (dark gray) |
| **B2 Super Droid** | `#9d8b5c` (muted tan) | `#6b5d3f` (dark brown) | `#444444` (charcoal) | `#1a1a1a` (near-black) |
| **Tactical Droid** | `#b8956d` (lighter tan) | `#7a6a4f` (weathered brown) | `#ff6b35` (orange accent) | `#2a2a2a` (dark) |

### Imperial / Stormtrooper (Optional expansion)
| Role | Primary | Secondary | Accent | Detail |
|------|---------|-----------|--------|--------|
| **Stormtrooper** | `#f5f5f5` (off-white) | `#222222` (black trim) | `#555555` (gray panel) | `#111111` (pure black) |
| **Officer** | `#f5f5f5` (off-white) | `#1a1a1a` (black) | `#d4a017` (brass/gold) | `#666666` (charcoal) |

---

## 3. Stylization Transformation Steps

Follow this pipeline to convert a high-detail model into a DINO-ready unit:

### Step 1: Silhouette Exaggeration
- **Helmets**: Increase dome/height by 10–20% (Republic: rounded, CIS: angular)
- **Shoulders**: Broaden pauldrons; exaggerate armor plate edges
- **Weapons**: Scale up by 15–25% for visual impact from distance
- **Limbs**: Maintain proportions but sharpen geometric edges

**Rationale**: Distant units need high visual contrast. Oversized helmet = instant "soldier" read.

### Step 2: Material Simplification
- Strip all PBR (physically-based rendering) complexity:
  - Remove metallic maps, roughness maps, normal maps, ambient occlusion maps
  - Convert all textures to single flat diffuse color per material
- Merge similar-color surfaces (e.g., all blue trim into one material)
- Target: **2–4 materials max** per unit

**Tools**: Blender Principled BSDF shader, delete texture nodes, set solid RGB color

### Step 3: Detail Removal
- Delete all elements < 0.1 unit dimensions:
  - Rivets, panel lines, seams, small vents
  - Fine mechanical detail, cables, small attachments
  - Intricate belt buckles, insignia (if < 0.15 units)
- Keep large geometric shapes: armor plates, major joints, helmet domes

**Rule of thumb**: If you can't see it from 50 units away in-game, it goes.

### Step 4: Shape Enhancement
- **Smooth curves**: Apply Bevel modifier (0.01–0.05 unit) to harden silhouette edges without over-smoothing
- **Simplify geometry**: Consider Remesh modifier (voxel method, 0.08–0.1 voxel size) for subtle polygon reduction while preserving shape
- **Preserve military geometry**: Keep angular shoulders, helmet visor slots, weapon barrels—these are recognizable landmarks

### Step 5: Faction Recoloring
- Apply faction color palette:
  - **Primary material**: Main faction color (white for Republic, tan for CIS)
  - **Secondary material**: Trim/accents (blue for Republic, brown for CIS)
  - **Metal accents**: Gold (Republic) or gunmetal gray (CIS)
  - **Black/dark**: Weapon barrels, shadows, undersuit

**Note**: If original model uses wrong faction colors, recolor all Diffuse values in Blender Principled BSDF nodes.

### Step 6: LOD Variant Creation
- Create 50% decimated version for distance rendering:
  1. Duplicate model
  2. Apply Decimate modifier: ratio = 0.5, threshold = 0.01
  3. Export as `stylized_lod.glb`
- DINO's asset pipeline can swap LOD versions at runtime (optional feature, future version)

---

## 4. Material Assignment Rules

Consistent material structure across all units:

| Part Type | Base Color | Metallic | Roughness | Notes |
|-----------|-----------|----------|-----------|-------|
| **Armor plating** | Faction primary (white/tan) | 0.2–0.4 | 0.6–0.8 | Slight metalness for worn look |
| **Undersuit / cloth** | Darker faction shade or `#666666` | 0.0 | 0.9 | Fully matte |
| **Weapons** | `#1a1a1a` (matte black) or `#4a4a4a` (gunmetal) | 0.1–0.3 | 0.7–0.9 | Minimal shine, practical look |
| **Visor / eyes** | Bright faction accent (`#00ddff` cyan, `#ffaa00` orange) | 0.0 | 0.3 | Glossy for "active" feel |
| **Gold trim** | `#d4af37` (Republic) or `#b8956d` (CIS) | 0.6 | 0.4 | More metallic than armor |
| **Utility details** | `#4a7ba7` (blue trim) or `#8b6914` (brown trim) | 0.0 | 0.8 | Match secondary faction color |

**Assignment in Blender**:
1. Create material per type (e.g., `M_Armor_White`, `M_Undersuit_Gray`, `M_Weapon_Black`)
2. Use Principled BSDF with **only** Diffuse, Metallic, Roughness inputs (no texture nodes)
3. Assign to mesh faces by selecting faces in Edit Mode → Assign Material

---

## 5. Blender Stylization Workflow

**Prerequisites**: Blender 4.0+, source model (normalized.glb), faction assignment

### 5.1 Import & Setup
```
1. File → Import → glTF 2.0 (.glb/.gltf) → normalized.glb
2. Select all objects: A (deselect all) → Shift+Click each mesh
3. Ctrl+J to join into single object (optional, for simplicity)
```

### 5.2 Material Creation
```
1. Switch to Shading workspace (top bar)
2. Select mesh → in Shader Editor, add new material
3. Delete default Principled BSDF's texture inputs:
   - Select Image Texture nodes → X → Delete
   - Select Normal Map nodes → X → Delete
   - Disconnect everything except Principled BSDF output
4. Left on Principled BSDF node only:
   - Base Color (RGB color picker)
   - Metallic (0.0–0.4)
   - Roughness (0.6–0.9)
5. Duplicate material for each faction color needed
```

### 5.3 Material Assignment by Color
```
1. Switch to Edit Mode (Tab)
2. Select all faces: A (all)
3. In Shader Editor, select first material (e.g., M_Armor_White)
4. Deselect all: Alt+A
5. Select only white faces: (use Select → By Color in UV Editor or Face Select mode)
6. Assign to M_Armor_White in material properties panel
7. Repeat for each faction color
```

### 5.4 Silhouette & Edge Hardening
```
1. Switch to Modifiers tab (wrench icon)
2. Add modifier: Bevel
   - Amount: 0.02–0.05 unit
   - Segment count: 2 (crisp)
   - Weight: enabled
3. Add modifier: Decimate (for LOD)
   - Mode: Collapse
   - Ratio: 0.5 (50% reduction)
   - Use Collapse Degenerate
   - Apply to separate copy for LOD variant
4. Optional: Add Remesh modifier
   - Mode: Voxels
   - Voxel Size: 0.08–0.1
   - Smoothness: 1 (subtle smoothing)
```

### 5.5 Final Cleanup & Export
```
1. Object Mode (Tab)
2. Apply all modifiers: Ctrl+Shift+A → Apply All Modifiers
3. Select All objects → File → Export → glTF 2.0 (.glb)
   - Name: stylized.glb
   - Export settings:
     * Format: glTF Binary (.glb)
     * Include: Normals, Vertex Colors
     * Compression: Draco (if file size > 5MB)
4. Export Blender file: File → Save As → stylized.blend (for future edits)
```

### 5.6 Save Editable Blender File
```
File → Save As → stylized.blend
(Keep this for future revisions or LOD tweaking)
```

---

## 6. Comparison Table: High-Detail vs Stylized

Reference for how much each metric should change:

| Aspect | High-Detail (Sketchfab Original) | Stylized (DINO Target) | Reduction |
|--------|----------------------------------|----------------------|-----------|
| **Polycount** | 4,000–60,000 | 800–3,000 | 85–93% ✓ |
| **Materials** | 3–8 with PBR | 2–4 solid colors | 50–75% ✓ |
| **Textures** | 2k–4k (detailed) | None (solid color) or 512×512 (flat) | ~100% or 75% ✓ |
| **Texture maps** | Diffuse, Normal, Metallic, Roughness, AO, Emissive | Diffuse only | 100% ✓ |
| **Silhouette** | Film-accurate, subtle | Exaggerated, instantly readable | +15–20% size |
| **Animation rig** | Full skeleton (60+ bones) | Simplified or none (static model) | Deferred to animation pipeline |
| **File size** | 2–15 MB | 0.3–1 MB | 90% reduction ✓ |

**Goal**: Stylized should **feel like TABS**, not like a realistic film prop. Player should recognize unit type from silhouette alone.

---

## 7. Faction-Specific Stylization Guidelines

### 7.1 Republic Clone Trooper

**Visual markers**:
- Rounded helmet dome (not angular)
- White armor base with blue trim
- Bright cyan visor / eyes
- Kama (waist cloth) as flat plane or simple geometry
- Blaster rifle with rounded stock

**Stylization steps**:
1. Exaggerate helmet dome height by 15% (make it iconic)
2. Simplify kama to 2–4 flat rectangular planes
3. Widen shoulder pauldrons by 10%
4. Color: `#FFFFFF` armor, `#1a4d7a` blue trim, `#00ddff` visor glow
5. Lighten entire model slightly (Republic aesthetic = clean, bright)

**Polycount target**: 1,200–1,800 (with bevel modifier applied)

### 7.2 CIS B1 Battle Droid

**Visual markers**:
- Thin, spindly limbs (exaggerated proportions)
- Angular joints with visible square joint blocks
- Large head relative to body (reads "droid")
- Tan/gold chassis with brown accents
- Simple angular visor slit (no glow)

**Stylization steps**:
1. Exaggerate limb thinness: decrease arm/leg cylinder radius by 10–15%
2. Make joint blocks **more angular**, add beveled edges for mechanical feel
3. Enlarge head by 5–10% relative to body
4. Color: `#c9a84c` tan primary, `#8b6914` brown secondary
5. Add `#333333` stripe along torso centerline (droid design language)
6. Simplify hands to flat plates if possible

**Polycount target**: 1,000–1,600

### 7.3 Republic Heavy Weapons Specialist

**Visual markers**:
- Heavier armor plating (thicker silhouette)
- Oversized weapon (minigun, repeater cannon)
- Pauldrons extend further
- Ammo pack visible on back or shoulder

**Stylization steps**:
1. Increase torso width by 15–20% for "heavy" feel
2. Scale weapon up by 25% (readability + intimidation)
3. Add exaggerated ammo pack on back: simple rectangular geometry
4. Color same as Clone: white + blue
5. Add subtle `#888888` metal panel on weapon

**Polycount target**: 1,600–2,200

### 7.4 CIS B2 Super Battle Droid

**Visual markers**:
- More massive, more angular than B1
- Thick limbs with visible actuators
- Flat, blocky head
- Often has red markings or darker coloring
- Heavier weaponry (shoulder-mounted cannons)

**Stylization steps**:
1. Double arm/leg thickness vs B1 (exaggerate size difference)
2. Add beveled joint blocks at each articulation (sharp, industrial)
3. Blocky head with minimal features
4. Color: `#9d8b5c` muted tan, `#6b5d3f` dark brown
5. Optional: add `#ff4444` accent stripe on upper chest (visual marker)
6. Simplify shoulder cannon to geometric cone or cylinder

**Polycount target**: 1,800–2,400

### 7.5 Tactical Droid (Optional)

**Visual markers**:
- More "intelligent" appearance than B1/B2
- Sometimes has unique head design with antenna
- Often gold or orange accents
- Slightly more detailed than common droids (rank marker)

**Stylization steps**:
1. Refine head shape: more rounded but still droid-like
2. Add simple antenna (cylinder, 2–4 units tall)
3. Color: `#b8956d` lighter tan, `#7a6a4f` weathered brown
4. Accent: `#ff6b35` orange stripe or panel (leadership marker)
5. Keep body proportions mid-way between B1 and B2

**Polycount target**: 1,400–1,900

---

## 8. Preview & Validation Checklist

Before finalizing any stylized model, verify all criteria:

- [ ] **Silhouette recognizable at 100+ units**: Stand back in viewport, dim lighting, can you instantly ID faction + unit type?
- [ ] **Faction colors dominant**: Primary color covers ~60–70% of visible surface
- [ ] **No tiny details**: All visual elements > 0.1 unit dimension. No rivets, cables, or sub-0.1 bits
- [ ] **Material count ≤ 4**: Usually 2–3 (armor, undersuit/weapon, accent)
- [ ] **Polycount within budget**: 800–2,200 (specific target by unit type)
- [ ] **Proportions exaggerated slightly**: Helmet ~15–20% larger, weapons ~15–25% larger
- [ ] **No floating vertices**: Run Mesh → Cleanup → Remove Doubles
- [ ] **Smooth shading enabled**: Object → Shade Smooth (for Bevel + visual appeal)
- [ ] **Normals recalculated**: Select all faces → Mesh → Normals → Recalculate Normals
- [ ] **File exports cleanly**: File → Export → glTF 2.0, open re-imported file to verify integrity
- [ ] **Comparison screenshot created**: In-game or viewport, side-by-side original vs stylized

**Blockers**: If any checkbox fails, do not export. Fix and re-verify.

---

## 9. Output Artifacts

For each stylized unit, create and commit these files:

### 9.1 Main Model Files
```
packs/warfare-starwars/assets/models/[unit-type]/
├── stylized.glb              # Game-ready, optimized
├── stylized.blend            # Editable Blender file (for future revisions)
└── stylized_lod.glb          # 50% decimated LOD variant (optional, future)
```

### 9.2 Documentation & Metadata
```
packs/warfare-starwars/assets/models/[unit-type]/
├── stylization_report.json   # Metadata: what was changed, before/after stats
└── preview.png               # In-game or viewport screenshot (2k, side-by-side if possible)
```

### 9.3 Stylization Report Format (JSON)

```json
{
  "model_name": "Clone_Trooper_Standard",
  "faction": "Republic",
  "unit_type": "Infantry",
  "stylization_date": "2026-03-11T00:00:00Z",
  "source": {
    "filename": "Clone_Trooper_CT-5555_Tup.glb",
    "polycount": 12847,
    "material_count": 6,
    "texture_count": 4,
    "largest_texture": "2048x2048"
  },
  "target": {
    "filename": "stylized.glb",
    "polycount": 1456,
    "material_count": 3,
    "texture_count": 0,
    "file_size_mb": 0.45
  },
  "reduction_metrics": {
    "polycount_reduction_percent": 88.7,
    "material_reduction_percent": 50.0,
    "texture_reduction_percent": 100.0,
    "file_size_reduction_percent": 92.3
  },
  "stylization_steps_applied": [
    "silhouette_exaggeration_helmet_15pct",
    "material_simplification_pbr_to_diffuse",
    "detail_removal_rivets_panels",
    "shape_enhancement_bevel_0.03",
    "faction_recoloring_republic_white_blue",
    "lod_creation_50pct_decimation"
  ],
  "faction_colors": {
    "primary": "#FFFFFF",
    "secondary": "#1a4d7a",
    "accent": "#d4af37"
  },
  "validation_checklist": {
    "silhouette_readable_100m": true,
    "faction_colors_dominant": true,
    "no_tiny_details": true,
    "material_count_under_4": true,
    "polycount_in_budget": true,
    "proportions_exaggerated": true,
    "floating_vertices_removed": true,
    "smooth_shading_applied": true,
    "normals_recalculated": true,
    "exports_cleanly": true,
    "comparison_screenshot_created": true
  },
  "notes": "Clone Trooper standard infantry unit. Exaggerated helmet dome for readability. Simplified kama to 2 planes. Blue visor glow at 0x00ddff."
}
```

### 9.4 Example Directory Structure
```
packs/warfare-starwars/assets/models/
├── clone_trooper_standard/
│   ├── stylized.glb
│   ├── stylized.blend
│   ├── preview.png
│   └── stylization_report.json
├── b1_battle_droid/
│   ├── stylized.glb
│   ├── stylized.blend
│   ├── preview.png
│   └── stylization_report.json
├── clone_trooper_heavy/
│   ├── stylized.glb
│   ├── stylized.blend
│   ├── preview.png
│   └── stylization_report.json
└── b2_super_battle_droid/
    ├── stylized.glb
    ├── stylized.blend
    ├── preview.png
    └── stylization_report.json
```

---

## 10. Quality Gate: Side-by-Side Comparison

For final sign-off, create a visual comparison showing:

1. **Original (high-detail)**: Rotated 45° angle, good lighting
2. **Stylized (DINO)**: Same angle, same lighting
3. **In-game preview** (if available): Standing on battlefield at ~50 units distance

**Success criteria**:
- Stylized feels like a **distinct artistic choice**, not a degradation
- Silhouette is **more readable**, not less
- Faction identity is **crystal clear**
- Unit type (heavy vs standard) is **instantly obvious**

---

## 11. Troubleshooting & Edge Cases

### Problem: Stylized model looks "lumpy" after Remesh
**Solution**: Reduce voxel size from 0.1 to 0.08, or skip Remesh entirely. Bevel alone often sufficient.

### Problem: Material colors look different in-game than in Blender
**Solution**:
1. Check lighting model (DINO uses ECS PBS shading)
2. Export without Draco compression (may degrade color fidelity)
3. Verify Principled BSDF Metallic/Roughness values are within 0.0–1.0

### Problem: Polycount still > 3,000 after optimization
**Solution**:
1. Increase Decimate ratio: 0.4–0.5 (60–50% remaining)
2. Remove non-essential geometry (small details, internal faces)
3. Merge co-planar faces in Edit Mode (Select > Linked Flat Faces)

### Problem: Visor / eyes look flat and unreadable
**Solution**:
1. Increase Metallic to 0.6–0.8 for glossy shine
2. Reduce Roughness to 0.2–0.3 for bright highlight
3. Use bright accent color (`#00ddff` cyan, `#ffff00` yellow)
4. Optionally add small emissive glow (future: Blender 4.1+ supports emissive in glTF)

### Problem: Weapon looks too small compared to unit
**Solution**: Scale weapon up by additional 10–15% via proportional scale (S key, then number + confirm)

---

## 12. Integration with DINO Asset Pipeline

Once stylized models are complete:

1. **Place in content pack**:
   ```
   packs/warfare-starwars/assets/models/[unit]/stylized.glb
   ```

2. **Create unit manifest entry** (`packs/warfare-starwars/units.yaml`):
   ```yaml
   - id: unit_clone_trooper
     name: Clone Trooper
     model_path: assets/models/clone_trooper_standard/stylized.glb
     faction: Republic
     cost: 20
     ...
   ```

3. **Validate schema**: `dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars/`

4. **Build pack**: `dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars`

5. **Load in-game**: `dinoforge reload packs/warfare-starwars` (CLI)

6. **Verify**: F10 in-game overlay, check unit appears with correct colors and proportions

---

## 13. References & Resources

### Blender Documentation
- [Principled BSDF shader](https://docs.blender.org/manual/en/latest/render/shader_nodes/shader/principled.html)
- [Bevel Modifier](https://docs.blender.org/manual/en/latest/modeling/modifiers/generate/bevel.html)
- [Decimate Modifier](https://docs.blender.org/manual/en/latest/modeling/modifiers/generate/decimate.html)
- [glTF 2.0 Export](https://docs.blender.org/manual/en/latest/addons/import_export/scene_gltf2.html)

### TABS Aesthetic Inspiration
- Totally Accurate Battle Simulator: low-poly, exaggerated proportions, readable at distance
- Team Fortress 2: strong silhouettes, flat color, minimal texture detail

### DINO Integration
- See: `src/SDK/Assets/` (AssetsTools.NET, AddressablesCatalog)
- See: `packs/warfare-modern/` (reference pack structure)
- See: `schemas/unit.json` (unit model requirements)

---

## Appendix: Blender Quick Reference Card

### Material Setup (Copy-Paste)
```
Principled BSDF:
  Base Color:  [RGB picker]
  Metallic:    [0.0–0.4]
  Roughness:   [0.6–0.9]
  [All other inputs: DISCONNECT]
```

### Modifier Stack (in order)
```
1. Bevel (Amount: 0.03, Segments: 2)
2. Decimate (optional, Ratio: 0.5, Collapse mode)
3. Remesh (optional, Voxels, Voxel Size: 0.1)
```

### Export Checklist
```
[ ] Smooth Shading applied (Object → Shade Smooth)
[ ] Normals recalculated (Mesh → Normals → Recalc)
[ ] Doubles removed (Mesh → Cleanup → Remove Doubles)
[ ] All modifiers applied (Ctrl+Shift+A → Apply All)
[ ] File → Export → glTF 2.0 (.glb)
[ ] Format: Binary (.glb)
[ ] Include: Normals, Vertex Colors
[ ] Compression: Draco (if > 5MB)
[ ] Exported file re-imported and verified
```

### Common Hotkeys
| Action | Hotkey |
|--------|--------|
| Select All | A |
| Deselect All | Alt+A |
| Toggle Edit Mode | Tab |
| Wireframe view | Z → Wireframe |
| Solid view | Z → Solid |
| Switch to Shading | Top bar: Shading tab |
| Apply modifier | Modifiers panel → Apply |
| Scale | S → [number] → Enter |
| Rotate | R → [axis] → [degrees] → Enter |

---

**Document version**: 1.0
**Last updated**: 2026-03-11
**Maintained by**: DINOForge Agent Team
**Status**: Approved for production use
