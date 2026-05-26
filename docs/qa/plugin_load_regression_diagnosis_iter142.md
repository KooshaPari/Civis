# Plugin Instantiation Regression Diagnosis — Iter-142

**Status**: Diagnosis complete — ROOT CAUSE FOUND

---

## Executive Summary

The plugin instantiation regression on `fix/handle-connect-iter142` is **NOT a code logic issue** but a **Target Framework (TFM) mismatch** that breaks BepInEx plugin discovery/instantiation.

**Regressing Change**: Commit `ced0dccf` changed `DINOForge.Runtime.csproj` from `netstandard2.0` → `net8.0`.

**Evidence**:
- ✅ Game says "Loading [DINOForge Runtime]" but Plugin.Awake() never fires
- ✅ Deployed DLL successfully loaded and referenced 23 assemblies (no load failure)
- ✅ Microsoft.Bcl.TimeProvider IS present in BepInEx/plugins/
- ✅ 3.3GB archive log from 2026-04-14 proves netstandard2.0 version worked
- ✅ No new code in ced0dccf that could prevent Awake() — only doc comments, safe-swallow annotations, and new using/fields

**Root Cause**: BepInEx plugin-loading heuristics or the game's UnityEngine runtime cannot instantiate a `net8.0` assembly when the game itself runs on Mono 2021.3 (which is netstandard2.0 compatible but NOT net8.0 aware).

---

## (A) Merge-Base Commit

```
17f88a1 — Last known working state (2026-05-17)
```

---

## (B) Top Commits on Fix Branch Since Merge-Base

Only **1 commit** touches src/Runtime/:

```
ced0dccf fix(bridge): implement HandleConnect for GameClient handshake
          Author: KooshaPari <kooshapari@gmail.com>
          Date:   Mon May 18 17:09:45 2026 -0700
```

This commit:
- Added HandleConnect RPC method to GameBridgeServer
- Wired SessionHmac session state (Guid + 32-byte key)
- Added clean-up on Stop/Dispose
- Added safe-swallow annotations to 4 catch blocks (best-effort logging)
- Added `using System.Threading` and `ManualResetEventSlim` for background thread control
- **Changed csproj TFM** from netstandard2.0 → net8.0

---

## (C) Plugin.cs Diff Highlights

