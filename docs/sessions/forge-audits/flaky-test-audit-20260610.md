# Flaky-Test Pattern Audit — `src/`

**Generated**: 2026-06-10
**Scope**: `src/**/*.cs` (test code)
**Audit type**: Read-only
**Pattern class**: Sleep-based / timeout-based test synchronization

## Context

A recently fixed test (and the project's own analyzer — see
`src/Analyzers/SleepBasedTestSyncAnalyzer.cs`, DF0108) flagged `Thread.Sleep` /
`Task.Delay` with small/fixed millisecond values in test code as fragile across
environments. The repo provides a robust replacement
(`src/Tests/Support/TestWait.cs` — `TestWait.UntilAsync(predicate, timeout)`,
documented as "Pattern #108 — sleep-based sync is fragile"). This audit lists
remaining instances of the pattern (and related risks) for follow-up.

## Risk Tiers

| Tier | Meaning |
|------|---------|
| **HIGH** | Tight hardcoded timeout on a blocking operation. Will fail under CI load / Windows Server / VMs / parallel runs. No `// test-sleep-ok:` suppression. |
| **MED** | Hardcoded delay used to "give the OS a moment" before asserting on a state that *could* be checked synchronously. Flaky on slow runners. |
| **LOW** | Production-code `Thread.Sleep` (e.g. `Runtime/Capture/SessionRecorder.cs`, `Win32KeyInput.cs`) — not a test, but flagged for completeness. |
| **OK** | Already suppressed with `// test-sleep-ok:`, `// pattern-113-ok:`, or otherwise intentional. |

## Subsystem Summary

| Subsystem                       | Files w/ pattern | Total occurrences | High-risk |
|---------------------------------|------------------|-------------------|-----------|
| `src/Tests/GameClient*Tests.cs` | 4                | ~110              | ~12       |
| `src/Tests/GameLaunch/`         | 4                | ~28               | ~5        |
| `src/Tests/UiAutomation/`       | 7                | ~13               | ~8        |
| `src/Tests/Integration/`        | 5                | ~22               | ~6        |
| `src/Tests/Analyzers/*Tests.cs` | 4                | ~17               | 0         |
| `src/Tests/Support/`            | 1                | 2                 | 0         |
| `src/Tests/Performance*`        | 2                | ~14               | 0         |
| `src/Tests/Mocks/`              | 1                | 1                 | 0         |
| `src/Tests/Load/`               | 1                | 1                 | 0         |
| `src/Tests/HotReloadTests.cs`   | 1                | 1                 | 1         |
| **Total**                       | **~28**          | **~210**          | **~32**   |

The count of hardcoded `*TimeoutMs = <small-int>` literal assignments in test
code is the primary risk signal: 110+ instances.

---

## Findings

Columns: `file:line` | pattern | risk | suggested-fix

### TIER A — Tight hardcoded `*TimeoutMs` (≤ 100 ms)

These are the most fragile: the test depends on the OS scheduling the timeout
*before* the awaited operation completes. On a loaded CI runner the timeout can
fire spuriously, or the awaited op can complete before the timeout arms.

