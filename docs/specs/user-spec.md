# DINOForge User Specification

**Version**: 0.11.0
**Last Updated**: 2026-03-20
**Target Users**: Modders, Game Enthusiasts, Plugin Developers
**Status**: Active Development

---

## Table of Contents

1. [Overview](#overview)
2. [Target Users](#target-users)
3. [User Personas](#user-personas)
4. [Core User Workflows](#core-user-workflows)
5. [Feature Specifications](#feature-specifications)
6. [Current Implementation Status](#current-implementation-status)
7. [Known Issues & Limitations](#known-issues--limitations)
8. [Future Roadmap](#future-roadmap)

---

## Overview

DINOForge is a general-purpose mod platform and SDK for **Diplomacy is Not an Option (DINO)** that enables modders to create content packs, balance mods, and total conversions without deep reverse-engineering knowledge.

### Key Promise

> Create new DINO mods primarily by defining validated pack manifests and content through the DINOForge SDK and toolchain, with minimal fresh reverse engineering.

### Product Tiers

| Tier | User | Responsibility |
|------|------|-----------------|
| **End-User** | Game players | Install packs, experience mods |
| **Mod Author** | Content creators | Create packs, define units/buildings/factions |
| **Plugin Developer** | Advanced modders | Extend DINOForge with custom domain plugins |
| **Platform Developer** | Project maintainer | Core runtime, SDK, toolchain maintenance |

---

## Target Users

### Primary: Mod Authors

**Who**: Gameplay modders, content creators, balance designers

**Needs**:
- Create mods without learning Unity internals or reverse-engineering
- Iterate quickly on gameplay and visuals
- Understand compatibility and conflicts
- Get clear errors when something breaks

**Success Metric**: Can create a new mod pack in <4 hours using YAML + assets

### Secondary: Plugin Developers

**Who**: Advanced modders, domain-specific extension creators

**Needs**:
- Understand DINOForge architecture and extension points
- Create custom domain plugins (e.g., custom economy, AI, UI)
- Build on stable SDK contracts
- Debug integration issues

**Success Metric**: Can create a custom domain plugin in <2 days

### Tertiary: End-Users

**Who**: Game players installing mods

**Needs**:
- Install packs safely
- Understand compatibility and conflicts
- Get understandable error messages
- Stable gameplay behavior

**Success Metric**: 95% pack installations succeed without errors

---

## User Personas

### Persona A: "Thematic Content Creator" (Lisa)

**Background**: Enjoys modding games, loves Star Wars, moderate technical skills

**Goals**:
- Create a Star Wars Clone Wars total conversion
- Keep existing gameplay mechanics mostly intact
- Add new unit visuals, factions, and names

**Pain Points**:
- Doesn't know how to reverse-engineer DINO
- No 3D modeling skills
- Wants to avoid writing custom C# code

**DINOForge Fit**: ✅ Perfect — YAML packs + asset bundles solve her problem

**Typical Workflow**:
```
1. Download assets (existing models from community)
2. Create pack.yaml with manifest
3. Define factions/units/buildings in YAML
4. Map visual assets to definitions
5. Test with /test-swap command
6. Deploy and share
```

---

### Persona B: "Balance Tuner" (Marcus)

**Background**: Competitive player, wants to balance gameplay, some technical skills

**Goals**:
- Create subtle balance changes to unit costs/damage/HP
- A/B test different economy rates
- Share balance patches with friends

**Pain Points**:
- Doesn't want to touch code
- Needs fast iteration (test → tweak → test)
- Wants to compose multiple balance mods

**DINOForge Fit**: ✅ Excellent — stat overrides + hot reload are perfect

**Typical Workflow**:
```
1. Create balance pack with stat overrides
2. Launch game with /launch-game
3. Tweak YAML values
4. Use F10 mod menu to reload
5. Verify changes in gameplay
6. Repeat until balanced
```

---

### Persona C: "Plugin Architect" (Dev)

**Background**: Professional developer, wants to extend DINOForge itself

**Goals**:
- Create custom domain plugin (e.g., "Diplomacy" domain)
- Contribute back to DINOForge core
- Build complex gameplay mechanics

**Pain Points**:
- Needs clear SDK contracts and stability guarantees
- Wants examples and templates
- Needs to understand internal architecture

**DINOForge Fit**: ✅ Good — SDK is stable and documented

**Typical Workflow**:
```
1. Read domain plugin architecture docs
2. Create new domain project
3. Define registry + schemas
4. Implement orchestrator integration
5. Add 30+ unit tests
6. Create example pack
7. Submit PR with ADR
```

---

## Core User Workflows

### Workflow 1: Install & Play

**Goal**: End-user installs a mod pack and plays

**Precondition**: Game installed, BepInEx + DINOForge installed

**Steps**:
1. Download pack from Thunderstore or GitHub
2. Extract to `BepInEx/dinoforge_packs/`
3. Launch game
4. Pack loads automatically on startup
5. Play with mod active

**Success Criteria**:
- ✅ Pack loads without errors
- ✅ Visuals and gameplay changes are visible
- ✅ No crashes or performance degradation

**Known Issues**:
- Asset loading can fail silently if bundle mismatch (fallback to placeholder)
- Pack conflicts may prevent some packs from loading together

---

### Workflow 2: Create Balance Mod

**Goal**: Mod author creates a cost/stats balance pack

**Precondition**: DINOForge installed, modding tools available

**Steps**:
1. Create `packs/my-balance-pack/` directory
2. Write `pack.yaml` manifest:
   ```yaml
   id: my-balance-pack
   name: My Balance Mod
   version: 1.0.0
   type: balance
   ```
3. Create `units.yaml` with stat overrides:
   ```yaml
   units:
     - id: infantry
       stats:
         cost: 50  # reduced from 60
         hp: 100   # increased from 80
   ```
4. Run `/pack-deploy` to validate and build
5. Launch game with `/launch-game`
6. Press F10 to open mod menu and verify changes
7. Use hot reload to test iteratively

**Success Criteria**:
- ✅ Pack validates without schema errors
- ✅ Game launches successfully
- ✅ Changes visible in gameplay
- ✅ Can reload without restart

**Known Issues**:
- Hot reload may miss some system updates (restart recommended for major changes)
- Stat overrides apply at pack load time, not dynamically

---

### Workflow 3: Create Total Conversion Pack

**Goal**: Mod author creates a Star Wars themed total conversion

**Precondition**: Assets sourced (models + textures), DINOForge tools installed

**Steps**:

**Phase 1: Setup**
1. Create pack directory and manifest
2. Define new factions: `Republic`, `CIS`, `Neutral`
3. Define unit archetypes (Order, Industrial Swarm, Asymmetric)

**Phase 2: Content Definition**
4. Create `units.yaml` with all unit definitions
5. Create `buildings.yaml` with all structures
6. Create `factions.yaml` mapping factions to units/tech
7. Add `visual_asset` references to each unit

**Phase 3: Asset Integration**
8. Build asset bundles in Unity 2021.3.45f2
9. Deploy bundles to `packs/my-pack/assets/bundles/`
10. Register addressable keys in asset catalog

**Phase 4: Testing & Deployment**
11. Run `/pack-deploy` to validate all references
12. Launch game with `/launch-game`
13. Verify all visuals and gameplay
14. Publish to Thunderstore

**Success Criteria**:
- ✅ All 30+ units defined
- ✅ All 15+ buildings defined
- ✅ 80%+ visual coverage (assets loaded)
- ✅ Zero schema validation errors
- ✅ Game launches and plays stably
- ✅ Compatible with other packs (no conflicts)

**Estimated Time**: 40-60 hours (including asset creation)

---

### Workflow 4: Debug & Troubleshoot

**Goal**: Mod author diagnoses why a pack doesn't work

**Precondition**: Pack created but has issues

**Steps**:

1. **Check manifest**:
   ```bash
   dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack
   ```
   → Reports schema violations, missing dependencies, conflicts

2. **Check asset references**:
   ```bash
   dotnet run --project src/Tools/PackCompiler -- assets validate packs/my-pack
   ```
   → Reports missing bundles, invalid asset IDs, load errors

3. **Launch game and check debug overlay**:
   - Press **F9** to open debug overlay
   - Check "Loaded Packs" section
   - Look for error messages in red text

4. **Check log files**:
   ```bash
   tail -50 "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log"
   ```
   → Shows pack loading errors, ECS integration issues

5. **Query entities in-game**:
   - Press **F9 → Entity Inspector** tab
   - Search for affected unit types
   - Check component values vs. pack definition

6. **Use MCP bridge for detailed diagnostics**:
   ```bash
   game_query_entities component=Health
   game_get_stat entity_id=12345 stat=AttackDamage
   ```

**Success Criteria**:
- ✅ Error message clearly identifies the problem
- ✅ Log file has relevant trace information
- ✅ Root cause found without reverse-engineering code

---

## Feature Specifications

### F1: Pack Loading & Manifest System

**Description**: Users can create mods as declarative YAML packs with manifest metadata

**User Stories**:

**US-F1.1** "As a mod author, I can create a pack with a `pack.yaml` manifest so the game recognizes it as a mod"

**Acceptance Criteria**:
- [ ] Pack with valid `pack.yaml` loads on startup
- [ ] Pack with missing `pack.yaml` fails with clear error
- [ ] Pack with invalid YAML fails with line number
- [ ] Framework version check works (`framework_version: ">=0.11.0 <1.0.0"`)
- [ ] Dependency resolution respects declared `depends_on`

**Implementation Status**: ✅ COMPLETE (v0.5.0+)

**Current Behavior**:
- All packs load from `BepInEx/dinoforge_packs/` directory
- Manifest parsed via YamlDotNet
- Schema validated against `pack.schema.json`
- Circular dependencies detected and reported

**Known Limitations**:
- Pack load order may not respect complex dependency graphs (workaround: chain dependencies linearly)

---

### F2: Debug Overlay (F9 Key)

**Description**: In-game overlay showing mod status, entity counts, and errors

**User Stories**:

**US-F2.1** "As a mod author, I can press F9 to see loaded packs and entity statistics without restarting"

**Acceptance Criteria**:
- [ ] F9 toggles overlay visibility
- [ ] Shows list of loaded packs with version
- [ ] Shows entity count per pack/faction
- [ ] Shows system performance stats (FPS, frame time)
- [ ] Shows error messages in red
- [ ] Overlay doesn't impact game performance (<1ms overhead)

**Implementation Status**: ✅ COMPLETE (v0.6.0+)

**Current Behavior**:
- RuntimeDriver watches Win32 input for F9 key
- DebugOverlay system renders UI via Immediate Mode GUI
- Shows sections: Loaded Packs, Entity Inspector, System Stats, Error Log
- Toggles on/off without affecting gameplay

**Runtime Flow**:
```
KeyInputSystem (Win32 watcher)
    ↓
RuntimeDriver.Update()
    ↓
DebugOverlay.OnUpdate()
    ↓
ImGuiNET render
```

**Known Limitations**:
- Cannot see overlay in fullscreen exclusive mode (windowed or borderless required)
- Entity Inspector may lag with 10K+ entities

---

### F3: Mod Menu (F10 Key)

**Description**: In-game menu to toggle packs and access mod settings

**User Stories**:

**US-F3.1** "As a mod author, I can press F10 to toggle individual packs on/off without restarting"

**Acceptance Criteria**:
- [ ] F10 opens mod menu overlay
- [ ] Menu shows checkbox list of loaded packs
- [ ] Toggling checkbox writes `disabled_packs.json`
- [ ] Game respects disabled packs on next launch
- [ ] Menu has settings for each pack (if defined)
- [ ] Menu is responsive and doesn't lag

**Implementation Status**: ✅ COMPLETE (v0.8.0+)

**Current Behavior**:
- RuntimeDriver watches F10 key
- HudStrip system renders menu overlay
- Packs can be toggled; state persists to `disabled_packs.json`
- Settings per-pack are template-ready but not yet in example packs

**Runtime Flow**:
```
KeyInputSystem (F10 detection)
    ↓
RuntimeDriver.Update() → ModPlatform.ShowModMenu()
    ↓
HudStrip.OnUpdate() renders overlay
    ↓
Overlay writes disabled_packs.json on toggle
```

**Known Limitations**:
- Can't add new packs from menu (only toggle existing ones)
- Settings not yet editable in UI (defined in YAML only)

---

### F4: Hot Module Reload (F10 → Reload)

**Description**: Reload pack YAML without restarting game

**User Stories**:

**US-F4.1** "As a mod author, I can edit `units.yaml`, press F10 → Reload, and see changes instantly"

**Acceptance Criteria**:
- [ ] Reload reads all modified YAML files from disk
- [ ] Registry updates with new definitions
- [ ] New entities use updated stats
- [ ] Existing entities reflect partial updates (health, cost, etc.)
- [ ] No crashes or memory leaks during reload
- [ ] Players are notified of reload via chat message

**Implementation Status**: ⚠️ PARTIAL (v0.9.0+)

**Current Behavior**:
- F10 → Reload option triggers `PackHotReloadBridge.Reload()`
- Reads YAML files from disk
- Reloads registries (units, buildings, factions, etc.)
- New spawned units use updated stats
- Existing entities updated via HotReloadBridge component queries

**Known Limitations**:
- **Existing entities**: Only health, armor, damage updated; role/archetype changes require restart
- **Systems state**: Some systems (WaveController, TechSystem) may not sync fully
- **Performance**: Reload takes 500-1000ms; recommended to avoid during battles

---

### F5: Asset Swap System

**Description**: Load custom 3D models for units and buildings

**User Stories**:

**US-F5.1** "As a pack author, I can define `visual_asset: my-unit-model` and the game uses my custom 3D model instead of the vanilla one"

**Acceptance Criteria**:
- [ ] Asset bundles load from `packs/my-pack/assets/bundles/`
- [ ] Addressables catalog maps asset IDs to bundles
- [ ] Live entity swap replaces vanilla prefab with custom one
- [ ] Catalog patch updates game's internal asset references
- [ ] Fallback to vanilla model if custom asset fails to load
- [ ] LOD variants reduce polycount for distant units

**Implementation Status**: ✅ COMPLETE (v0.7.0+)

**Current Behavior**:
- Phase 1: Catalog patch updates Addressables manifest
- Phase 2: Live entity swap via `AssetSwapSystem`
  - Queries entities with old prefab
  - Loads custom bundle from addressables
  - Instantiates replacement prefab
  - Copies transform + components
  - Destroys old prefab
- Performance: 600-frame delay to allow cascade

**Critical Facts**:
- ✅ Only works with Unity 2021.3.45f2 asset bundles
- ✅ Bundles must be named exactly as `visual_asset` key
- ✅ Fallback: if bundle fails, uses vanilla model without error
- ✅ LOD system reduces visible polycount by ~60% at distance

---

### F6: Pack Validation & Compiler

**Description**: CLI tool to validate packs before deployment

**User Stories**:

**US-F6.1** "As a mod author, I can run `pack-deploy` and get immediate feedback on schema violations, missing assets, and conflicts"

**Acceptance Criteria**:
- [ ] Validates `pack.yaml` against schema
- [ ] Reports missing YAML files (units, buildings, factions)
- [ ] Checks all asset references exist
- [ ] Detects circular dependencies
- [ ] Flags conflicting packs
- [ ] Provides clear error messages with line numbers
- [ ] Builds pack artifact (ready to deploy)

**Implementation Status**: ✅ COMPLETE (v0.6.0+)

**Current Behavior**:
```bash
dotnet run --project src/Tools/PackCompiler -- \
  validate packs/my-pack \
  build packs/my-pack
```

**Output**:
```
✓ pack.yaml: Valid
✓ units.yaml: 30 units defined
✗ buildings.yaml: Missing file
  → Suggestion: Create buildings.yaml or set empty array
✓ Assets: 25/30 visual assets found (83%)
✓ Dependencies: All resolved
✗ Conflicts: This pack conflicts with "starwars-old-v1.0"
  → Consider: Ask users to uninstall competing pack

Build completed: packs/my-pack/dist/
```

---

### F7: Entity Inspector

**Description**: Debug tool to inspect in-game entities and components

**User Stories**:

**US-F7.1** "As a mod author, I can press F9 → Entity Inspector and search for units to see their component values (health, damage, cost, etc.)"

**Acceptance Criteria**:
- [ ] Search by unit name or ID
- [ ] Display all components on entity
- [ ] Show calculated stats vs. base stats
- [ ] Show stat overrides from packs
- [ ] Allow live editing of values (for testing)
- [ ] No performance impact when hidden

**Implementation Status**: ✅ COMPLETE (v0.7.0+)

**Current Behavior**:
- F9 → Entity Inspector tab
- Search box filters by unit name/type
- Shows components: Health, AttackCooldown, Cost, ArmorData, etc.
- Shows calculated values and overrides
- Can edit values in real-time for testing (changes not saved)

---

### F8: Desktop Companion App

**Description**: Standalone desktop app to manage packs without launching game

**User Stories**:

**US-F8.1** "As a mod author, I can open the Desktop Companion, see all installed packs, toggle them on/off, and see their status without launching DINO"

**Acceptance Criteria**:
- [ ] Companion launches in <3 seconds
- [ ] Shows list of packs with version, author, status
- [ ] Can toggle packs on/off (writes `disabled_packs.json`)
- [ ] Shows pack dependencies and conflicts
- [ ] Shows F9/F10 debug panel snapshots (from file dump)
- [ ] Real-time YAML file watcher updates UI when pack files change
- [ ] No game process required

**Implementation Status**: ✅ COMPLETE (v0.9.0+)

**Current Behavior**:
- WinUI 3 app targeting `net8.0-windows`
- Reads packs from `BepInEx/dinoforge_packs/`
- UI layout matches F10 in-game menu
- File watcher monitors pack YAML changes (500ms debounce)
- Companion state persists independently from game

---

## Current Implementation Status

### Version: 0.11.0 (Active)

#### Completed Features (✅)

| Feature | Version | Status | Notes |
|---------|---------|--------|-------|
| Pack loading & manifest system | 0.5.0 | ✅ STABLE | YAML + schema validation |
| Registry system (units, buildings, factions) | 0.5.0 | ✅ STABLE | Extensible typed registries |
| Debug overlay (F9) | 0.6.0 | ✅ STABLE | Entity inspector, system stats |
| Mod menu (F10) | 0.8.0 | ✅ STABLE | Toggle packs, settings |
| Hot module reload | 0.9.0 | ✅ STABLE | YAML reload without restart |
| Asset swap system | 0.7.0 | ✅ STABLE | Custom models + LOD |
| Pack validator/compiler | 0.6.0 | ✅ STABLE | CLI tool with detailed errors |
| Entity inspector | 0.7.0 | ✅ STABLE | Live component inspection |
| Desktop companion | 0.9.0 | ✅ STABLE | Pack manager app |
| Warfare domain plugin | 0.6.0 | ✅ STABLE | Factions, doctrines, waves |
| Example packs | 0.8.0 | ✅ STABLE | Modern, Star Wars, Guerrilla |
| MCP bridge | 0.10.0 | ✅ STABLE | 13 game automation tools |

#### In Progress (🔄)

| Feature | Version | Status | ETA |
|---------|---------|--------|-----|
| Asset pipeline refactor | 0.11.0+ | 🔄 IN PROGRESS | v0.12.0 (April 2026) |
| Economy domain plugin | 0.11.0+ | 🔄 IN PROGRESS | v0.12.0 |
| Scenario domain plugin | 0.11.0+ | 🔄 IN PROGRESS | v0.13.0 (May 2026) |
| UI domain plugin | 0.12.0+ | 🔄 QUEUED | v0.13.0 |

#### Test Coverage

- **678 tests passing** (664 unit + 14 integration)
- **Target**: 130+ tests by v0.12.0
- Coverage areas: pack loading, validation, ECS bridge, warfare domain, registry system

---

## Known Issues & Limitations

### Critical Issues

None identified.

---

### High Priority Issues

#### Issue 1: Asset Loading Silent Failure (MEDIUM-HIGH)

**Symptoms**:
- Unit displays placeholder model instead of custom asset
- No error message in log or UI

**Root Cause**:
- Bundle loading fails silently when:
  - Asset ID doesn't match bundle filename
  - Bundle is corrupted
  - Bundle was built with wrong Unity version

**Workaround**:
1. Check bundle filename matches `visual_asset` ID
2. Verify bundle built with Unity 2021.3.45f2
3. Check `dinoforge_debug.log` for "AssetSwap" lines

**Fix ETA**: v0.12.0

**Tracking**: ADR-010 (Asset Intake Pipeline)

---

#### Issue 2: Pack Reload Doesn't Update All Systems (MEDIUM)

**Symptoms**:
- After F10 → Reload, new units still have old stats from previous spawn
- Tech tree or wave system still using old definitions

**Root Cause**:
- HotReloadBridge updates existing entities but not:
  - WaveController spawn queue
  - TechSystem tech nodes
  - Economic balance (rates)

**Workaround**:
- Restart game for complete system reset
- Avoid reloading during active waves

**Fix ETA**: v0.12.0

**Tracking**: #87

---

### Medium Priority Issues

#### Issue 3: Entity Inspector Lag (10K+ entities)

**Symptoms**:
- F9 → Entity Inspector becomes unresponsive with 10K+ entities
- Query takes >100ms

**Root Cause**:
- Linear search through all entities for each keystroke
- UI re-renders every frame

**Workaround**:
- Use narrower search filters (by faction or unit type)
- Avoid inspecting during massive battles

**Fix ETA**: v0.13.0 (index-based lookup)

---

#### Issue 4: Mod Menu UI Clipping (Low Resolution)

**Symptoms**:
- F10 menu items cut off at <1024x768 resolution
- Settings panel overlaps mod list

**Root Cause**:
- Hard-coded layout assumptions

**Workaround**:
- Use 1024x768 or higher resolution

**Fix ETA**: v0.12.0 (responsive layout)

---

### Low Priority Issues

#### Issue 5: Audio Latency (Audio)

**Symptoms**:
- Custom audio packs may have slight lip-sync delay
- Not critical for gameplay

**Status**: Known limitation, low priority

---

#### Issue 6: Colorblind Palette Support (UX)

**Symptoms**:
- Faction colors not validated for colorblind accessibility
- Red/green factions may be indistinguishable

**Status**: Design issue, queued for v1.0.0

---

## Future Roadmap

### v0.12.0 (April 2026)

**Scope**: Asset pipeline refinement + Economy domain

- ✅ Asset import optimization (parallel processing)
- ✅ Hot reload system completeness (WaveController sync)
- ✅ Economy domain plugin (rates, tech tree balance)
- ✅ Pack conflict resolution UI improvements

**User Impact**:
- Faster iteration on assets
- Better reload experience
- Economy balance packs available

---

### v0.13.0 (May 2026)

**Scope**: Scenario domain + UI domain

- ✅ Scenario pack support (mission scripting)
- ✅ Custom UI pack support (HUD themes)
- ✅ Entity Inspector performance improvements

**User Impact**:
- Custom missions and campaign mods
- HUD customization packs
- Better debugging tools

---

### v1.0.0 (Q2-Q3 2026)

**Scope**: Feature-complete, production-ready

- ✅ Full asset coverage (80%+)
- ✅ Third-party pack ecosystem
- ✅ Official mod template generator
- ✅ Community distribution (Thunderstore)

**User Impact**:
- Stable API guarantees
- Active modding community
- Official templates and tutorials

---

## Appendix: User Acceptance Testing

### UAT Checklist: New Pack Installation

- [ ] Pack downloads successfully from Thunderstore
- [ ] Extract to `BepInEx/dinoforge_packs/my-pack/`
- [ ] Game launches without errors
- [ ] Pack appears in F10 mod menu
- [ ] Gameplay changes are visible
- [ ] No performance degradation
- [ ] Can toggle pack on/off
- [ ] Uninstall (delete directory) works cleanly

### UAT Checklist: Balance Mod Creation

- [ ] Create pack with stat overrides
- [ ] Run `/pack-deploy` → validates successfully
- [ ] Game launches with pack
- [ ] F9 shows pack as loaded
- [ ] Changes visible (e.g., units cost less)
- [ ] Hot reload works (F10 → Reload)
- [ ] Iterative tweaking possible

### UAT Checklist: Total Conversion Pack

- [ ] All units defined in YAML
- [ ] All buildings defined in YAML
- [ ] All visual assets referenced and found
- [ ] Factions load correctly
- [ ] Gameplay stable with 30+ units
- [ ] Models render with correct LOD
- [ ] Both heroes visible
- [ ] No crashes after 30min gameplay

---

**Last Updated**: 2026-03-20
**Next Review**: After v0.12.0 release (April 2026)
