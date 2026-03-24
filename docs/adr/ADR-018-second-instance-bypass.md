# ADR-018: Second Instance Detection and Bypass

**Status**: Accepted — RESOLVED 2026-03-24
**Date**: 2026-03-24
**Deciders**: kooshapari, DINOForge agents
**Supersedes**: Extends ADR-013 (Duplicate Instance Detection & Win32 Mutex Bypass) with BepInEx-layer options

---

## Context

Diplomacy is Not an Option (DINO) enforces a single-instance constraint and presents a fatal error dialog:

```
Another instance is already running. Only one instance of the game is allowed.
```

This constraint blocks DINOForge testing workflows that need to relaunch the game (e.g., after a BepInEx plugin reload, a pack deploy, or a test-swap cycle). Because the game uses DINO's own Unity-ECS player loop (not the standard Unity loop), BepInEx runs inside the process, and the game's single-instance check fires before BepInEx's Awake phase completes. The error dialog appears, the process exits, and automated test workflows fail.

### Runtime Context

DINOForge's `Plugin.cs` loads as a standard BepInEx `BaseUnityPlugin`. Key constraints:

- **Awake() fires synchronously** during BepInEx bootstrap, before DINO's own scene loader runs.
- **MonoBehaviour.Update() never fires** — DINO replaces Unity's PlayerLoop entirely.
- **KeyInputSystem** (ECS `SystemBase`) is the only reliable execution context for recurring logic.
- **Background threads** must not call `Resources.FindObjectsOfTypeAll` — it deadlocks during asset loading.
- The game binary is `Diplomacy is Not an Option.exe`, built for Unity 2021.3.45f2, Mono runtime.

### Known Execution Timeline (where the error occurs)

```
Steam.exe launches game.exe
  └─ game.exe CRT startup
       └─ Unity player init (UnityPlayer.dll)
            └─ [HERE] Single-instance mutex check fires → fatal error dialog → process exit
                 └─ (BepInEx Preloader runs BEFORE this if doorstop is configured early enough)
                 └─ BepInEx Awake / Plugin bootstrap (never reached if mutex check fails)
```

The exact position of the mutex check relative to BepInEx's doorstop hook is the central open question. If doorstop (the BepInEx injector, `winhttp.dll` proxy) intercepts control before `UnityPlayer.dll` initializes, a preloader patcher can release the mutex before the check runs. If the check fires inside native code before any managed entry point, no managed approach can intercept it.

### What is Known About the Lock Mechanism

DINO is a Steam game. Steam games commonly use one or more of:

1. **Win32 named mutex** — `CreateMutexW` in global or session namespace. Most common for Unity/Steam games.
2. **Steam's own mutex** — Steam client holds `Global\Valve_SteamMutex` and related handles while the game is registered as running.
3. **TCP port bind** — Game binds a port; a second instance fails to bind and exits.
4. **Temp file lockfile** — An exclusively-locked file in `%TEMP%` or `%LOCALAPPDATA%`.

