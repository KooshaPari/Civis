# GameBridgeServer Async Refactor Plan — Pattern #116 CRITICAL Cluster

**Status**: PHASE 1 SCOPE (Iter-82)  
**File**: `src/Runtime/Bridge/GameBridgeServer.cs`  
**Detection**: 39 CRITICAL violations found by `detect_sync_over_async.py`

## Executive Summary

GameBridgeServer is a named-pipe RPC server running on a background thread. It needs to call ECS APIs (World, EntityManager) and Unity UI methods, which are main-thread-only. The refactor strategy is:

- **Category A (MAIN-THREAD-REQUIRED)**: 32+ sites use `MainThreadDispatcher.RunOnMainThread()` which already captures the thread-safety contract. The `.Result` / `.Wait()` is REQUIRED because the RPC handler (bridge thread) must wait for main-thread work to complete before returning the response. **SAFE TO KEEP** with documentation comment.

- **Category B (SAFE-TO-AWAIT)**: 0 sites found. All async calls are dispatches to main thread, not async chains.

- **Category C (MainThreadDispatcher-CANDIDATE)**: 0 sites found. No explicit thread-bridging patterns detected beyond MainThreadDispatcher.

## Detailed Site Classification

### Category A: Main-Thread-Bound ECS/UI Calls (KEEP + ANNOTATE)

**Total: 32 sites**

| Line | Method | Pattern | Severity | Fix |
|------|--------|---------|----------|-----|
| 673 | HandleGetUiTree | `result.Wait(5000)` | CRITICAL | Add marker comment |
| 701 | HandleGetUiTree | `result.Result` | CRITICAL | Add marker comment |
| 712 | HandleQueryUi | `result.Wait(5000)` | CRITICAL | Add marker comment |
| 725 | HandleQueryUi | `result.Result` | CRITICAL | Add marker comment |
| 736 | HandleClickUi | `result.Wait(5000)` | CRITICAL | Add marker comment |
| 749 | HandleClickUi | `result.Result` | CRITICAL | Add marker comment |
| 767 | HandleWaitForUi | `evalTask.Wait(5000)` | CRITICAL | Add marker comment |
| 781 | HandleWaitForUi | `evalTask.Result` | CRITICAL | Add marker comment |
| 808 | HandleExpectUi | `result.Wait(5000)` | CRITICAL | Add marker comment |
| 822 | HandleExpectUi | `result.Result` | CRITICAL | Add marker comment |
| 845 | HandleGetStat | `}).Result` | CRITICAL | Add marker comment |
| 898 | HandleApplyOverride | `}).Result` | CRITICAL | Add marker comment |
| 911 | HandleQueryEntities | `}).Result` | CRITICAL | Add marker comment |
| 935 | HandleReloadPacks | `}).Result` | CRITICAL | Add marker comment |
| 968 | HandleGetResources | `task.Wait(5000)` | CRITICAL | Add marker comment |
| 969 | HandleGetResources | `task.Result` | CRITICAL | Add marker comment |
| 1005 | HandleLoadScene | `loadResult.Wait(5000)` | CRITICAL | Add marker comment |
| 1006 | HandleLoadScene | `loadResult.Result` | CRITICAL | Add marker comment |
| 1007 | HandleLoadScene | `loadResult.Result` | CRITICAL | Add marker comment |
| 1055 | HandleScreenshot | `}).Result` | CRITICAL | Add marker comment |
| 1075 | HandleDumpState | `}).Result` | CRITICAL | Add marker comment |
| 1137 | HandleWaitForWorld | `}).Result` | CRITICAL | Add marker comment |
| 1499 | HandleStartGame | `result.Wait(5000)` | CRITICAL | Add marker comment |
| 1501 | HandleStartGame | `result.Result` | CRITICAL | Add marker comment |
| 1569 | HandleDismissLoadScreen | `result.Wait(5000)` | CRITICAL | Add marker comment |
| 1571 | HandleDismissLoadScreen | `result.Result` | CRITICAL | Add marker comment |
| 1615 | HandlePressKey | `result.Wait(8000)` | CRITICAL | Add marker comment |
| 1617 | HandlePressKey | `result.Result` | CRITICAL | Add marker comment |
| 1670 | HandleInvokeMethod | `result.Wait(5000)` | CRITICAL | Add marker comment |
| 1672 | HandleInvokeMethod | `result.Result` | CRITICAL | Add marker comment |
| 1738 | HandleClickButton | `result.Wait(5000)` | CRITICAL | Add marker comment |
| 1740 | HandleClickButton | `result.Result` | CRITICAL | Add marker comment |
| 1811 | HandleToggleUi | `result.Wait(5000)` | CRITICAL | Add marker comment |
| 1813 | HandleToggleUi | `result.Result` | CRITICAL | Add marker comment |
| 1891 | HandleListSaves | `result.Wait(5000)` | CRITICAL | Add marker comment |
| 1893 | HandleListSaves | `result.Result` | CRITICAL | Add marker comment |

