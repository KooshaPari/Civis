# Engine-UI Wedge — Native Dump Diagnosis (iter-144 methodology) — 2026-05-29

## Verdict (TL;DR)

**This is NOT a native kernel-syscall wedge. It is NOT a recurrence of the iter-144
`mono_jit_cleanup → WaitForMultipleObjectsEx` gray-freeze.**

The full-memory minidump proves the process is **healthy**: the main thread is parked in the
**normal Unity main loop** (`UnityMain → WaitForSingleObjectEx`), not in any teardown / Mono
cleanup / pipe wait. There is **no DINOForge/BepInEx/Harmony managed thread present at all** in
the dump — the plugin's worker threads have already **returned/exited cleanly**. No thread is in
`ConnectNamedPipe`, `NtFsControlFile`, or a thread-`Join`.

The observed "wedge" (debug-log mtime frozen at the `[RuntimeDriver] OnDestroy: returning to
Unity` line while the process stays alive + Responding=True) is a **managed-side dormancy**, not a
native deadlock: after the `InitialGameLoader` scene's `OnDestroy` runs to completion, the plugin
returns control to Unity and is **never called again** — the RuntimeDriver does not resurrect and
the ResurrectionFallback heartbeat thread stops emitting. The engine keeps rendering MainMenu fine;
the plugin is just silently inert.

## Capture details

| Item | Value |
|---|---|
| Deployed DLL | `33A98D0E` (probes-disabled build, as specified) |
| Repro | Deterministic — wedge signature appears ~20-45s after launch every run |
| Wedged PID (dumped) | 606712 |
| Wedge confirmation | log tail = `...returning to Unity`, mtime stale ≥22s, process alive + Responding=True |
| Dump path | `G:\dino-dumps\dino-606712-202521.dmp` |
| Dump size | 8137.7 MB (full memory, `MiniDumpWithFullMemory`) |
| Capture method | `MiniDumpWriteDump` P/Invoke (`scripts/game/write-dump.ps1`) |
| Debugger | `cdb.exe` 10.0.26100.3624 (Windows Kits 10 x64) |
| Stack dump | `G:\dino-dumps\cdb-allstacks-606712.txt` (1581 lines, `~* k 50`) |

