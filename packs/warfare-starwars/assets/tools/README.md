# Star Wars Building Asset Tools

This directory contains scripts for generating faction-color textures and assembling game-ready building models from Kenney source assets.

## Overview

The asset pipeline has three main stages:

1. **Texture Generation** (`generate_faction_textures.py`) - Create faction-color variants (Republic: white+blue, CIS: grey+orange)
2. **FBX Conversion** (`convert_kenney_to_game_fbx.py`) - Prepare Kenney models for use in game
3. **Blender Assembly** (`blender_assemble_buildings.py`) - Full material application and detail work (optional, manual)

## Quick Start

### 1. Generate All Textures

```bash
cd packs/warfare-starwars/assets
python tools/generate_faction_textures.py --output textures/buildings
```

Output: 24 PNG files in `textures/buildings/`
- 12 Republic textures (white + navy blue palette)
- 12 CIS textures (grey + rust orange palette)

### 2. Create FBX Placeholder Models

```bash
cd tools
python convert_kenney_to_game_fbx.py --pilot
```

Output: 2 pilot FBX files in `../meshes/buildings/`
- `rep_house_clone_quarters.fbx`
- `cis_house_droid_pod.fbx`

These are Kenney source FBX copies. To apply faction materials and optimize:

### 3. [Optional] Full Blender Assembly (Manual for now)

Requires Blender 3.6+ installed:

```bash
blender --python blender_assemble_buildings.py -- --output assets/meshes/buildings
```

This script:
- Imports Kenney FBX
- Applies faction texture + material
- Adds faction-specific details (decals, glow effects)
- Exports game-ready FBX

## Script Details

### generate_faction_textures.py

**Purpose**: Batch-generate faction-color texture variants from procedural/source textures.

**Usage**:
```bash
python generate_faction_textures.py [OPTIONS]

Options:
  --source DIR              Path to Kenney source (default: assets/source/kenney)
  --output DIR              Output texture directory (default: assets/textures/buildings)
  --faction {republic,cis,all}  Which faction to generate (default: all)
  --force                   Regenerate even if exists
  --verbose                 Show detailed progress
```

**Output**:
- Republic: 12 x `rep_*_albedo.png` (256x256, white+blue palette)
- CIS: 12 x `cis_*_albedo.png` (256x256, grey+orange palette)

**Color Palettes**:
- **Republic**: #F5F5F5 (white), #1A3A6B (navy blue), #EEEEEE (trim)
- **CIS**: #444444 (grey), #B35A00 (rust orange), #2A2A2A (shadow)

### convert_kenney_to_game_fbx.py

**Purpose**: Convert Kenney FBX source models to game-ready format.

**Usage**:
```bash
python convert_kenney_to_game_fbx.py [OPTIONS]

Options:
  --source DIR              Path to Kenney source
  --output DIR              Output mesh directory
  --textures DIR            Path to faction textures
  --pilot                   Only convert pilot buildings (for testing)
```

**Current Implementation**:
- If `trimesh` library available: converts FBX with poly optimization
- Fallback: copies source FBX as placeholder
- **Limitation**: Fallback method doesn't apply materials; requires Blender for full assembly

**Output**:
- Pilot (--pilot): 2 FBX files
  - `rep_house_clone_quarters.fbx`
  - `cis_house_droid_pod.fbx`
- Full (no --pilot): 24 FBX files

### blender_assemble_buildings.py

**Purpose**: Full Blender assembly workflow (material application, detail work, optimization).

**Usage**:
```bash
blender --python blender_assemble_buildings.py -- --output assets/meshes/buildings
```

**Features**:
- Loads Kenney source FBX
- Creates faction material with texture + colors
- Applies faction-specific details
- Optimizes poly count (target: 300-600 tris)
- Exports game-ready FBX

**Current Status**:
- Blender integration ready
- Material framework complete
- Detail pass placeholders in place

