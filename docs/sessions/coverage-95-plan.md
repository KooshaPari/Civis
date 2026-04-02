# 95% Coverage Audit & Plan

## Current State

| Metric | Current | Target |
|--------|---------|--------|
| Line Coverage | 82.62% | 95% |
| Branch Coverage | 67.72% | 80% |
| Method Coverage | 92.26% | 95% |
| Total Tests | 1,924 | ~2,200 |
| Skipped Tests | 24 | 0 |

## Coverage Gaps (by priority)

### Critical: 0% Coverage (Must Fix)
| Class | Current | Lines Uncovered | Strategy |
|-------|---------|-----------------|----------|
| `GoDependencyResolver` | 0% | ~200 | Mock Go process, test resolver logic |
| `RustAssetPipeline` | 0% | ~400 | Mock Rust process, test pipeline logic |
| NativeInterop models | 0% | ~100 | Unit test data model behavior |

### High: 70-90% Coverage (Quick Wins)
| Class | Current | Target | Gap |
|-------|---------|--------|-----|
| `RegistryImportService` | 73.3% | 95% | +22% |
| `FactionNamingRules` | 75.0% | 95% | +20% |
| `AerialProperties` | 75.0% | 95% | +20% |
| `UniverseLoader` | 81.5% | 95% | +14% |
| `FileDiscoveryService` | 88.1% | 95% | +7% |
| `ContentLoadResult` | 88.9% | 95% | +6% |

### Skipped Tests (Must Run Real Game)
| File | Skipped | Reason |
|------|---------|--------|
| `CatalogTests.cs` | 3 | Requires live game |
| `GameWorkflowTests.cs` | 2 | Mock incomplete (USE REAL GAME) |
| `BridgeLifecycleTests.cs` | 1 | Mock incomplete (USE REAL GAME) |
| `GameLaunch*.cs` | 11 | Requires live game |
| `EnvironmentMatrixTests.cs` | 4 | RDP/Sandbox specific |
| `GameLaunchTests.cs` | 3 | MCP connectivity |

## Implementation Plan

### Phase 1: Fix Integration Tests (Real Game, No Mocks)
1. Create `GameTestRunner` base class using MCP `game_launch` tool
2. Add headless game launch for CI
3. Rewrite skipped tests to use real game execution
4. **Policy**: Integration tests MUST run against real game

### Phase 2: NativeInterop Coverage (Biggest Gap)
1. Create `GoProcessMock` for GoDependencyResolver tests
2. Create `RustProcessMock` for RustAssetPipeline tests
3. Add tests for async state machines (CallMcpAsync, ImportAssetAsync, etc.)
4. Test error paths: Go/Rust process not found, crashes, timeouts

### Phase 3: SDK Gap Closure
1. RegistryImportService: Add error path tests
2. FileDiscoveryService: Add edge case tests (symlinks, permissions)
3. ContentLoadResult: Add failure scenario tests
4. UniverseLoader: Add validation failure tests

### Phase 4: CI Pipeline
1. Add `test-game` job to CI (headless game launch)
2. Add `test-game-integration` job (full game automation)
3. Gate merge on game tests passing
4. Add test instance path configuration

## No-Mock Policy

**Integration tests MUST NOT use mocks for game-related functionality.**

Instead of:
```csharp
var bridge = new FakeGameWorkflowBridge(); // BAD
```

Use:
```csharp
var runner = new GameTestRunner(); // GOOD
await runner.LaunchGameAsync();
await runner.WaitForWorldAsync();
```

### GameTestRunner Pattern
```csharp
public class GameTestRunner : IDisposable
{
    public async Task LaunchGameAsync(string? testInstance = null);
    public async Task<GameStatus> StatusAsync();
    public async Task<WaitResult> WaitForWorldAsync(int timeoutMs = 60000);
    public async Task<T> QueryAsync<T>(string method, object? parameters = null);
    public void Dispose();
}
```

## Test Classification

| Type | Location | Mocks? | CI? |
|------|----------|--------|-----|
| Unit | `src/Tests/*.cs` | Yes | Yes |
| Integration | `src/Tests/Integration/` | NO - real game | Yes (headless) |
| Game | `src/Tests/GameLaunch/` | NO - MCP tool | Yes (headless) |
| E2E | `src/Tests/e2e/` | NO - real game + UI | Manual |

## Quick Wins Estimate

- NativeInterop tests: +12% line coverage
- SDK gap closure: +5% line coverage  
- Integration tests with real game: +3% line coverage
- **Total estimated: 95%+ line coverage**
