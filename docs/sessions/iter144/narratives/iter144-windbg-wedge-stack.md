# Iter-144 Wedge Dump Analysis (WinDbg/cdb)

## Tool

- **Tool used**: `cdb.exe` (CLI WinDbg)
- **Path**: `C:\Program Files (x86)\Windows Kits\10\Debuggers\x64\cdb.exe`
- **Version**: `cdb version 10.0.26100.3624`
- **Install status**: WinDbg already installed via winget (`Microsoft.WinDbg` — no upgrade available; cdb shipped with Windows 10 SDK Debuggers x64)
- **Dump**: `C:\Users\koosh\Dino\docs\sessions\iter144-wedge-dump.dmp` (1.4 GB, full mem, captured by procdump64 `-ma -64`, PID 242516)
- **Process uptime at dump**: 17 seconds (process was wedged shortly after startup)
- **Full output**: `C:\Users\koosh\Dino\docs\sessions\iter144-wedge-windbg-analysis.txt` (74 KB, 1700+ lines)
- **!analyze bucket**: `BREAKPOINT_80000003_mono-2.0-bdwgc.dll!Unknown` (procdump-injected break, expected)

## Main Thread (TID 0 / 3b354.39790) — The Wedge

```
00 ntdll!NtWaitForMultipleObjects+0x14
01 KERNELBASE!WaitForMultipleObjectsEx+0x123
02 mono_2_0_bdwgc!mono_os_event_wait_multiple+0xe83
03 mono_2_0_bdwgc!mono_os_event_wait_multiple+0x165
04 mono_2_0_bdwgc!mono_thread_info_uninstall_interrupt+0x657
05 mono_2_0_bdwgc!mono_thread_set_manage_callback+0xdb
06 mono_2_0_bdwgc!mono_threads_set_shutting_down+0x7a0
07 mono_2_0_bdwgc!mono_jit_cleanup+0x6c
08 UnityPlayer+0x62df58       <- PlayerCleanup / Application::Cleanup
09 UnityPlayer+0x4af682       <- PlayerMain shutdown
10 UnityPlayer+0x6a40ea
11 UnityPlayer+0x6a30db
12 UnityPlayer+0x6a7a77
13 UnityPlayer+0x6a7a77 (UnityMain return path)
14 UnityPlayer!UnityMain+0xb
15 Diplomacy_is_Not_an_Option+0x11f2
16 kernel32!BaseThreadInitThunk
17 ntdll!RtlUserThreadStart
```

**Reading**: The main thread is inside Mono's `mono_jit_cleanup` -> `mono_threads_set_shutting_down`, blocked on `WaitForMultipleObjects` waiting for managed worker/finalizer threads to finish so the JIT runtime can be torn down. **The game is in shutdown** and one or more managed threads are refusing to exit, so the main thread hangs forever in this wait. This is the "gray-freeze on exit" / "OnDestroy deadlock" class of bug already tracked as task #547.

## Locked / Waiting Threads of Interest

**85 threads total** in the dump. Key non-trivial waiters:

### Thread 82 (3b354.38dd8) — SMOKING GUN: GameBridgeServer pipe accept thread

