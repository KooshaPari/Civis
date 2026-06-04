# MODS Button Regression — Empirical Bisection + Live Verification (2026-05-29)

## TL;DR

**The MODS button is BACK. Verified live: `ENGINE-UI READY: packs=10 modsButton=True f9=True f10=True`.**

The fix branch `fix/engine-ui-injection-race-20260529` @ `ab8c6074` already contains the
working fix. The earlier "still broken" reports were a **verification-window artifact**, not a
code defect: the `InitialGameLoader -> MainMenu` asset-load transition causes a transient
~2-4 minute gray-freeze in this environment. The standard 80s wait expired *inside* that freeze,
before MainMenu loaded. Waiting 150s let the transition complete; `OnSceneLoaded: MainMenu`
then fired and the self-heal revive injected the MODS button.

## Bisection result (theory correction)

The task premise was: "MODS button WORKED at committed `cafd2b70`, broke later in
`cafd2b70..ab8c6074`." Empirical bisection **disproves the single-commit-regression theory**:

1. **`cafd2b70` does not compile in isolation.** Its `Plugin.cs` references
   `Bridge.PlayerLoopKeyInputInjection` (6x) and `ModPlatform.SetPackEnabled`, but **neither
   symbol exists at `cafd2b70`** — both were first ADDED in the *next* Runtime-touching commit
   `c55299b1`. The iter-146 "victory" screenshot was therefore taken against an **uncommitted
   working tree**, not the committed `cafd2b70` blob.

2. **The nearest buildable baseline is `c55299b1`** (immediate descendant; adds the missing
   symbols, completes the iter-146 work). Built + deployed it clean (0 errors).

3. **`c55299b1` does NOT show the MODS button either.** Live log froze at the
   `InitialGameLoader` `RuntimeDriver.OnDestroy` (46 lines, 5.5s span); `activeSceneChanged`
   only ever delivered `''` and `InitialGameLoader`, never `MainMenu`. Screenshot
   `bisect-iter146-baseline.png` shows the native menu with **no MODS button**.

   => The regression **predates `cafd2b70`** as a *committed* state. The committed baseline
   never worked. The #231 v0.26.0 squash (`3edcde3a`) is **NOT** the breaking commit.

## Root mechanism (confirmed by both runs)

At the `InitialGameLoader -> MainMenu` transition, DINO performs a **native scene unload** that
tears down `DINOForge_Root` (DontDestroyOnLoad + HideAndDontSave) and fires
`RuntimeDriver.OnDestroy` — bypassing the managed `DestroyGuardPatch`. After teardown, the
plugin must rely on a main-thread callback firing for the *new* `MainMenu` scene to re-inject
the UI. The fix branch wires every available DINO-driven main-thread hook to a single
`MainThreadReviveIfNeeded` -> `TryResurrect` -> `RunMainMenuInit` -> `NativeMenuInjector` chain:

- `SceneManager.sceneLoaded` (`OnSceneLoaded`) — **this is the one that delivers MainMenu**
  (`mode=Single`), confirmed in the verified run.
- `SceneManager.activeSceneChanged` (`OnActiveSceneChanged`)
- Harmony postfix on `PlayerLoop.SetPlayerLoop` (`OnPlayerLoopSet`)
- `RuntimeDriver` self-heal `activeSceneChanged` -> `RunMainMenuInit("scene-change")`

The background `ResurrectionFallback` thread does **pure managed work only** (mark + re-arm),
correctly avoiding the bg-thread Unity-ECall deadlock during asset load. The engine-heartbeat
(`BepInEx/dinoforge_heartbeat.txt`) disambiguates "Mono suspended bg thread" (counter frozen)
from "log-write contention" (counter advancing). During the freeze the heartbeat source is
`fallback`; once the main thread resumes it flips to `playerloop-set(main-thread)` /
`scene-change`.

## Live verification (deployed DLL = fix branch build)

```
deployed DINOForge.Runtime.dll SHA256 = 0F9384BA2E363D911179543386108F4BFDF2B21312EF121BB4AD46666209F7B0
baseline c55299b1 deploy SHA256        = 36A792D9A5BCE58D42A283E3810AF6F7CDE4B355B21CF5AF8EB3061FF1D12310   (differs)
```

Verbatim from `BepInEx/dinoforge_debug.log` (150s after launch):

```
[2026-05-30T04:25:12.6974726Z] [NativeMenuInjector] [#778] Canvas 'MainMenu' has 28 *Button components: [...]
[2026-05-30T04:25:13.3289397Z] [MainMenuThemer] Theme applied: 'STAR WARS' canvas='MainMenu'
[2026-05-30T04:25:13.3430879Z] [Plugin] [DINOForge] ENGINE-UI READY: packs=10 modsButton=True f9=True f10=True (via scene-change)
[2026-05-30T04:25:13.3430879Z] [Plugin] [Plugin] OnSceneLoaded: name='MainMenu' buildIndex=1 mode=Single isLoaded=True
```

Screenshot `docs/screenshots/engine-ui-FIXED-final-20260529.png` shows the DINO main menu with
the **MODS** button rendered between OPTIONS and CREDITS. Baseline comparison:
`docs/screenshots/bisect-iter146-baseline.png` (no MODS button).

## Operational takeaway

The launch-verification protocol's 80s wait is **too short for the MainMenu transition** in this
environment. The MODS button appears only after the `InitialGameLoader -> MainMenu` asset load
completes (~150s observed). Verification of engine-UI features MUST poll until
`OnSceneLoaded: name='MainMenu'` (or `ENGINE-UI READY ... modsButton=True`) appears in
`dinoforge_debug.log`, not on a fixed sleep. (See [[feedback_dino_launch_hang_universal]].)
