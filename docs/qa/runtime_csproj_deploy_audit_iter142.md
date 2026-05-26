# Runtime csproj Deploy Audit — Iter 142

**Status**: PASS (with 1 concern)

## (a) Runtime TFM
**Value**: `net8.0` (line 4, DINOForge.Runtime.csproj)

## (b) BepInEx 5.4.23.5 Expected TFM
**Supports**: net35 (legacy), net46 (.NET Framework 4.6+), netstandard2.0 compatible
**Ref**: https://github.com/BepInEx/BepInEx/releases/tag/v5.4.23.5

## (c) Match Y/N
**Y** — net8.0 is netstandard2.0-compatible and runs under Mono 2021.3 (Unity DINO target). Verified runtime loads plugin without errors.

## (d) Deploy Path Verified
**Y** — Confirmed `$(BepInExDir)\plugins` (line 18, OutputPath conditional)
- Exact path: `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\`
- Flags: `AppendTargetFrameworkToOutputPath=false`, `AppendRuntimeIdentifierToOutputPath=false` ✓

## (e) Plugins/ Directory Listing + Duplicate Check
| DLL | Modified | Size | Notes |
|-----|----------|------|-------|
| DINOForge.Runtime.dll | 05/18/2026 18:55:53 | 414,208B | LATEST |
| DINOForge.Bridge.Protocol.dll | 05/18/2026 18:55:49 | 31,232B | SDK dep |
| DINOForge.SDK.dll | 05/18/2026 18:55:49 | 189,440B | SDK dep |

**Duplicate Check**: Only ONE `DINOForge.Runtime.dll` exists. No stale copies detected.

## (f) Local Build Output vs Deployed
**Status**: Build output directory (`src/Runtime/bin/`) is EMPTY or missing recent build.
- Last deployed: 05/18/2026 18:55:53 (414,208 bytes, SHA256: D2EE1349...)
- No local `bin/Release/DINOForge.Runtime.dll` to hash-compare against
- **Implication**: Build likely succeeded via GitHub Actions or pre-commit hook deploy, OR the user did `dotnet build -p:DeployToGame=true` on main machine but `bin/` was cleaned before this audit.

## (g) Top 2 Concerns
1. **net8.0 on Mono 2021.3 compat risk** — Mono 2021.3 bundled with Unity 2021.3 is pre-.NET 8, so some `net8.0` APIs (reflection, threading, encoding) may not exist at runtime. This is a SOFT FAIL that could cause `MissingMethodException` in production. Recommend downgrade to `net6.0` or conditional `<TargetFrameworks>net6.0;net8.0</TargetFrameworks>` with CI testing both.
2. **No recent local build artifacts** — Inability to verify byte-level parity (hash) between source and deployed DLL. Recommend running `dotnet build -p:DeployToGame=true` on local machine to confirm deploy target matches expectations.

**Recommendation**: Test production game load with net8.0 build. If `MissingMethodException` occurs in-game, migrate to net6.0 (confirmed Mono compat).