```
00 ntdll!NtFsControlFile+0x14
01 KERNELBASE!ConnectNamedPipe+0x76          <- blocking named-pipe accept
02..09  <managed JIT frames at 0x027f`xxxxxxxx — DINOForge.Bridge.GameBridgeServer.*>
10 mono_2_0_bdwgc!mono_jit_set_domain+0x5e8e
11 mono_2_0_bdwgc!mono_object_get_virtual_method+0x454
12 mono_2_0_bdwgc!mono_runtime_delegate_invoke+0x34d
13 mono_2_0_bdwgc!mono_profiler_init_etw+0x3d7d    (thread-start trampoline)
14 mono_2_0_bdwgc!mono_profiler_init_etw+0x3f3e
15 kernel32!BaseThreadInitThunk
```

This is a managed delegate thread parked inside a blocking `ConnectNamedPipe` syscall. It is the GameBridgeServer's pipe-accept loop. Because `ConnectNamedPipe` was issued without `OVERLAPPED` (synchronous), it cannot be interrupted by Mono shutdown — `mono_threads_set_shutting_down` cannot get this thread to acknowledge the shutdown request, so the main thread on stack #7 above stays in `WaitForMultipleObjects` forever.

### Thread 19 (3b354.3ae98) — "Finalizer"
```
ntdll!NtWaitForSingleObject
KERNELBASE!WaitForSingleObjectEx
mono_2_0_bdwgc!mono_os_event_wait_multiple+0x9da
mono_2_0_bdwgc!mono_os_sem_timedwait+0x30
mono_2_0_bdwgc!mono_gc_finalize_notify+0x834   <- finalizer queue sleep
```
Idle waiting for finalizer work. Not the wedge cause.

### Threads 4-18 ("AssetGarbageCollectorHelper") and 20-80
All parked in `WaitForSingleObjectEx` / `NtWaitForWorkViaWorkerFactory`. Standard idle Unity worker pool, nvwgf2umx driver helpers, and CLR ThreadPool workers. **None hold critical sections**, none are blocking the main shutdown directly. The main thread is exclusively waiting on the un-interruptible thread #82.

## Modules of Interest

```
00007ff7`812a0000 00007ff7`81345000   Diplomacy_is_Not_an_Option.exe
00007ff8`86d30000 00007ff8`87784000   mono-2.0-bdwgc.dll
00007fff`a1670000 00007fff`a336e000   UnityPlayer.dll
```

`DINOForge.Runtime.dll` does NOT show as a separately mapped native module — it's a managed assembly loaded into the Mono domain (JIT addresses `0x027e`/`0x027f` on thread 82 are AOT/JIT-compiled IL). No BepInEx native module appears in the loaded-module list; the BepInEx loader injects via Mono, not as a Win32 DLL load.

## ROOT CAUSE (1 sentence)

`GameBridgeServer`'s pipe-accept thread is parked inside a synchronous `ConnectNamedPipe(handle, NULL)` call, which Mono cannot interrupt during runtime shutdown, so `mono_threads_set_shutting_down` blocks the main thread in `WaitForMultipleObjects` indefinitely — this is task #547's "native deadlock in OnDestroy chain" and is triggered by the synchronous named-pipe wait, not by `UnloadUnusedAssets` or `ShutdownNonBridge`.

## Harmony Patch Target (concrete)

**File**: `src/Runtime/Bridge/GameBridgeServer.cs`

**Method**: the pipe accept loop (look for `ConnectNamedPipe` / `NamedPipeServerStream.WaitForConnection` / `WaitForConnectionAsync` inside `GameBridgeServer.AcceptLoopAsync` or the `Start()` / `RunListenerAsync` method — whichever method spawns the accept thread).

**Fix shape** (do NOT modify src per orchestrator rule, but the target shape):
1. Wrap accept in `WaitForConnectionAsync(CancellationToken)` (overlapped IO).
2. Drive the CT from a `ShutdownToken` exposed by the plugin's `OnDestroy()` so Mono shutdown can cancel the pending IO.
3. On `OnApplicationQuit` / `Plugin.OnDestroy`, call `Cancel()` then `_pipe.Dispose()` to force the kernel handle closed and unblock the syscall.
4. Surround the loop with `try { ... } catch (OperationCanceledException) { /* expected */ }` and ensure the listener thread joins within a bounded timeout (e.g. 500 ms) before plugin tear-down returns.

Pairs directly with task #547 (Gray-freeze H5 — native deadlock in OnDestroy chain). The wedge is `GameBridgeServer.AcceptLoop`, not `AssetSwapSystem.OnDestroy` (#534 already fixed) and not `RuntimeDriver` root-destroy (#535 isolated). This is the next layer of the same shutdown-deadlock onion.