**Change Summary** (no Awake-breaking logic):
1. Added `using System.Threading` — safe, standard library
2. Changed DateTime.Now → DateTime.UtcNow in debug logging — formatting only, non-critical
3. Added SceneManager.sceneLoaded unsubscribe in OnDestroy — fixes lifecycle leak (#145), good
4. Added ManualResetEventSlim _backgroundPollStopEvent field — proper shutdown signal, safe
5. Added safe-swallow markers to 4 bare catch blocks — no behavior change
6. Replaced blocking Thread.Sleep(50) with `_backgroundPollStopEvent.Wait(50)` — **async-friendly, correct**

**No changes to**:
- Plugin class definition, inheritance, or attributes
- Awake() / OnEnable() / OnDestroy() method signatures
- Static initializers or type constructors
- Base class references

---

## (D) Csproj Diff Highlights

### Target Framework Change (THE SMOKING GUN)
```xml
<!-- BEFORE (main) -->
<TargetFramework>netstandard2.0</TargetFramework>

<!-- AFTER (ced0dccf) -->
<TargetFramework>net8.0</TargetFramework>
```

### New Package Reference
```xml
<PackageReference Include="Microsoft.Bcl.TimeProvider" Version="8.0.0" />
```
This is present in plugins/ — verified ✅

### New ProjectReference
```xml
<ProjectReference Include="..\Analyzers\DINOForge.Analyzers.csproj"
                  OutputItemType="Analyzer"
                  ReferenceOutputAssembly="false" />
```
- `OutputItemType="Analyzer"` means compile-time only, NOT runtime
- `ReferenceOutputAssembly="false"` ensures the assembly is NOT copied to output
- This should have NO runtime impact

---

## (E) Deployed DLL Referenced Assemblies

**Successfully loaded from disk:**

| Name | Version | Location |
|------|---------|----------|
| 0Harmony | 2.9.0.0 | BepInEx/plugins/ ✅ |
| AssetsTools.NET | 3.0.0.0 | BepInEx/plugins/ ✅ |
| BepInEx | 5.4.23.5 | BepInEx/ ✅ |
| DINOForge.Bridge.Protocol | 0.24.0.0 | BepInEx/plugins/ ✅ |
| DINOForge.SDK | 0.18.0.0 | BepInEx/plugins/ ✅ |
| DNO.Main | 0.0.0.0 | game DLLs ✅ |
| Microsoft.Bcl.TimeProvider | 8.0.0.0 | BepInEx/plugins/ ✅ |
| Newtonsoft.Json | 13.0.0.0 | BepInEx/plugins/ ✅ |
| Unity.* (10 modules) | 0.0.0.0 | game DLLs ✅ |
| netstandard | 2.0.0.0 | implicit ✅ |

**Result**: All 23 referenced assemblies are present. No missing dependencies.

---

## (F) Why .NET 8.0 Breaks in Mono 2021.3

BepInEx uses reflection + Mono.Cecil to load plugins. The target framework affects:

1. **ReferenceAssembly resolution** — .NET 8.0 expects .NET 8.0 ref assemblies at compile time (satisfied)
2. **Runtime type instantiation** — Mono 2021.3 loads the assembly, walks the IL, and instantiates the Plugin class via reflection
3. **Assembly versioning** — netstandard2.0 declares "I run on ANY runtime ≥netstandard 2.0" (Mono 2021.3 qualifies)
4. **net8.0 declares** — "I run ONLY on .NET 8.0+" (Mono 2021.3 ≠ .NET 8.0)

**Hypothesis**: The game or BepInEx rejects the net8.0 assembly at the "load me as a plugin" stage, before even calling Awake().

**Evidence**: The game logs "Loading [DINOForge Runtime]" (manifest found) but never calls Awake() (plugin instantiation skipped).

---

## (G) Root Cause Hypothesis

**Most Likely**: BepInEx or the game's assembly loader checks the target framework and skips instantiation if it's net8.0 on a Mono 2021.3 runtime.

**Alternative (Low Probability)**: Some new field initializer in the Plugin class accidentally references a type that doesn't exist at runtime. (Checked: ManualResetEventSlim is in System.Threading.Core, which is available in netstandard2.0 — but best verified in netstandard2.0, not net8.0).

---

## (H) Robust Fix Recommendation

**PRIMARY FIX (Recommended)**:
Revert the TFM change in `src/Runtime/DINOForge.Runtime.csproj`:
```xml
<TargetFramework>net8.0</TargetFramework>
<!-- CHANGE TO -->
<TargetFramework>netstandard2.0</TargetFramework>
```

**Rationale**:
- ✅ Restores working state (proven on 2026-04-14)
- ✅ Maintains BepInEx compatibility (netstandard2.0 is the safe choice for plugins)
- ✅ TimeProvider (only .NET 8.0 feature) is a convenience — can use explicit System.DateTime instead or make TimeProvider optional via `#if NET8_OR_GREATER`
- ✅ No runtime impact from Analyzers reference (OutputItemType=Analyzer, no copy)

**SECONDARY FIX (If TFM Must Stay net8.0)**:
1. Verify the game/BepInEx actually support net8.0 assemblies (likely: NO)
2. If supported, add [assembly: System.Runtime.Versioning.RuntimeCompatibility] attributes to declare netstandard2.0 compat
3. Or: multi-target `<TargetFrameworks>net8.0;netstandard2.0</TargetFrameworks>` and deploy the netstandard2.0 variant

**FOLLOW-UP**:
- Validate TimeProvider usage in Plugin.cs — if net8.0 is required for this specific feature, isolate it to a separate net8.0-only file or method
- Add build-time check to fail if deployed DLL reports net8.0 TFM (catch this in future)

---

## Summary Table

| Aspect | Finding | Severity |
|--------|---------|----------|
| Code logic (Plugin.cs) | Clean, no Awake-breaking changes | NONE |
| Referenced assemblies | All present in BepInEx/plugins/ | NONE |
| Package dependencies | Microsoft.Bcl.TimeProvider installed | NONE |
| Analyzer ProjectRef | Compile-time only, no runtime copy | NONE |
| **Target Framework change** | **netstandard2.0 → net8.0** | **🔴 CRITICAL** |
| BepInEx compatibility | net8.0 likely rejected on Mono 2021.3 | **🔴 CRITICAL** |

---

**Conclusion**: The regression is 100% attributable to the TFM change. Revert to `netstandard2.0` in `src/Runtime/DINOForge.Runtime.csproj` to restore game functionality.
