# Diplomacy is Not an Option - Technical Game Notes

**Last Updated**: 2026-03-09
**Status**: Research complete (initial pass)

---

## Game Overview

- **Genre**: Medieval siege defense RTS / city-builder
- **Developer**: Door 407 (door407.com)
- **Engine**: Unity with full ECS (DOTS) architecture
- **Scripting Backend**: Mono (NOT IL2CPP)
- **Platform**: Steam (PC / Windows)
- **Release**: February 9, 2022 (Early Access), October 4, 2024 (Full Release)

---

## Modding Landscape

### Official Support
- **Map Editor** (added May 2025) with full Steam Workshop integration
  - Adjustable terrain, enemy horde waves, timing
  - Mission objectives (defend/attack, survive/prosper)
  - Random or manual landscape options
  - Resource difficulty settings
- **December 2025**: Winter update making Undead faction playable in Map Editor
- No official overhaul/mod SDK
- No official API documentation
- Developer stance: "open to community mods but prioritize core game development"

### Community Modding
- Small Nexus Mods community
- BepInEx-based modding via **modified BepInEx 5 with Doorstop pre-loader fork**
- Standard BepInEx from GitHub DOES NOT WORK - requires specialized ECS build
- Custom `BepInEx/ecs_plugins/` install path (differs from standard BepInEx)
- Nexus/Vortex support confirmed (Vortex extension auto-installs BepInEx)

### Existing Mods (Nexus)
1. **BepInEx 5 with Unity ECS Support** - Foundation loader
2. **Resource Fairy** - Configurable resource/storage multipliers (F1 ConfigManager)
3. **Trainer and Multipliers** - Unit damage/speed/health, building health, population multipliers

### Existing Code Repos
- **devopsdinosaur/dno-mods** (GitHub) - Primary mod source repo
  - Visual Studio solution (`dno-mods.sln`)
  - Directories: `resource_fairy/`, `shared/`, `system_test/`, `testing/`, `third-party/`
  - 100% C#, 29 commits, single contributor
  - All built mods published on Nexus

---

## Key Technical Details

### ECS Architecture
- Uses **full Unity ECS** (Entity Component System) via DOTS
- **Not hybrid ECS** - full DOTS implementation
- Burst compilation for performance
- Custom `ecs_plugins` flow for game-specific plugins

### BepInEx Setup (CRITICAL)
- **Standard BepInEx from GitHub does NOT work**
- Required: BepInEx 5 with modified Doorstop pre-loader (C++ winhttp.dll)
- Doorstop fork allows multiple assembly pre-loading
- Source: Nexus Mods (diplomacyisnotanoption/mods/1)

**Folder Structure**:
```
GameRoot/
  winhttp.dll              # Modified Doorstop
  doorstop_config.ini
  BepInEx/
    plugins/               # Standard plugins folder (EMPTY for DINO)
    ecs_plugins/            # WHERE DINO MODS GO
    config/                 # Configuration files
```

### Modding Path
1. Install modified BepInEx/Doorstop for Unity ECS
2. Mods go into `BepInEx/ecs_plugins/`, NOT standard `BepInEx/plugins/`
3. ECS-native mods preferred over Harmony patches
4. Plugins use attribute-based registration (standard BepInEx pattern)

### Harmony Compatibility
- Prefix patches (run before methods): **COMPATIBLE**
- Postfix patches (run after methods): **COMPATIBLE**
- Transpiler patches: **NOT RECOMMENDED** (ECS architecture limitations)
- **Performance Warning**: Steam guide (id:3348001330) warns severe perf cost
- Removing Burst-generated DLL can halve frame rate
- Recommends writing ECS-based mods directly

---

## Key Game Systems

### Economy
- **5 resource types**: Food, Wood, Stone, Iron, Gold
- Trading system via "merchant dirigible"
- Gold obtained exclusively through trading (high exchange rates)
- Resource production and storage critical to strategy
- Late-game heavily dependent on gold conversion strategy

### Military
- Real-time combat with pause mechanics
- Unit positioning and flanking critical
- Diverse unit types (cavalry, infantry, archers, etc.)
- Progressive unit unlock via tech tree
- Unit upgrades through research

### Building
- Economic buildings (farms, woodcutters, quarries, mines)
- Military buildings (barracks, stables, towers)
- Production buildings (markets, smithies)
- Research buildings (universities)
- Defense structures (walls, gates)

### Technology & Upgrades
- University reduces research time
- 14+ technologies available
- Tech tree system with dependencies
- Unit and building upgrades

