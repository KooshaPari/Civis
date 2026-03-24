# DINOForge Technical Specification

**Version**: 0.11.0
**Status**: Active
**Last Updated**: 2026-03-20
**Audience**: Plugin Developers, Runtime Maintainers, Advanced Modders

---

## 1. Architecture Overview

DINOForge is a **BepInEx plugin framework** that bridges the DINO Unity ECS runtime with a declarative pack system. The architecture follows a three-layer design:

```
┌─────────────────────────────────────────────────┐
│ CONTENT LAYER                                   │
│  Packs (YAML manifests, assets, registries)   │
└─────────────────────────────────────────────────┘
                        ↑
┌─────────────────────────────────────────────────┐
│ SDK LAYER                                       │
│  ContentLoader, Registry, Schemas, Validators  │
└─────────────────────────────────────────────────┘
                        ↑
┌─────────────────────────────────────────────────┐
│ RUNTIME LAYER                                   │
│  Plugin.cs → RuntimeDriver → ECS Bridge        │
│  • KeyInputSystem (F9/F10 input)               │
│  • ModPlatform (orchestration)                 │
│  • DebugOverlay (F9 display)                   │
│  • HudStrip (F10 display)                      │
│  • ComponentMap (ECS glue)                     │
│  • AssetSwapSystem (visual replacement)        │
└─────────────────────────────────────────────────┘
                        ↑
┌─────────────────────────────────────────────────┐
│ GAME (DINO)                                     │
│  Unity 2021.3.45f2 ECS (DOTS) + BepInEx 5.4.23.5
└─────────────────────────────────────────────────┘
```

---

## 2. Core Components

### 2.1 Plugin.cs (BepInEx Entry Point)

**Path**: `src/Runtime/Plugin.cs`

**Responsibility**: Bootstrap DINOForge on game launch

**Flow**:
```
BepInEx Load Phase (game boot)
    ↓
Plugin.Awake()
    • Create DontDestroyOnLoad root GameObject
    • Set HideAndDontSave flags (persist across scene transitions)
    • Instantiate RuntimeDriver
    • Start version detection
    ↓
Plugin.Start()
    • Wait for scene load
    • Initialize RuntimeDriver.Initialize()
    ↓
Plugin lifecycle ends; RuntimeDriver takes over
```

**Key Facts**:
- Runs ONCE at game boot (BepInEx plugin lifecycle)
- Creates root GameObject that persists across scene transitions (`DontDestroyOnLoad`)
- Sets `HideAndDontSave` to hide from scene hierarchy
- All subsequent logic delegates to RuntimeDriver

---

### 2.2 RuntimeDriver.cs (Orchestrator)

**Path**: `src/Runtime/RuntimeDriver.cs`

**Responsibility**: Coordinate all runtime systems and respond to input events

**Lifecycle**:
```
RuntimeDriver.OnAwake() [frame 0 of first scene load]
    ↓
    • Detect ECS world and systems
    • Boot ComponentMap
    • Initialize ModPlatform
    • Register KeyInputSystem
    ↓
RuntimeDriver.OnDestroy() [scene transition or shutdown]
    • Shutdown all systems gracefully
    • Cleanup component mappings
    ↓
RuntimeDriver.Update() [EVERY frame after initialization]
    • Check KeyInputSystem for F9/F10 presses
    • Call DebugOverlay.OnUpdate() or HudStrip.OnUpdate()
    • Pump ModPlatform systems
    • Coordinate asset swaps
```

**Critical Design Pattern**:
```csharp
// RuntimeDriver persists via HideAndDontSave + DontDestroyOnLoad
// This survives scene transitions like Main Menu → Gameplay

private void OnDestroy()
{
    // When scene reloads, this fires
    // We immediately recreate on next frame
    StartCoroutine(RebindAfterSceneTransition());
}
```

**Why This Matters**:
- DINO has a **two-boot cycle**: Main Menu scene → Gameplay scene
- RuntimeDriver must survive both transitions to maintain pack state
- F9/F10 must be responsive across scene changes

---

### 2.3 KeyInputSystem (F9/F10 Detection)

**Path**: `src/Runtime/KeyInputSystem.cs`

**Responsibility**: Watch for F9 and F10 keypresses via Win32 API