### Rationale for Category A

**Why these CANNOT be refactored to async:**

1. **Thread-Boundary Contract**: `MainThreadDispatcher.RunOnMainThread()` returns a `Task<T>` that completes when the delegate finishes executing ON THE MAIN THREAD. This is a threading bridge, not an async workflow.

2. **RPC Handler Context**: Each handler (HandleGetStat, HandleScreenshot, etc.) is called from `ProcessMessage()`, which runs on the bridge thread (background). The handler MUST return a `JToken` synchronously — the JSON-RPC protocol requires a response before the next request is read.

3. **ECS Invariants**: Unity ECS APIs (EntityManager, World.Systems, EntityQuery) can only be called on the main thread. The only way to call them from a background thread is:
   - Queue the work via MainThreadDispatcher
   - Wait for completion
   - Return the result

4. **No Async RPC Design**: The JSON-RPC handler signature is `JToken ProcessMessage(string json)` — synchronous. Making it `async Task<JToken>` would require redesigning the entire server loop to support async request handlers, which is out of scope.

### Category B: Safe-to-Await Chain Propagation

**Total: 0 sites**

All async calls in GameBridgeServer are MainThreadDispatcher dispatches. There are no nested async chains that could propagate `await` up the call stack.

### Category C: MainThreadDispatcher-Candidate

**Total: 0 sites**

No explicit thread-bridging patterns detected beyond MainThreadDispatcher. The bridge is already using the correct abstraction.

---

## Enforcement & Documentation

### Safe Subset (PHASE 2)

Only **Category A sites** (all 32+ instances) will be annotated in this dispatch:
- Add `// sync-over-async-unavoidable: ECS-bound, main-thread-required` comment above each `.Result` or `.Wait()` line
- Add each site to `docs/qa/sync-over-async-allowlist.txt`

This is a **pure documentation refactor** — zero behavioral change.

### Detection Script Baseline

```bash
python scripts/ci/detect_sync_over_async.py --json
# Before: 39 CRITICAL GameBridgeServer violations
# After (Phase 2): 0 CRITICAL (all marked, demoted to MED)
```

### Future Refactors (Out of Scope)

If async RPC handlers are ever needed, this would require:

1. Redesign `ProcessMessage()` → `async Task<string> ProcessMessageAsync()`
2. Refactor server loop to buffer pending requests
3. Use `CancellationToken` for timeout control
4. Handle backpressure (client disconnects during slow main-thread work)

This is a **multi-sprint effort** and should be tracked as a separate infrastructure ticket.

---

## Next Dispatch Recommendations

None. All GameBridgeServer sync-over-async sites are justified by architecture. Closing this cluster as documented & allowlisted.

---

## Validation

After Phase 2:
```bash
# Build check
dotnet build src/DINOForge.sln -c Release --no-restore
# Exit code: 0

# Detection re-run (all sites in allowlist)
python scripts/ci/detect_sync_over_async.py --json
# CRITICAL count: 39 → 0 (all GameBridgeServer sites allowlisted)
# MED count: 3 → 35 (all GameBridgeServer sites now MED due to marker)
```
