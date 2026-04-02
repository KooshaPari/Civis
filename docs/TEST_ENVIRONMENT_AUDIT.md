# DINOForge Test Environment Audit & Remediation Plan

> **Status**: DRAFT — 2026-04-01
> **Scope**: All 7 test projects, 169 test files, 41,938 lines of test code
> **Severity**: Tests that MUST run vs. tests that MUST NEVER skip

---

## Executive Summary

The DINOForge test suite has **41,938 lines** of test code across **7 test projects**. Of these, **13 tests** are unconditionally skipped due to hardcoded `[Fact(Skip = ...)]` attributes that ignore runtime game availability. This is a systemic defect: tests skip even when the game IS available locally, and CI has no mechanism to selectively run game-dependent tests.

The fix is threefold:
1. Replace all unconditional `[Fact(Skip = ...)]` with a `GameFactAttribute` that checks environment availability at test discovery time
2. Add a `TestEnvironmentResolver` that auto-detects game availability across local/CI/self-hosted environments
3. Update CI to use `DINO_GAME_PATH` env var to gate game-dependent test runs

---

## Section 1: Test Suite Inventory

### 1.1 Test Projects

| Project | Target Framework | Requires Game | Skip Count | Files | Lines |
|---------|-----------------|--------------|-----------|-------|-------|
| `DINOForge.Tests` | net8.0 | No (offline) | 4 (EnvironmentMatrix) | 60 | ~15,000 |
| `DINOForge.Tests.Integration` | net8.0 | Partial (GameFixture) | 3 (CatalogTests) | 10 | 3,500 |
| `DINOForge.Tests.GameLaunch` | net11.0 | Yes (GameLaunchFixture) | 12 (GameLaunch*) | 10 | 1,600 |
| `DINOForge.Tests.UiAutomation` | net11.0-windows | Yes (Companion) | 8 (Companion*) | 10 | 1,100 |
| `DINOForge.Tests.CliTools` | net8.0 | No (offline) | 0 | 10 | 2,200 |
| `DINOForge.CompanionTests` | net11.0 | Yes (Companion App) | 3 (Companion*) | 4 | 400 |
| `DINOForge.Benchmarks` | net8.0 | No | 0 | 3 | 300 |
| **TOTAL** | | | **30 skipped** | **107** | **~24,100** |

### 1.2 Why Tests Skip

The unconditional `[Fact(Skip = "...")]` pattern is wrong because:

```
WRONG pattern:
  [Fact(Skip = "Game unavailable in CI")]   ← Skips EVEN when game IS available

RIGHT pattern:
  [GameFact]                               ← Skips only when DINO_GAME_PATH unset
```

The `[Fact(Skip = ...)]` attribute is evaluated at **compile time** (as a constant string). It cannot be made conditional on runtime environment. The correct approach is to use an `IFactMetadata` attribute that xUnit evaluates at **test discovery time**.

---

## Section 2: Skipped Tests Analysis

### 2.1 Integration Tests (CatalogTests) — 3 skipped

**File**: `src/Tests/Integration/Tests/CatalogTests.cs`

| Test | Reason Skipped | Should Skip in CI? | Should Skip on Local? |
|------|--------------|-------------------|---------------------|
| `Catalog_HasUnits` | Game not running | YES | NO (if game available) |
| `Catalog_HasBuildings` | Game not running | YES | NO (if game available) |
| `Catalog_HasProjectiles` | Game not running | YES | NO (if game available) |

**Root cause**: `GameFixture` already detects game availability (`GameAvailable` property). The `[Fact(Skip = ...)]` overrides this check entirely. Tests check `if (!_fixture.GameAvailable) return;` but by then the skip has already been recorded.

**Fix**: Use `[GameFact(RequiresCatalog = true)]` — skips only when `GameAvailable=false`.

### 2.2 GameLaunch Tests — 12 skipped

**File**: `src/Tests/GameLaunch/GameLaunch*.cs`

| Test Class | Skipped Tests | CI Status | Local Status |
|-----------|--------------|-----------|-------------|
| `GameLaunchOverlayTests` | 3 | SKIP | SKIP (if no game) |
| `GameLaunchNativeMenuTests` | 3 | SKIP | SKIP (if no game) |
| `GameLaunchHotReloadTests` | 1 | SKIP | SKIP (if no game) |
| `GameLaunchEconomyTests` | 4 | SKIP | SKIP (if no game) |
| `GameLaunchAssetSwapTests` | 1 (implicit) | SKIP | SKIP (if no game) |
| `GameLaunchSmokeTests` | 0 (smoke only) | RUN | RUN |