**Implementation**:
```csharp
private static bool IsKeyPressed(Keys key)
{
    // Win32 GetAsyncKeyState (non-blocking, works in background)
    return (GetAsyncKeyState((int)key) & 0x8000) != 0;
}

public override void OnUpdate()
{
    if (IsKeyPressed(Keys.F9))
    {
        _runtimeDriver.OnF9Pressed();
        // Debounce: skip next 10 frames to avoid repeat firing
    }

    if (IsKeyPressed(Keys.F10))
    {
        _runtimeDriver.OnF10Pressed();
    }
}
```

**Why Not Use Unity Input Manager?**
- Input.GetKeyDown() only works when game window is focused
- Win32 GetAsyncKeyState works even in background
- Modders want to toggle overlays without focusing game window
- Win32 is cross-platform via BepInEx compatibility layer

**Debouncing**:
- 10-frame cooldown between F9 toggles (166ms at 60 FPS)
- Prevents accidental double-toggles
- User experiences instant response

---

### 2.4 ModPlatform.cs (Pack Orchestration)

**Path**: `src/Runtime/ModPlatform.cs`

**Responsibility**: Coordinate pack loading, content application, and mod menu state

**Core Methods**:
```csharp
public async Task InitializeAsync()
{
    // Called once on boot
    // 1. Load all packs from BepInEx/dinoforge_packs/
    // 2. Validate schemas + resolve dependencies
    // 3. Apply stat overrides to ContentLoader
    // 4. Notify domain plugins of loaded packs
}

public void ShowModMenu()
{
    // Called on F10 press
    // Toggle _isModMenuOpen flag
    // HudStrip.OnUpdate() will render menu
}

public void ReloadPacks()
{
    // Called on F10 → Reload
    // 1. Re-read all YAML files from disk
    // 2. Validate schemas
    // 3. Update registries
    // 4. Apply HotReloadBridge updates to live entities
}

public void ApplyStatOverrides(IRegistry<Unit> unitRegistry)
{
    // Apply pack-defined stat changes to all units
    // Runs AFTER all packs loaded but BEFORE entities spawn
}
```

**Pack Load Order**:
```
dependency graph resolution (topological sort)
    ↓
framework version filtering (>=0.11.0 <1.0.0 check)
    ↓
conflict detection (pack A conflicts_with pack B)
    ↓
sequential load: for each pack:
    • Parse pack.yaml (manifest)
    • Load units.yaml, buildings.yaml, factions.yaml
    • Register with typed registries
    • Apply stat overrides
```

---

### 2.5 DebugOverlay.cs (F9 Display)

**Path**: `src/Runtime/DebugOverlay.cs`

**Responsibility**: Render F9 debug information to screen

**Sections**:
| Section | Content | Updates |
|---------|---------|---------|
| Header | DINOForge version, FPS, frame time | Every frame |
| Loaded Packs | Pack name, version, status | On pack load |
| Entity Counts | Breakdown by archetype/faction | Every 10 frames (cached) |
| Active Errors | Pack load errors, missing assets | On events |
| System Stats | ComponentMap entries, memory | Every frame |
| Entity Inspector | Search + inspect individual entities | On keystroke |

**Rendering**:
```csharp
private void OnGUI()
{
    if (!_isVisible) return;

    // Layout: top-left corner, 400px wide
    GUILayout.BeginArea(new Rect(10, 10, 400, 600));

    // Render each section using immediate-mode GUI
    RenderHeader();
    RenderLoadedPacks();
    RenderEntityCounts();
    RenderErrors();
    RenderEntityInspector();

    GUILayout.EndArea();
}
```

**Performance**:
- Entity count queries: cached every 10 frames (avoid 45K entity scan every frame)
- Individual entity lookup: uses indexed EntityQuery, O(log n)
- Overlay overhead: <2ms per frame when visible
- Invisible: 0ms (early-out at top of OnGUI)

---

### 2.6 HudStrip.cs (F10 Display)

**Path**: `src/Runtime/UI/HudStrip.cs`

**Responsibility**: Render F10 mod menu overlay