### Magic System
- Magical monuments
- Soul crystal harvesting
- Summoned units/ghosts
- Beam attack spells

### Waves & AI
- Procedurally generated maps with enemy spawns
- Wave-based enemy assault system
- Multiple attack directions
- Progressive difficulty scaling
- "Vast hordes" (25,000+ enemies reported)

---

## Vanilla Unit-to-Role Mapping (for mod conversion)

These vanilla game elements map to abstract modding roles:

| Vanilla Element | Abstract Role |
|----------------|---------------|
| Peasant worker | Economy worker |
| Swordsman | Line infantry |
| Archer / ranged | Ranged infantry |
| Cavalry | Fast strike unit |
| Catapult / trebuchet | Siege / artillery |
| Walls / towers | Static defense |
| Farms | Economy primary |
| Town hall / castle | Command center |
| Magic / soul | Special ability proxy |

---

## Tool Chain

| Tool | Purpose |
|------|---------|
| BepInEx 5 (ECS build) | Plugin/mod framework (modified for DINO) |
| HarmonyX | Method patching (use sparingly - perf cost) |
| AssetRipper | Unity asset extraction |
| dnSpy / ILSpy | Managed assembly inspection / decompilation |
| Unity.Entities | ECS-native system creation |
| Visual Studio | Mod development (C#) |
| ConfigurationManager | In-game config UI (F1 key) |
| Vortex | Mod manager with DINO extension |

---

## Open Questions

- [ ] Exact Unity version used by DINO
- [x] ~~Mono vs IL2CPP runtime~~ -> **Mono confirmed**
- [ ] Complete entity/component namespace map (requires decompilation)
- [ ] Prefab organization and naming conventions
- [ ] Asset bundle structure
- [ ] Localization table format
- [ ] Research tree data format
- [ ] Wave composition data format
- [ ] Available hook/patch points in ECS systems

---

## References

### Primary
- [Nexus Mods - Diplomacy is Not an Option](https://www.nexusmods.com/diplomacyisnotanoption) -- Mod downloads and community hub
- [BepInEx 5 with Unity ECS Support](https://www.nexusmods.com/diplomacyisnotanoption/mods/1) -- Required mod loader (modified Doorstop fork)
- [Steam Community Modding Guide](https://steamcommunity.com/sharedfiles/filedetails/?id=3348001330) -- "Modding is an Option (with BepInEx)" -- primary BepInEx setup reference
- [devopsdinosaur/dno-mods](https://github.com/devopsdinosaur/dno-mods) -- Community mod source code (Resource Fairy, Trainer)
- [Steam Workshop](https://steamcommunity.com/app/1272320/workshop/) -- Official map sharing (Map Editor maps only, not code mods)

### Nexus Mods Available
- [BepInEx 5 with Unity ECS Support](https://www.nexusmods.com/diplomacyisnotanoption/mods/1) -- Foundation loader
- [Resource Fairy](https://www.nexusmods.com/diplomacyisnotanoption/mods/2) -- Configurable resource/storage multipliers (F1 ConfigManager)
- [Trainer and Multipliers](https://www.nexusmods.com/diplomacyisnotanoption/mods/3) -- Unit damage/speed/health, building health, population multipliers

### Secondary
- [Vortex Extension for DINO](https://www.nexusmods.com/site/mods/1070) -- Adds DINO support to Vortex Mod Manager (auto-installs BepInEx)
- [PCGamingWiki](https://pcgamingwiki.com/wiki/Diplomacy_Is_Not_an_Option)
- [Fandom Wiki](https://diplomacy-is-not-an-option.fandom.com)
- [ModDB](https://www.moddb.com/games/diplomacy-is-not-an-option/mods)
- [SteamDB Patch Notes](https://steamdb.info/app/1272320/patchnotes/)
- [Door 407 (Developer)](https://door407.com)
- [Steam Store Page](https://store.steampowered.com/app/1272320/Diplomacy_is_Not_an_Option/)

### Map Editor (Official)
The May 2025 update added a full Map Editor with Steam Workshop integration:
- Adjustable terrain, enemy horde waves, timing
- Mission objectives (defend/attack, survive/prosper)
- Random or manual landscape options
- Resource difficulty settings
- December 2025 update made Undead faction playable in Map Editor
- Maps only -- the Workshop does not support code mods

### Comparable BepInEx Modding Scenes (for reference)
- Valheim (mature BepInEx + Thunderstore ecosystem)
- Risk of Rain 2 (BepInEx reference game)
- The Long Dark (Harmony patching reference)
- RimWorld (modding-friendly architecture reference)
