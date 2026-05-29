# AssetSwap Real Bundles Spec
## Issue #101: Star Wars Models Render as Nothing (0/36 in-game)

**Date**: 2026-05-28
**Status**: OPEN — root cause confirmed, fix path specified below

---

## 1. Why the Current Bundles Are Stubs

### The Three Bundle Classes in `packs/warfare-starwars/assets/bundles/`

| Class | Count | Size | Content | Loads in-game? |
|-------|-------|------|---------|---------------|
| 90-byte stubs | 12 | 90 B | `UnityFS` header + version string, zero assets | No — `LoadAsset<T>` returns null for anything |
| Primitive placeholder bundles | 48 | ~44 KB | Unity capsule/cube/sphere GameObject built from Unity primitives; real material color applied | Yes — renders colored shapes, NOT Star Wars meshes |
| Real mesh bundles | 16 | 345 KB–2.5 MB | Actual FBX geometry imported into Unity, baked into bundle | Yes — these render real 3D models |

**The 90-byte stubs are the direct cause of the 0/36 rendering failure for building-type units.** They have the correct bundle name (e.g. `sw-guard-tower`) and appear in `AssetBundles.manifest`, but their `UnityFS` block carries zero asset data. `AssetBundle.LoadFromFile()` succeeds (the file is a valid but empty UnityFS), then every `LoadAsset<T>` call returns null, so `TrySwapRenderMeshFromBundle` returns false and no swap is applied.

The 48 primitive placeholder bundles *do* render but render as Unity primitive shapes (capsule, cube, sphere), not as Star Wars characters or buildings. These account for the remaining ~24 entries that show colored geometry instead of real models.

### Why the PackCompiler Path Does NOT Build Real Bundles

Reading `src/Tools/PackCompiler/Services/`:

- **`AssetImportService.cs`** — uses AssimpNet to parse GLB/FBX into a C# `ImportedAsset` object (vertices, normals, UVs, materials as `float[]` arrays). It never touches `UnityEditor` or `BuildPipeline`. Output is a POCO, not a `.bundle` file.

- **`PrefabGenerationService.cs`** — serializes the `OptimizedAsset` to a hand-written YAML string that mimics `.prefab` file format. The YAML uses `GenerateGUID()` (random GUIDs) and `GenerateFileID()` (a simple counter). This YAML is **not a Unity-compiled prefab**; it is a text file that Unity's runtime cannot deserialize into a live GameObject. It is never fed into `BuildPipeline.BuildAssetBundles()`.

- **`AddressablesService.cs`** — generates Addressables catalog YAML / settings YAML. Again, these are text configuration stubs. No bundle compilation occurs.

**Conclusion**: The entire PackCompiler asset pipeline (`assets import`, `assets optimize`, `assets generate`) produces JSON/YAML metadata files and never calls `BuildPipeline.BuildAssetBundles()`. It cannot produce Unity AssetBundles. The real bundles that exist in the bundles directory were produced by a separate Unity Editor project (`unity-assetbundle-builder/`), not by PackCompiler.

### Why 12 Bundles Are Still 90-Byte Stubs

The 90-byte stubs correspond to building-type entries that were **never added to the `GenerateStarWarsPrefabs.cs` or `GenerateStarWarsPrefabsFromModels.cs` definition arrays**. Looking at the arrays in those scripts, the stub bundle IDs (`sw-guard-tower`, `sw-weapons-factory`, `sw-heavy-foundry`, `sw-mining-facility`, `sw-processing-plant`, `sw-skyshield-generator`, `sw-tech-union-lab`, `sw-tibanna-refinery`, `sw-vulture-nest`, `sw-assembly-line`, `sw-blast-wall`, `sw-durasteel-barrier`) do not appear in either `Definitions[]` / `Defs[]`. They appear to have been created as placeholder files (`echo "" > bundle-name`) with only a minimal UnityFS version header, then were never backed by an actual `BuildPipeline` invocation that included them.

---

## 2. The Correct End-to-End Pipeline

### The Authoritative Pipeline (Unity 2021.3 Required)

```
source FBX/GLB
  └─> [unity-assetbundle-builder/Assets/Models/] copy FBX here
        └─> GenerateStarWarsPrefabsFromModels.Generate()
              creates prefab in Assets/Prefabs/{faction}/{bundle-key}.prefab
              assigns assetBundleName = bundle-key on the prefab
              └─> BuildAssetBundles.BuildHeadless()
                    BuildPipeline.BuildAssetBundles(
                      "AssetBundles",
                      ChunkBasedCompression,
                      BuildTarget.StandaloneWindows64)
                    └─> unity-assetbundle-builder/AssetBundles/{bundle-key}  (real bundle)
                          └─> copy to packs/warfare-starwars/assets/bundles/{bundle-key}
```