**Note**: `GameLaunchFixture` already implements graceful skipping via `IsInitialized` — but the tests themselves have `[Fact(Skip = ...)]`. These should NOT have skips at all; the fixture handles it.

**Fix**: Remove `[Fact(Skip = ...)]` from all GameLaunch tests. Let `GameLaunchFixture` handle the skip.

### 2.3 EnvironmentMatrix Tests — 4 skipped

**File**: `src/Tests/EnvironmentMatrixTests.cs`

| Test | Reason | Valid Skip? |
|------|--------|------------|
| `TestDesktopLaunchSucceeds` | Manual env setup | YES |
| `TestRdpCompatibility` | RDP env required | YES |
| `TestSandboxEnvironmentHandling` | Sandbox env required | YES |
| `LogEnvironmentDetails` | Informational only | YES |

**Note**: These are **correct** skips — they require specific hardware/environment that cannot be auto-detected.

### 2.4 UiAutomation Tests — 8 skipped

**File**: `src/Tests/UiAutomation/Companion*.cs`

These tests require the Companion app and Steam overlay. They should use `[SteamFact]` attribute (defined in `CompanionFixture`).

---

## Section 3: Root Cause Analysis

### 3.1 Why Unconditional Skips Are Wrong

1. **Local development**: Developer has game installed → tests skip anyway → bugs not caught
2. **CI environment**: Game not installed → correct skip → BUT no differentiation from local
3. **Self-hosted runner**: Game IS installed → tests skip anyway → E2E tests never run in CI
4. **Test discovery**: `[Fact(Skip = ...)]` is a compile-time constant → cannot depend on runtime state

### 3.2 The Missing Abstraction

The codebase has:
- `GameFixture` — detects game availability ✓
- `GameLaunchFixture` — launches game, detects `DINO_GAME_PATH` ✓
- `CompanionFixture` — companion app detection ✓

But it is **missing**:
- `GameFactAttribute` — conditionally skips based on `GameFixture.IsAvailable`
- `TestEnvironmentResolver` — single source of truth for "what environment am I in?"
- `SkipWhenNoGame` — xUnit `IMethod life-cycle` hook

---

## Section 4: Proposed Architecture

### 4.1 TestEnvironment Enum

```csharp
public enum TestEnvironment
{
    /// <summary>No game available. All game-fact tests skip.</summary>
    None = 0,
    /// <summary>Game installed but not running. Bridge-dependent tests skip.</summary>
    InstalledOnly = 1,
    /// <summary>Game installed and running with bridge connected.</summary>
    GameRunning = 2,
    /// <summary>Full E2E: game running + Steam overlay + Companion app.</summary>
    FullE2E = 3,
}
```

### 4.2 TestEnvironmentResolver

```csharp
public static class TestEnvironmentResolver
{
    public static TestEnvironment Current { get; }
    public static bool IsGameAvailable { get; }
    public static bool IsBridgeConnected { get; }
    public static bool IsSteamAvailable { get; }
    public static string? GamePath { get; }
    public static string? BridgePort { get; }

    static TestEnvironmentResolver()
    {
        // Priority 1: Explicit env var (CI/self-hosted)
        string? explicitPath = Environment.GetEnvironmentVariable("DINO_GAME_PATH");
        if (!string.IsNullOrEmpty(explicitPath) && File.Exists(explicitPath))
            GamePath = explicitPath;

        // Priority 2: Auto-detect Steam install
        if (GamePath == null)
            GamePath = AutoDetectSteamGamePath();

        // Priority 3: Docker/sandbox game stub
        if (GamePath == null)
            GamePath = Environment.GetEnvironmentVariable("DINO_DOCKER_GAME_STUB");

        IsGameAvailable = !string.IsNullOrEmpty(GamePath);
    }
}
```

### 4.3 GameFactAttribute

