# Steamworks.NET TFM Audit (Iter 142)

## Package Analysis
- **Current**: `Steamworks.NET` v15.0.* (via NuGet)
- **Target TFM**: netstandard2.1 + net8.0
- **Issue**: v15.0+ targets netstandard2.1 only; netstandard2.0 not supported

## Mocked Types & Methods
Plugin uses Harmony reflection patching to intercept:
1. `SteamAPI.Init()` → returns `true`
2. `SteamAPI.IsSteamRunning()` → returns `true`
3. `SteamUser.GetSteamID()` → returns mock `CSteamID(76561197960265728UL)`
4. `SteamApps.BIsSubscribedApp(uint)` → returns `true`
5. `SteamFriends.GetPersonaName()` → returns `"MockUser"`

## Runtime Type Requirements
**YES** — Code requires concrete Steamworks.NET types:
- `typeof(SteamAPI)`, `typeof(SteamUser)`, `typeof(SteamApps)`, `typeof(SteamFriends)` lookups via reflection
- `CSteamID` constructor call (line 142)
- Cannot be replaced with interface stubs or reference-only assembly

## Steamworks.NET Version History
- **v14.x & earlier**: netstandard2.0 ✓ supported
- **v15.0+ (2024)**: netstandard2.1 only (dropped netstandard2.0)
- Release notes confirm deliberate removal of netstandard2.0 TFM

## Recommendation
**(a) Downgrade to Steamworks.NET v14.x** — Mocked API surface is stable across v14/v15; netstandard2.0 compatibility maintained; minimal risk.

**Rationale**: Harmony patches rely on public static methods (Init, IsSteamRunning, etc.) which haven't changed between v14/v15. Downgrade is lowest-friction path to restore CI headless support.