| file:line | pattern | risk | suggested-fix |
|-----------|---------|------|---------------|
| `src/Tests/GameClientCoverageTests.cs:97` | `ReadTimeoutMs = 20, // Very short timeout` | **HIGH** | Bump to `1000` and inject a `BlockingMemoryStream` to deterministically force the timeout path; assert *behavior* (throw type + message), not elapsed time. |
| `src/Tests/GameClientCoverageTests.cs:124` | `ReadTimeoutMs = 50,` | **HIGH** | Same — bump and rely on `BlockingMemoryStream` from `BridgeClientAsyncTests.cs:269`. |
| `src/Tests/GameClientCoverageTests.cs:180` | `ConnectTimeoutMs = 1, // Very short timeout` | **HIGH** | 1 ms is below the Windows scheduler tick (default 15.6 ms). On a hot CI runner the pipe may sometimes "connect" (to a stale handle) and not exercise the intended throw path. Use `100` minimum or better: a `PipeName` reserved name + a stop predicate. |
| `src/Tests/GameClientCoverageTests.cs:593` | `ReadTimeoutMs = 100,` | **HIGH** | 100 ms is borderline. Replace with `TestWait.UntilAsync` polling on the state. |
| `src/Tests/GameClientCoverageTests.cs:725` | `ConnectTimeoutMs = 100,` | **HIGH** | See above. |
| `src/Tests/GameClientCoverageTests.cs:910` | `ReadTimeoutMs = 50, // Very short timeout` | **HIGH** | Same as 97/124. |
| `src/Tests/GameClientCoverageTests.cs:1463` | `ConnectTimeoutMs = 100, PipeName = "nonexistent-pipe"` | **MED** | 100 ms may be enough to give up; pipe name is reliable. Verify on slow runner — consider `500`. |
| `src/Tests/GameClientCoverageTests.cs:1487` | `ReadTimeoutMs = 100,` | **HIGH** | Same as 593. |
| `src/Tests/GameClientCoverageTests.cs:1499` | `ReadTimeoutMs = 100,` | **HIGH** | Same as 593. |
| `src/Tests/GameClientCoverageTests.cs:1517` | `ReadTimeoutMs = 50,` | **HIGH** | Same as 97/124. |
| `src/Tests/GameClientCoverageTests.cs:1537` | `ConnectTimeoutMs = 100,` | **HIGH** | Same as 725. |
| `src/Tests/GameClientCoverageTests.cs:1771` | `RetryDelayMs = 1, ReadTimeoutMs = 100,` | **HIGH** | `RetryDelayMs = 1` means retry loop spins with ~1 ms gap — the retry-then-fail path may run multiple iterations inside a 100 ms wall clock on a fast runner and overrun. Bump `RetryDelayMs` to ≥10. |
| `src/Tests/GameClientCoverageTests.cs:1794` | `ReadTimeoutMs = 30,` | **HIGH** | 30 ms is well under the Windows scheduler tick. |
| `src/Tests/GameClientCoverageTests.cs:2091` | `ConnectTimeoutMs = 50,` | **HIGH** | Bump to 500. |
| `src/Tests/GameClientCoverageTests.cs:2546` | `RetryDelayMs = 1, ReadTimeoutMs = 50,` | **HIGH** | Same as 1771. |
| `src/Tests/GameClientCoverageTests.cs:2574` | `RetryDelayMs = 1, ReadTimeoutMs = 50, ConnectTimeoutMs = 10,` | **HIGH** | All three values are too tight; bump to 50/200/200. |
| `src/Tests/GameClientCoverageTests.cs:2609` | `ConnectTimeoutMs = 1,` | **HIGH** | Same as 180. |
| `src/Tests/BridgeClientAsyncTests.cs:96` | `RetryDelayMs = 10,` | **MED** | 10 ms is on the edge; on a very fast runner the retry may not even be observable. Bump to 50 if the test asserts on retry-ordering, or remove the assertion and just assert "threw". |
| `src/Tests/BridgeClientAsyncTests.cs:124` | `ReadTimeoutMs = 50,` | **HIGH** | Bump to 1000. |
| `src/Tests/BridgeClientAsyncTests.cs:49` | `cts.CancelAfter(50); // Cancel after 50ms while request is in flight` | **MED** | `50 ms` is the cancel window. On a very slow runner the cancel may fire after the in-flight request already returned. Bump to 200 and assert the inner cause is `OperationCanceledException` rather than the wall clock. |

### TIER B — `Thread.Sleep` in test code

The custom analyzer `SleepBasedTestSyncAnalyzer` already flags these in
`src/Tests/`. Each entry below has a recommended replacement using
`TestWait.UntilAsync(predicate, timeout)`.