**Manual Workflow** (if script limitations encountered):
1. Open Blender
2. File → Import → FBX → Load `source/kenney/space-kit/Models/FBX format/structure.fbx`
3. Create new Material:
   - Base Color: faction palette primary (#F5F5F5 or #444444)
   - Metallic: 0.2
   - Roughness: 0.8
4. Add texture image if available
5. File → Export → FBX → Save to `meshes/buildings/{building_id}.fbx`

## Building Checklist

See `../BUILD_CHECKLIST.md` for:
- All 24 building definitions
- Source Kenney models for each
- Texture palette assignments
- Assembly notes and complexity ratings
- Status tracking (todo, in-progress, done)

## Asset Structure

```
packs/warfare-starwars/assets/
├── source/kenney/               # Kenney source (CC0 license)
│   ├── space-kit/Models/FBX format/
│   └── modular-space-kit/Models/FBX format/
├── textures/buildings/          # OUTPUT: faction textures (24 PNG files)
├── meshes/buildings/            # OUTPUT: game-ready FBX models (24 FBX files)
├── tools/
│   ├── generate_faction_textures.py
│   ├── convert_kenney_to_game_fbx.py
│   ├── blender_assemble_buildings.py
│   └── README.md (this file)
├── BUILD_CHECKLIST.md           # All 24 buildings, status, sources
├── ASSET_PIPELINE.md            # Art style guide, color palettes
└── crosswalk_*.yaml             # Asset swap manifests (Republic/CIS)
```

## Dependencies

### Required
- Python 3.9+
- Pillow (for texture generation)
  ```bash
  pip install Pillow
  ```

### Optional (for enhanced FBX conversion)
- trimesh (for FBX manipulation without Blender)
  ```bash
  pip install trimesh pyassimp
  ```
- Blender 3.6+ (for full assembly and material application)
  Download: https://www.blender.org/

## Workflow Recommendations

### Phase 1: Texture Generation (Quick)
```bash
python generate_faction_textures.py --all
```
Time: ~1 minute
Output: 24 faction-color textures ready for use

### Phase 2: Pilot FBX Assembly (Manual)
1. Run texture generation (Phase 1)
2. In Blender:
   - Import `source/kenney/space-kit/structure.fbx`
   - Apply white+blue material (Republic) or grey+orange (CIS)
   - Add simple details (stripes, decals)
   - Export to `meshes/buildings/{building_id}.fbx`
3. Test in game (scale, materials, performance)

Time: ~2 hours per pilot building
Output: 2 validated FBX + lessons learned for batch

### Phase 3: Batch Assembly (Repeatable)
1. Use Blender assembly script for remaining 22 buildings
2. Refine material templates based on pilot learnings
3. Batch-process via command line or custom Blender macro
4. Validate all 24 in game

Time: ~4-6 hours total
Output: 24 complete, game-ready building models

## Troubleshooting

### Textures not generating
```bash
python generate_faction_textures.py --verbose
```
Check that `source/kenney/` directory exists and is readable.

### FBX conversion fails
If trimesh not available, falls back to simple copy. For full conversion:
```bash
pip install trimesh pyassimp
```

Or use Blender:
```bash
blender --python blender_assemble_buildings.py
```

### Material not applying in Blender
1. Check that texture PNG exists: `textures/buildings/{building_id}_albedo.png`
2. Verify texture path in script
3. Create material manually in Blender if needed

### Models too high poly count
Use Blender's Modifier → Decimate to reduce polygons. Target: 300-600 tris.

## References

- **Kenney Assets**: https://kenney.nl (CC0 license)
- **Art Style Guide**: `ASSET_PIPELINE.md`
- **Building Definitions**: `BUILD_CHECKLIST.md`
- **Crosswalk Manifests**: `crosswalk_republic_vanilla.yaml`, `crosswalk_cis_vanilla.yaml`

## Next Steps

1. Generate textures (Phase 1)
2. Manually assemble 2 pilot buildings in Blender (Phase 2)
3. Refine Blender assembly script based on learnings
4. Batch-assemble remaining 22 buildings (Phase 3)
5. Validate all 24 in game
6. Document final assembly workflow for team

---

**Last Updated**: 2026-03-12
**Status**: Texture generation ✓ complete; FBX assembly ✓ framework ready (manual work needed)