**Menu Structure**:
```
┌─────────────────────────────────────────────────┐
│ DINOForge Mod Menu (v0.11.0)                   │
├─────────────────────────────────────────────────┤
│ ☑ Star Wars Clone Wars (v0.7.0)                │
│   Dependencies: (none)                          │
│   Conflicts: Classic Balance Pack v1.0          │
│                                                 │
│ ☑ Modern Warfare (v0.11.0)                     │
│   Dependencies: Base Framework                  │
│   Conflicts: (none)                            │
│                                                 │
│ ☐ Experimental Physics (v0.1.0) [disabled]     │
│   Dependencies: (unmet)                         │
│   Conflicts: (none)                            │
│                                                 │
├─────────────────────────────────────────────────┤
│ [Space] Toggle · [R] Reload · [ESC] Close      │
└─────────────────────────────────────────────────┘
```

**Interaction**:
- Arrow Up/Down: scroll pack list
- Space: toggle selected pack (writes `disabled_packs.json`)
- R: reload all packs (HotReload)
- Escape: close menu

**State Persistence**:
```csharp
private void WritePersistentState()
{
    // Write to: BepInEx/dinoforge_packs/disabled_packs.json
    var disabled = _visiblePacks
        .Where(p => !p.IsEnabled)
        .Select(p => p.Id)
        .ToList();

    File.WriteAllText("disabled_packs.json",
        JsonSerializer.Serialize(new { disabled_packs = disabled }));
}
```

---

### 2.7 ECS Bridge Layer

#### ComponentMap.cs

**Path**: `src/Runtime/Bridge/ComponentMap.cs`

**Responsibility**: Map vanilla ECS components to modder-friendly abstractions

**Example Mapping**:
```csharp
// Vanilla DINO
var healthComponent = entity.GetComponentData<Health>();

// Via ComponentMap
var stat = _componentMap.GetStat(entity, StatType.Health);
// No need to know component type; just know the stat

// Applying overrides
_componentMap.SetStat(entity, StatType.AttackDamage, 15f);
// ComponentMap finds correct component and updates it
```

**Critical Facts**:
- **30+ component mappings** (Health, Armor, AttackCooldown, ArmorData, CostComponent, etc.)
- **Typed strongly**: StatType enum, not strings
- **Lazy registration**: components discovered at runtime via ECS reflection
- **Fallback safety**: if component missing, returns 0 or default value

**Design Pattern**:
```csharp
private readonly Dictionary<StatType, Type> _typeMap = new()
{
    { StatType.Health, typeof(Health) },
    { StatType.AttackDamage, typeof(AttackData) },
    { StatType.Armor, typeof(ArmorData) },
    // ... 27 more mappings
};

public T GetStat<T>(Entity entity, StatType type) where T : unmanaged
{
    if (!_typeMap.TryGetValue(type, out var componentType))
        return default; // Unknown stat; safe fallback

    // Use reflection to get component
    var componentData = entity.GetComponentData(componentType);
    return (T)Extract(componentData, type);
}
```

#### EntityQueries.cs

**Path**: `src/Runtime/Bridge/EntityQueries.cs`

**Responsibility**: Provide safe, reusable entity queries for common modding patterns

**Examples**:
```csharp
// Query all infantry units (by component + tag)
var infantryQuery = _entityQueries.GetUnitsOfRole("infantry");

// Query all entities belonging to a faction
var republicUnits = _entityQueries.GetFactionEntities("rep");

// Query all damaged entities (health < max)
var damagedUnits = _entityQueries.GetDamagedEntities();
```

**Critical Detail**:
```csharp
// WRONG (returns 0 results for DINO)
var query = _world.EntityManager.CreateEntityQuery(
    ComponentType.ReadOnly<Health>()
);

// RIGHT (includes prefab entities)
var query = _world.EntityManager.CreateEntityQuery(
    ComponentType.ReadOnly<Health>()
);
query.SetOptions(EntityQueryOptions.IncludePrefab); // MUST have this
```

**Why This Matters**:
- **ALL DINO entities are prefab entities** (spawned from prefabs, not instantiated dynamically)
- Without `IncludePrefab`, EntityQuery returns 0 results
- This breaks modding badly if not understood

#### AssetSwapSystem

**Path**: `src/Runtime/Bridge/AssetSwapSystem.cs`