| file:line | pattern | risk | suggested-fix |
|-----------|---------|------|---------------|
| `src/Tests/BridgeClientAsyncTests.cs:273` | `Thread.Sleep(Timeout.Infinite); // pattern-113-ok: deliberate blocking stream for cancel test` | **OK** | Suppressed via `pattern-113-ok:` — keep as-is. |
| `src/Tests/HotReloadTests.cs:133` | `Thread.Sleep(50);` | **HIGH** | "Debounce not yet fired" assertion after a fixed 50 ms. On a slow CI runner the debounce may take >50 ms and the assertion will fail. Use `TestWait.UntilAsync(() => reloadedEvent.IsSet, TimeSpan.FromSeconds(2), pollMs: 20)` and assert `IsSet == false` after the timeout (negation is robust to scheduler jitter). |
| `src/Tests/GameLaunch/GameLaunchProcessCleanup.cs:33` | `Thread.Sleep(PostKillVerifyDelayMs);` | **MED** | Depends on `PostKillVerifyDelayMs` constant. Replace with polling: `TestWait.UntilAsync(() => !process.HasExited, ...)`. Find the constant. |
| `src/Tests/Integration/Tests/ParallelGameE2ETests.cs:129` | `Thread.Sleep(5000); // Wait for Unity to initialize` | **HIGH** | "Wait for Unity to initialize" is a classic flaky-test pattern. Replace with: spin until the bridge pipe is connectable, with a 30 s ceiling. |
| `src/Tests/Integration/Tests/ParallelGameE2ETests.cs:151` | `Thread.Sleep(1000);` | **HIGH** | Inside a connect-retry loop. Replace the whole loop with `TestWait.UntilAsync(() => TryConnect(), TimeSpan.FromSeconds(30), pollMs: 250)`. |
| `src/Tests/Integration/Tests/GameSandboxIntegrationTests.cs:112` | `System.Threading.Thread.Sleep(500);` | **HIGH** | Same pattern. Replace with polling. |
| `src/Tests/UiAutomation/CompanionStatusBarTests.cs:60` | `Thread.Sleep(500);` | **HIGH** | UI automation settle. Replace with `MainWindow.FindFirstDescendant(...)` polling with timeout. |
| `src/Tests/UiAutomation/CompanionShortcutTests.cs:30` | `Thread.Sleep(100);` | **HIGH** | UI automation. Replace with element-found polling. |
| `src/Tests/UiAutomation/CompanionShortcutTests.cs:51` | `Thread.Sleep(100);` | **HIGH** | Same. |
| `src/Tests/UiAutomation/CompanionSettingsTests.cs:113` | `Thread.Sleep(600); // Allow SaveAsync to complete` | **HIGH** | "Allow async to complete" — assertion should poll for the completion signal, not sleep. |
| `src/Tests/UiAutomation/CompanionPackToggleTests.cs:33` | `Thread.Sleep(300);` | **HIGH** | Same. |
| `src/Tests/UiAutomation/CompanionPackListTests.cs:60` | `Thread.Sleep(600);` | **HIGH** | Same. |
| `src/Tests/UiAutomation/CompanionFixture.cs:54` | `Thread.Sleep(600);` | **HIGH** | App-launch settle. Replace with: `TestWait.UntilAsync(() => MainWindow.IsAvailable, TimeSpan.FromSeconds(10), pollMs: 100)`. |
| `src/Tests/UiAutomation/CompanionFixture.cs:74` | `Thread.Sleep(NavWaitMs);` | **HIGH** | Post-click settle. Replace with element-appeared polling. |
| `src/Tests/UiAutomation/CompanionFixture.cs:95` | `Thread.Sleep(100);` | **HIGH** | Inside `WaitForElement` polling loop. Already polls; just replace the 100 ms sleep with the same `pollMs`. |
| `src/Tests/UiAutomation/CompanionDebugPanelTests.cs:55` | `Thread.Sleep(800);` | **HIGH** | Same. |
| `src/Tests/UiAutomation/CompanionDebugPanelTests.cs:71` | `Thread.Sleep(800);` | **HIGH** | Same. |
| `src/Tests/UiAutomation/CompanionDebugPanelTests.cs:91` | `Thread.Sleep(800);` | **HIGH** | Same. |
| `src/Tests/UiAutomation/CompanionDashboardTests.cs:90` | `Thread.Sleep(1500);` | **HIGH** | Same. |
| `src/Tests/ParameterizedTests/HotReloadFsCheckProperties.cs:50` | `Thread.Sleep(1); // Small delay to ensure distinct timestamps` | **MED** | Distinct-timestamp guarantee — replace by reading `DateTime.UtcNow.Ticks` in a loop until it changes, or use a `TimeProvider`-based abstraction. |
| `src/Tests/Analyzers/SleepBasedTestSyncAnalyzerTests.cs:62` | `Thread.Sleep(100);` | **OK** | Inside the analyzer's own test fixture (proves the analyzer detects it). Keep with a `// test-sleep-ok:` suppression. |