### What Gets Built Into a Real Bundle

A real bundle contains a Unity-serialized `GameObject` prefab with:
- A `MeshFilter` holding the imported mesh (Class 33)
- A `MeshRenderer` holding the material (Class 23)
- A `Material` with `Standard` shader and faction color (Class 21)
- The `Mesh` asset itself (Class 43)
- A `Transform` component (Class 4)

The bundle filename is the `assetBundleName` value (e.g. `sw-rep-clone-trooper`). The asset name inside the bundle is the `GameObject.name` (also set to the bundle key, e.g. `sw-rep-clone-trooper`).

### How AssetSwapSystem Loads the Bundle

`AssetSwapSystem.TrySwapRenderMeshFromBundle()` does:
1. `AssetBundle.LoadFromFile(modBundlePath)` — loads the bundle file
2. `bundle.LoadAsset<Mesh>(assetName)` — null for prefab-based bundles
3. `bundle.LoadAsset<Material>(assetName)` — null for prefab-based bundles
4. **Fallback**: `bundle.LoadAsset<GameObject>(assetName)` — succeeds when the prefab's `name` matches `assetName`
5. Extracts `MeshFilter.sharedMesh` and `MeshRenderer.sharedMaterial[0]` from the prefab hierarchy
6. Sets these on matched ECS entities via `SetSharedComponentData<RenderMesh>`

**Critical**: `assetName` in the swap request must equal `GameObject.name` in the bundle. The existing `GenerateStarWarsPrefabs.cs` and `GenerateStarWarsPrefabsFromModels.cs` both set `go.name = def.BundleKey` — this is correct and matches the expected behavior.

---

## 3. Automating Unity 2021.3 Headlessly

### Unity Installation Status

Unity **2021.3.45f1** is installed at:
```
C:\Program Files\Unity\Hub\Editor\2021.3.45f1\Editor\Unity.exe
```

**Note**: CLAUDE.md specifies `2021.3.45f2` but the installed version is `2021.3.45f1`. These are functionally identical (patch-level difference); the existing real bundles in `bundles/` have the header `UnityFS    5.x.x  2021.3.45f1`, confirming this version was used to produce the working bundles. Use `f1`.

The existing `unity-assetbundle-builder/` Unity project is already set up at `C:\Users\koosh\Dino\unity-assetbundle-builder\` with:
- `Assets/Editor/BuildAssetBundles.cs` — headless bundle builder
- `Assets/Editor/GenerateStarWarsPrefabs.cs` — primitive placeholder generator
- `Assets/Editor/GenerateStarWarsPrefabsFromModels.cs` — mesh-from-FBX generator (preferred)
- `Assets/Models/` — FBX source files
- `Assets/Materials/{faction}/` — pre-generated materials
- `Assets/Prefabs/{faction}/` — pre-generated prefabs (currently primitives or mesh-backed)

### Headless Build Commands

**Step 1 — Regenerate prefabs** (required when adding/updating FBX models):
```powershell
& "C:\Program Files\Unity\Hub\Editor\2021.3.45f1\Editor\Unity.exe" `
  -batchmode `
  -nographics `
  -projectPath "C:\Users\koosh\Dino\unity-assetbundle-builder" `
  -executeMethod GenerateStarWarsPrefabsFromModels.Generate `
  -logFile "C:\Users\koosh\Dino\docs\sessions\unity-generate-prefabs.log" `
  -quit
```

**Step 2 — Build bundles**:
```powershell
& "C:\Program Files\Unity\Hub\Editor\2021.3.45f1\Editor\Unity.exe" `
  -batchmode `
  -nographics `
  -projectPath "C:\Users\koosh\Dino\unity-assetbundle-builder" `
  -executeMethod BuildAssetBundles.BuildHeadless `
  -logFile "C:\Users\koosh\Dino\docs\sessions\unity-build-bundles.log" `
  -quit
```

**Step 3 — Deploy bundles**:
```powershell
Copy-Item "C:\Users\koosh\Dino\unity-assetbundle-builder\AssetBundles\*" `
  -Destination "C:\Users\koosh\Dino\packs\warfare-starwars\assets\bundles\" `
  -Force
```

**Verification** — check log for success:
```powershell
Select-String -Path "C:\Users\koosh\Dino\docs\sessions\unity-build-bundles.log" `
  -Pattern "\[BuildAssetBundles\]"
```

### Important: Unity Batchmode Process Management