```csharp
/// <summary>
/// Marks a test as requiring the DINO game to be available.
/// Skips automatically when:
///   - DINO_GAME_PATH is not set OR
///   - Game executable does not exist at DINO_GAME_PATH
///   - Bridge connection fails (when RequiresBridge=true)
///
/// Does NOT skip when the game IS available (even in CI with self-hosted runner).
/// </summary>
[AttributeUsage(AttributeTargets.Method, AllowMultiple = false)]
public sealed class GameFactAttribute : FactAttribute
{
    public bool RequiresBridge { get; init; }
    public bool RequiresCatalog { get; init; }
    public bool RequiresSteam { get; init; }
    public bool RequiresCompanion { get; init; }

    protected override async Task<RunSummary> RunAsync(IMethodInfo method, ReflectionAssemblyInfo assembly, 
        IAttributeInfo factAttribute, IMessageBus messageBus, ExceptionAggregator aggregator, 
        CancellationTokenSource cancellationTokenSource)
    {
        // Check 1: Game path available
        if (!TestEnvironmentResolver.IsGameAvailable)
            return GameFactResult(method, "Game not found at DINO_GAME_PATH or auto-detected path");

        // Check 2: Bridge required
        if (RequiresBridge && !TestEnvironmentResolver.IsBridgeConnected)
            return GameFactResult(method, "Game bridge not connected");

        // Check 3: Catalog required
        if (RequiresCatalog && !VanillaCatalog.IsBuilt)
            return GameFactResult(method, "VanillaCatalog not built — build VanillaCatalog first");

        // Check 4: Steam required
        if (RequiresSteam && !TestEnvironmentResolver.IsSteamAvailable)
            return GameFactResult(method, "Steam not available");

        // Check 5: Companion required
        if (RequiresCompanion && !TestEnvironmentResolver.IsCompanionAvailable)
            return GameFactResult(method, "Companion app not running");

        // All checks pass — run the test
        return await base.RunAsync(method, assembly, factAttribute, messageBus, aggregator, cancellationTokenSource);
    }

    private static RunSummary GameFactResult(IMethodInfo method, string reason)
    {
        // Return a skipped run summary
        return new RunSummary { Skipped = 1 };
    }
}
```

### 4.4 Environment Detection Strategy

| Environment | Detection Mechanism | GameAvailable | BridgeConnected | Notes |
|------------|-------------------|-------------|----------------|-------|
| **Local (Steam)** | Auto-detect Steam path | YES | YES (if launched) | Normal dev workflow |
| **Local (explicit)** | `DINO_GAME_PATH` env var | YES | YES (if launched) | Override auto-detect |
| **CI (ubuntu)** | No DINO_GAME_PATH | NO | NO | All GameFact tests skip |
| **CI (windows-latest)** | `DINO_GAME_PATH` secret | YES | YES (if launched) | Full game validation |
| **Self-hosted** | `DINO_GAME_PATH` env var | YES | YES | Runs GameLaunch E2E |
| **Docker/Stub** | `DINO_DOCKER_GAME_STUB` | YES | YES (mock) | Headless game simulation |
| **Steam Headless** | `STEAM_HEADLESS=1` | YES | YES (if launched) | Linux CI game testing |

---

## Section 5: Docker/Headless Steam Strategy

### 5.1 Problem

CI runs on `ubuntu-latest` (Linux). DINO is a Windows game. No way to run it natively.

### 5.2 Solution Options

| Option | Feasibility | Cost | Fidelity |
|--------|------------|------|---------|
| **A. Self-hosted Windows runner** | High — already configured | Free (already running) | 100% real game |
| **B. Steam Game Server (headless)** | Medium — SteamCMD can download some games | Free | Partial (no rendering) |
| **C. Wine/Proton** | Low — DINO uses Unity ECS | Unknown | Unlikely to work |
| **D. Mock game process** | High — fake bridge responses | Free | 70% (bridge only) |
| **E. GitHub Actions Windows runner** | Medium — $0.008/min vs free self-hosted | $$ | 100% real game |

### 5.3 Recommended: Option A + D (Hybrid)

**For GameLaunch E2E tests**: Use existing self-hosted Windows runner (`game-launch.yml`) which already runs `GameLaunch*` tests via `GameLaunchFixture`. The runner has `DINO_GAME_PATH` set.

**For Integration tests with bridge**: Use Docker container with a **mock game process** that:
1. Accepts HTTP bridge connections on port 7474
2. Responds to `/api/status` with mock game state
3. Can simulate scene transitions, entity counts, catalog responses
4. Can be extended to simulate failure modes (RuntimeDriver dead, slow responses)