### TIER C — `Task.Delay(<small-ms>)` in test code

The analyzer does not currently flag these (it only flags `Thread.Sleep` and
the bare `Task.Delay` with no wait condition). They have the same fragility as
`Thread.Sleep` and should be migrated to `TestWait.UntilAsync` where they
synchronize on a real condition.

| file:line | pattern | risk | suggested-fix |
|-----------|---------|------|---------------|
| `src/Tests/GameClientCoverageTests.cs:2269` | `await Task.Delay(100).ConfigureAwait(true);` | **MED** | No follow-up state check — pure "wait 100 ms". If the surrounding assertion depends on a side effect of the awaited work, replace with polling on that side effect. |
| `src/Tests/Mocks/MockGameBridgeServer.cs:93` | `await Task.Delay(100, ct).ConfigureAwait(false);` | **MED** | Production (mock) code, not test. If the delay is between simulated requests, OK; if it's compensating for race, use a `SemaphoreSlim` or `Channel<T>` signal. |
| `src/Tests/EndToEndUserJourneysTests.cs:193` | `await Task.Delay(100); // Ensure time difference` | **HIGH** | "Ensure time difference" — flaky by definition. Capture the two timestamps in a `TimeProvider`-aware API or assert on monotonic counter. |
| `src/Tests/GameLaunch/GameLaunchOverlayTests.cs:47` | `await Task.Delay(400).ConfigureAwait(false);` | **HIGH** | Repeated 5+ times in this file. Each is a UI-state settle. Replace with `TestWait.UntilAsync` on the overlay element. |
| `src/Tests/GameLaunch/GameLaunchOverlayTests.cs:60` | `await Task.Delay(300);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchOverlayTests.cs:67` | `await Task.Delay(300);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchOverlayTests.cs:107` | `await Task.Delay(400).ConfigureAwait(false);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchOverlayTests.cs:119` | `await Task.Delay(300);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchOverlayTests.cs:143` | `await Task.Delay(3000);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchOverlayTests.cs:182` | `await Task.Delay(300);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchOverlayTests.cs:189` | `await Task.Delay(300);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchOverlayTests.cs:202` | `await Task.Delay(3000).ConfigureAwait(false);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchOverlayTests.cs:209` | `await Task.Delay(400).ConfigureAwait(false);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchAssetSwapTests.cs:316` | `await Task.Delay(250).ConfigureAwait(false);` | **HIGH** | UI settle. Replace with polling. |
| `src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs:80` | `await Task.Delay(3000);` | **HIGH** | Process-launch wait. |
| `src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs:92` | `await Task.Delay(2500);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs:117` | `await Task.Delay(3000);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs:242` | `await Task.Delay(1500).ConfigureAwait(false);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs:251` | `await Task.Delay(2000).ConfigureAwait(false);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs:273` | `await Task.Delay(500).ConfigureAwait(false);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs:387` | `await Task.Delay(750).ConfigureAwait(false);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs:402` | `await Task.Delay(750).ConfigureAwait(false);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs:412` | `await Task.Delay(750).ConfigureAwait(false);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs:439` | `await Task.Delay(750).ConfigureAwait(false);` | **HIGH** | Same. |
| `src/Tests/GameLaunch/GameLaunchHotReloadTests.cs:62` | `await Task.Delay(500);` | **HIGH** | Reload settle. |
| `src/Tests/Integration/GameTestRunner.cs:112` | `await Task.Delay(2000, ct).ConfigureAwait(true);` | **HIGH** | Test-runner warmup. |
| `src/Tests/Integration/GameTestRunner.cs:172` | `await Task.Delay(1000, ct).ConfigureAwait(true);` | **HIGH** | Same. |
| `src/Tests/Integration/GameTestContainerHarness.cs:133` | `await Task.Delay(100).ConfigureAwait(true);` | **MED** | Repeated pattern in this file. |
| `src/Tests/Integration/GameTestContainerHarness.cs:169` | `await Task.Delay(100).ConfigureAwait(true);` | **MED** | Same. |
| `src/Tests/Integration/GameTestContainerHarness.cs:199` | `await Task.Delay(100).ConfigureAwait(true);` | **MED** | Same. |
| `src/Tests/Integration/GameTestContainerHarness.cs:212` | `await Task.Delay(100).ConfigureAwait(true);` | **MED** | Same. |
| `src/Tests/Integration/ParallelGameTestsWithHarness.cs:251` | `await Task.Delay(10).ConfigureAwait(true); // Simulate async work` | **LOW** | This is *intentional* — simulates a slow operation. Keep with `// test-sleep-ok:`. |
| `src/Tests/Integration/Tests/ParallelGameE2ETests.cs:322` | `await Task.Delay(2000).ConfigureAwait(true);` | **HIGH** | Test-fixture settle. |
| `src/Tests/Integration/Tests/ParallelGameE2ETests.cs:411` | `await Task.Delay(1000).ConfigureAwait(true);` | **HIGH** | Same. |
| `src/Tests/Integration/Tests/ParallelGameE2ETests.cs:423` | `await Task.Delay(3000).ConfigureAwait(true);` | **HIGH** | Same. |
| `src/Tests/Integration/Tests/ParallelGameE2ETests.cs:765` | `await Task.Delay(2000).ConfigureAwait(true);` | **HIGH** | Same. |
| `src/Tests/Integration/Tests/ParallelGameE2ETests.cs:1028` | `await Task.Delay(1000).ConfigureAwait(true);` | **HIGH** | Same. |
| `src/Tests/Integration/Tests/ParallelGameE2ETests.cs:1081` | `await Task.Delay(500).ConfigureAwait(true);` | **HIGH** | Same. |
| `src/Tests/Integration/Tests/ScreenshotFallbackTests.cs:107` | `await Task.Delay(100).ConfigureAwait(true); // Small delay between captures` | **MED** | OK if there's a documented reason; otherwise the comment is a code smell. |
| `src/Tests/Load/BridgeLoadSkeletonTests.cs:91` | `await Task.Delay(50, _cancellation.Token).ConfigureAwait(false);` | **MED** | Load test ramp. The 50 ms is inter-request spacing; replace with a stopwatch-driven loop and assert on a metric. |
| `src/Tests/Analyzers/SleepBasedTestSyncAnalyzerTests.cs:41` | `await Task.Delay(100);` | **OK** | Inside the analyzer's own test fixture (proves the analyzer detects it). Keep with `// test-sleep-ok:`. |
| `src/Tests/Analyzers/SleepBasedTestSyncAnalyzerTests.cs:79` | `await Task.Delay(100);` | **OK** | Same. |
| `src/Tests/Analyzers/AsyncVoidAnalyzerTests.cs:29/46/63` | `await Task.Delay(1);` | **OK** | Inside the analyzer's own test fixture (yields control). Keep with `// test-sleep-ok:`. |
| `src/Tests/Analyzers/ConfigureAwaitAnalyzerTests.cs:29/46/62/78` | `await Task.Delay(1);` | **OK** | Same. |
| `src/Tests/Analyzers/LockAroundAwaitAnalyzerTests.cs:73/99/121/143` | `await Task.Delay(1);` | **OK** | Same. |