**Responsibility**: Replace vanilla unit/building visuals with custom 3D models

**Two-Phase Approach**:

**Phase 1: Catalog Patch (Pre-Boot)**
```
1. Read addressables catalog.json
2. For each pack with visual_asset refs:
   • Find vanilla asset ID
   • Override with custom bundle path
   • Write updated catalog back
3. Game loads custom catalog on startup
```

**Phase 2: Live Entity Swap (Post-Boot)**
```
frame 600+ (allows engine to stabilize)
    ↓
AssetSwapSystem.OnUpdate() runs
    ↓
Query entities using vanilla prefab
    ↓
For each entity:
    • Load custom bundle from addressables
    • Instantiate custom prefab
    • Copy transform + components from vanilla
    • Destroy vanilla entity
    • Register swap in debug log
```

**Critical Pattern**:
```csharp
// Load custom model
var bundle = Addressables.LoadAssetAsync<GameObject>(assetId).WaitForCompletion();
if (bundle == null)
{
    // FALLBACK: keep vanilla model, don't crash
    _logger.Warn($"Asset '{assetId}' not found; using vanilla");
    return;
}

// Instantiate and copy state
var customInstance = Instantiate(bundle, entity.transform);
customInstance.transform.position = entity.transform.position;
customInstance.transform.rotation = entity.transform.rotation;
// ... copy all relevant components

// Cleanup
Destroy(entity);
```

---

## 3. Key Design Decisions

### Decision 1: HideAndDontSave + DontDestroyOnLoad for Persistence

**Why**:
- DINO has a two-boot cycle (Main Menu → Gameplay scene)
- RuntimeDriver must survive scene transitions to preserve pack state
- Standard GameObject parenting doesn't persist across scene boundaries

**Implementation**:
```csharp
// In Plugin.Awake()
var root = new GameObject("DINOForge");
root.hideFlags = HideFlags.HideAndDontSave;
DontDestroyOnLoad(root);

// Result: object appears in every scene, survives transitions
```

**Consequence**:
- Must manually handle cleanup on scene transitions
- Must re-bind to ECS world after scene load
- Requires OnDestroy + RebindAfterSceneTransition coroutine

---

### Decision 2: Win32 KeyInputSystem Instead of Unity Input

**Why**:
- Modders want F9/F10 to work even when game window is unfocused
- Unity's Input manager only fires when window is active
- Win32 GetAsyncKeyState is cross-platform via BepInEx