```dockerfile
# docker/DINOForge.GameStub/Dockerfile
FROM mcr.microsoft.com/dotnet/aspnet:8.0
WORKDIR /app
COPY src/Tests/GameStub/ .
ENTRYPOINT ["dotnet", "DINOForge.GameStub.dll"]
EXPOSE 7474
```

The mock game stub implements the same HTTP JSON-RPC interface as the real game bridge. Tests tagged `[GameFact(RequiresBridge = false)]` run with the stub. Tests tagged `[GameFact(RequiresBridge = true)]` require the real game.

### 5.4 Steam Headless Mode for Linux CI

For the `game-launch-validation.yml` workflow on `windows-latest`:

```yaml
- name: Launch Steam headless
  shell: pwsh
  run: |
    steamcmd +runscript steam_headless.vdf +quit
    
# steam_headless.vdf
// install and launch game headlessly
```

This allows the game to run on a Windows CI runner without a visible desktop.

---

## Section 6: CI/CD Pipeline Updates

### 6.1 Proposed CI Matrix

| Job | Runner | DINO_GAME_PATH | Tests Run | GameFact Behavior |
|-----|--------|---------------|-----------|------------------|
| `build-and-test` | ubuntu-latest | NOT SET | Unit + offline Integration | Skips all GameFact |
| `integration-with-stub` | ubuntu-latest + Docker | Docker stub IP | Integration + GameStub | Runs with mock game |
| `game-launch-e2e` | self-hosted windows | SET | GameLaunch* | Runs with real game |
| `game-validation` | windows-latest | SET (secret) | Full suite | Runs with real game |
| `ui-automation` | windows-latest | SET | Companion* | Runs with real game |

### 6.2 New CI Job: `integration-with-stub`

```yaml
integration-with-stub:
  name: integration-with-stub
  runs-on: ubuntu-latest
  services:
    game-stub:
      image: dinoforge/game-stub:latest
      ports:
        - 7474:7474
      env:
        DINO_STUB_ENTITY_COUNT: "45776"
        DINO_STUB_CATALOG_SIZE: "500"
        DINO_STUB_WORLD_NAME: "Default World"
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-dotnet@v5
      with:
        dotnet-version: 8.0.x
    - name: Run integration tests with stub
      env:
        DINO_GAME_PATH: "http://game-stub:7474"
        DINO_BRIDGE_PORT: "7474"
      run: |
        dotnet test src/Tests/Integration/... --filter "Category=Integration"
```

### 6.3 Updated `game-launch-validation.yml`

```yaml
- name: Run integration + game-fact tests
  env:
    DINO_GAME_PATH: ${{ secrets.DINO_GAME_PATH }}
    DINO_BRIDGE_PORT: "7474"
  run: |
    dotnet test src/Tests/DINOForge.Tests.csproj \
      -c Release \
      --filter "Category=Integration" \
      -v normal
    dotnet test src/Tests/Integration/DINOForge.Tests.Integration.csproj \
      -c Release \
      --filter "Category=Integration || Category=GameWorkflow || Category=AssetSwap || Category=BridgeLifecycle" \
      -v normal
```

---

## Section 7: Implementation Roadmap

### Phase 1: Core Infrastructure (Do First)

| Task | File | Lines | Priority |
|------|-------|--------|---------|
| Define `TestEnvironment` enum | `Tests/TestEnvironment.cs` | 50 | P0 |
| Implement `TestEnvironmentResolver` | `Tests/TestEnvironment.cs` | 120 | P0 |
| Define `GameFactAttribute` | `Tests/Xunit/GameFactAttribute.cs` | 90 | P0 |
| Define `SteamFactAttribute` | `Tests/Xunit/SteamFactAttribute.cs` | 40 | P1 |
| Define `CompanionFactAttribute` | `Tests/Xunit/CompanionFactAttribute.cs` | 40 | P1 |

### Phase 2: Fix All Skipped Tests

| File | Current Skips | Action |
|------|--------------|--------|
| `Integration/Tests/CatalogTests.cs` | 3 `[Fact(Skip)]` | Replace with `[GameFact(RequiresCatalog = true)]` |
| `GameLaunch/GameLaunch*.cs` | 12 `[Fact(Skip)]` | Remove `[Fact(Skip)]` — fixture handles it |
| `UiAutomation/Companion*.cs` | 8 `[Fact(Skip)]` | Replace with `[CompanionFact]` |
| `EnvironmentMatrixTests.cs` | 4 `[Fact(Skip)]` | KEEP (valid hardware requirements) |