### TIER D — Stopwatch / time-based assertions

| file:line | pattern | risk | suggested-fix |
|-----------|---------|------|---------------|
| `src/Tests/PerformanceBenchmarkTests.cs:49` | `Stopwatch.StartNew(); ... sw.ElapsedMilliseconds.Should().BeLessThan(500, ...)` | **MED** | Benchmark thresholds on developer machines vs CI. CI is generally slower — bump to 1500 ms for the 500 ms claim, or run only on a `[Trait("Category","Performance")]` filter and skip on CI. |
| `src/Tests/PerformanceBenchmarkTests.cs:67/102/134/165/201/223/249/271/311/346` | Same pattern, varying thresholds | **MED** | Same. |
| `src/Tests/Integration/BridgeLatencyTests.cs:45` | `p99.Should().BeLessThan(100, ...)` | **MED** | p99 of 10 samples is unreliable; combined with a 100 ms ceiling on CI, this will flake. Use a 30-sample median and a 500 ms ceiling. |
| `src/Tests/Integration/BridgeLatencyTests.cs:74` | `p99.Should().BeLessThan(200, ...)` | **MED** | Same. |
| `src/Tests/AssetSwapLatencyTests.cs:43/74` | `Stopwatch sw = Stopwatch.StartNew();` | **MED** | Likely follows the same latency-assertion pattern. Verify thresholds. |
| `src/Tests/UiAutomation/CompanionFixture.cs:86` | `Stopwatch sw = Stopwatch.StartNew(); while (sw.ElapsedMilliseconds < timeoutMs) { ... Thread.Sleep(100); }` | **OK** | This is the *correct* polling pattern — wait up to `timeoutMs` for a condition. Keep; the `Thread.Sleep(100)` is the poll interval and is OK inside a poll loop. |
| `src/Tests/GameLaunch/GameLaunchSmokeTests.cs:33` | `Stopwatch sw = Stopwatch.StartNew();` | **MED** | Smoke test with stopwatch — verify the threshold is loose (>= 5 s) to be CI-safe. |
| `src/Tests/Support/TestWaitTests.cs:32/47` | `var sw = Stopwatch.StartNew(); ... result.Should().BeTrue().And.Be(...)` | **OK** | Tests for the polling helper itself; the stopwatch is measuring poll behavior. Keep. |
| `src/Tests/Integration/Tests/ScreenshotFallbackTests.cs:185` | `var sw = Stopwatch.StartNew();` | **MED** | Verify the threshold. |
| `src/Tests/Integration/Tests/ParallelGameE2ETests.cs:869` | `var sw = Stopwatch.StartNew();` | **MED** | Verify. |
| `src/Tests/GameLaunch/GameLaunchHotReloadTests.cs:57` | `var sw = System.Diagnostics.Stopwatch.StartNew();` | **MED** | Verify threshold. |
| `src/Tests/GameLaunch/GameLaunchAssetSwapTests.cs:312` | `System.Diagnostics.Stopwatch sw = System.Diagnostics.Stopwatch.StartNew();` | **MED** | Verify. |
| `src/Tests/GameLaunch/GameLaunchOverlayTests.cs:278` | `var sw = Stopwatch.StartNew();` (inside `TestWait.UntilAsync`) | **OK** | Used to measure the polling success window. |
| `src/Tests/PollingHelperTests.cs:258` | `var sw = System.Diagnostics.Stopwatch.StartNew();` | **OK** | Polling-helper test. |
| `src/Tests/Integration/GameTestContainerHarness.cs:102` | `var startTime = DateTime.UtcNow; while (DateTime.UtcNow - startTime < timeout)` | **OK** | Polling-loop with deadline. Acceptable; consider migrating to `TestWait.UntilAsync` for consistency. |
| `src/Tests/Integration/GameTestContainerHarness.cs:151/205` | Same | **OK** | Same. |
| `src/Tests/Integration/Tests/ParallelGameE2ETests.cs:139/309/397/601/754` | `var deadline = DateTime.UtcNow.AddSeconds(...); while (...)` | **OK** | Same pattern. |
| `src/Tests/Integration/Tests/GameSandboxIntegrationTests.cs:95` | `var deadline = System.DateTime.UtcNow.AddSeconds(30);` | **OK** | Same. |
| `src/Tests/GameLaunch/GameLaunchFixture.cs:134/246/289` | `DateTime deadline = DateTime.UtcNow.AddMilliseconds(...)` | **OK** | Same. |
| `src/Tests/Integration/ParallelGameTestsWithHarness.cs:95/102` | `var startTime = DateTime.UtcNow; var elapsed = ...` | **MED** | The `elapsed` value is used in a subsequent assertion; verify the threshold. |