### Capture-tooling notes (why P/Invoke, not procdump/comsvcs)
- `procdump` not installed.
- `rundll32 comsvcs.dll,MiniDump` produced **corrupt** dumps ("Minidump does not have system
  info", HRESULT 0x80004005) because a **parallel bisect agent was dumping the same game
  concurrently** — comsvcs interleaved writes (one file grew to 8.2 GB). It also wrote files with a
  restrictive DACL (SYSTEM + Administrators only), causing "Access is denied" from the
  non-elevated shell.
- Switched to `MiniDumpWriteDump` via P/Invoke with a **self-owned file handle** (own DACL, no
  concurrency clash) writing to **G:** (C: had filled to 0 bytes from the multi-GB dumps). This
  produced a clean, debuggable dump on the first try.

## Thread analysis

94 threads total. Module histogram of all return frames:

```
297 ntdll!     226 nvwgf2umx!   144 kernel32!   139 KERNELBASE!
 55 UnityPlayer!  24 fmodstudio!   5 steamclient64!   5 mono_2_0_bdwgc!   4 combase!   1 mswsock!
```

**Zero** `DINOForge!` / `BepInEx` / `Harmony` / `0Harmony` / `dobby` / `System.IO.Pipes` /
`NamedPipe` / `Thread.Join` frames anywhere. Only **5 mono frames total**, all on a single idle
GC-finalizer thread.

### Thread 0 — the MAIN / teardown thread (the one iter-144 found wedged)

```
ntdll!NtWaitForSingleObject+0x14
KERNELBASE!WaitForSingleObjectEx+0xae
UnityPlayer+0x57a92c
UnityPlayer+0x57a71c
UnityPlayer+0x4572e6
UnityPlayer+0x4595a6
UnityPlayer+0x4ac4ef
UnityPlayer+0x49f05a
UnityPlayer+0x49f23a
UnityPlayer+0x6a43b9
UnityPlayer+0x6a30db
UnityPlayer+0x6a7a77
UnityPlayer!UnityMain+0xb
Diplomacy_is_Not_an_Option+0x11f2
kernel32!BaseThreadInitThunk+0x17
ntdll!RtlUserThreadStart+0x20
```

This is the **normal Unity main-loop idle wait** (event/present pump under `UnityMain`). In
iter-144 this same thread was instead in `mono_jit_cleanup → WaitForMultipleObjectsEx`. Here it is
**not in Mono and not in teardown** — it has fully returned to Unity's loop. This is why the
process stays Responding=True and keeps rendering MainMenu.

### The only Mono thread — idle GC finalizer (not blocking anyone)

```
mono_2_0_bdwgc!mono_os_event_wait_multiple+0x9da
mono_2_0_bdwgc!mono_os_sem_timedwait+0x30
mono_2_0_bdwgc!mono_gc_finalize_notify+0x834
mono_2_0_bdwgc!mono_profiler_init_etw+0x3d2b   (finalizer thread entry)
```

Normal idle finalizer wait. Not holding any lock the main thread needs.

### All other threads
Idle pool waits: 132 threads in `NtWaitForSingleObject`, 4 in `NtWaitForMultipleObjects` — all are
NVIDIA driver (`nvwgf2umx`), Unity job workers (`UnityPlayer`), FMOD audio, Steam client, COM/RPC
(`combase`), and one winsock (`mswsock!SockAsyncThread`). None are DINOForge code; none are blocked
on a plugin handle.

## What the log says (corroborates the dump)

This run's tail (mtime frozen 03:24:51Z):

```
OnActiveSceneChanged: old='' new='InitialGameLoader'
[EventSystem] reconcile ... DINOForge_EventSystem_Restored
KeyInputSystem.RecreateInCurrentWorld ... Registered in 'Default World'
OnSceneLoaded: name='InitialGameLoader' buildIndex=0 mode=2
KeyInputSystem.OnDestroy ...
[AssetSwapSystem] OnDestroy SKIPPED bundle unload (NeedsResurrection=True)
[RuntimeDriver] OnDestroy: GameBridgeServer.RequestShutdown() invoked (sync pipe unwedge).
[RuntimeDriver] OnDestroy: background poll stopped ... BridgeServerThreadAlive=False.
[RuntimeDriver] OnDestroy: metrics snapshot written ...
[RuntimeDriver] OnDestroy: returning to Unity (resurrection flags set, fallback thread will revive).
<<< LOG ENDS — no further heartbeat >>>
```

Just before the end, the fallback thread WAS healthy and looping:

```
03:24:45 ResurrectionFallback: loop entered.
03:24:47 ResurrectionFallback heartbeat #4 NeedsRes=False NeedsDefRes=False rootNull=False
03:24:49 ResurrectionFallback heartbeat #8 NeedsRes=False NeedsDefRes=False rootNull=False
```

Key observations:
1. `BridgeServerThreadAlive=False` — the bridge/pipe server thread was **already stopped before
   OnDestroy** in this probes-disabled build. So the iter-144 `ConnectNamedPipe` culprit thread
   **does not exist here** (confirmed by the dump: no pipe thread).
2. `RequestShutdown()` (the iter-144 fix that disposes the pipe handle synchronously) ran and
   returned — `OnDestroy` completed all the way to "returning to Unity".
3. The ResurrectionFallback heartbeats report `NeedsRes=False` right up to OnDestroy. After
   OnDestroy returns, **no heartbeat #12+ is ever emitted** and the dump shows **no managed thread
   left alive** → the fallback/resurrection thread has **exited**, and nothing re-arms it.

## Root cause (revised — this is a different bug from iter-144)

The teardown does **not** deadlock natively. The actual failure is a **managed lifecycle gap**:

- On the `InitialGameLoader → MainMenu` scene transition, the RuntimeDriver's `OnDestroy` tears
  down (stops poll, RequestShutdown, snapshot) and **returns**, expecting the ResurrectionFallback
  thread (or a scene-changed/OnEnable callback) to **revive** the driver in the new scene.
- In this build that revival **never happens**: the fallback thread stops heartbeating after the
  OnDestroy (it has exited per the dump), `NeedsRes` is never set True for this transition, and no
  PlayerLoop/OnEnable callback re-attaches the driver. The plugin goes **permanently dormant**.
- Because the engine itself is unaffected, the process stays alive + Responding=True and reaches
  MainMenu — which is exactly the "alive but plugin-dead" symptom that was mis-classified as the
  iter-144 native gray-freeze.

The "frozen log mtime past OnDestroy while alive" heuristic used to confirm an iter-144 wedge is
**not specific** — it also fires for this benign-engine / dormant-plugin case. The native dump is
what disambiguates them.

## Fix recommendations

1. **Primary — fix the resurrection lifecycle, not a native wait.** The ResurrectionFallback
   thread must NOT exit at/after `OnDestroy`; it must remain alive across the scene transition and
   actively re-create the RuntimeDriver in the new (MainMenu) scene. Verify:
   - the fallback thread is started once and `IsBackground=true`, looping on a durable flag, and is
     never joined/aborted from `OnDestroy`;
   - `NeedsResurrection` is actually **set True** on the `InitialGameLoader → MainMenu` `OnDestroy`
     (the heartbeats show it staying False — so the arm-condition is mis-gated for this transition);
   - resurrection is **also** driven from `SceneManager.activeSceneChanged` (reliable in DINO)
     rather than relying solely on the fallback poll, so revival is event-driven not race-prone.

2. **No native interruptibility change is needed** for THIS dump — there is no synchronous kernel
   wait to break. (The iter-144 `RequestShutdown()` pipe-handle dispose is already present and
   already returning cleanly; `BridgeServerThreadAlive=False` means there is no pipe-accept thread
   to interrupt in this probes-disabled build.)

3. **Improve the wedge classifier.** The "log-mtime stale + alive + Responding" probe cannot tell a
   native deadlock from a dormant-plugin. Add a discriminator: a heartbeat the engine ALSO drives
   (e.g. a PlayerLoop-marker counter file) — if it keeps advancing while the plugin log is frozen,
   it's a dormant-plugin lifecycle bug (this case); if everything is frozen, it's a native wedge
   (iter-144 class).

## Artifacts
- Dump: `G:\dino-dumps\dino-606712-202521.dmp` (8.1 GB, full memory)
- All-thread native stacks: `G:\dino-dumps\cdb-allstacks-606712.txt`
- Modules + uniqstack: `G:\dino-dumps\cdb-targeted.txt`
- Capture helper: `scripts/game/write-dump.ps1`

---

## iter-149e LIVE FOLLOW-UP (2026-05-29) — root mechanism CONFIRMED beyond the dump

Branch `fix/engine-ui-injection-race-20260529`. Three deploy/relaunch cycles on
DLLs `BDF954FB`, `60356F4B`, `3af573fd` (differ from prior `0CEC4650`/`94ff7dcc`).
Engine heartbeat (`BepInEx/dinoforge_heartbeat.txt`) added this session.

### What the live runs proved (corroborates + sharpens the dump)
1. The engine is healthy every run: process ALIVE, Responding=True, ~4.1–4.3 GB WS,
   title="Diplomacy is Not an Option", **MainMenu fully rendered** (screenshots
   `docs/screenshots/engine-ui-iter149e-PREFIX-*.png`,
   `docs/screenshots/engine-ui-FIXED-iter149e-20260529.png`) — but **NO MODS button**.
2. The plugin goes **totally dormant** at `[RuntimeDriver] OnDestroy: returning to
   Unity` during the **InitialGameLoader** phase. After that line:
   - **NO MainMenu `sceneLoaded`/`activeSceneChanged` ever reaches our static handlers**
     (every run shows only `''`→`InitialGameLoader`, never `MainMenu`).
   - **NO second `KeyInputSystem.OnCreate`** — DINO creates exactly ONE ECS world
     (`Default World` during InitialGameLoader); MainMenu is **non-ECS**, so the
     OnCreate revive hook never fires for it.
   - **NO further `PlayerLoop.SetPlayerLoop` postfix** — DINO does not rebuild the
     PlayerLoop again in the MainMenu-settled window.
   - **The ResurrectionFallback bg thread STOPS.** Decisive new evidence: with the
     Unity ECall removed AND a per-iteration engine-heartbeat write added, the
     heartbeat froze at `27 … fallback#11` for 100 s straight. The thread does pure
     managed work (flag read + DateTime + two `File.WriteAllText`) and STILL halts.
   - BepInEx's own `LogOutput.log` also stops at the same instant.
3. **Conclusion (definitive):** Mono **suspends / stops scheduling DINOForge's
   background threads** across the InitialGameLoader→MainMenu asset load and never
   resumes them, AND DINO invokes **zero** managed callbacks of ours after the
   teardown. There is **no execution vehicle left** for any resurrection path. This
   is exactly the dump's "worker threads gone / no DINOForge frame" state, now
   reproduced live with an instrumented counter.

### Why resurrection is the wrong layer (next-session pivot)
The RuntimeDriver is attached to `DINOForge_Root` (DontDestroyOnLoad +
HideAndDontSave). It is **destroyed anyway** during the InitialGameLoader scene
unload — Unity tears DontDestroyOnLoad objects via the **native scene-unload path
(`DestroyGameObjectHierarchy`), NOT managed `Object.Destroy`**, so `DestroyGuardPatch`
(which only patches managed `Object.Destroy`/`DestroyImmediate`) never fires for the
real teardown. Once destroyed, nothing of ours runs again → permanent dormancy.

**The fix must PREVENT the teardown, not resurrect after it.** Candidate directions
for the next iteration (each must be verified live, not by build-green):
- Patch the native-routed destroy the way iter-146 did when the MODS button last
  worked (regression bisect `cafd2b70` ← current). Find what changed so the root
  stopped surviving.
- Re-parent `DINOForge_Root` under a DINO-owned object that survives
  InitialGameLoader→MainMenu, or recreate it from a Harmony postfix on a DINO
  MainMenu bootstrap method (a callback DINO actually invokes at MainMenu).
- Investigate whether `HideFlags.HideAndDontSave` is what makes Unity reap the root
  (HideAndDontSave objects are eligible for cleanup in some unload paths); try plain
  DontDestroyOnLoad without HideAndDontSave.

### Changes landed this session (do NOT revert — they are correct hardening + the
### classifier the diagnosis asked for, even though they don't revive at MainMenu)
- `ResurrectionFallbackLoop`: removed the bg-thread Unity ECall
  (`ResurrectionSucceeded()`→`GetComponent`); pure managed mark+re-arm only.
- `OnSceneLoaded`/`OnActiveSceneChanged`: revive-first ordering; EventSystem/world
  fixups made best-effort so they can't gate the revive.
- `MainThreadReviveIfNeeded`: resets `_resurrectionAttempts` per scene event so a
  loader-phase cap exhaustion can't poison a later revive.
- `KeyInputSystem.OnCreate`: no longer prematurely clears `NeedsResurrection`; revives
  directly on the main thread when `Plugin.ResurrectionParamsReady`.
- `OnPlayerLoopSet`: drives `MainThreadReviveIfNeeded` on the main thread.
- **Engine heartbeat** (`BumpEngineHeartbeat` → `dinoforge_heartbeat.txt`) from scene
  events, PlayerLoop, and the fallback loop — the wedge classifier the diagnosis
  requested. A frozen counter + alive/Responding process = dormant-plugin (this
  class); all-frozen = native wedge (iter-144 class). **Validated live this session.**

### Status: MODS button NOT yet up. Resurrection cannot fire (no execution vehicle).
Commits on branch: `ed985450`, `43c350fc`, `73f413c0`, `d5038cb6`. No PR (gated on
verified modsButton=True). Game left killed/clean.
