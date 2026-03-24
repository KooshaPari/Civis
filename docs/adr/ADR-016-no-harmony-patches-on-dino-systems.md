# ADR-005: No Harmony Patches on DINO Systems

**Status**: Accepted
**Date**: 2026-03-24
**Deciders**: kooshapari (Runtime Lead)
**Related Issues**: ISSUE-036 (MonoBehaviour lifecycle crashes), ISSUE-041 (PlayerLoop injection)

---

## Context

DINOForge's runtime layer must interoperate with DINO's custom game engine, which replaces Unity's standard initialization flow and PlayerLoop with a custom ECS-based scheduler. Early attempts to Harmony-patch DINO's own systems (MonoBehaviour lifecycle, scene transitions, system groups) caused hard crashes and deadlocks.

### History of Patch Failures

#### 1. MonoBehaviour Lifecycle Patches (FAILED)

**What we tried**: Patch `MonoBehaviour.OnEnable()` and `OnDisable()` in DINO's own systems to detect scene transitions.

**What happened**:
```
mono_runtime_delegate_invoke: exception
System.ExecutionEngineException: Attempting to JIT compile method...
```

**Root cause**: During DINO's teardown of scenes, it calls Unity APIs (SceneManager, GetComponent, etc.) from contexts where the Mono GC is in an inconsistent state. Harmony patches on MonoBehaviour callbacks inserted additional delegate invocations that violated Mono's internal state machine.

**Lesson**: Patches that inject delegates into the MonoBehaviour lifecycle are unsafe when the target MonoBehaviours are managed by a custom engine that replaces the normal lifecycle.

#### 2. SceneManager.LoadScene Patches (FAILED)

**What we tried**: Patch `SceneManager.LoadScene()` to inject custom logic before/after scene loads.

**What happened**: Silent scene loading failures, or scenes loaded but DINO's world setup didn't proceed.

**Root cause**: DINO has its own scene loading pipeline (GameStateUpdateHandler) that expects scenes to load in a specific order with specific callbacks. Harmony patches were firing during that pipeline, causing initialization race conditions.

**Lesson**: The game engine's scene pipeline is tightly coupled to its ECS initialization. Patching scene loading breaks invariants we can't guarantee.

#### 3. FightGroup.OnUpdate() Patches (SUCCEEDED, BUT FRAGILE)

**What we tried**: Patch `FightGroup.OnUpdate()` to detect when gameplay starts (to resume f9/f10 key input after main menu).

**What happened**: Works, but only because:
- `FightGroup` is a DINO system group (not MonoBehaviour-based)
- We're only wrapping the method entry/exit, not injecting new callbacks
- The patch does not call Unity APIs (no scene/asset access)

**Current implementation** (in src/Runtime/Patches/ECSSystemPatches.cs):

```csharp
[HarmonyPatch(typeof(FightGroup), nameof(FightGroup.OnUpdate))]
public static class FightGroupPatches
{
    public static void Prefix()
    {
        // Simple flag set — no Unity API calls, no exceptions possible
        _inGameplay = true;
    }
}
```

**Why this specific patch survived**: It's minimal, doesn't interact with lifecycles, and FightGroup is a pure ECS system (no MonoBehaviour involvement).

---

## Decision

**Establish a hard rule**:

- ❌ **DO NOT** patch DINO's MonoBehaviour-based systems (InitialGameLoader, any scene MonoBehaviours, etc.)
- ❌ **DO NOT** patch DINO's scene loading or world initialization pipeline
- ✓ **ALLOWED**: Patch Unity engine methods (PlayerLoop.SetPlayerLoop, Input, Application callbacks) — these are external to DINO
- ✓ **ALLOWED**: Minimal wraps on DINO ECS system groups **IF** the patch does not call Unity APIs that might be in an inconsistent state

### Corollary: Controlled Exceptions

The following exceptions are documented and controlled:

| Patch | Target | Status | Justification |
|-------|--------|--------|---------------|
| PlayerLoop.SetPlayerLoop | Unity engine (external) | ✓ OK | Injecting DINOForge delegates into Unity's loop is safe; we control our own delegate |
| FightGroup.OnUpdate | DINO ECS system | ✓ OK (Exception) | Minimal entry/exit flag, no Unity APIs, pure data mutation |
| DinoSysGroup detection | DINO ECS system | ✓ OK (Exception) | Same reasoning as FightGroup — pure data mutation on entry/exit |

All other patches on DINO systems are **forbidden**.