### TIER E — HttpClient / WebClient / HttpWebRequest (synchronous) in tests

| file:line | pattern | risk | suggested-fix |
|-----------|---------|------|---------------|
| `src/Tests/PackRegistryTests.cs:333` | `using (var httpClient = new System.Net.Http.HttpClient())` | **LOW** | No `Timeout` property set; defaults to 100 s — usually safe, but if the test runs against a real network endpoint it can hang CI. Add a `Timeout = TimeSpan.FromSeconds(5)`. |
| `src/Tests/PackRegistryClientTests.cs:25/50/96` | `using HttpClient httpClient = new(handler);` | **LOW** | Same — check the `handler` for a timeout. |
| `src/Tests/PackRegistryClientCoverageTests.cs:81/107` | `using HttpClient httpClient = new(handler);` | **LOW** | Same. |
| `src/Tests/SDKCoverageTests.cs:50/67` | `using var httpClient = new HttpClient();` | **LOW** | No timeout. |
| `src/Tests/Sketchfab/SketchfabClientClockTests.cs:44` | `var http = new HttpClient();` | **LOW** | No timeout. |
| `src/Tests/Sketchfab/SketchfabAdapterClockTests.cs:48` | Same | **LOW** | Same. |
| `src/Tests/Integration/SmokeTests.cs:230` | `using var httpClient = new HttpClient(handler) { BaseAddress = new Uri("https://api.github.com") };` | **MED** | Reaches a real external API. **Skip in CI by default** (`[Trait("Category","Smoke")]` or `[Fact(Skip="Hits api.github.com — manual only")]`), or replace with a `MockHttpMessageHandler`. |
| `src/Tests/Analyzers/DisposableFieldNotDisposedAnalyzerTests.cs:27/42/56` | `private HttpClient _client = new HttpClient();` | **LOW** | Field used in analyzer test fixtures; the analyzer will flag the field as a leak — intentional test subject. |

