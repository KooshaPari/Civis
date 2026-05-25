# warfare-starwars Asset Pipeline Guide

This guide covers how to create, source, import, and contribute visual assets for the
**Star Wars Clone Wars** pack. All assets are currently placeholders; community contributions
are welcomed and encouraged.

---

## Art Style Guide: Low-Poly TABS Aesthetic

### Philosophy

DINO's engine runs a low-poly art style similar to **Totally Accurate Battle Simulator (TABS)**:
simple geometric shapes, flat pastel/matte textures, exaggerated proportions, and no fine surface
detail. This style is an excellent fit for Star Wars fan assets because the iconic silhouettes of
clone troopers, battle droids, and vehicles are defined by their broad shapes - not fine detail.

A clone trooper reads as a clone trooper from its helmet dome and T-visor silhouette alone.
A battle droid reads as a battle droid from its spindly stick-figure proportions alone.
Lean into this: **silhouette over surface detail.**

### Polygon Budgets

| Asset Category | Budget | Notes |
|---|---|---|
| Infantry (standing) | 250-400 tris | TABS-style exaggerated proportions |
| Infantry (hero/elite) | 400-500 tris | Extra detail for prominence |
| Light vehicle / speeder | 400-600 tris | Rider + vehicle combined |
| Medium vehicle / tank | 800-1200 tris | Main battle unit |
| Walker (AT-TE) | 1000-1500 tris | Six legs, must read cleanly at distance |
| Small building / turret | 60-250 tris | Keeps scene draw-call budget low |
| Standard building | 300-600 tris | Most buildings |
| Large command building | 500-700 tris | Tallest structures |
| Prop (weapon, crate) | 20-80 tris | Keep minimal |
| VFX mesh (bolt, blade) | 8-60 tris | Emissive, high visual impact per tri |

**Hard limit**: no single asset may exceed 1500 tris. The engine must render hundreds of units
simultaneously; budget discipline is essential.

### Shape Language

**Republic (Galactic Republic)**
- Rounded, clean, organized geometry
- Helmet domes, curved pauldrons, smooth barrel forms
- Conveys order, discipline, institutional authority
- Units should look like a coherent professional army

**CIS (Confederacy of Independent Systems)**
- Angular, mechanical, segmented geometry
- Thin interconnected rods (B1 droids), boxy industrial forms (AAT, factory)
- Conveys mass production, expendability, industrial menace
- Units should look like they came off an assembly line

### Color Palettes

**Republic**

| Element | Hex | Notes |
|---|---|---|
| Armor primary | `#F5F5F5` | Near-white, slight warm tint |
| Squad blue stripe | `#1A3A6B` | Deep navy blue |
| Heavy yellow stripe | `#FFD700` | Gold-yellow for heavy troopers |
| Medical red | `#CC2222` | Cross insignia only |
| ARC / ARF green | `#4A7A4A` | Muted military green |
| Commando dark | `#444444` | Under-suit charcoal |
| Jedi tan robe | `#C8A87A` | Warm sand tan |
| Jedi lightsaber | `#4488FF` | Emissive blue |
| Building walls | `#EEEEEE` to `#DDDDDD` | Off-white durasteel |
| Building accent | `#1A3A6B` | Navy blue trim panels |
| Shield generator glow | `#4488FF` | Emissive blue |

**CIS**

| Element | Hex | Notes |
|---|---|---|
| B1 droid base | `#C8A87A` | Sandy tan |
| B1 joint recesses | `#5C3D1E` | Dark brown |
| B1 photoreceptor | `#CC2222` | Red dot eye |
| B2 chest armor | `#3A4A5A` | Steel blue-gray |
| BX / MagnaGuard dark | `#333333` | Near-black |
| AAT hull | `#C8A87A` | Tan matching B1 |
| AAT panels | `#4A5A2A` | Olive green |
| Factory exhaust glow | `#FF6600` | Orange heat |
| Sentry sensor | `#CC2222` | Red photoreceptor |
| Ray shield glow | `#CC2222` | Red emissive |
| Durasteel barrier | `#555555` | Medium gray |
| Droideka shield | `#4488FF` @ 20% alpha | Transparent blue bubble |