### Phase 3: Update CI/CD

| File | Change |
|-------|--------|
| `.github/workflows/ci.yml` | Add `integration-with-stub` job |
| `.github/workflows/game-launch.yml` | Update filter to run ALL GameFact tests |
| `.github/workflows/game-launch-validation.yml` | Add integration test suite run |

### Phase 4: Mock Game Stub (Optional but Recommended)

| Task | File | Lines | Priority |
|------|-------|--------|---------|
| Define `IGameBridgeStub` HTTP server | `Tests/GameStub/` | 300 | P2 |
| Implement `StubGameBridge` | `Tests/GameStub/` | 500 | P2 |
| Add Docker compose for stub | `docker-compose.yml` | 20 | P2 |
| Write stub integration tests | `Integration/Tests/StubTests.cs` | 150 | P2 |

---

## Section 8: Immediate Actions (This Session)

1. **Create `TestEnvironmentResolver.cs`** — single source of truth for "is game available?"
2. **Create `GameFactAttribute.cs`** — replaces all unconditional `[Fact(Skip)]` in game-dependent tests
3. **Fix `CatalogTests.cs`** — change 3 `[Fact(Skip)]` to `[GameFact(RequiresCatalog = true)]`
4. **Fix `GameLaunch*.cs`** — remove unconditional skips, rely on fixture
5. **Update `ci.yml`** — add game-stub Docker service for stub-based integration tests
6. **Document** the pattern in `AGENTS.md` so future tests use `GameFact` not `[Fact(Skip)]`

---

## Appendix A: Files Referenced

### Test Infrastructure Files
- `src/Tests/Integration/Fixtures/GameFixture.cs` — game detection for Integration tests
- `src/Tests/Integration/Fixtures/GameCollection.cs` — xUnit collection for GameFixture
- `src/Tests/GameLaunch/GameLaunchFixture.cs` — game launch + bridge bootstrap
- `src/Tests/GameLaunch/GameLaunchCollection.cs` — xUnit collection for GameLaunchFixture

### Skipped Test Files (13 tests to fix)
- `src/Tests/Integration/Tests/CatalogTests.cs` — 3 skips
- `src/Tests/GameLaunch/GameLaunchOverlayTests.cs` — 3 skips
- `src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs` — 3 skips
- `src/Tests/GameLaunch/GameLaunchHotReloadTests.cs` — 1 skip
- `src/Tests/GameLaunch/GameLaunchEconomyTests.cs` — 4 skips
- `src/Tests/GameLaunch/GameLaunchAssetSwapTests.cs` — 1 skip (implicit)
- `src/Tests/EnvironmentMatrixTests.cs` — 4 skips (KEEP — valid hardware requirements)
- `src/Tests/GameLaunchTests.cs` — 3 skips (MCP connectivity)

### CI Workflow Files
- `.github/workflows/ci.yml` — main CI (ubuntu, no game)
- `.github/workflows/game-launch.yml` — self-hosted E2E (Windows with game)
- `.github/workflows/game-launch-validation.yml` — Windows latest game validation

---

## Appendix B: Game Install Path Registry

```csharp
// Known game installation paths (used by TestEnvironmentResolver)
private static readonly string[] KnownGamePaths =
[
    @"C:\Program Files (x86)\Steam\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
    @"C:\Program Files\Steam\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
    @"D:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
    @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
];
```

These are already used by `GameFixture` and should be centralized in `TestEnvironmentResolver`.

---

## Appendix C: Coverage Impact

After implementation, expected test counts:

| Test Project | Before (skipped) | After (active) | Notes |
|-------------|-----------------|-----------------|-------|
| `Tests.Integration` | 42 passed / 3 skipped | 45 passed / 0 skipped | CatalogTests now run |
| `Tests.GameLaunch` | 0 run (all skipped) | 15 run / 0 skipped | Run on self-hosted |
| `Tests.UiAutomation` | 0 run (all skipped) | 8 run / 0 skipped | Run on windows-latest |
| `CI (ubuntu)` | 1955 passed / 4 skipped | 1955 + 3 stub-run | Stub tests added |
| **Total CI-passing** | 1955 | **2000+** | |