**Trade-off**:
- Slightly less portable (but BepInEx abstracts this away)
- Must debounce manually (can't rely on Input.GetKeyDown debouncing)
- Works in background, which is the modder experience goal

---

### Decision 3: ECS Systems for F9/F10 Overlay Rather Than MonoBehaviour

**Why**:
- MonoBehaviour.Update() never fires in DINO (destroyed at frame 0)
- ECS SystemBase.OnUpdate() is guaranteed to run every frame
- Consistent with game's architecture

**Implementation**:
```csharp
public partial class DebugOverlaySystem : SystemBase
{
    protected override void OnUpdate()
    {
        // This runs every frame
        // Render F9 overlay if visible
    }
}
```

---

### Decision 4: ComponentMap for Stat Abstraction

**Why**:
- Pack authors shouldn't know DINO's internal component names
- ComponentMap provides a stable abstraction layer
- Isolates modding surface from ECS implementation details

**Example**:
```csharp
// Pack author perspective: just set a stat
modPlatform.ApplyStatOverride("rep-clone-trooper", "health", 27f);

// ComponentMap implementation: find the right component
var component = GetComponentByType(typeof(Health));
component.Value = 27f;

// If DINO updates internal components, only ComponentMap changes
// Packs are unaffected
```

---

### Decision 5: Hot Reload via YAML Re-read, Not Live System Patching

**Why**:
- Modders want to edit YAML and see changes instantly
- Live system patching (Harmony) is complex and fragile
- Re-reading YAML + applying stat overrides is safer and more predictable

**Trade-off**:
- Not all changes can be hot-reloaded (e.g., system additions)
- Scope is YAML-only content: units, buildings, factions, stats
- Restart required for code/behavior changes

---

## 4. Critical Facts About DINO's ECS

### Fact 1: All Entities Are Prefab Entities

```csharp
// Vanilla code
var prefabEntity = em.CreateEntity(typeof(Unit), typeof(Health));
var instance = em.Instantiate(prefabEntity); // This creates the "prefab" entity

// Querying
var query = em.CreateEntityQuery(typeof(Health));
query.SetOptions(EntityQueryOptions.IncludePrefab); // MUST include prefabs or get 0 results
```

**Why This Matters**:
- DINO spawns units from prefab entity templates
- The visible entities in-game are actually instantiated prefabs
- Without `IncludePrefab`, modders' queries return empty sets
- This is the #1 source of "mod doesn't work" bugs

---

### Fact 2: World.Systems Is NoAllocReadOnlyCollection, Not IEnumerable

```csharp
// WRONG
var systems = world.Systems.Cast<SystemBase>().ToList();

// RIGHT
var systems = new SystemBase[world.Systems.Count];
for (int i = 0; i < world.Systems.Count; i++)
{
    systems[i] = (SystemBase)world.Systems[i];
}
```

**Why This Matters**:
- Unity optimized this for memory; no IEnumerable support
- Trying to LINQ will throw
- Modders must iterate by index
- ComponentMap uses index-based lookup internally

---

### Fact 3: Scene Transitions Destroy All Entities and Systems

```
Main Menu Scene
    ↓ [scene load]
ECS world cleared
All entities destroyed
All systems reinitialized
    ↓
Gameplay Scene
```

**Consequence**:
- RuntimeDriver must survive via DontDestroyOnLoad
- Must re-detect game systems after scene load
- Packs must remain loaded across transitions
- Asset swaps must re-apply if bundles reload

---

### Fact 4: Two-Boot Cycle (Main Menu → Gameplay)

```
Frame 0: Game boots
    ↓
Frame 1-30: Main Menu scene loads
    • ECS systems initialize
    • Player sees menu
    ↓
User clicks "New Game"
    ↓
Frame 31: Scene unload → Main Menu world destroyed
Frame 32: Gameplay scene loads → new ECS world created
    ↓
Frame 33+: Playing gameplay
    • All entities spawned fresh
    • Packs re-applied
```

**Consequence**:
- RuntimeDriver sees two separate ECS worlds
- Must re-initialize ComponentMap on scene transition
- F9/F10 must remain responsive across transitions
- Stat overrides must re-apply to new entities

---

## 5. Component Interactions (Data Flow)

### Flow 1: Pack Loading on Boot

```
Plugin.Awake()
    ↓
Plugin.Start()
    ↓
RuntimeDriver.OnAwake() [first scene stabilizes]
    ↓
ModPlatform.InitializeAsync()
    • ContentLoader.LoadAllPacks(packDirectory)
    • Resolve dependencies (DependencyResolver)
    • Validate schemas (SchemaValidator)
    • Load YAML files (YamlDotNet)
    • Register with typed registries
    ↓
For each pack:
    ApplyStatOverrides()
        → ComponentMap.UpdateRegistry()
    ApplyAssetReferences()
        → AssetSwapSystem.Queue()
    ApplyDoctrineModifiers() [Warfare domain]
    ↓
Game ready; entities start spawning
```

---

### Flow 2: F9 Pressed (Debug Overlay Toggle)

```
KeyInputSystem.OnUpdate()
    ↓
IsKeyPressed(F9) == true
    ↓
_runtimeDriver.OnF9Pressed()
    ↓
DebugOverlay.Toggle()
    _isVisible = !_isVisible
    ↓
DebugOverlay.OnGUI() [next frame]
    ↓
Render:
    • Header (version, FPS)
    • Loaded packs (from ModPlatform._loadedPacks)
    • Entity counts (via EntityQuery with cached results)
    • Active errors (from ModPlatform._errorLog)
    • Entity inspector (search + inspect current entity)
```

---

### Flow 3: F10 Pressed → Reload

```
KeyInputSystem.OnUpdate()
    ↓
IsKeyPressed(F10) == true
    ↓
_runtimeDriver.OnF10Pressed()
    ↓
ModPlatform.ShowModMenu()
    _isModMenuOpen = true
    ↓
HudStrip.OnUpdate() [next frame]
    ↓
User presses "R" for reload
    ↓
ModPlatform.ReloadPacks()
    ↓
For each pack:
    • Re-read YAML files from disk
    • Validate schemas
    • Update registries
    • Apply stat overrides
    ↓
HotReloadBridge.UpdateLiveEntities()
    ↓
Query all entities
For each entity:
    • Update health component (if overridden)
    • Update cost component (if overridden)
    • Update armor/damage (if overridden)
    • Keep role/archetype unchanged
    ↓
ModPlatform._reloadedThisFrame = true
    ↓
HudStrip renders confirmation: "Reloaded 3 packs"
```

---

### Flow 4: Asset Swap (Custom Model Loading)

```
Phase 1 [Pre-Boot]
    AssetSwapSystem.PatchCatalog()
        ↓
    Read: addressables/catalog.json
    For each pack with visual_asset refs:
        Override entry in catalog
    Write: catalog.json (updated)
    ↓

Phase 2 [Post-Boot, frame 600+]
    AssetSwapSystem.OnUpdate()
        ↓
    _elapsedFrames++
    if (_elapsedFrames < 600) return; // wait for stabilization
        ↓
    Query entities with vanilla prefab
        ↓
    For each matched entity:
        • Determine custom asset ID from unit registry
        • Load bundle: Addressables.LoadAssetAsync(assetId)
        • If fails → log warning, keep vanilla (don't crash)
        • If succeeds:
          - Instantiate custom prefab
          - Copy transform + components
          - Destroy vanilla entity
          - Log "Swapped unit X"
        ↓
    After all swaps complete:
        Set _swapComplete = true
        (don't run this system again)
```

---

## 6. F9/F10 Key Input Architecture

```
┌──────────────────────────────────────────┐
│ Win32 System (GetAsyncKeyState)          │
│ Monitors F9/F10 globally                 │
└──────────────────────────────────────────┘
         ↓
┌──────────────────────────────────────────┐
│ KeyInputSystem (ECS System)              │
│ .OnUpdate() → IsKeyPressed(F9/F10)?      │
│ Yes → Call RuntimeDriver.OnF9Pressed()   │
└──────────────────────────────────────────┘
         ↓
┌──────────────────────────────────────────┐
│ RuntimeDriver.OnF9Pressed()              │
│ DebugOverlay.Toggle()                    │
│ OR                                       │
│ ModPlatform.ShowModMenu()                │
└──────────────────────────────────────────┘
         ↓
┌──────────────────────────────────────────┐
│ DebugOverlay.OnGUI() [F9]                │
│ Renders in OnGUI() callback              │
│ OR                                       │
│ HudStrip.OnUpdate() [F10]                │
│ Renders text overlay                     │
└──────────────────────────────────────────┘
```

---

## 7. Pack System Architecture

### Pack Directory Structure

```
BepInEx/dinoforge_packs/my-pack/
├── pack.yaml                 # Manifest (id, version, deps, conflicts)
├── units.yaml               # Unit definitions + overrides
├── buildings.yaml           # Building definitions
├── factions.yaml            # Faction definitions + doctrines
├── weapons.yaml             # Weapon definitions
├── projectiles.yaml         # Projectile definitions
├── skills.yaml              # Skill definitions (Warfare domain)
├── waves.yaml               # Wave templates (Warfare domain)
├── assets/
│   ├── bundles/             # Asset bundles (built with Unity 2021.3.45f2)
│   │   ├── sw-rep-clone-trooper
│   │   ├── sw-rep-general
│   │   └── ...
│   └── raw/                 # Source models (GLB/FBX)
│       ├── clone_trooper.glb
│       └── ...
└── README.md                # User-facing documentation
```

### Pack Manifest (pack.yaml)

```yaml
id: warfare-starwars
name: Star Wars - Clone Wars
version: 0.7.0
author: DINOForge Community
framework_version: ">=0.5.0 <1.0.0"
description: Total conversion to Star Wars Clone Wars era
type: total_conversion
depends_on:
  - base-framework    # Optional: specify pack dependencies
conflicts_with:
  - warfare-modern-old   # Packs that can't coexist
  - classic-balance-v1

loads:
  factions:
    - factions.yaml
  units:
    - units.yaml
  buildings:
    - buildings.yaml
  weapons: []         # Empty if not defined
  projectiles: []
  skills: []
  waves: []
```

### ContentLoader Pipeline

```
for each pack (in dependency order):
    ↓
Load pack.yaml (manifest)
    ↓
Validate pack_id + version format
    ↓
Check framework_version compatibility
    ↓
for each load section (units, buildings, factions):
    ↓
Read YAML file from disk
    ↓
Validate against schema (units.schema.json, etc.)
    ↓
Register in typed registry
    • UnitRegistry.Register(unit)
    • BuildingRegistry.Register(building)
    • etc.
    ↓
Check for conflicts:
    • Duplicate IDs within pack? Error
    • ID exists in previous pack? Log warning (last pack wins)
    ↓
After all packs loaded:
    ↓
Apply stat overrides (from packs)
    ↓
Notify domain plugins:
    • WarfareDomain.OnAllPacksLoaded()
    • EconomyDomain.OnAllPacksLoaded()
    ↓
Publish ModPlatform.OnPacksReady event
```

---

## 8. Registry Pattern

### Core Interface

```csharp
public interface IRegistry<T> where T : IModel
{
    void Register(T item);
    T Get(string id);
    IReadOnlyList<T> GetAll();
    bool Contains(string id);
    event EventHandler<RegistrationConflictEventArgs> Conflict;
}
```

### Typed Registries

| Registry | Key Type | Example |
|----------|----------|---------|
| `UnitRegistry` | string | "rep-clone-trooper" |
| `BuildingRegistry` | string | "jedi-temple" |
| `FactionRegistry` | string | "republic" |
| `WeaponRegistry` | string | "blaster-rifle" |
| `DoctrineRegistry` | string | "republic-training" |
| `SkillRegistry` | string | "charge-ability" |
| `WaveRegistry` | string | "wave-1" |
| `SquadRegistry` | string | "clone-squad-1" |

### Registration Flow

```csharp
// Pack loading
foreach (var unit in unitsYaml.Units)
{
    _unitRegistry.Register(unit);
    // If conflict (unit.Id already registered):
    //   → Fire Conflict event
    //   → Last registration wins (overwrites previous)
    //   → Log warning in debug overlay
}

// Modding surface (safe)
var cloneTrooper = _unitRegistry.Get("rep-clone-trooper");
var health = cloneTrooper.Stats.Health;  // type-safe access
```

---

## 9. Schema Validation

### Validation Pipeline

```
for each YAML file:
    ↓
Load raw YAML (YamlDotNet)
    ↓
Load canonical schema (e.g., units.schema.json)
    ↓
Validate YAML against schema using NJsonSchema
    ↓
Check for missing required fields
Check for type violations (number vs string)
Check for enum violations (invalid archetype)
    ↓
If validation fails:
    • Collect all errors
    • Report with line numbers
    • Prevent pack load
    • Log to debug overlay
    ↓
If validation passes:
    Deserialize YAML → typed object (Unit, Building, etc.)
```

### Example Schema (units.schema.json)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "units": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": { "type": "string", "pattern": "^[a-z0-9-]+$" },
          "name": { "type": "string" },
          "health": { "type": "number", "minimum": 1 },
          "attack_damage": { "type": "number", "minimum": 0 },
          "armor": { "type": "number", "minimum": 0 },
          "cost": { "type": "number", "minimum": 0 },
          "archetype": { "enum": ["infantry", "heavy", "vehicle", "specialized"] },
          "visual_asset": { "type": "string" }
        },
        "required": ["id", "name", "health", "attack_damage", "cost"]
      }
    }
  }
}
```

---

## 10. Known Technical Constraints

### Constraint 1: Two-Boot ECS Worlds

**Issue**: Scene transitions destroy and recreate the ECS world

**Mitigation**:
- RuntimeDriver uses DontDestroyOnLoad to persist across scenes
- Must re-initialize ComponentMap on scene transition
- Must re-detect systems and entities after scene load

**Code Pattern**:
```csharp
private void OnDestroy()
{
    // Fire when scene transitions
    StartCoroutine(RebindAfterSceneTransition());
}

