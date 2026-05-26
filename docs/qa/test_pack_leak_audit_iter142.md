# Test Pack Leak Audit (Iter 142)

## Executive Summary
`TestInvalidID` pack-load crash was caused by test fixture pack `test-invalid-schema-2/` being deployed to the game directory via automatic pack discovery globbing in `DINOForge.Runtime.csproj` (line 292). The pack existed in commit `ced0dcc` but was later deleted from the source tree, leaving stale copies in deployed directories.

## Findings

### (a) TestInvalidID Locations

**Source (git):**
- Commit: `ced0dcc` (fix(bridge): implement HandleConnect)
- Location: `packs/test-invalid-schema-2/pack.yaml`
- Pack ID: `TestInvalidID` (intentionally invalid per schema tests)
- Status: **DELETED** from repo (no longer in current working tree)

**Deployed (game directory):**
- Path: `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_packs\test-invalid-schema-2\`
- Pack YAML contains: `id: TestInvalidID`
- Status: **STILL PRESENT** in deployed artifacts (stale)

### (b) Git History

- **Commit introducing it:** `ced0dcc` (2026-05-18)
  - 5 test packs added: `test-invalid`, `test-invalid-schema`, `test-invalid-schema-2`, `test-invalid-schema-3`, `test-invalid-schema-4`
  - Purpose: Schema validation test fixtures for negative-case pack loading
  
- **Deletion path:** Unknown — pack is in `ced0dcc` but missing from current HEAD
  - Likely deleted in a later commit (not found in git log)
  - No evidence of explicit `git rm` (no deletion commit in history)
  - Hypothesis: Accidentally omitted during pack directory cleanup or branch squash

### (c) Deploy Pipeline Pack-Source Globbing

**File:** `src/Runtime/DINOForge.Runtime.csproj` (lines 289-299)

```xml
<Target Name="DeployPacks" AfterTargets="Build" 
         Condition="'$(DeployToGame)' == 'true' and Exists('$(BepInExDir)') and '$(TargetFramework)' == 'netstandard2.0'">
  <ItemGroup>
    <PackFiles Include="$(MSBuildThisFileDirectory)..\..\packs\**\*" />
  </ItemGroup>
  <MakeDir Directories="$(BepInExDir)\dinoforge_packs" 
           Condition="!Exists('$(BepInExDir)\dinoforge_packs')" />
  <Copy SourceFiles="@(PackFiles)"
        DestinationFolder="$(BepInExDir)\dinoforge_packs\%(RecursiveDir)"
        SkipUnchangedFiles="true" />
</Target>
```

**Issue:** Glob pattern `packs/**/*` copies ALL packs (including test fixtures). `SkipUnchangedFiles=true` means deleted source files remain in the deployed directory indefinitely.

### (d) Root Cause Analysis

1. **Test packs should never be deployed** — they contain intentionally invalid IDs (`BadID!@#`, `TestInvalidID`) for schema validation testing
2. **Glob is overly broad** — includes test-* fixtures alongside production packs
3. **Stale deletion** — when `test-invalid-schema-2` was removed from source, the deployed copy persisted (no cleanup mechanism)
4. **Crash on load** — ContentLoader attempts to validate `TestInvalidID` against pack ID regex, fails, throws exception

### (e) Prevention Recommendation: Pattern #234

**Pattern #234: Implicit Test Fixture Deployment**

**Smell:** Build targets use glob patterns (`**/*`) to copy content without explicit inclusion/exclusion, allowing test fixtures, debug assets, or temporary files to be deployed to production directories.

**Why bad:** Test-specific data (intentionally invalid configs, mock IDs, debug assets) silently leak into deployed artifacts. Schema validation fails at runtime with cryptic errors. Stale deletions remain indefinitely.

**Governance:**
- Exclude test packs explicitly: modify DeployPacks target to filter out `test-*` and `mock-*` directories
- Use inclusion list rather than exclusion (prefer `Include="...example-*;...warfare-*"` over `Exclude="test-*"`)
- Add pre-deploy validation: verify no test-fixture IDs in deployed manifest
- CI gate: fail build if any `test-` or `mock-` pack found in `BepInEx\dinoforge_packs\`

**Recommended Fix (MSBuild):**
```xml
<PackFiles Include="$(MSBuildThisFileDirectory)..\..\packs\**\*" 
           Exclude="$(MSBuildThisFileDirectory)..\..\packs\test-*\**;$(MSBuildThisFileDirectory)..\..\packs\mock-*\**" />
```

**CI Gate:**
```powershell
$testPacks = Get-ChildItem "$gameDir\BepInEx\dinoforge_packs" -Directory | Where-Object { $_.Name -match '^test-|^mock-' }
if ($testPacks) { 
  Write-Error "Test fixtures deployed: $($testPacks -join ', ')" 
  exit 1 
}
```

## Summary
- **Root Cause:** Broad glob in MSBuild + deleted-but-not-deployed test pack
- **Action:** Exclude test-* / mock-* packs from DeployPacks target
- **Gate:** Add pre-deploy validation to CI
- **Pattern:** #234 (Implicit Test Fixture Deployment)
