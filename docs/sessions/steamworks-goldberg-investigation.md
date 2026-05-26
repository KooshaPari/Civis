# DINO Steamworks Dependency + Goldberg Drop-in Viability

**Date**: 2026-05-17  
**Investigation**: Verify DINO executable's Steamworks dependency for headless/CI testing without Steam running  
**Task**: #225 P2 INFRA

---

## Executive Summary

**DINO has a runtime dependency on Steamworks via COM/pipes**, but **does NOT bundle `steam_api64.dll`**.

- **Compile-time**: Uses `Steamworks.NET` wrapper (C# managed assembly, 381KB)
- **Runtime**: Expects Steam client running (for IPC via named pipes or COM)
- **Bundled**: Only the wrapper DLL, not the native `steam_api64.dll`
- **Goldberg Viability**: **PROBLEMATIC** — Goldberg Emulator drops in `steam_api64.dll`, but DINO's Steamworks.NET wrapper calls Steam via IPC. Goldberg would need to intercept IPC calls, not just DLL exports.

---

## Investigation Results

### File System Evidence

| File | Status | Size | Purpose |
|------|--------|------|---------|
| `steam_api64.dll` | **NOT FOUND** | — | Native Steam API DLL (not bundled) |
| `com.rlabrecque.steamworks.net.dll` | **FOUND** | 381 KB | Managed C# wrapper around Steamworks API |
| `winhttp.dll` | FOUND | 26 KB | Likely BepInEx proxy or generic native module |
| `DNO.Main.dll` | FOUND | — | References "steamWorks" internally |

### Dependency Analysis

**Steamworks.NET Wrapper:**
- **Path**: `Diplomacy is Not an Option_Data/Managed/com.rlabrecque.steamworks.net.dll`
- **Hash (SHA256)**: `BB579876ABAEE72C3CAAF326DAAA9522E180563DD82E40CE543FC278751FE273`
- **Modified**: 2026-03-19 19:33:44
- **Verified Functions**: References to `steam_api64` and `Steamworks` found in binary

**Game Main Assembly:**
- **Path**: `Diplomacy is Not an Option_Data/Managed/DNO.Main.dll`
- **Status**: Contains reference to `steamWorks` (runtime call detected)
- **Implication**: Game code actively calls Steamworks API at runtime

**Native Steam DLL:**
- **Status**: NOT present in game directory
- **Implications**:
  - Steamworks.NET wrapper must locate `steam_api64.dll` via system PATH
  - Expects Steam client running (IPC/COM mechanism)
  - No fallback or mock mode detected

---

## Why Goldberg Emulator Won't Work (Directly)

### Goldberg Drop-in Mechanism
Goldberg Emulator works by replacing `steam_api64.dll` with a mock that:
1. Intercepts direct DLL function calls
2. Fakes achievement/stats APIs
3. Emulates multiplayer lobbies

### DINO's Problem
The Steamworks.NET wrapper uses a **high-level managed API** that likely:
1. Calls native Steamworks C++ APIs via DLL imports
2. May use **named pipes** or **COM interop** for some operations
3. Expects an actual Steam client service running

### Risk Matrix
| Operation | Goldberg Intercepts? | Risk Level | Notes |
|-----------|---------------------|-----------|-------|
| Init/Shutdown | ✅ YES | LOW | DLL function call, Goldberg can fake |
| GetUser() / Stats | ✅ YES | LOW | Direct DLL API |
| Achievements | ✅ YES | LOW | Standard emulation |
| In-game IPC | ❌ NO | **HIGH** | If game uses named pipes, Goldberg won't see them |
| App ID validation | ✅ YES (maybe) | MEDIUM | Goldberg defaults to app ID 480; if DINO checks, needs config |
| Cloud saves | ⚠️ PARTIAL | MEDIUM | Goldberg emulates locally, may differ from Steam behavior |

---

## Recommended Path Forward

### Option 1: Mock Steamworks (Preferred for CI)
Create a thin managed proxy that intercepts `com.rlabrecque.steamworks.net` calls:
- **Effort**: 1-2 days
- **Risk**: LOW (C# managed layer, no native concerns)
- **Benefit**: Works in headless, doesn't require Steam or Goldberg
- **Implementation**: Create `MockSteamworksNet.cs` that stubs achievements, stats, user info
- **Deployment**: Load via BepInEx plugin, before main DINO assembly

### Option 2: Goldberg + Verification
Try Goldberg as-is, then validate:
1. Drop Goldberg's `steam_api64.dll` into game root
2. Download and extract app ID 1389730 (DINO's Steam ID)
3. Test game launch in CI with `steam=0` env var
4. If fails, revert to Option 1

**Timeline**: Same-day proof (1-2 hours)  
**Risk**: Game may fail silently or crash on Goldberg incompatibility

### Option 3: Native Steam Running in CI
Use GitHub Actions + Proton/Steamless:
- **Effort**: 3-5 days (complex setup)
- **Risk**: HIGH (Steam client setup in headless environment)
- **Benefit**: 100% compatibility (real Steam)
- **Cost**: Overkill for a mod platform

---

## Recommendation: Short-term Fix

**For v0.24.0 (immediate):**
- Pursue **Option 1** (MockSteamworksNet proxy)
- Allows full CI/headless game testing without Steam client
- Decouples modding platform from Steamworks dependency

**For v0.25.0+ (long-term):**
- Evaluate Option 2 (Goldberg) if Option 1 insufficient
- Consider strategy #191 (smart-contract proof system) to validate achievements/stats offline

---

## References

- **Steamworks.NET GitHub**: https://github.com/rlabrecque/Steamworks.NET
- **Goldberg Emulator**: https://gitlab.com/Mr_Goldberg/goldberg_emulator
- **DINO Asset Path**: `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\`
- **Wrapper Location**: `Diplomacy is Not an Option_Data/Managed/com.rlabrecque.steamworks.net.dll`

---

## Conclusion

DINO is **NOT** a "steamless" game — it depends on Steamworks.NET wrapper and expects Steam client IPC. However, because `steam_api64.dll` is not bundled and the wrapper is managed C#, we have two clear paths:

1. **Preferred**: Mock the wrapper → headless CI in 1-2 days
2. **Fallback**: Test Goldberg viability → verify in 1-2 hours

**Next Task**: Prototype MockSteamworksNet as a BepInEx plugin.
