# Menu/Sprite Forensic Read (iter-144 post-66bba825)

## Timeline (key events from LogOutput.log + dinoforge_debug.log)

| timestamp / line | source | event | significance |
|---|---|---|---|
| LogOutput L15 | Plugin | `Plugin.Awake() ENTRY` | BepInEx loads, TFM=netstandard2.0 OK |
| L34 | RuntimeDriver | `Initialize() ENTRY` | runs once at startup |
| L42 | DFCanvas | `EventSystem not found - created DINOForge_EventSystem` | DFCanvas creates ES with `DontDestroyOnLoad` (DFCanvas.cs:139-143) |
| L65-66 | RuntimeDriver | DFCanvas + NativeMenuInjector added | UI components live |
| L76 | NativeMenuInjector | `Scene changed: <empty>` (Attempt#1) | first scene transition |
| L93 | NativeMenuInjector | `Scene changed: InitialGameLoader` (Attempt#2) | second scene |
| L125 | NativeMenuInjector | `Scene changed: MainMenu` (Attempt#3) | THIRD scene = MainMenu visible |
| L257-259 | NativeMenuInjector | Found `Options` button in canvas `MainMenu` → INJECTING | injection logic runs |
| L262 | NativeMenuInjector | `STEP 1 OK: Cloned button 'DINOForge_ModsButton'` | clone succeeded |
| L300-301 | NativeMenuInjector | `Canvas 'MainMenu' renderMode=ScreenSpaceOverlay`; `GraphicRaycaster: enabled=True` | graphics OK |
| **L305** | NativeMenuInjector | **`⚠ EventSystem.current is NULL!`** | **ROOT CAUSE — UI cannot route input** |
| L306 | NativeMenuInjector | `STEP 7 OK: EventSystem configuration complete` | **misleading log** — code logged OK but did nothing because ES was null |
| L319-320 | NativeMenuInjector | `MODS BUTTON INJECTION FULLY SUCCESSFUL` | misleading — injection structurally OK but unusable |
| debug L all | AssetSwapSystem | `TrySwapRenderMeshFromBundle: GetSharedComponentData/SetSharedComponentData not found` (every unit) | reflection lookup fails on every single swap (36/36) |
| debug L all | AssetSwapSystem | `ApplySwap: entity swap result=False` (×36) | every sprite swap silently fails |

## EventSystem state machine

- **Created**: DFCanvas.BuildCanvas() at L42 of LogOutput, with `DontDestroyOnLoad(esGo)` (DFCanvas.cs:139-140). One-time only — only fires when `EventSystem.current == null` at canvas-build time.
- **Survives**: across `<empty>` → InitialGameLoader → MainMenu scene transitions (L76, L93, L125 show NativeMenuInjector still running).
- **NULL at click time**: by NativeMenuInjector Attempt#3 STEP 7 (L305), `EventSystem.current` is `null`.
- **Why**: DINO's MainMenu scene almost certainly contains its own EventSystem GameObject that gets created on scene-load and destroyed when MainMenu unloads. While our DontDestroyOnLoad'd `DINOForge_EventSystem` GameObject still exists, Unity's `EventSystem.current` static is whichever EventSystem most recently called `OnEnable` — when DINO's scene-bound ES is destroyed during transition, `current` is reset to `null`, NOT to our orphan one. Our ES is alive in the hierarchy but is not the "active" one. NativeMenuInjector STEP 7 logs the NULL but does nothing to recover.

## NativeMenuInjector retry behavior

- Reached **Attempt#3** at MainMenu, found Options button, injected Mods clone (L259-320).
- Attempt#4 (L358): `Already injected + button alive, skipping scan` — no re-validation of EventSystem.
- Button is gameObject.activeInHierarchy=True, interactable=True, raycastTarget=True (L311-318), but no EventSystem means clicks die in the GraphicRaycaster → InputModule routing.

## AssetSwapSystem re-Initialize gap — but DIFFERENT root cause

- Initialize fires on world startup and DOES call ApplySwap for all 36 units (debug log 23:59-00:01 timeframe shows full cycle running).
- **`TrySwapRenderMeshFromBundle`** at AssetSwapSystem.cs:351-356 returns false on EVERY entity because reflection lookup for `GetSharedComponentData(Entity, ComponentType)` returns null. This is **Pattern #101** (still in MEMORY task #101 as completed but verified-broken). The `MethodInfo getSharedNonGeneric = typeof(EntityManager).GetMethod("GetSharedComponentData", new[] { typeof(Entity), typeof(ComponentType) });` returns null at runtime — DINO's Unity 2021.3 EntityManager either doesn't expose that exact overload OR returns it under a different signature.
- Bundles load successfully (`LoadBundle: loaded ...`), but every swap aborts before touching ECS. Chicken/skeleton sprites remain because the swap NEVER mutates entities.

## ROOT CAUSE (specific)

Two independent bugs both visible:

1. **Menu unclickable**: `EventSystem.current` is null at button-injection time because DINO's MainMenu-scene-bound EventSystem was destroyed on a prior scene transition, and our DontDestroyOnLoad'd `DINOForge_EventSystem` (DFCanvas.cs:139) was created **once at startup**, never re-promoted to `EventSystem.current` after DINO's ES died. `NativeMenuInjector.ValidateRaycastAndEventSystem` (NativeMenuInjector.cs:849-885) **detects** the null but the `else` branch (L874-877) only **logs a warning** — it never recreates / re-activates an EventSystem.

2. **Sprite swaps no-op**: `AssetSwapSystem.TrySwapRenderMeshFromBundle` (src/Runtime/Bridge/AssetSwapSystem.cs:335-354) uses `typeof(EntityManager).GetMethod("GetSharedComponentData", new[] { typeof(Entity), typeof(ComponentType) })` which returns null against DINO's Unity 2021.3 EntityManager — likely the overload signature is different (e.g. takes `Entity` only with a generic, or signature returns a ref struct). Every swap exits at L353-357.

## RECOMMENDED FIX (concrete)

1. **NativeMenuInjector.cs:874-877** (EventSystem null branch): instead of logging-only, create a fresh EventSystem on the active scene (NOT DontDestroyOnLoad — let it die with MainMenu, and re-create on next scene). Use the same pattern as DFCanvas.cs:138-144 but inline. After creation, also re-assign `EventSystem.current = newEs` explicitly.

2. **Better**: hook `SceneManager.activeSceneChanged` inside RuntimeDriver to re-ensure `EventSystem.current != null` on EVERY scene change (cheap idempotent check). This guarantees both DFCanvas's overlay clicks and the injected Mods button work across all scenes. Roughly 10 lines in RuntimeDriver.cs.

3. **AssetSwapSystem.cs:335-345**: replace the brittle `GetMethod(name, types[])` lookup with `EntityManager.GetMethods(BindingFlags.Public | BindingFlags.Instance).FirstOrDefault(m => m.Name == "GetSharedComponentData" && m.GetParameters().Length == 2 && m.GetParameters()[0].ParameterType == typeof(Entity) && !m.IsGenericMethodDefinition)`. Mirror the same defensive arity-filter pattern already used at L343-349 for `SetSharedComponentData`. Add a one-time dump of all `EntityManager.GetSharedComponentData*` method signatures on first failure to confirm what overloads DINO actually exposes.