---

## Implementation Consequences

### What We Must Do Instead of Patches

For any feature that requires interaction with DINO's systems:

1. **Use event/callback injection** — if DINO provides hooks (SceneManager.activeSceneChanged, etc.), use them
2. **Use ECS composition** — create new ECS systems that observe existing components/tags
3. **Use background threads** — for polling-based features (like F9/F10 key detection via GetAsyncKeyState)
4. **Use PostLoadAttribute** — for code that runs after scenes are fully initialized
5. **Use OnEnable on our own MonoBehaviours** — only in our DontDestroyOnLoad persistent root

### Example: F9/F10 Key Input

❌ **Wrong approach** (attempted early):
```csharp
[HarmonyPatch(typeof(UpdateManager), nameof(UpdateManager.Update))]
public static class KeyInputPatches
{
    public static void Prefix()
    {
        // Try to detect F9 press here — crashes because UpdateManager is DINO's lifecycle
    }
}
```

✓ **Correct approach** (current):
```csharp
// Background thread (Win32 API, not Harmony)
private void KeyPollThread()
{
    while (_running)
    {
        if ((GetAsyncKeyState(VK_F9) & 0x8000) != 0)
        {
            Plugin.PendingF9Toggle = true;  // Signal flag
        }
        Thread.Sleep(50);
    }
}

// ECS system (safe hook point)
public partial struct KeyInputSystem : ISystem
{
    public void OnUpdate(ref SystemState state)
    {
        if (Plugin.PendingF9Toggle)
        {
            Plugin.PendingF9Toggle = false;
            OnF9Pressed?.Invoke();  // Delegate to UGUI
        }
    }
}
```

---

## Testing This ADR

### Verification Checklist

- [ ] No Harmony patches exist on DINO MonoBehaviours (audit src/Runtime/Patches/)
- [ ] No Harmony patches on DINO scene loading pipeline
- [ ] All DINO ECS system patches are documented as controlled exceptions
- [ ] F9/F10 key detection uses background thread (Win32), not Harmony
- [ ] Menu injection uses OnActiveSceneChanged (Unity engine hook), not Harmony
- [ ] Resurrection and panel visibility use ECS systems and DontDestroyOnLoad, not Harmony
- [ ] No patches are added without explicit exception entry in this ADR

### Audit Command

```bash
# Find all Harmony patches (search for [HarmonyPatch] in src/Runtime/)
grep -r "\[HarmonyPatch" src/Runtime/ --include="*.cs"

# Expected results: Only patches on Unity.* types or documented DINO ECS exceptions
```

---

## Acceptance Criteria

This ADR is considered **successfully implemented** when:

1. ✓ All runtime features work without MonoBehaviour lifecycle patches
2. ✓ F9/F10 keys work reliably (no crashes)
3. ✓ Menu injection completes without deadlocks
4. ✓ No "Attempting to JIT compile" or "ExecutionEngineException" errors in logs
5. ✓ `/prove-features` runs autonomously without user interaction
6. ✓ Feature tests pass repeatedly (no race conditions from patch fragility)

---

## Related Documents

- **SPEC-runtime-features-baseline.md**: Feature specs that rely on this ADR (no MonoBehaviour patches)
- **project_dino_runtime_execution_model.md**: Execution context map (what works where)
- **ISSUE-036**: Original crash from MonoBehaviour patch attempts
- **ISSUE-041**: PlayerLoop injection (allowed exception)

---

## Maintenance & Future Decisions

If a new feature requires "just a small Harmony patch on DINO's system", the decision process is:

1. **Can this use an existing Unity/DINO hook?** (SceneManager.activeSceneChanged, OnEnable, etc.) → Use hook
2. **Can this use a background thread?** (polling, timers) → Use background thread
3. **Can this create new ECS systems?** (if the feature fits in ECS) → Create ECS system
4. **Is this patch only on a Unity engine method?** (PlayerLoop, Input, Time) → Document exception and proceed
5. **Is this patch ONLY a minimal entry/exit flag with no Unity APIs?** (like FightGroup exception) → Document exception and proceed
6. **Otherwise**: Reject patch, iterate on design using allowed techniques

**Any exception must be documented in this ADR with clear justification.**

---

## Ownership & Review

**ADR Owner**: Runtime Lead (kooshapari)
**Reviewers**: Architecture Council (before major feature patches)
**Last Review**: 2026-03-24
**Next Review**: Before any new Harmony patch is added to src/Runtime/
