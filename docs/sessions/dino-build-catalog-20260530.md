# DINO Build Menu + Building Catalog — Discovery (2026-05-30)

Branch: `feat/build-catalog-injection-20260530` (base `integration/v0.27.0-reconcile-20260530` @ f75c3105)

Goal: make DINOForge pack buildings (warfare-aerial airfields, warfare-starwars
landing platform / droid hangar, warfare-naval) APPEAR + be BUILDABLE in DINO's
live build menu, and spawn the correct ECS prefab + (for aerial) AerialUnitComponent.

All findings below were obtained by **live reflection against a running game**
(PID 56140, world ready, 49152 entities, packs `warfare-aerial`, `warfare-naval`,
`warfare-starwars` loaded) via the named-pipe bridge `dinoforge-game-bridge`
(`discoverTypes`) and **reflection-only metadata** of
`DNO.Main.dll` (`scripts/game/reflect-types.ps1`).

## 1. What drives the player's build menu

The build menu (the "building shop") is **data-driven by compiled game config
structs**, NOT by a runtime-mutable registry:

- `ScriptableObjectDefinitions.BuildingsCategory` (a **struct / ValueType**):
  - `int index`
  - `List<BuildingTypeContainer> types`  ← the buttons in a build-menu category
  - `UI.HelpersStorage.HintData hintData`
  - `Icon`
- `ScriptableObjectDefinitions.BuildingTypeContainer` (**struct / ValueType**) — one build-menu entry:
  - `Utility.EnumsStorage.BuildingType type`   ← **the key that identifies the building**
  - `List<int> buildingUniqueIds`              ← integer unique IDs of the concrete building prefab(s)
  - `int firstLevelUIIndex`
  - `int maxCount`

So every buildable structure the player sees is keyed by a value of the
**`Utility.EnumsStorage.BuildingType` enum** plus integer `buildingUniqueIds`.

### `BuildingType` is a CLOSED compiled enum (the load-bearing constraint)

`[Enum]::GetNames(BuildingType)` =
`Enemy, Academy, University, Barraks, BerryPickerHouse, BuildersHouse, Cemetery,
FisherHouse, Farm, ForesterHouse, Fountain, Gate, Granary, GravediggerHouse,
Hospital, House, IronMine, StoneMine, SawmillHouse, Market, Obelisk, Storage,
Tavern, TownHall, Workshop, Wall, SmallTower, Tower, BigTower, EngineerGuild,
Stables, Port, MineTrap, SpikeTrap, EnemyLowPriority, SoulStoneMine, CurseSpreader,
DeathMetalMine, UndeadBarraksRanged, UndeadBarraksMechs, UndeadBarraksTank,
UndeadBarraksGhost, UndeadMarket, StorageDeathMetal`.

**There is no spare/"custom" enum slot.** A C# enum is a compile-time constant
set; you cannot synthesise a new `BuildingType` member at runtime and have the
game's construction / placement / UI systems recognise it (the placement
ghost, cost system, `CurrentConstructionSingleton`, and `ConstructionBuildCommand`
all switch on `BuildingType`). This is why **defining a building in pack YAML is
not enough, and also why a literally-NEW building type cannot be injected into the
native menu** — the engine has no enum value, no prefab uniqueId, and no
placement/cost wiring for it.

## 2. How a build-menu entry maps to an ECS prefab