The AppData directory for DINO (`C:\Users\koosh\AppData\LocalLow\Door 407\Diplomacy is Not an Option\`) was inspected on 2026-03-24. Its root contains:

```
CurrentSettings.json
DNOPersistentData/
DNOWorkshop/
Player.log
Player-prev.log
Unity/
```

No `.lock`, `.pid`, or other explicit lockfile was found at the root. Subdirectory contents of `DNOPersistentData/` and `Unity/` were not fully inspected — those directories may contain per-session lock artifacts.

The game installation directory (`C:\Program Files (x86)\Steam\steamapps\common\Diplomacy is Not an Option\`) was inaccessible at investigation time — likely installed to a non-default Steam library path (confirmed: `G:\SteamLibrary\` based on `launch-game.md`).

### Existing Workaround (launch-game.md)

The current `/launch-game` skill:
1. Kills the game process via `Stop-Process`.
2. Clears the debug log.
3. Launches the `.exe` directly (bypassing the Steam launcher, which avoids Steam's own mutex).

This works when the previous game process terminates cleanly and the OS releases its mutex handles before the next launch. It fails intermittently when the kernel mutex table has not yet been flushed, or when the game process hung before exit and mutex handles were not properly abandoned.

---

## Options Considered

### Option A: Named Mutex Release via BepInEx Preloader Patcher

**Mechanism**: BepInEx's preloader system runs managed C# code via a `doorstop` DLL proxy (`winhttp.dll`) before `UnityPlayer.dll` completes initialization. Preloader patchers (assemblies in `BepInEx/patchers/`) are loaded at this stage. A patcher can call Win32 APIs — including `OpenMutex` / `ReleaseMutex` — before the game's single-instance check fires.

**Implementation sketch**:

```csharp
// In BepInEx/patchers/DINOForge.Patcher.dll
using System.Runtime.InteropServices;

public static class SecondInstancePatcher
{
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode)]
    static extern IntPtr OpenMutex(uint desiredAccess, bool inheritHandle, string name);

    [DllImport("kernel32.dll")]
    static extern bool ReleaseMutex(IntPtr hMutex);

    [DllImport("kernel32.dll")]
    static extern bool CloseHandle(IntPtr hObject);

    const uint MUTEX_MODIFY_STATE = 0x0001;
    const uint SYNCHRONIZE = 0x00100000;

    public static void Initialize()
    {
        // Candidate mutex names — must be confirmed via Handle.exe investigation
        var candidates = new[]
        {
            "DiploNotAnOption_SingleInstance",
            "Global\\DiploNotAnOption_SingleInstance",
            "Diplomacy is Not an Option",
            "Global\\Valve_SteamMutex",
        };

        foreach (var name in candidates)
        {
            IntPtr handle = OpenMutex(MUTEX_MODIFY_STATE | SYNCHRONIZE, false, name);
            if (handle != IntPtr.Zero)
            {
                ReleaseMutex(handle);
                CloseHandle(handle);
                Logger.LogInfo($"[DINOForge.Patcher] Released mutex: {name}");
            }
        }
    }
}
```

**Critical caveat**: `ReleaseMutex` on Windows requires that the calling thread be the one that currently owns (acquired) the mutex. If the mutex is owned by the Steam process or a previous game process, a non-owning caller will receive `ERROR_NOT_OWNER` (0x0120) and the call fails silently. The practical effect: this approach only works if the mutex is in an **abandoned** state (owning process died without releasing it). In that case, the OS marks it abandoned and any waiting thread can acquire it — but `ReleaseMutex` from an unrelated process still fails. What actually works in the abandoned case is simply calling `CreateMutex` with the same name: the OS re-creates or re-opens it and assigns ownership to the calling thread.

**Alternative within this option**: Instead of releasing the existing mutex, the patcher calls `CreateMutex(null, true, mutexName)` which will either create a new mutex (if none exists) or open the existing one. If `GetLastError()` returns `ERROR_ALREADY_EXISTS`, the game's check will still fail. However, if the patcher runs early enough and creates the mutex first, the game's own `CreateMutex` call returns `ERROR_ALREADY_EXISTS` to the game — which triggers the "another instance" error. This is the wrong approach.

The correct approach for the patcher is: open the existing mutex, check if it is abandoned (via `WaitForSingleObject` with timeout=0 returning `WAIT_ABANDONED`), then release it so the game can re-acquire it cleanly.

**Feasibility gate**: This option is only viable if the BepInEx doorstop hook runs before the game's `CreateMutex` call. This must be confirmed empirically.

**Pros**:
- Most surgical approach — no process killing required
- Runs transparently before game logic
- No user interaction needed

**Cons**:
- Requires knowing the exact mutex name
- Win32 ownership semantics may prevent release from non-owning thread
- Preloader is more invasive to ship (adds a BepInEx patcher assembly)
- May break if DINO updates and changes mutex name

---

### Option B: Process Detection Before Launch (in MCP/CLI Tooling)

**Mechanism**: Before launching the game, the `launch-game` skill and/or the MCP `game-test` tool checks whether the game process is already running. If it is, it kills it and waits for full cleanup before launching.

The current `launch-game.md` already does this partially (it calls `Stop-Process`), but the sleep between kill and launch may be insufficient for kernel mutex cleanup. Enhancements:

1. After `Stop-Process`, poll for the process handle to disappear entirely (not just request sent).
2. After process handle disappears, wait an additional configurable grace period (default 5 seconds).
3. Attempt the launch and detect the error dialog (via process exit code or window title).
4. If the error dialog appears, retry with progressively longer waits (exponential backoff, max 30 seconds).

**PowerShell implementation sketch**:

```powershell
function Wait-GameExit {
    param([int]$TimeoutSeconds = 15)
    $deadline = [DateTime]::Now.AddSeconds($TimeoutSeconds)
    Stop-Process -Name "Diplomacy is Not an Option" -Force -ErrorAction SilentlyContinue
    while ([DateTime]::Now -lt $deadline) {
        $proc = Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue
        if (-not $proc) { break }
        Start-Sleep -Milliseconds 500
    }
    # Additional mutex drain grace period
    Start-Sleep -Seconds 5
}
```

**Pros**:
- No BepInEx patcher assembly required
- Entirely in tooling layer — no game binary or plugin changes
- Works even if mutex name is unknown
- Reliable for the common case where the game died cleanly

**Cons**:
- Does not address the kernel mutex hold time if a process was forcibly killed
- Race condition remains: OS may not release mutex even after process handle is gone
- Does not help if Steam is holding its own mutex separately

---

### Option C: Lockfile Detection and Deletion

**Mechanism**: Identify any file-based lock used by DINO or Unity and delete it before launching.

Unity games built with `Application.persistentDataPath` sometimes write a session lock file. Common locations:

- `%APPDATA%\..\LocalLow\Door 407\Diplomacy is Not an Option\` (confirmed path)
- `%TEMP%\` (Unity editor uses this; standalone builds may too)
- Game installation root

If a lockfile is found, delete it before launch. This requires identifying the exact file.

**Investigation required**: No lockfile was found in `C:\Users\koosh\AppData\LocalLow\Door 407\Diplomacy is Not an Option\` at the root level. Subdirectories (`DNOPersistentData/`, `Unity/`) need inspection. Also check `%TEMP%` for files named after DINO at launch time.

**Pros**:
- Very simple if a lockfile is confirmed to exist
- No P/Invoke, no mutex semantics, no race conditions
- Deterministic — delete the file and it's gone

**Cons**:
- May not be the mechanism DINO actually uses (mutex is more likely)
- Deleting certain lock files could corrupt save data if game writes to them
- Must confirm which file to delete without causing data loss

---

### Option D: Accept Limitation — Document and Require Manual Game Restart

**Mechanism**: Accept that automated relaunch is not always reliable and document the manual procedure. The test workflow becomes:

1. Agent deploys pack / plugin changes.
2. Agent notifies user: "Please restart the game manually."
3. User restarts game.
4. Agent proceeds with post-launch verification.

**Pros**:
- Zero implementation risk
- No mutex or lockfile analysis required
- Does not depend on OS internals

**Cons**:
- Breaks the autonomous testing loop — requires human in the loop for every test cycle
- Violates the agent-testing mandate ("never ask user to launch/interact with game")
- Acceptable only as a temporary workaround while investigation proceeds

---

## Decision (TBD — Pending Investigation)

No decision has been made. The approach depends on:

1. **Whether the BepInEx doorstop hook fires before the game's mutex check** — determines if Option A is viable at all.
2. **The exact mutex name** — required to implement Option A and to write the PowerShell release script for Option B.
3. **Whether a lockfile exists** — determines if Option C is viable.

**Tentative preference order** (subject to investigation results):

1. Option B (process detection + extended wait) as the immediate no-risk improvement
2. Option A (preloader patcher) if mutex name is confirmed and doorstop timing allows
3. Option C (lockfile) if a lockfile is found to exist
4. Option D as fallback of last resort only

---

## Investigation Needed

The following steps must be completed before any implementation begins. See WORK-002 for the tracked work item.

### Step 1: Identify the Mutex Name

Use Sysinternals `Handle.exe` while the game is running:

```powershell
# Download Handle.exe from https://docs.microsoft.com/en-us/sysinternals/downloads/handle
# Run while game is running:
.\handle.exe -p "Diplomacy is Not an Option" -a | Where-Object { $_ -match "Mutant" }
```

Or use PowerShell's `Get-Process` + kernel handle enumeration via WMI:

```powershell
$proc = Get-Process "Diplomacy is Not an Option"
# Enumerate handles via NtQuerySystemInformation P/Invoke (advanced)
```

Alternatively, use Process Explorer (Sysinternals) → Lower pane → Handles → filter by type = Mutant.

### Step 2: Confirm Doorstop Timing

Launch the game with a preloader patcher that writes a log line immediately on `Initialize()`. Compare the timestamp against the game's error dialog appearance. If the patcher log appears before the dialog, Option A is viable.

### Step 3: Check for Lockfile

```powershell
# Run immediately after launching game — capture new files
$before = Get-ChildItem -Recurse "C:\Users\koosh\AppData\LocalLow\Door 407\" | Select-Object FullName, LastWriteTime
# Launch game, wait 5s
$after = Get-ChildItem -Recurse "C:\Users\koosh\AppData\LocalLow\Door 407\" | Select-Object FullName, LastWriteTime
Compare-Object $before $after -Property FullName
```

Also check `$env:TEMP` for DINO-related files created at startup.

### Step 4: Test Mutex Release via P/Invoke

Write a minimal PowerShell script that:
1. Launches the game (first instance — succeeds).
2. Opens the mutex by name via `OpenMutex`.
3. Calls `WaitForSingleObject` with timeout=0 to detect abandoned state.
4. Kills the game.
5. Attempts to acquire and release the mutex.
6. Launches the game again and checks if it succeeds.

### Step 5: Measure Kernel Mutex Hold Time After Process Kill

```powershell
Stop-Process -Name "Diplomacy is Not an Option" -Force
$start = Get-Date
# Attempt launch every 1 second, record when it succeeds
while ($true) {
    $proc = Start-Process -PassThru "...\Diplomacy is Not an Option.exe"
    Start-Sleep 3
    if (-not ($proc.HasExited -and $proc.ExitCode -ne 0)) {
        Write-Host "Success after $((Get-Date) - $start)"
        break
    }
    Start-Sleep 1
}
```

---

## References

- ADR-013: Duplicate Instance Detection & Win32 Mutex Bypass (related prior ADR — focuses on PowerShell-layer options)
- WORK-002: Second Instance Mutex/Lockfile Bypass (tracked implementation work item)
- `.claude/commands/launch-game.md` — current workaround
- `src/Runtime/Plugin.cs` — BepInEx plugin entry point
- `src/Runtime/Bridge/KeyInputSystem.cs` — ECS system (execution context)
- Windows named mutex docs: https://docs.microsoft.com/en-us/windows/win32/sync/named-mutexes
- BepInEx preloader patcher docs: https://docs.bepinex.dev/articles/dev_guide/preloader_patchers.html
- Sysinternals Handle.exe: https://docs.microsoft.com/en-us/sysinternals/downloads/handle
- `ReleaseMutex` ownership constraints: https://docs.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-releasemutex

---

## Resolution (2026-03-24)

Root cause identified: not a game Mutex but Unity's **built-in single-instance enforcement** controlled via `boot.config`.

### Finding
`Diplomacy is Not an Option_Data/boot.config` contained:
```
single-instance=
```
An empty value for the `single-instance` key is treated as **truthy** by UnityPlayer.dll, enabling Unity's native single-instance dialog ("Another instance is already running") from within the engine binary itself — not from any managed C# code.

### Fix Applied
```
single-instance=0
```
One-line edit to `boot.config`. No code changes, no Harmony patches, no preloader patcher.

### Decision: Option A (Accepted — Simplest)
The boot.config edit was chosen over all options considered in this ADR:
- Trivially reversible
- No process killing
- No mutex name discovery required
- No BepInEx preloader patcher needed
- Permanent until Steam overwrites boot.config on game update (reapply if so)
