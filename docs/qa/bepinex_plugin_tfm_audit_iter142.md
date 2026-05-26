# BepInEx Plugin TFM Audit (Iter-142)

**Date**: 2026-05-18  
**Pattern**: #233 (BepInEx Plugin Silent-Load Risk)  
**Scope**: All csproj files producing BepInEx plugins

## Summary
- **Plugins Found**: 2 actual BepInEx plugins
- **Status**: 1 SAFE (multi-target), 1 HIGH RISK (pure net8.0)

## Plugin Classification

| Project | TFM | Status | Risk |
|---------|-----|--------|------|
| `src/Runtime/DINOForge.Runtime.csproj` | netstandard2.0;net8.0 | SAFE | None |
| `src/Tools/MockSteamworksNet/MockSteamworksNet.csproj` | net8.0 | **HIGH RISK** | Silent-load failure |

## Plugin Details

### 1. DINOForge.Runtime.csproj (✅ SAFE)
- **Plugin Class**: `Plugin.cs`, `AviationPlugin.cs` (both in Runtime)
- **TFM**: `<TargetFrameworks>netstandard2.0;net8.0</TargetFrameworks>` (multi-target)
- **Status**: COMPLIANT — follows iter-142 remediation
- **Assessment**: Loads cleanly on .NET 8.0 BepInEx runtime; backward-compat via netstandard2.0

### 2. MockSteamworksPlugin (⚠️ HIGH RISK)
- **Path**: `src/Tools/MockSteamworksNet/MockSteamworksPlugin.cs`
- **Csproj TFM**: `<TargetFramework>net8.0</TargetFramework>` (pure net8.0)
- **Attribute**: `[BepInPlugin(MockSteamworksPluginInfo.GUID, ...)]`
- **Parent Assembly**: `DINOForge.Tools.MockSteamworksNet.csproj`
- **Risk**: Per Pattern #233, BepInEx loader may silently skip loading pure net8.0 assemblies if it's running on a different runtime context
- **Remediation**: Change TFM to `<TargetFrameworks>netstandard2.0;net8.0</TargetFrameworks>` (same as Runtime)

## Non-Plugins Checked
- `DINOForge.SDK.csproj`: netstandard2.0 (not a plugin, library only)
- `DINOForge.Bridge.Client.csproj`: netstandard2.0 (not a plugin, library only)
- `DINOForge.Bridge.Protocol.csproj`: netstandard2.0 (not a plugin, library only)
- Domain plugins (Economy, Scenario, UI, Warfare): all netstandard2.0 (loaded as DLL imports by Runtime, not standalone plugins)

## Remediation Order
1. **IMMEDIATE**: Update `MockSteamworksNet.csproj` TFM to multi-target
2. **VERIFY**: Rebuild both plugins; confirm CI HeadlessCI workflow exercises both

## Cross-Reference
- **Pattern #233**: BepInEx Plugin Silent-Load Risk — single-TFM plugin DLLs may fail to load silently if runtime context mismatches
- **Iter-142 Fix**: DINOForge.Runtime already corrected to multi-target; audit ensures no other plugins left behind