**CIS blaster bolts**: `#FF4400` red-orange
**Republic blaster bolts**: `#4488FF` bright blue
**Lightsaber blades**: `#4488FF` blue (Jedi), `#44FF44` green, `#FF44FF` purple (Grievous trophies)
**Electrostaff tips**: `#FFFF44` yellow emissive

### Proportions (TABS Style)

- Infantry heads are slightly oversized (1.2-1.5x realistic scale)
- Limbs are slightly shortened and thickened vs realistic anatomy
- Vehicles are squat and wide rather than tall and thin
- Heroes are visually larger than standard infantry (1.3-1.5x scale)
- Weapons are slightly oversized for readability at camera distance

---

## Free Asset Sources

All assets used in this pack must be **CC0** (public domain) or **CC-BY** (attribution required).
Attribution for CC-BY assets must be added to `/packs/warfare-starwars/assets/ATTRIBUTION.md`.
Do not use assets with NC (non-commercial), ND (no-derivatives), or SA (share-alike) licenses in
game-distributed packs. Fan/parody status does not override license restrictions on third-party assets.

### Primary Sources

**Kenney.nl** - https://kenney.nl/assets
- License: CC0 (all packs) - no attribution required
- Best packs for this project:
  - **Character Pack** - humanoid soldier base meshes perfect for clone/droid variants
  - **Robot Pack** - mechanical humanoid variants ideal for B1/B2 droids
  - **Space Kit** - hover vehicles, pylons, emitter shapes
  - **Modular Buildings** - tileable building components
  - **Platformer Kit** - small props (crates, barriers)
  - **Particle Pack** - 2D sprites for VFX bolt/explosion effects
  - **Vehicle Pack** - ground vehicles as base for AAT/BARC

**PolyPizza** - https://poly.pizza
- License: CC0 and CC-BY (check per-asset)
- Search terms to try: "clone trooper", "battle droid", "sci-fi soldier", "hover tank",
  "sci-fi building", "robot", "mech walker", "spider robot", "military barracks"

**Sketchfab Free Section** - https://sketchfab.com/search?features=downloadable&sort_by=-likeCount&type=models
- License: varies - check per-asset, look for CC0 or CC-BY only
- Many Star Wars fan assets available; confirm license before use
- Search terms: "AT-TE low poly", "clone trooper low poly", "battle droid low poly",
  "B1 droid", "droideka", "AAT tank", "BARC speeder", "General Grievous low poly"

**OpenGameArt** - https://opengameart.org
- License: CC0 and CC-BY available (filter by license)
- Good for 2D VFX sprites (bolt/explosion sheets)

### Recommended Workflow: Starting from Kenney

