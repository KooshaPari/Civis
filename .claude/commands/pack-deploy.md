# pack-deploy

Build, validate, and deploy a content pack + Runtime DLL to the game installation.

**Usage**: `/pack-deploy [<pack-id>] [--dll-only] [--pack-only] [--watch]`

**Arguments**: $ARGUMENTS

## Purpose

One-command deploy for iterative development. Validates the pack, builds the Runtime DLL, copies everything to the game, and optionally watches the debug log for results.

## Steps

1. **Validate pack** (skip if `--dll-only`):
   ```bash
   dotnet run --project src/Tools/PackCompiler -- validate packs/<pack-id>/
   ```
   Abort if validation fails.

2. **Build Runtime DLL + deploy**:
   ```bash
   dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true
   ```
   The `DeployToGame=true` flag copies:
   - `DINOForge.Runtime.dll` → `BepInEx/plugins/`
   - Pack files → `BepInEx/dinoforge_packs/<pack-id>/`

3. **Verify deployment**:
   - Confirm DLL timestamp is fresh: `ls -la "G:\...\BepInEx\plugins\DINOForge.Runtime.dll"`
   - Confirm pack files exist: `ls "G:\...\BepInEx\dinoforge_packs\<pack-id>/"`
   - Count bundle files deployed

4. **If `--watch`**:
   - Remind user to restart the game (or use `/launch-game`)
   - Tail `dinoforge_debug.log` for 60 seconds
   - Report first `batch complete` result

## Quick Pack-Only Deploy

If you only changed pack YAML files (no C# changes), use `--pack-only` to skip the DLL build:
```bash
# Just copy updated pack files
cp -r packs/<pack-id>/ "G:\SteamLibrary\...\BepInEx\dinoforge_packs\"
```

## Deployment Paths

| Artifact | Source | Destination |
|----------|--------|-------------|
| Runtime DLL | `src/Runtime/bin/Release/` | `BepInEx/plugins/DINOForge.Runtime.dll` |
| Pack files | `packs/<pack-id>/` | `BepInEx/dinoforge_packs/<pack-id>/` |
| Bundle files | `packs/<pack-id>/assets/bundles/` | `BepInEx/dinoforge_packs/<pack-id>/assets/bundles/` |