Unity batchmode can take 2–5 minutes on first run (asset import / Library cache cold). Do not assume the process is hung if it takes 3–4 minutes. Wait for exit code before proceeding. Unity `-quit` ensures the process terminates; without it, Unity hangs after batch operations.

---

## 4. Fallback: Can AssetsTools.NET Write Real Bundles Without Unity?

**Short answer: No — not for functional Unity-runtime bundles.**

AssetsTools.NET (used by `DINOForge.SDK.Assets.AssetService` for catalog reading and bundle patching) can:
- Read and patch existing Unity AssetBundle files (replacing raw bytes of an already-serialized asset)
- Read `Mesh` type data (vertices, indices) and write it back to an existing bundle slot

What it **cannot** do:
- Create a new AssetBundle file from scratch that Unity's runtime will accept as containing a `Mesh` or `GameObject`
- Serialize a `Mesh` asset in the Unity binary format that `AssetBundle.LoadAsset<Mesh>()` can deserialize — this requires the Unity type tree, which varies per Unity version and is embedded in bundles built by that version's `BuildPipeline`
- Create the `TypeTreeBlob` header that Unity uses to validate the asset's serialized fields against the current runtime's type definitions

Attempting to write a raw `Mesh` binary into an AssetsTools bundle file would produce a file that passes `AssetBundle.LoadFromFile()` (the header is valid) but causes `LoadAsset<Mesh>()` to either return null or throw a `UnityException` — the same symptom as the 90-byte stubs.

**Therefore**: Unity 2021.3.45f1 batchmode is the required and only viable path for producing loadable bundles.

**One narrow exception**: If a bundle was already built by Unity 2021.3 (e.g. the 44KB primitive bundles), AssetsTools.NET *could* in principle overwrite the Mesh bytes with new geometry in-place, since the type tree is already correct for this Unity version. This would be a "bundle patch" approach — copy a working primitive bundle and replace its mesh data. This is feasible for swapping geometry but is fragile (requires exact size matching or raw binary surgery) and would not give a clean solution. It is not recommended.

---

## 5. Current Bundle Inventory: Real vs Stub

### 90-Byte Stubs (12) — Render NOTHING in-game

These must be rebuilt with at least a primitive placeholder geometry, or with a real FBX if one is available.

| Bundle Name | Category | FBX in Models dir? |
|------------|----------|-------------------|
| `sw-guard-tower` | Building (CIS/Rep generic) | No |
| `sw-weapons-factory` | Building | No |
| `sw-heavy-foundry` | Building | No |
| `sw-mining-facility` | Building | No |
| `sw-processing-plant` | Building | No |
| `sw-skyshield-generator` | Building | No |
| `sw-tech-union-lab` | Building | No |
| `sw-tibanna-refinery` | Building | No |
| `sw-vulture-nest` | Building | No |
| `sw-assembly-line` | Building | No |
| `sw-blast-wall` | Building | No |
| `sw-durasteel-barrier` | Building | No |

### Real Mesh Bundles (16, 345 KB–2.5 MB) — Render actual 3D geometry

These are functioning correctly. The large file sizes indicate the FBX geometry is embedded.

| Bundle Name | Size | Source FBX |
|------------|------|-----------|
| `sw-cis-droideka` / `sw-cis-droideka` | 2.5 MB | `cis_droideka.fbx` (3.9 MB) |
| `sw-cis-sniper-droid` / `sw-sniper-droid` | 1.97 MB | `cis_sniper_droid.fbx` (4 MB) |
| `sw-b2-super-droid` / `sw-cis-b2-super-droid` | 1.27 MB | `cis_b2_super_droid.fbx` (2 MB) |
| `sw-rep-clone-medic` / `sw-clone-medic` | 1.19 MB | `rep_clone_medic.fbx` (2.3 MB) |
| `sw-rep-clone-trooper` / `sw-clone-trooper-republic` | 1.18 MB | `sw_clone_trooper_phase2.fbx` |
| `sw-rep-arc-trooper` | 1.1 MB | `rep_arf_trooper.fbx` (2.9 MB) |
| `sw-bx-commando-droid` / `sw-cis-commando-droid` / `sw-commando-droid` | 701 KB | `cis_bx_commando_droid.fbx` (2.6 MB) |
| `sw-clone-barracks` | 346 KB | `rep_clone_barracks` (via engineer FBX) |
| `sw-rep-clone-engineer` | 346 KB | `rep_clone_engineer.fbx` (560 KB) |
| `sw-rep-clone-sniper` | 345 KB | `rep_clone_sharpshooter.fbx` (560 KB) |

### Primitive Placeholder Bundles (48, ~44 KB) — Render colored capsule/cube/sphere shapes

