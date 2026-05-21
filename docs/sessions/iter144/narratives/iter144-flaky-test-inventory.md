# Flaky-Test Candidate Inventory (iter-144)

## Already-fixed this session
- AssetSwapRegistry_BulkRegister_Under10Ms (d7342a02)
- JsonRpcRequest_MethodName_AssignmentStable (d625e18e)
- MockGameBridgeServer pipe-name + ScreenshotFallback skip (7de6fd37)
- Server_Starts_ExposesPipeName assertion + LiveGame x13 SkippableFact (b0538a48)
- PackFileWatcher.FileChanged debounce (9bc88f9c)

## Remaining candidates (ranked by risk)

### Risk: HIGH (likely to flake on next CI/lefthook run)
- src/Tests/AssetSwapLatencyTests.cs:55 - p99 < 1ms - perf threshold absurdly tight on CI noise.
- src/Tests/AssetSwapLatencyTests.cs:86 - sw < 50ms - perf threshold; allocator pauses spike it.
- src/Tests/GameLaunch/GameLaunchSmokeTests.cs:40 - sw < 100ms vs live RPC - perf+timing.
- src/Tests/Integration/BridgeLatencyTests.cs:45,74 - p99 < 100/200ms - perf threshold; named-pipe jitter.
- src/Tests/Bridge/GameClientFramingTests.cs:68,88 - sw < 4500/2500ms - tight timeout; flakes under heavy lefthook concurrency.
- src/Tests/Integration/ParallelGameTestsWithHarness.cs:108 - <30s for 4 parallel launches - resource-constrained CI.
- src/Tests/Integration/Tests/ParallelGameE2ETests.cs:124,146,309,398,410,712,963,1011 - 8x Thread.Sleep/Task.Delay polling boot - race-prone.
- src/Tests/Integration/Tests/ParallelGameE2ETests.cs:209 - hardcoded pipe `dinoforge-game-bridge-test-{pid}` - PID can recycle on Windows, collide.
- src/Tests/GameLaunch/GameLaunchOverlayTests.cs:91 - await Task.Delay(10_000) wall-clock sleep, no predicate - either over- or under-shoots.
- src/Tests/GameLaunch/GameLaunchOverlayTests.cs:24,50,82 - `[Fact]` on tests that hit live game; should be `[SkippableFact]` (LiveGame attribute missing).
- src/Tests/GameLaunch/GameLaunchSmokeTests.cs:18,31 / GameLaunchPackTests.cs:17,34 / GameLaunchEconomyTests.cs:21,37,52,73 / GameLaunchUiTests.cs:20,39 / GameLaunchStatTests.cs:16 / GameLaunchHotReloadTests.cs:22 / GameLaunchAssetSwapTests.cs:25,52 / GameLaunchNativeMenuTests.cs:21,38,64 - 19 [Fact]s in GameLaunch/ that connect to live bridge but lack [SkippableFact] skip; will hard-fail when DINO absent.
- src/Tests/PropertyTests.cs:60,163,250,402,420,480,497 - 7 `[Property]` accepting raw `NonEmptyString` for pack-id / version where domain expects regex-constrained ASCII; no `.Where(...)` filter -> FsCheck discard exhaustion or assertion failure on Unicode garbage. (PropertyTests.cs:30 comment claims "constrained generator" but only methods 32+52 use it.)

### Risk: MEDIUM
- src/Tests/UiAutomation/CompanionDashboardTests.cs:90 (1500ms), CompanionDebugPanelTests.cs:55,71,91 (800ms x3), CompanionFixture.cs:66,97 (NavWaitMs), CompanionPackListTests.cs:60, CompanionPackToggleTests.cs:33, CompanionSettingsTests.cs:113, CompanionStatusBarTests.cs:60 - Thread.Sleep race-prone for UIA element appearance.
- src/Tests/HotReloadTests.cs:133 - Thread.Sleep(50) before reload assertion; can lose race on slow disk.
- src/Tests/ParameterizedTests/HotReloadFsCheckProperties.cs:50 - Thread.Sleep(1) for "distinct timestamps" - FAT/NTFS resolution can swallow it.
- src/Tests/PackFileWatcherTests.cs:177 - await Task.Delay(250) debounce wait, may need re-tune vs 9bc88f9c.
- src/Tests/Mocks/MockGameBridgeServer.cs:80 - await Task.Delay(100) inside mock RPC; if production timeout drops below 100ms test hangs.
- src/Tests/Integration/Tests/ErrorPathTests.cs:66 - Task.Delay(100) "give client time to notice" - timing-based.
- src/Tests/EndToEndUserJourneysTests.cs:193 - Task.Delay(100) "ensure time difference" - timing-based.
- src/Tests/GameClientCoverageTests.cs:2267 - Task.Delay(100) - timing-based.
- src/Tests/Integration/Tests/GameSandboxIntegrationTests.cs:98 - Thread.Sleep(500) inside polling.
- src/Tests/Integration/Tests/MockGameServerTests.cs (no SkippableFact / [Fact] needing review).
- src/Tests/ParameterizedTests/ValidationFsCheckProperties.cs:98,121,142,163,196 - `[Property(MaxTest=100)]` chains 3-4 PositiveInts that flow into version-string formatting; under boundary values (int.MaxValue) version parse can overflow.
- src/Tests/EnvironmentMatrixTests.cs:93,132 - Task.Delay(5s) wall-clock per case; cumulative flake risk on slow runners.

