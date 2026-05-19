# Tier 1 Deploy Target Spec Verification — Iter 142

**Date**: 2026-05-18  
**Verdict**: **SPEC ACCURATE**  
**Confidence**: HIGH

## Property Verification

| Property | Status | Location | Finding |
|----------|--------|----------|---------|
| `$(GameInstalled)` | ✓ | DINOForge.Runtime.csproj:11-12 | Conditional detection: `Exists('$(ManagedDir)\UnityEngine.dll')` → true/false |
| `$(DeployToGame)` | ✓ | DINOForge.Runtime.csproj:13 | Defaults to `false`; can be overridden via `-p:DeployToGame=true` |
| `$(BepInExDir)` | ✓ | DINOForge.Runtime.csproj:18 (used) | Set via Directory.Build.props (external); referenced in output redirect & file ops |
| `AfterTargets="Build"` | ✓ | DINOForge.Runtime.csproj:247, 267, 290 | Three deploy targets use this pattern (DeployUiAssets, DeployRustAssetPipelineDll, DeployPacks) |
| `SkipUnchangedFiles="true"` | ✓ | DINOForge.Runtime.csproj:256, 278, 297 | Present in all Copy tasks (idempotent behavior) |
| `$(TargetPath)` | ✓ | DINOForge.Runtime.csproj:79 (spec assumes it) | MSBuild standard; resolves to built `MockSteamworksNet.dll` |
| `$(ManagedDir)` | ✓ | DINOForge.Runtime.csproj:11, 61+ | Set via Directory.Build.props (game-install detection) |

## Target Syntax Consistency

**Spec snippet vs. actual patterns**:
- ✓ Condition syntax matches Runtime patterns (lines 247, 267, 290)
- ✓ MakeDir usage consistent with line 252-253, 294
- ✓ Copy usage matches lines 254-257, 295-297
- ✓ Message importance="high" matches line 258, 298
- ✓ Property defaults (DeployMockSteamworks=false) match DINOForge.Runtime.csproj:13

## Divergences

**None detected.** The spec adheres to established patterns in DINOForge.Runtime.csproj.

## Notes

- MockSteamworksNet.csproj already detects `$(GameInstalled)` (lines 14-16)
- No output redirection needed; MockSteamworksNet.csproj has no custom OutputPath override
- Copy destination in spec (`$(BepInExDir)\plugins`) is identical to Runtime plugin output (line 18)

## Conclusion

The proposed `DeployMockSteamworksNet` target XML can be dropped into MockSteamworksNet.csproj without modification. All cited MSBuild properties and syntax patterns are **verified to exist and match actual usage** in the codebase.