These work (AssetSwap can load and render them) but show Unity primitive geometry, not Star Wars models.

All 48 names in the `GenerateStarWarsPrefabs.cs` definitions table: `sw-rep-clone-*`, `sw-rep-at-*`, `sw-rep-laat-*`, `sw-rep-jedi-fighter`, `sw-rep-*-facility`, `sw-rep-*-tower`, `sw-cis-b1-battle-droid`, `sw-cis-magna-guard`, etc.

Of these, **many have FBX source files** in `unity-assetbundle-builder/Assets/Models/` and `packs/warfare-starwars/assets/source/` that were never promoted to real bundles:

| Bundle | FBX Available in Models dir |
|--------|----------------------------|
| `sw-cis-b1-battle-droid` | `cis_b1_battle_droid.fbx` (211 KB) — YES |
| `sw-rep-clone-heavy` | `rep_clone_heavy.fbx` (56 KB) — YES |
| `sw-rep-clone-engineer` | `rep_clone_engineer.fbx` (560 KB) — YES (already has real bundle) |
| `sw-cis-spider-droid` | `cis_dwarf_spider_droid` — mapped but FBX check needed |
| `sw-cis-stap` | `cis_stap_speeder` — mapped |

---

## 6. Concrete Next Actions: ONE Real Bundle End-to-End Proof

The fastest proof of concept is fixing the 12 stubs via primitive placeholders (no new FBX needed), then promoting one unit to a real mesh bundle. Follow this sequence:

### Action 1: Fix 90-Byte Stubs (Priority — these render NOTHING)

Add the 12 missing bundle IDs to `unity-assetbundle-builder/Assets/Editor/GenerateStarWarsPrefabs.cs` Definitions array. Use `PrimitiveType.Cube` for all building-type entries. Example additions:

```csharp
("sw-guard-tower",          "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
("sw-weapons-factory",      "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
("sw-heavy-foundry",        "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube),
("sw-mining-facility",      "CIS",      CisGrey,       CisDark,       PrimitiveType.Cube),
("sw-processing-plant",     "CIS",      CisGrey,       CisDark,       PrimitiveType.Cube),
("sw-skyshield-generator",  "Republic", RepublicBlue,  RepublicWhite, PrimitiveType.Sphere),
("sw-tech-union-lab",       "CIS",      CisGrey,       CisDark,       PrimitiveType.Cube),
("sw-tibanna-refinery",     "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
("sw-vulture-nest",         "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube),
("sw-assembly-line",        "CIS",      CisDark,       CisGrey,       PrimitiveType.Cube),
("sw-blast-wall",           "Republic", RepublicWhite, RepublicBlue,  PrimitiveType.Cube),
("sw-durasteel-barrier",    "CIS",      CisGrey,       CisDark,       PrimitiveType.Cube),
```

Then run:
```powershell
# Step 1: Generate prefabs
& "C:\Program Files\Unity\Hub\Editor\2021.3.45f1\Editor\Unity.exe" `
  -batchmode -nographics `
  -projectPath "C:\Users\koosh\Dino\unity-assetbundle-builder" `
  -executeMethod GenerateStarWarsPrefabs.Generate `
  -logFile "C:\Users\koosh\Dino\docs\sessions\unity-gen-stubs.log" -quit

# Step 2: Build bundles
& "C:\Program Files\Unity\Hub\Editor\2021.3.45f1\Editor\Unity.exe" `
  -batchmode -nographics `
  -projectPath "C:\Users\koosh\Dino\unity-assetbundle-builder" `
  -executeMethod BuildAssetBundles.BuildHeadless `
  -logFile "C:\Users\koosh\Dino\docs\sessions\unity-build-stubs.log" -quit

# Step 3: Deploy
Copy-Item "C:\Users\koosh\Dino\unity-assetbundle-builder\AssetBundles\*" `
  "C:\Users\koosh\Dino\packs\warfare-starwars\assets\bundles\" -Force