private IEnumerator RebindAfterSceneTransition()
{
    // Wait for new scene to stabilize
    yield return new WaitForSeconds(1f);

    // Re-detect world and systems
    var newWorld = World.DefaultGameObjectInjectionWorld;
    _componentMap.SetWorld(newWorld);
    _entityQueries.UpdateWorld(newWorld);
}
```

---

### Constraint 2: Asset Bundles Must Be Built with Unity 2021.3.45f2

**Issue**: DINO uses Unity 2021.3.45f2; bundles built with other versions won't load

**Mitigation**:
- Document in pack guidelines (create bundles with specific version)
- AssetSwapSystem has fallback: if bundle fails to load, keep vanilla asset
- Warn modders in compilation step

---

### Constraint 3: IncludePrefab Required for Entity Queries

**Issue**: All DINO entities are prefab instances; queries without `IncludePrefab` return 0 results

**Mitigation**:
- EntityQueries.cs pre-sets `IncludePrefab` on all queries
- Modders use EntityQueries API, not raw EntityManager queries
- Prevents "invisible entities" bugs

---

### Constraint 4: 600-Frame Delay for Asset Swaps

**Issue**: Game engine needs stabilization before swapping visuals

**Mitigation**:
- AssetSwapSystem.OnUpdate() waits 600 frames (~10 seconds at 60 FPS)
- Allows entity spawning, initial rendering, LOD calculation to settle
- Swap doesn't cause visual pop-in

---

## 11. Extension Points (For Plugin Developers)

### Extension Point 1: Custom Domain Plugin

**Path**: Implement `IDomainPlugin`

```csharp
public interface IDomainPlugin
{
    string Id { get; }
    Version Version { get; }

