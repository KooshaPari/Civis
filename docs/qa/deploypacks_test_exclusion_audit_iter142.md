# DeployPacks MSBuild Target: Test Pack Exclusion Audit

**Date**: 2026-05-18  
**Iteration**: 142 (incident recovery)  
**Pattern**: #234 — Test Fixture IDs Leaking Into Deployed Packs

---

## Findings

### (a) Deploy Target Location
- **File**: `C:\Users\koosh\Dino\src\Runtime\DINOForge.Runtime.csproj`
- **Lines**: 290-299
- **Target Name**: `DeployPacks`
- **Trigger**: `AfterTargets="Build"` when `DeployToGame == 'true'` and `TargetFramework == 'netstandard2.0'`

### (b) Current Pack Source Glob
```xml
<PackFiles Include="$(MSBuildThisFileDirectory)..\..\packs\**\*" />
```
**Resolves to**: `C:\Users\koosh\Dino\packs\**\*` (all files, all subdirectories)

**Status**: NO EXCLUSION. Test packs deploy to game directory.

### (c) Existing Test Pack Exclusion
**Current**: NONE (NO clause, NO condition, no `Exclude` attribute)

**Risk**: All 4 test packs in scope currently deploy to `BepInEx\dinoforge_packs\`:
- `packs\test-valid\pack.yaml`
- `packs\test-bad-version\pack.yaml`
- `packs\test-invalid-schema\pack.yaml`
- `packs\test-invalid-schema-4\pack.yaml`

Pattern #234 governance mandates: **Test fixtures MUST live in `src/Tests/Fixtures/` not `packs/`.**

### (d) Recommended Fix

**Option A** (preferred): Add `Exclude` to glob — minimally intrusive, keeps test packs in `packs/test-*` for visibility.

```xml
<PackFiles Include="$(MSBuildThisFileDirectory)..\..\packs\**\*" 
           Exclude="$(MSBuildThisFileDirectory)..\..\packs\test-*\**\*" />
```

**Option B** (long-term): Migrate test packs to `src/Tests/Fixtures/packs/` per Pattern #234 governance. Removes them from repo `packs/` visibility entirely. Requires:
1. Move `packs/test-*/*` → `src/Tests/Fixtures/packs/test-*/*`
2. Update ContentLoader tests to load from `src/Tests/Fixtures/packs/`
3. No MSBuild change needed (different source tree, no glob conflict)

### (e) Cross-Link
- **Pattern #234**: Test Fixture IDs Leaking Into Deployed Packs  
  - Governance line: _"Test pack fixtures live in `src/Tests/Fixtures/` (excluded from DeployPacks MSBuild target)."_
  - Root cause: Iter-142 incident where `TestInvalidID` in `test-invalid-schema.pack.yaml` deployed and crashed game with duplicate-key exception.

---

## Recommendation

**Apply Option A immediately** (1-line change, low risk):
- Adds `Exclude="packs\test-*\**\*"` to `<PackFiles>` glob
- Prevents test packs from deploying to game without relocating test fixtures
- Option B (migration to `src/Tests/Fixtures/`) can follow in a separate refactor session

---

## Verification

Post-fix, verify:
```powershell
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true
# Check BepInEx\dinoforge_packs: only production packs, no test-* entries
ls "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_packs"
```

Expected: 3 production packs (`example-balance`, `warfare-modern`, `warfare-starwars`); 0 test packs.