### Risk: LOW (theoretical, but worth tracking)
- src/Tests/BridgeClientAsyncTests.cs:273,281 / GameClientCoverageTests.cs:1110,1123 - Task.Delay(Timeout.Infinite) inside cancellation-test handlers; correct pattern but flake if CT is never wired.
- src/Tests/BridgeClientTests.cs:20 / GameClientOptionsUnitTests.cs:22 / GameClientCoverageTests.cs:166,2057 / GameClientUnitTests.cs:80 - Assert default PipeName == "dinoforge-game-bridge"; harmless unless the default ever changes for parallel safety.
- src/Tests/HotReloadTests.cs:106,126,127 / PackFileWatcherTests.cs:140,169,170 - `+=` event subscriptions in tests without matching `-=` in Dispose; leaks survive only across that test, but can fire on stale watcher.
- src/Tests/AviationStarWarsTests.cs (many BeGreaterThan(-1)/(0)) - open-ended count semantics (Pattern #110); not flaky, just weak.
- src/Tests/Integration/Tests/ScreenshotFallbackTests.cs:106 - Task.Delay(100) between captures.
- src/Tests/Integration/GameTestRunner.cs:112,172 / GameTestContainerHarness.cs:133,169,199,212 - 100-2000ms polling waits; tolerate slow CI but may extend wall-clock.

## Categorized counts
- Hardcoded timing thresholds: ~10 (perf assertions); ~80 open-ended count assertions (Pattern #110, not strictly flaky)
- Thread.Sleep in tests: ~14 (production); Task.Delay in tests: ~30
- Unfiltered FsCheck generators: 7 (PropertyTests.cs) + 5 boundary-risk (ValidationFsCheckProperties)
- Hardcoded pipe-name assertions: 5 (default-check, low risk) + 1 PID-suffix collision (ParallelGameE2ETests.cs:209)
- Broken skip helpers: 0 confirmed remaining (both SkipIfGameNotAvailable now use `Skip.IfNot`)
- Live-DINO without SkippableFact: 19 in src/Tests/GameLaunch/* (all [Fact]; need [SkippableFact] + GameAvailable guard like GameLaunchFixture exposes)
- Event +=/-= asymmetry in tests: ~6 (HotReload/PackFileWatcher)

## Recommended fix order for next session
1. **Convert 19 GameLaunch/[Fact] -> [SkippableFact]** with `Skip.IfNot(_fixture.GameAvailable, ...)`. Highest blast radius; one commit closes biggest source of CI flake.
2. **Fix ParallelGameE2ETests.cs:209 PID-based pipe name** -> GUID suffix per #443 convention; remove 8 hardcoded Thread.Sleep/Task.Delay in same file (replace with TestWait.UntilAsync).
3. **Relax perf thresholds** in AssetSwapLatencyTests (p99<1ms -> p99<5ms), GameLaunchSmokeTests:40 (<100ms -> <500ms), Bridge/GameClientFramingTests (<2500/<4500 -> doubled or env-gated).
4. **Add `.Where(IsValidId)` filter** to 7 NonEmptyString FsCheck generators in PropertyTests.cs (idNonEmpty, packIdNonEmpty, versionNonEmpty).
5. **Replace Thread.Sleep in CompanionUIA fixtures** with WaitForCondition helper (TestWait.UntilAsync exists already).
6. **GameLaunchOverlayTests.cs:91 10s sleep** -> predicate-based wait on EntityCount stability.
7. **Audit event +=/-= asymmetry** in HotReloadTests + PackFileWatcherTests; add Dispose unsubscribe.
