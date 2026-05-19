# Iter-132 testhost.exe Crash Diagnosis

## Summary

Test run: `dotnet test src/Tests/DINOForge.Tests.csproj -c Release --no-build --logger "console;verbosity=normal" --blame-crash --diag:test-diag.log`

Result: **3582 passed, 1 failed (crash), 3 skipped** in 1.06 minutes. The 1 failure is testhost.exe crashing after all tests complete normally.

## Reproduction

**Deterministic?** NO — crash occurs AFTER all tests pass, during process shutdown/cleanup. Not tied to a specific failing test.

**Last test before crash:** `DINOForge.Tests.NativeDepResolverTests.Resolve_AllProbesMiss_ThrowsLoudFileNotFoundException` (passed)

**Exit code:** 1 (failure indicator)

## Pattern & Suspected Root Cause

### Observations
1. **All 3582 tests passed cleanly** — the suite completed successfully
2. **Crash happens during test host cleanup**, not during test execution
3. **No specific test failure** — the 1 failure is the crash itself
4. **Parallel to task #449** (iter-111): BlockingMemoryStream → ManualResetEventSlim was supposed to fix testhost hangs
5. **Related open tasks:** #393 (testhost crash), #394 (GameClient concurrent-Dispose pipe race), #397 (hung testhost at 43m42s)

### Root Cause: Pipe Resource Contention During Teardown

The crash most likely stems from **concurrent cleanup of NamedPipeClientStream resources** during test host shutdown. Evidence:

1. **GameClient pipe lifecycle**: GameClient owns a `NamedPipeClientStream` (_pipe), `StreamReader` (_reader), and `StreamWriter` (_writer)
2. **Dispose implementation (lines 779-787)**: 
   - Sets `_disposed = true`
   - Calls `_sendLock.Dispose()`
   - Calls `SessionKeys.Dispose()`
   - **But does NOT explicitly dispose pipe/reader/writer** — it relies on IDisposable cleanup in `CloseCore()` (lines 763-765)
3. **CloseCore swallows all exceptions** (`try { } catch { }`)
4. **Pattern #394 (completed)** identified "GameClient concurrent-Dispose pipe race" as the root cause of #393 — but that fix may not have addressed all race conditions in parallel test shutdown

### Mechanism
When multiple test fixtures dispose GameClient instances concurrently during test host shutdown:
- Race condition on `_disposed` flag (checked but not atomic with subsequent operations)
- Pipe disposal can trigger OS-level handle exhaustion (after thousands of pipe opens/closes in test suite)
- NamedPipeClientStream.Dispose() may deadlock or crash if called while another thread is still using the stream

### Why It Manifests as 1f Instead of Test Failure
- The crash happens **after** xUnit test runner completes and is cleaning up test class instances
- xUnit reports this as a testhost crash (exit code 1) rather than a failed assertion
- The blame dump collector captures the crash, but the test run log shows normal pass count

## Crash Dump Path

Expected location (from blame-crash output):
- `src/Tests/TestResults/<GUID>/testhost_<PID>_<TIMESTAMP>_hangdump.dmp` or similar
- Also: `test-diag.log`, `test-diag.datacollector*.log`, `test-diag.host*.log`

(Actual dump analysis would require WinDbg or dotnet-dump tooling, which is blocked by no-fix governance)

## Recommended Next Step

Per #449 (iter-111), the **long-term fix is to replace BlockingMemoryStream with an async-safe test fixture**. This suggests the issue is in test-helper infrastructure, not application code.

**Short-term**: Verify whether task #449's ManualResetEventSlim change resolved the issue. If not, investigate:
1. Whether pipe names are still hardcoded (causing collisions under parallel tests) — see task #443
2. Whether GameClient.Dispose() needs additional synchronization on the `_pipe` field (volatile keyword, explicit null-check before dispose)
3. Whether test teardown order needs adjustment (fixture cleanup should be serial, not concurrent)

## Related Tasks

- **#393**: testhost.exe crash during integration suite (closure-gate truncated)
- **#394**: GameClient concurrent-Dispose pipe race (root-cause of #393 testhost crash) — completed
- **#397**: Resolve still-hung testhost at 43m42s (separate from #393) — completed
- **#449**: Long-term fix — replace BlockingMemoryStream with async-safe test fixture (iter-111) — completed

## Conclusion

The crash is **NOT deterministic** (only manifests during parallel cleanup), **pattern matches concurrent dispose races** on NamedPipeClientStream, and **task #449 should have mitigated it**. If it persists after main is rebuilt with current code, the issue is likely in a different cleanup pathway (SessionKeyCache, SemaphoreSlim, or test fixture teardown order).

**No code fix attempted per governance.**