# Step 4: Verify stubs are now real
(Get-Item "C:\Users\koosh\Dino\packs\warfare-starwars\assets\bundles\sw-guard-tower").Length
# Should be ~44000 (not 90)
```

**Expected outcome**: All 12 stub bundles become ~44 KB primitive placeholder bundles. In-game rendering changes from "nothing" to colored cube shapes for those buildings.

### Action 2: Promote `sw-cis-b1-battle-droid` to Real Mesh (Highest-Impact Single Unit)

`cis_b1_battle_droid.fbx` (211 KB) is already in `unity-assetbundle-builder/Assets/Models/`. The `GenerateStarWarsPrefabsFromModels.cs` entry for `sw-cis-b1-battle-droid` correctly maps to this file (`"cis_b1_battle_droid"`). This unit appears in the most entities (B1 droids are the CIS swarm unit).

Run `GenerateStarWarsPrefabsFromModels.Generate` + `BuildAssetBundles.BuildHeadless` to produce a real-mesh bundle for `sw-cis-b1-battle-droid`. This is the single highest-ROI proof: swarm-scale CIS units go from grey capsules to actual B1 droid geometry.

```powershell
& "C:\Program Files\Unity\Hub\Editor\2021.3.45f1\Editor\Unity.exe" `
  -batchmode -nographics `
  -projectPath "C:\Users\koosh\Dino\unity-assetbundle-builder" `
  -executeMethod GenerateStarWarsPrefabsFromModels.Generate `
  -logFile "C:\Users\koosh\Dino\docs\sessions\unity-gen-models.log" -quit

& "C:\Program Files\Unity\Hub\Editor\2021.3.45f1\Editor\Unity.exe" `
  -batchmode -nographics `
  -projectPath "C:\Users\koosh\Dino\unity-assetbundle-builder" `
  -executeMethod BuildAssetBundles.BuildHeadless `
  -logFile "C:\Users\koosh\Dino\docs\sessions\unity-build-models.log" -quit
```

Verify: `sw-cis-b1-battle-droid` should be ~200 KB+ after the build (currently ~44 KB primitive).

### Action 3: Acquire Missing Building FBXs

The 12 stub buildings have no FBX sources anywhere in the repo. To get real geometry:
1. Source from Kenney.nl Modular Buildings pack (CC0) — recommended per `ASSET_PIPELINE.md`
2. Place FBX files in `unity-assetbundle-builder/Assets/Models/` 
3. Add entries to `GenerateStarWarsPrefabsFromModels.Defs[]` with the FBX filename
4. Run the two Unity batchmode steps above

---

## 7. Recommended Approach

**Unity batchmode is the correct and only viable approach.** The tooling is already present:

- Unity 2021.3.45f1 is installed
- The `unity-assetbundle-builder/` project is set up and functional (it produced the 16 real-mesh bundles and 48 primitive bundles already deployed)
- Both editor scripts (`GenerateStarWarsPrefabs.cs`, `GenerateStarWarsPrefabsFromModels.cs`, `BuildAssetBundles.cs`) are correct and functional
- The asset name convention (`go.name = bundleKey`) is correct for AssetSwapSystem's `LoadAsset<GameObject>(assetName)` fallback path

AssetsTools.NET cannot replace Unity batchmode for this use case.

**Immediate priority ordering**:
1. Fix the 12 stubs (Action 1 above) — no new assets needed, pure code edit + 2 batchmode runs
2. Run `GenerateStarWarsPrefabsFromModels` to promote the ~15 units that already have FBX source files from primitive placeholders to real mesh bundles
3. Source building FBXs from Kenney.nl for the stub buildings

**File modified**: `unity-assetbundle-builder/Assets/Editor/GenerateStarWarsPrefabs.cs` (add 12 stub entries)
**Commands**: 2× Unity batchmode invocations + file copy
**Expected result**: 12 → 0 stubs; 48 → 33 primitive placeholders + 15+ real mesh bundles

---

## Key File Paths

| File | Purpose |
|------|---------|
| `C:\Users\koosh\Dino\unity-assetbundle-builder\Assets\Editor\GenerateStarWarsPrefabs.cs` | Primitive placeholder generator — add 12 stub entries here |
| `C:\Users\koosh\Dino\unity-assetbundle-builder\Assets\Editor\GenerateStarWarsPrefabsFromModels.cs` | Real-mesh generator — maps bundle IDs to FBX filenames |
| `C:\Users\koosh\Dino\unity-assetbundle-builder\Assets\Editor\BuildAssetBundles.cs` | Headless bundle builder — invokes `BuildPipeline.BuildAssetBundles()` |
| `C:\Users\koosh\Dino\unity-assetbundle-builder\Assets\Models\` | FBX source files imported into Unity |
| `C:\Users\koosh\Dino\unity-assetbundle-builder\AssetBundles\` | Build output — copy to bundles/ after build |
| `C:\Users\koosh\Dino\packs\warfare-starwars\assets\bundles\` | Deployed bundles loaded by AssetSwapSystem at runtime |
| `C:\Users\koosh\Dino\src\Runtime\Bridge\AssetSwapSystem.cs` | Bundle loader — uses `LoadAsset<GameObject>(assetName)` fallback path |
| `C:\Program Files\Unity\Hub\Editor\2021.3.45f1\Editor\Unity.exe` | Unity editor binary for batchmode builds |