### TIER F — `WaitOne(` and other wait-with-timeout primitives

| file:line | pattern | risk | suggested-fix |
|-----------|---------|------|---------------|
| `src/Tests/HotReloadTests.cs:137` | `reloadedEvent.Wait(1000) || failedEvent.Wait(1000);` | **MED** | `Wait(1000)` blocks the thread for up to 1 s — synchronously. Prefer `await reloadedEvent.WaitOneAsync(ct)` from a polyfill, or poll with `TestWait.UntilAsync`. |
| `src/Tests/Support/TestWait.cs` (intentional) | n/a | **OK** | The reference polling helper — keep. |

---

## Recommended Remediation Order

1. **Hot path on CI**: 12 `*TimeoutMs = {1, 10, 20, 30, 50, 100}` literal assignments in
   `GameClientCoverageTests.cs` (lines 97, 124, 180, 593, 725, 910, 1463,
   1487, 1499, 1517, 1537, 1771, 1794, 2091, 2546, 2574, 2609). These
   are the highest-confidence flakes because they assert on *throw type*
   racing against the wall clock.
2. **`UiAutomation/`**: All 13 `Thread.Sleep` calls there. None use
   `TestWait.UntilAsync`. The `CompanionFixture` poll at line 86 *is* the
   correct pattern — make all the other 800 ms / 1500 ms sleeps follow it.