`BuildingTypeContainer.buildingUniqueIds : List<int>` holds integer **unique
prefab IDs**. The construction pipeline (`Components.Commands.ConstructionCommands.*`,
`Components.RawComponents.CurrentBuildingForConstruction`,
`Components.RawComponents.ApplyCreateConstruction`,
`Components.SingletonComponents.CurrentConstructionSingleton`) resolves a chosen
`BuildingType` → uniqueId → baked ECS **prefab entity** (all DINO entities are
Prefab entities — queries need `EntityQueryOptions.IncludePrefab`). The actual
mesh/visual is on that prefab entity (the target of #964 mesh-swap).

Confirms `AerialBuildingMapper.cs`'s comment: *"Buildings are baked into the DINO
game world at scene load time — there is no runtime building-spawner equivalent to
PackUnitSpawner."* Correct — buildings are placed by the **player via the menu**,
not spawned by us.

## 3. How a building produces units (the UnitsShop)

- `ScriptableObjectDefinitions.UnitsShop` (struct): `int objectId`,
  `List<Components.Structs.UnitsShopItemData> unitShopItems`.
- `ScriptableObjectDefinitions.UnitsShopChain` (struct):
  `Utility.EnumsStorage.UnitTrainerType type`, `List<UnitsShop> unitShops`.
- `Components.UnitsShopItem` + `Components.Structs.UnitsShopItemData` are the ECS
  side — the per-building production list shown when you select the building.

So the units a building can train are also keyed by integer item data inside a
`UnitsShop`, selected per building via `UIBridgeCommandSelectBuildingShopCategory`.

## 4. Vanilla registration timing

The `BuildingsCategory`/`UnitsShop` config is loaded from Addressables at world
load into a config ScriptableObject (referenced by the construction UI systems),
then read by `Systems.*` construction/placement systems during active gameplay
(those system groups only fire in-game, not at the main menu — matches the
DINO execution model).

## 5. Conclusion — the only viable injection strategy

Because `BuildingType` is closed, the robust, declarative approach is **ALIAS +
RESKIN + RE-PRODUCE**, driven from pack YAML (no hardcoded content IDs):

1. **Alias**: each pack building declares (or is auto-mapped to) an existing
   vanilla `BuildingType` slot that matches its role:
   - aerial/SW **Production** buildings (aviary, wyvern_roost, phoenix_nest,
     dragon_cave, rep_gunship_bay, cis_droid_hangar) → `Stables` (vanilla
     mounted-unit trainer; closest "produces a special movement-class unit"
     analogue) or `Barraks`.
   - **Defense** buildings (ballista_tower, thunder cannon) → `Tower` / `BigTower`.
   - naval buildings → `Port`.
   This makes them **already appear + already buildable** (they reuse a vanilla,
   fully-wired buildable slot).
2. **Reskin**: swap the aliased prefab's mesh/material to the pack's
   `visual_asset` bundle (depends on #964) and rename the build-menu label/icon
   via the existing ThemeEngine / mesh-swap path so the player sees the pack
   building, not the vanilla one.
3. **Re-produce**: inject the pack building's `production:` units into that
   building's `UnitsShop` so it trains the new aerial/naval/SW units; spawned
   aerial units get `AerialUnitComponent` (via existing `AerialUnitMapper` /
   `AerialSpawnSystem`) so the Aviation systems fly them.

This is mirror-of-vanilla (clone an existing buildable, retarget it) — the same
pattern `PackUnitSpawner` uses for units (clone vanilla archetype, restyle).

### What CANNOT be done (report honestly)
- Add a brand-new `BuildingType` enum member at runtime → impossible without
  patching the game assembly / an engine enum slot. A truly novel building can
  only ride on an aliased vanilla slot.
- Inject into the menu purely from disk YAML with no runtime hook → the config
  is compiled struct data behind Addressables; mutation must happen at world-ready
  via reflection on the live config instance.

## 6. Injection mechanism (this PR)

`src/Runtime/Bridge/BuildMenuInjector.cs` — at world-ready, for each loaded pack
building with a `build_alias` (vanilla `BuildingType` name) it:
- resolves the vanilla `BuildingsCategory`/`BuildingTypeContainer` for that
  `BuildingType` on the live config instance (`Resources.FindObjectsOfTypeAll`),
- records the alias→pack-building mapping so the reskin (mesh-swap, label) +
  UnitsShop production injection can target the placed entity,
- is fully driven by pack YAML (`build_alias`, `visual_asset`, `production:`),
  no hardcoded content IDs.

Pack YAML gains an optional `build_alias: <VanillaBuildingType>` on building defs.
When absent, `BuildMenuInjector` auto-maps by `building_type` (Production→Stables,
Defense→Tower, naval→Port).
