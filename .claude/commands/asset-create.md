# asset-create

Create a Unity AssetBundle for a DINOForge mod asset from a GLB/FBX source file.

**Usage**: `/asset-create <asset-id> <source-file> [--pack <pack-id>] [--faction <republic|cis|neutral>]`

**Arguments**: $ARGUMENTS

## Purpose

Guides the full pipeline from a raw 3D file (GLB/FBX) to a deployable Unity AssetBundle:
normalize geometry → apply faction palette → build bundle → register in pack definition.

## Steps

1. **Validate inputs**:
   - Confirm `<source-file>` exists (GLB/FBX)
   - Confirm `<pack-id>` pack exists in `packs/`
   - Determine faction palette from `--faction` or infer from `<asset-id>` prefix (`sw-rep-*` → republic, `sw-cis-*` → cis)

2. **Normalize** (Blender headless):
   ```bash
   blender --background --python scripts/blender/normalize_asset.py -- \
     <source-file> packs/<pack-id>/assets/working/<asset-id>/ 3000
   ```
   Confirm `normalized.glb` was created. Read `normalization_report.json` for polycount.

3. **Stylize** (Blender headless):
   ```bash
   dotnet run --project src/Tools/Cli -- assetctl stylize <asset-id> \
     --faction <faction> --pipeline-root packs/<pack-id>/assets
   ```
   Confirm `stylized.glb` was created.

4. **Build AssetBundle** (requires Unity Editor on PATH as `Unity`):
   - Check if `Unity` is available on PATH
   - If available: run the AssetBundle build script
   - If NOT available: output the manual Unity steps needed:
     ```
     1. Import packs/<pack-id>/assets/working/<asset-id>/stylized.glb into a Unity 2021.3 project
     2. Mark as Addressable with key = <asset-id>
     3. Build AssetBundles → copy output to packs/<pack-id>/assets/bundles/<asset-id>
     ```
   - Place bundle at: `packs/<pack-id>/assets/bundles/<asset-id>`

5. **Register in pack definition**:
   - Find the unit/building YAML that should reference this asset
   - Set `visual_asset: <asset-id>` in the definition
   - Run pack validation: `dotnet run --project src/Tools/PackCompiler -- validate packs/<pack-id>/`

6. **Deploy** (copies pack to game):
   ```bash
   dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true
   ```

7. Report bundle path, polycount stats, and next step (run `/test-swap` to verify).

## Notes

- Unity 2021.3.45f2 is required for bundle compatibility with DINO
- Bundles built with other Unity versions will fail to load at runtime
- The `visual_asset` key must match the bundle filename (without extension)