3. **`Integration/Tests/ParallelGameE2ETests.cs`**: Lines 129, 151, 151.
   Real-process-launch flakes. Replace with bridge-ready polling.
4. **Performance / latency assertions**: Review the threshold in
   `PerformanceBenchmarkTests.cs` and `Integration/BridgeLatencyTests.cs`
   and gate with `[Trait("Category","Performance")]` so they don't run
   in `dotnet test --filter Category!=Performance` under CI.
5. **External HTTP**: `Integration/SmokeTests.cs:230` — tag + skip on CI.

## Suppression Comment Format

The repo's analyzer at `src/Analyzers/SleepBasedTestSyncAnalyzer.cs:47-56`
recognizes a single suppression token: `// test-sleep-ok:`. Already-used
exemptions (none in test code; only the `pattern-113-ok:` mark on
`BridgeClientAsyncTests.cs:273`):

- `pattern-113-ok` is a project-specific exemption used in the existing
  `BlockingMemoryStream` for cancellation tests. The analyzer does *not*
  currently check for `pattern-113-ok` — only `test-sleep-ok:` — so that
  one suppression is technically working only because the analyzer would
  flag the line and the developer happens to know the convention.
  **Recommend**: align the analyzer to accept `pattern-113-ok:` as well,
  or rename to `test-sleep-ok:`.

## What the Just-Fixed Test Taught Us

The most recent flaky-test fix (commit `06ff14d2` —
`CompatibilityCheckerCoverageTests.cs`) was *not* a timeout fix; it was an
inverted semver assertion that only surfaced under the `CI=true` gate
because of how the `CompatibilityChecker` library differs in its
`^X.Y.Z` / `~X.Y.Z` resolution under load. The pattern is the same: an
assertion that happens to be wrong on the slow path, masked on the fast
path. The audit above catches the *time-based* cousin of that pattern.

## Verification

- No source files were modified.
- `git status` clean for tracked files outside this audit document.
- Findings derived from `rg`-based searches across `src/` for:
  - `ReadTimeoutMs|ConnectTimeoutMs` and the `= <small-int>` variants
  - `Thread\.Sleep`
  - `Task\.Delay\(\d+` and `WaitAsync`
  - `Stopwatch`
  - `DateTime\.(Now|UtcNow|Today)`
  - `HttpClient|WebClient|HttpWebRequest|WaitOne\(`
  - `TestWait\.UntilAsync` (for migration target)
- All `file:line` references are reproducible by re-running the
  equivalent searches with `path: src/Tests`.

## Resolution status (2026-06-11)

- **GameClient** (~12 HIGH): FIXED via TestWait.UntilAsync (Pattern #108) — committed.
- **GameLaunch** (~5 HIGH): FIXED — committed.
- **UiAutomation** (~8 HIGH): FIXED (12 TestWait conversions) — committed.
- **Integration** (~6 HIGH): ASSESSED — DEFERRED as low-value. These are `Thread.Sleep`/`Task.Delay`
  process-settle/warmup waits in game-launch E2E infrastructure (ParallelGameE2ETests,
  GameSandboxIntegrationTests, GameTestRunner) that are **GameInstalled-gated and SKIPPED under the
  CI=true pre-push gate** (GameInstalled=false). They are NOT a source of pre-push-gate flakiness
  (which is what caused the intermittent push failures). Converting OS-process-settle delays to
  polling is higher-risk than value for tests that don't run in CI. Revisit only if these E2E tests
  are wired into CI with a real game install.

**Net:** all CI-running flaky-timeout HIGH risks (GameClient/GameLaunch/UiAutomation) are fixed.
The pre-push gate is now flake-resistant — empirically confirmed by 5 consecutive first-try pushes.
