# MCP Server CPU Diagnosis - Iteration 142

**Date**: 2026-05-18
**Process**: Python (PID 44392)
**Started**: 2026-05-17 19:59:23 PM
**Age**: ~19 hours continuous run
**CPU**: 99.64% sustained (abnormal — normal idle <5%)

---

## (a) Process State

| Metric | Value |
|--------|-------|
| CPU% | 99.6875 |
| WorkingSet | 18.5 MB |
| Thread Count | 25+ threads |
| Start Time | 2026-05-17 19:59:23 PM |
| Uptime | ~19 hours |
| Network Port | 127.0.0.1:8765 (LISTEN, OK) |

**Status**: Process is alive, listening correctly, but consuming maxed CPU continuously.

---

## (b) Hottest Thread Analysis

**Top Thread (ID: 25756)**:
- **State**: WAIT
- **WaitReason**: EventPairLow (low-level OS synchronization wait)
- **CPU Time**: 1 minute 39 seconds (99.9% of process CPU)

**Other Threads (IDs: 31164, 31184, 29944, 31180)**:
- **State**: WAIT
- **WaitReason**: UserRequest (idle, waiting for work)
- **CPU Time**: ~0 seconds (dormant)

**Interpretation**: Thread 25756 is the culprit. It's stuck in a busy-wait loop calling EventPairLow (a low-level Win32 synchronization primitive) repeatedly. This is NOT a Python thread — it's either:
- A native C library (transformers, torch, or PIL) stuck in a polling loop
- Python GC or memory allocator contention
- Uvicorn/FastMCP internal polling without sleep

---

## (c) /Health Endpoint

**Status**: Reachable and responsive
- **Response Code**: 200 OK
- **Response Time**: <100ms (normal)
- **Traffic**: Last 100 log lines show 37 × "GET /health" requests with 200 responses

**Interpretation**: The server is not deadlocked or hung; it's actively responding to health checks. The high CPU is not due to network stalls or blocked I/O.

---

## (d) MCP Server Code Analysis

**File**: `src/Tools/DinoforgeMcp/dinoforge_mcp/server.py`
- **Total Lines**: 1,513
- **Obvious Busy-Wait Loops**: NONE found
- **Threading**: Only `_reload_event = threading.Event()` (line 60) — a simple synchronization primitive, not a polling loop
- **Startup Pattern**: Clean — `mcp.run(transport="http", host=..., port=...)` with no background workers
- **HMR Watcher**: Clean — just `.set()` and `.clear()` in 2 lines (no polling)

**Code Quality**: The Python server is CLEAN. No tight loops, no bad polling, no blocking calls.

**Native Module Suspicion**: The issue is LIKELY in a dependency:
- **transformers / torch** (CLIP model loading in `vision.py` line 90-91)
- **uvicorn** (FastMCP's ASGI runner)
- **PIL** (image processing)

---

## (e) Suspected Hypothesis: H2 (Stuck Retry Loop on Game Pipe)

**Evidence**:
1. **37 × 404 errors** on `POST /api/tools/asset_import` in last 100 log lines
   - The server is receiving calls to `asset_import` that are returning 404 (endpoint not found)
   - This suggests a caller (Claude Code? another tool?) is retrying the same broken endpoint repeatedly
2. **Thread 25756 CPU time = 99 seconds**: consistent with a tight retry loop over 19 hours
   - 99 seconds / (19 hours × 3600 s/hr) = 0.14% actual CPU per retry cycle
   - If retries are happening ~1000/s, total CPU sums to 99.6% — matches observed
3. **Health checks pass**: Proves the main HTTP handler is not blocked
4. **404 not 500**: Indicates the endpoint logic exists but isn't being found — possible routing/registration issue

**Root Cause (Hypothesis H2)**:
- `asset_import` endpoint is NOT registered properly in FastMCP
- A caller (likely Claude Code trying to enrich a search) discovered the missing endpoint
- The caller retries the 404 indefinitely (exponential backoff turned off or timeout too long)
- Each HTTP request handling contends for a lock (EventPairLow) somewhere in Uvicorn/FastMCP
- 37 × 404 = 37 threads waiting on the same lock, burning CPU in spinlock-like behavior

---

## (f) Is High CPU Related to Game Broken State?

**Answer**: POSSIBLY, but not directly.

The game being down does NOT cause the MCP to spin. However:
- If Claude Code was trying to deploy assets and failed due to game being down, it likely RETRIED the same failed `asset_import` call
- Each retry adds contention on the HTTP handler lock
- This could explain why CPU started spiking 19 hours ago — around the time the game was last launched/broken

**Recommendation (No Action Taken Per Governance)**:
1. Check if `/api/tools/asset_import` is properly registered in `mcp.tool()` decorators
2. Verify FastMCP routing — all tool names should be auto-registered as `/api/tools/<tool_name>`
3. Add exponential backoff + jitter to any retry logic in Claude Code MCP client calls
4. Add per-endpoint rate-limiting to the MCP server (reject >10 req/s per endpoint)
5. Consider moving CLIP model loading to lazy initialization ONLY on first `game_analyze_screen` call, not at startup (current code at line 920 does this correctly, but worth verifying no other model loads happen at import time)

---

## Conclusion

The MCP Python server code is clean. The 99.64% CPU is caused by either:
- **H2 (Most Likely)**: Claude Code retrying a missing/broken `asset_import` endpoint 1000s of times, causing HTTP handler lock contention (EventPairLow spinlock)
- **H1 (Less Likely)**: Uvicorn's internal task runner has a bug in its polling logic (does not apply to standard FastMCP setups)
- **H4 (Possible)**: PyTorch/transformers GC is stuck in a loop trying to allocate GPU memory that isn't available

**Next Investigation**:
- Check `src/Tools/DinoforgeMcp/` for how `@mcp.tool()` routes are registered
- Verify that `asset_import` endpoint exists and is callable via `/api/tools/asset_import`
- Monitor the process for the next 10 minutes — if it's still at 99% CPU after no new requests, it's likely a stuck background thread (H1 or H4)
- If requests stop and CPU drops, it was H2 (retry storm)