    void OnInitialize(DomainPluginContext context);
    void OnAllPacksLoaded(IReadOnlyList<Pack> loadedPacks);
    void OnShutdown();
}
```

**Example**: Warfare domain plugin

```csharp
public class WarfareDomainPlugin : IDomainPlugin
{
    public string Id => "warfare";

    public void OnInitialize(DomainPluginContext context)
    {
        // Register warfare-specific registries
        context.RegisterRegistry<Faction>(_factionRegistry);
        context.RegisterRegistry<Doctrine>(_doctrineRegistry);

        // Register warfare systems
        context.RegisterSystem<FactionSystem>();
        context.RegisterSystem<DoctrineModifierSystem>();
    }

    public void OnAllPacksLoaded(IReadOnlyList<Pack> loadedPacks)
    {
        // Apply doctrine modifiers to loaded factions
        foreach (var pack in loadedPacks)
        {
            ApplyDoctrineModifiers(pack);
        }
    }
}
```

---

### Extension Point 2: Custom Asset Processor

**Path**: Implement `IAssetProcessor`

```csharp
public interface IAssetProcessor
{
    string Name { get; }
    bool CanProcess(Asset asset);
    Task<ProcessedAsset> ProcessAsync(Asset asset, CancellationToken ct);
}
```

---

## 12. Glossary

| Term | Definition |
|------|-----------|
| **BepInEx** | Cross-platform game patching framework |
| **ECS** | Entity Component System (DOTS architecture) |
| **ComponentMap** | Abstraction layer mapping vanilla components to stat types |
| **RuntimeDriver** | Main orchestrator coordinating all runtime systems |
| **ModPlatform** | Pack loading and orchestration system |
| **DebugOverlay** | F9 in-game debug panel |
| **HudStrip** | F10 in-game mod menu |
| **ContentLoader** | Pack file reader and registry population |
| **Addressables** | Unity's asset loading system |
| **AssetSwap** | Live replacement of unit/building visuals |
| **HideAndDontSave** | GameObject flag to hide from editor and persist across scenes |
| **DontDestroyOnLoad** | Keep GameObject alive across scene transitions |
| **IncludePrefab** | EntityQuery option to include prefab entities |
| **Doctrine** | Set of stat multipliers for a faction |
| **Archetype** | Mechanical family (Order, Industrial Swarm, Asymmetric) |

---

**Last Updated**: 2026-03-20
**Prepared by**: Claude Haiku 4.5
**Next Review**: After v0.12.0 release (April 2026)