1. Download the relevant Kenney pack (CC0, no attribution needed)
2. Open the FBX/OBJ in **Blender** (free, https://blender.org)
3. Select the closest base mesh (humanoid soldier, robot variant, etc.)
4. Apply Star Wars-specific modifications:
   - Clone trooper helmet: add dome cap, T-visor slot cut
   - B1 droid: extend/thin limbs, elongate oval head
   - AT-TE: box body, six-leg rig from robot leg segments
5. Check poly count (Edit mode > Mesh Stats overlay)
6. Reduce if over budget using Decimate modifier (ratio ~0.5 start)
7. UV unwrap: use Smart UV Project for simple low-poly geometry
8. Export as FBX (File > Export > FBX, check "Selected Objects")

---

## Unity 2021.3 Import Workflow

### Prerequisites

- Unity 2021.3.x LTS (matching DINO's engine version)
- Unity Addressables package v1.21.18 (must match game version exactly)
- A fresh Unity project (use 3D URP template for consistent lighting)

### Step 1: Project Setup

```
File > New Project > 3D (URP) > Create Project
Window > Package Manager > Add package by name: com.unity.addressables (1.21.18)
```

Create the following folder structure inside `Assets/`:
```
Assets/
  StarWars/
    Meshes/
      Units/
        Republic/
        CIS/
      Buildings/
        Republic/
        CIS/
      Weapons/
    Textures/
      Units/
      Buildings/
    Materials/
    Prefabs/
      Units/
        Republic/
        CIS/
      Buildings/
        Republic/
        CIS/
    VFX/
```

### Step 2: Import Meshes

1. Drag FBX files into the appropriate `Assets/StarWars/Meshes/` subfolder
2. In the Inspector for each FBX:
   - **Scale Factor**: set to match DINO's unit scale (test in scene; infantry ~1.0-1.5 Unity units tall)
   - **Mesh Compression**: Low (preserve geometry for low-poly look)
   - **Read/Write Enabled**: check off (saves memory)
   - **Generate Lightmap UVs**: off (no baked lighting needed)
   - **Animation**: set to None for static meshes; Humanoid rig only if animating
3. Click Apply

### Step 3: Create Materials

For each unit/building, create a **URP Lit** material:
1. Right-click in Materials folder > Create > Material
2. Shader: Universal Render Pipeline/Lit
3. Set **Surface Type**: Opaque (transparent only for shield bubble)
4. Set **Base Map**: drag in the albedo texture PNG
5. Set **Smoothness**: 0.1-0.2 (matte TABS-style finish)
6. Set **Metallic**: 0 for organic/cloth, 0.3-0.5 for metal armor
7. For emissive elements (lightsabers, shield generators):
   - Enable **Emission**
   - Set emission color to the appropriate emissive hex value
   - Set HDR intensity to 1.5-2.0 for visible glow

### Step 4: Create Prefabs

1. Drag mesh into the scene viewport (creates a GameObject with MeshRenderer)
2. Assign the correct material in the Inspector
3. Add any child props (weapon mesh, shield dome, etc.) as child GameObjects
4. Set position/rotation to identity (0,0,0 / 0,0,0)
5. Drag from Hierarchy into `Assets/StarWars/Prefabs/` folder to create prefab
6. Delete from scene (prefab is saved)

### Step 5: Set Up Addressables Group

1. Window > Asset Management > Addressables > Groups
2. In the Groups window: Create > Group > Packed Assets
3. Name it `warfare-starwars`
4. In the Group Inspector:
   - **Build Path**: `[UnityEngine.AddressableAssets.Addressables.BuildPath]`
   - **Load Path**: `{UnityEngine.AddressableAssets.Addressables.RuntimePath}`
   - **Bundle Mode**: Pack Together (all assets in one bundle)
5. Select all prefabs in `Assets/StarWars/Prefabs/` (Ctrl+A in that folder)
6. In Inspector, check **Addressable** checkbox for each prefab
7. Set the Address for each prefab to match the pack id format:
   ```
   warfare-starwars/units/rep_clone_trooper
   warfare-starwars/units/cis_b1_battle_droid
   warfare-starwars/buildings/rep_command_center
   ...etc
   ```
   The address must match the `id` field in `manifest.yaml`.

### Step 6: Build the Bundle

1. Window > Asset Management > Addressables > Groups
2. Build > New Build > Default Build Script
3. Build output appears at: `Library/com.unity.addressables/aa/StandaloneWindows64/`
4. The relevant file is named `warfare-starwars_assets_all_[hash].bundle`

---

## Bundle Placement

After building, copy the bundle file to the pack directory:

```
packs/warfare-starwars/assets/warfare-starwars-assets.bundle
```

Update `pack.yaml` assets section to reference it:
```yaml
assets:
  manifest: assets/manifest.yaml
  bundle: assets/warfare-starwars-assets.bundle
```

The DINOForge SDK's `AddressablesCatalog` service will locate and load the bundle from this path
at runtime when the pack is loaded. The catalog JSON from the Addressables build also belongs here:

```
packs/warfare-starwars/assets/catalog_warfare-starwars.json
```

**Do not commit the `.bundle` file to git.** Bundle files are large binary artifacts. Add the
following to `.gitignore` or note that bundles should be distributed separately (GitHub Releases,
itch.io, etc.):
```
packs/*/assets/*.bundle
packs/*/assets/catalog_*.json
```

---

## Validation

After placing the manifest and before building the bundle, validate the pack structure:

```bash
dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars
```

This will:
- Parse `pack.yaml` and all YAML content files
- Validate against schemas in `schemas/`
- Check that all `id` references (units, weapons, buildings) are consistent
- Report any missing required fields

Expected output for a valid pack:
```
[PackCompiler] Validating packs/warfare-starwars...
[PackCompiler] pack.yaml OK
[PackCompiler] factions: 2 entries OK
[PackCompiler] units: 26 entries OK
[PackCompiler] weapons: 18 entries OK
[PackCompiler] buildings: 20 entries OK
[PackCompiler] doctrines: 6 entries OK
[PackCompiler] Asset manifest: assets/manifest.yaml present (placeholder mode)
[PackCompiler] Validation complete. 0 errors.
```

The `placeholder: true` flag in `manifest.yaml` suppresses errors for missing `.bundle` files,
allowing the pack to be used in content-only mode with vanilla DINO mesh fallbacks until real
assets are contributed.

---

## Contributing Assets

### Contribution Checklist

Before submitting a pull request with new assets:

- [ ] Asset is CC0 or CC-BY licensed
- [ ] If CC-BY: attribution added to `packs/warfare-starwars/assets/ATTRIBUTION.md`
- [ ] FBX source file placed in `assets/src/meshes/` (do not commit binary to main bundle)
- [ ] Poly count is within budget (see table above)
- [ ] Texture is power-of-two resolution (256x256, 512x512, or 1024x1024 max)
- [ ] Prefab address matches the `id` in `manifest.yaml`
- [ ] `placeholder: true` removed from the relevant manifest entry
- [ ] `source_hint` updated to reflect actual source used
- [ ] Bundle rebuilt and tested in DINO with `dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars`
- [ ] Screenshot of asset in-engine included in PR description

### Priority Asset List

Community contributors: start here for maximum impact.

**Highest priority (most visible in gameplay):**
1. `cis_b1_battle_droid` - appears in swarm quantities, needs to be very low poly
2. `rep_clone_trooper` - the "face" of the Republic faction
3. `cis_b2_super_battle_droid` - heavy unit, visually distinctive
4. `rep_atte_crew` / AT-TE walker - iconic siege vehicle
5. `cis_aat_crew` / AAT tank - CIS siege vehicle
6. `rep_jedi_knight` - hero unit, very visible
7. `cis_general_grievous` - hero unit, four-armed, distinctive silhouette

**Medium priority:**
- All standard buildings (command centers, factories, barracks)
- Speeder/STAP vehicles
- Support and scout infantry variants

**Lower priority (cosmetic variety):**
- Wall segments (blast wall, durasteel barrier)
- Economy buildings (refineries, mining facilities)
- Blaster bolt VFX (simple particle systems, quick to make)

---

## Quick-Start: Blender Modification of Kenney Assets

The fastest path to a passable TABS-style clone trooper from zero:

1. Download **Kenney Character Pack** from https://kenney.nl/assets/character-pack (CC0)
2. Open `soldier.fbx` in Blender
3. Enter Edit Mode, select the helmet mesh
4. Add a Subdivide (1 cut) to helmet top, scale into dome shape
5. Cut a rectangular T-visor slot: Loop Cut + Delete faces, fill with dark material
6. Select entire figure, apply the white material with blue stripe texture
7. Check face count: Viewport Overlays > Statistics > Faces
8. File > Export > FBX, name `rep_clone_trooper.fbx`

Total time from Kenney download to passable placeholder: approximately 30-60 minutes.
Total time for a polished low-poly model with proper UV paint: approximately 2-4 hours.

For B1 battle droids, start from Kenney's **Robot Pack** `robot_thin.fbx` variant - the
proportions are already nearly correct (thin limbs, segmented joints). Elongate the head
and add the characteristic horizontal slit visor.
