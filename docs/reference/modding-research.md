# Modding Systems Research: 15 Industry-Leading Games

**Purpose**: Analyze proven modding architectures to inform DINOForge's design as a BepInEx-based mod platform for Diplomacy is Not an Option (DINO).

**Date**: March 2026
**Scope**: Mod format, load systems, content scope, dependency management, distribution, no-code paths, strengths, weaknesses, and relevance to DINOForge.

---

## Quick Comparison Table

| Game | Format | No-Code Path | Content Scope | Distribution | Key Strength |
|------|--------|--------------|---------------|--------------|--------------|
| **Factorio** | Lua (structured) | Partial (data.lua) | Prototypes, scripts, assets | Factorio Mods portal | Data lifecycle standardization |
| **Stardew Valley** | JSON (Content Patcher) | Full | Graphics, data, maps, dialogue | Nexus, CurseForge, SMAPI | JSON-only content without C# |
| **RimWorld** | XML + C# (Harmony) | Full (XML only) | Defs, behaviors, mechanics | Steam Workshop | XML blueprint + optional patches |
| **Crusader Kings III** | PDX script | Partial | Game rules, map, cultures | Steam Workshop, Paradox Mods | Total conversions (CK2→CK3 converters) |
| **Stellaris** | PDX script | Partial | Empires, events, buildings | Steam Workshop, Paradox Mods | Community-driven total conversions |
| **Total War: Warhammer III** | Binary (.pack) + RPFM | No | Tables, scripts, data | Steam Workshop | RPFM tool ecosystem |
| **Kerbal Space Program** | C# + .cfg | Partial (.cfg only) | Parts, behaviors, plugins | CKAN (package manager) | CKAN dependency resolution |
| **Mount & Blade II: Bannerlord** | C# + XML | Partial (XML only) | Scenes, items, characters | Nexus, Steam Workshop | Official tools + SubModule.xml structure |
| **The Sims 4** | Python + .package | No | Game objects, behaviors, tuning | CurseForge, TSR | Package format + Python scripting |
| **Cities: Skylines** | C# (compiled DLL) | No | Buildings, behaviors, UI | Steam Workshop | Unified C# API |
| **Minecraft** | JSON (datapacks) | Full | Recipes, loot, structures, functions | Modrinth, CurseForge | JSON datapacks + Fabric/Forge loaders |
| **Skyrim / Fallout 4** | Binary (ESP/ESM) | No | Content via Creation Kit | Nexus Mods | Creation Kit GUI tool |
| **OpenTTD** | NewGRF (NML) | No (binary format) | Vehicles, buildings, cargo | Community forums, steam | Open source + NML high-level language |
| **Victoria 3** | PDX script | Partial | Rules, events, decisions | Paradox Mods | Jomini engine evolution |
| **Dwarf Fortress** | RAW (text) | Full | Creatures, items, materials | Steam Workshop, DFFD | Declarative text modification |

---

## Detailed Analysis by Game

### 1. Factorio - Data Lifecycle Mastery

**Mod Format**: Lua scripts organized by lifecycle stage
**File Structure**: `info.json`, `data.lua`, `data-updates.lua`, `data-final-fixes.lua`, `control.lua`, `settings.lua`

**Load System**:
- Mods loaded as zip or folder (`{name}_{version}` or `{name}`)
- Single shared Lua state per stage
- Three-phase data construction: data → updates → final-fixes
- Runtime Lua state per mod for event handling

**Content Types**:
- Prototypes (items, entities, technologies, recipes, buildings)
- Scripting (events, commands, remote interfaces)
- Settings (player-configurable mod options)
- Graphics and localizations

**Dependency Management**:
- Mods can declare dependencies in `info.json`
- Mod portal handles version compatibility
- Load order determined by dependency graph

**Distribution**: [Factorio Mods Portal](https://mods.factorio.com/) (built-in in-game mod manager)

**No-Code Path**: Partial—mods can be pure data (prototypes) without scripting, but dynamic behavior requires `control.lua`

**Key Strengths**:
1. **Data Lifecycle Standardization**: Three-phase load ensures consistent override ordering (data → updates → final-fixes)
2. **Modular Lua States**: Each mod gets isolated runtime state, reducing conflicts
3. **Built-in Mod Manager**: Integrated discovery, installation, versioning
4. **Simple Text Format**: `info.json` is human-readable and git-friendly

**Key Weaknesses**:
1. Lua required for anything beyond prototypes
2. No visual GUI for config (JSON-style settings only)
3. Factorio modding has steep learning curve for complex mechanics

**Relevance to DINOForge**:
- **Adopt**: Lifecycle concept (data → apply → final-fixes) maps well to pack loading phases
- **Adapt**: Three-phase could be `base-pack.yaml → balance-override.yaml → conflict-resolution.yaml`
- **Avoid**: Don't force Lua; JSON/YAML is sufficient for most RTS mod types (units, buildings, factions)

**Sources**:
- [Factorio Modding Wiki](https://wiki.factorio.com/Modding)
- [Factorio Lua API](https://lua-api.factorio.com/latest/)
- [Mod Structure Documentation](https://lua-api.factorio.com/latest/auxiliary/mod-structure.html)

---

### 2. Stardew Valley + SMAPI + Content Patcher - JSON-Only Content

**Mod Format**: JSON-based content patches (Content Patcher)
**File Structure**: `manifest.json`, `content.json`, asset folders

**Load System**:
- SMAPI is the mod loader (Stardew Modding API)
- Content Patcher is a SMAPI mod (not code mods)
- JSON patches applied in order: Load, EditData, EditImage, EditMap
- Validation via [smapi.io/json](https://smapi.io/json)

**Content Types**:
- Game assets (images, data, dialogue)
- Map modifications
- Data entry edits (conditional by date, weather, location, spouse, etc.)
- No code required

**Dependency Management**:
- Manifest lists dependencies + versions
- SMAPI enforces dependency order
- Conflict detection and reporting

**Distribution**: [Nexus Mods](https://www.nexusmods.com/stardewvalley), [CurseForge](https://www.curseforge.com/stardewvalley)

**No-Code Path**: **Full** — all modding possible without writing C#

**Key Strengths**:
1. **Pure JSON Content Patcher**: Zero code required; patches are declarative
2. **Conditional Logic**: Patches apply conditionally (date, location, NPC spouse, etc.)
3. **Automatic Compatibility Checks**: SMAPI validates manifests before load
4. **Asset Replacement**: Simple Load/Edit paradigm—no file format knowledge needed
5. **Auto-Update Support**: Built-in update checking and compatibility reports

**Key Weaknesses**:
1. Limited to asset/data replacement; can't add new game mechanics
2. SMAPI required (not part of base game)
3. Conditional syntax is JSON-heavy (not intuitive for non-coders)

**Relevance to DINOForge**:
- **Adopt**: Content Patcher's conditional patch system (date/location) is perfect for scenario modding
- **Adapt**: DINOForge should support no-code YAML pack format for balance/unit mods
- **Target Design**: `units.yaml` + `conditions.yaml` for faction/unit overrides without C#
- **Conflict Detection**: Implement similar manifest validation as SMAPI

**Sources**:
- [Content Patcher Documentation](https://stardewvalleywiki.com/Modding:Content_Patcher)
- [SMAPI Mod Compatibility](https://smapi.io/mods)
- [Nexus Content Patcher Page](https://www.nexusmods.com/stardewvalley/mods/1915)

---

### 3. RimWorld - XML Defs + Harmony Patching

**Mod Format**: XML definitions + optional C# Harmony patches
**File Structure**: `About/About.xml`, `Defs/` (XML files), `Assemblies/` (DLLs with Harmony patches)

**Load System**:
- Load order specified in `About.xml`
- XML defs inherit from C# Def classes
- Harmony patches applied to methods (pre/post hooks, replacements)
- Mod folders or `.rar` files

**Content Types**:
- Defs (items, buildings, factions, technologies, animals)
- Behaviors via Harmony method patches
- Custom C# code in DLLs

**Dependency Management**:
- `About.xml` lists dependencies and load order
- No version tracking (manual)
- Community tracks conflicts

**Distribution**: Steam Workshop, Nexus Mods

**No-Code Path**: **Full** — pure XML mods define new content (buildings, items, factions)

**Key Strengths**:
1. **Separation of Data and Code**: XML for defs, C# for complex behavior
2. **Harmony Ecosystem**: De facto standard for method patching (used by many games)
3. **Simple Inheritance**: XML defs inherit from base classes automatically
4. **Hot Reload Friendly**: Can add new Defs without recompiling
5. **Massive Community**: 10K+ mods, well-documented tooling

**Key Weaknesses**:
1. XML inheritance rules are implicit (not always obvious)
2. Load order errors cause silent failures or crashes
3. Harmony patches are fragile (game updates break them)
4. No dependency versioning (semver not enforced)

**Relevance to DINOForge**:
- **Adopt**: XML Def pattern is perfect for units, buildings, factions
- **Adapt**: YAML equivalent: `units.yaml` inherits from base unit archetype
- **Improve**: Add explicit schema validation (JSON Schema) instead of silent failures
- **Avoid**: Don't mimic Harmony's method-patching fragility; use registry-based overrides

**Sources**:
- [RimWorld Modding Wiki](https://rimworldmodding.wiki.gg/wiki/Basic_Concepts)
- [XML Defs Tutorial](https://rimworldwiki.com/wiki/Modding_Tutorials/XML_Defs)
- [Harmony RimWorld](https://github.com/pardeike/HarmonyRimWorld)

---

### 4. Crusader Kings III - Paradox PDX Scripting

**Mod Format**: PDX script (text-based declarative language)
**File Structure**: `/common/` (events, decisions, characters), `/map/` (provinces, borders)

**Load System**:
- Mod folders with `.mod` descriptor file
- Steam Workshop integration
- PDX script parser loads and merges files
- Load order by folder name and dependency tracking

**Content Types**:
- Rules and mechanics (events, decisions, claims, titles)
- Map and provinces
- Character traits and culture
- Government types and succession laws

**Dependency Management**:
- `.mod` files list dependencies
- Mod portal enforces compatibility
- Version checking built-in

**Distribution**: [Steam Workshop](https://steamcommunity.com/workshop/browse/?appid=1158310), [Paradox Mods](https://mods.paradoxplaza.com/games/ck3)

**No-Code Path**: Partial—new rules require PDX script; can't add purely data-driven rules without code

**Key Strengths**:
1. **Total Conversions**: Mods like Shogunate, Princes of Darkness redefine entire game
2. **Game Converters**: Community tools convert CK2→CK3, EU4→CK3
3. **Live Editing**: PDX syntax is relatively intuitive
4. **Built-in Workshop**: Seamless Steam distribution

**Key Weaknesses**:
1. PDX script is proprietary; documentation is sparse
2. Syntax errors cause silent failures or crashes
3. Requires understanding game's internal rule system
4. Total conversions are 5K+ file projects (high barrier to entry)

**Relevance to DINOForge**:
- **Avoid**: PDX script is too proprietary for an open-source project
- **Adapt**: Document YAML rule syntax clearly (DINOForge's equivalent of PDX script)
- **Learn**: Total conversions show value of well-designed base game abstraction

**Sources**:
- [CK3 Modding Wiki](https://ck3.paradoxwikis.com/Modding)
- [Best CK3 Total Conversions](https://gamerant.com/crusader-kings-3-ck3-best-total-conversion-mods/)
- [Paradox Mods](https://mods.paradoxplaza.com/games/ck3)

---

### 5. Stellaris - Total Conversions & Community Scale

**Mod Format**: PDX script (same as CK3)
**Content Types**: Star Wars, Warhammer 40K total conversions popular

**Load System**: Same as CK3 (Steam Workshop, dependency tracking)

**Distribution**: [Steam Workshop](https://steamcommunity.com/workshop/browse/?appid=281990&requiredtags%5B%5D=Total+Conversion), [Paradox Mods](https://mods.paradoxplaza.com/games/stellaris)

**Key Strengths**:
1. **Community Total Conversions**: Warhammer 40K has 1200+ systems, 32 empires, 25 planet types
2. **Active Modding Community**: Mods with detailed balance tuning, lore integration
3. **Scale**: Mods rival official DLC in scope (one person or small team creates 5K+ system empires)

**Key Weaknesses**:
1. Same as CK3: proprietary script, documentation gaps
2. Performance issues with large total conversions (1200+ systems)

**Relevance to DINOForge**:
- **Target**: DINOForge should enable Star Wars / WH40K scale mods for DINO
- **Learn**: Community's ability to create total conversions shows value of declarative modding
- **Plan**: DINO is smaller than Stellaris (single map), but same principle applies

**Sources**:
- [Warhammer 40K Total Conversion](https://forum.paradoxplaza.com/forum/threads/mod-warhammer-40k-total-conversion.988421/)
- [GitHub: Stellaris Warhammer Mod](https://github.com/kamip123/Stellaris-Warhammer-30k-Scenario)
- [Stellaris Workshop](https://steamcommunity.com/workshop/browse/?appid=281990)

---

### 6. Total War: Warhammer III - RPFM & Pack Files

**Mod Format**: Binary `.pack` files (edited via RPFM tool)
**File Structure**: `.pack` containers with DB tables, scripts, text (loc files)

**Load System**:
- RPFM (Rusted PackFile Manager) edits `.pack` files
- Mod folders loaded by launcher
- Pack files override vanilla data

**Content Types**:
- Database tables (units, buildings, factions, stats)
- Scripts and event handlers
- Localizations (text strings)

**Dependency Management**: Not formally versioned; manual override system

**Distribution**: Steam Workshop

**No-Code Path**: No—all modding requires RPFM binary editor

**Key Strengths**:
1. **Total War Tradition**: Large total conversion community (Warhammer Fantasy, Napoleonic Era)
2. **RPFM Ecosystem**: Open-source tool handles `.pack` format decoding/encoding
3. **Massive Tables**: Can override unit stats, building effects, faction mechanics

**Key Weaknesses**:
1. **Binary Format**: Mods are `.pack` files, not human-readable
2. **Tool Dependency**: RPFM is required; no other editors
3. **Fragile Compatibility**: Binary format changes break mods across game updates
4. **High Barrier**: Beginners need RPFM tutorial (steep learning curve)

**Relevance to DINOForge**:
- **Avoid**: Binary `.pack` format is not suitable for open-source, collaborative modding
- **Learn Negatively**: RPFM tooling is necessary but shows pain of binary-only modding
- **Contrast**: DINOForge's YAML-based packs are human-readable and git-friendly

**Sources**:
- [RPFM GitHub](https://github.com/Frodo45127/rpfm)
- [RPFM for Dummies](https://tw-modding.com/wiki/Tutorial:RPFM_For_Dummies)
- [Total War Warhammer III Modding](https://gameplay.tips/guides/total-war-warhammer-iii-modding-start.html)

---

### 7. Kerbal Space Program - CKAN Package Management

**Mod Format**: C# plugins (DLLs) + Module Manager `.cfg` config files
**File Structure**: `GameData/` folder hierarchy, `.ckan` metadata files

**Load System**:
- CKAN is a package manager (NixOS/Debian-inspired)
- Mods install with dependencies, version checking
- Module Manager patched game configs without overriding files
- Hot-reload capable

**Content Types**:
- Parts (custom spacecraft components)
- Plugins (mechanics, UI, integrations)
- Configurations (balance tweaks via Module Manager)

**Dependency Management**: **CKAN is exceptional** — semantic versioning, conflict resolution, dependency lockfiles

**Distribution**: CKAN registry, CurseForge, Nexus Mods

**No-Code Path**: Partial—Module Manager allows `.cfg` tweaks without code

**Key Strengths**:
1. **CKAN Package Manager**: Best-in-class dependency resolution (inspired by Debian/CPAN)
2. **Semantic Versioning**: Mods can specify `>=1.0.0, <2.0.0` constraints
3. **Module Manager**: Non-programmer can patch game configs
4. **Multi-Platform**: CKAN runs on Windows, macOS, Linux (Mono-based)
5. **Metadata Standard**: JSON schema ensures consistency

**Key Weaknesses**:
1. Two-tier system (CKAN + Module Manager) is confusing for new modders
2. KSP's modding is historically fragile (breaks on game updates)
3. Requires understanding C# for part definitions

**Relevance to DINOForge**:
- **Adopt**: CKAN's dependency resolution model is gold standard
- **Plan**: DINOForge should implement similar package manager (or wrap NuGet)
- **Target**: Semantic versioning for pack compatibility (`>=0.2.0, <1.0.0`)
- **Avoid**: Don't create two-tier system; keep YAML packs simple

**Sources**:
- [CKAN GitHub](https://github.com/KSP-CKAN/CKAN)
- [CKAN Wiki](https://github.com/KSP-CKAN/CKAN/wiki)
- [CKAN Package Manager (Forum)](https://forum.kerbalspaceprogram.com/topic/90246-the-comprehensive-kerbal-archive-network-ckan-package-manager-v1180-19-june-2016/)

---

### 8. Mount & Blade II: Bannerlord - Official Tools + XML

**Mod Format**: C# modules with `SubModule.xml` manifests
**File Structure**: `/Modules/{ModName}/SubModule.xml`, `/ModuleData/`, `/Bin/`

**Load System**:
- Launcher reads `SubModule.xml` for each mod folder
- Mods inherit from `MBSubModuleBase` (entry point)
- DLL files loaded in order, `OnSubModuleLoad()` called at startup
- XML patches applied to game data

**Content Types**:
- Scenes and maps
- Items and equipment
- Characters and troops
- Custom mechanics (via C# patches)

**Dependency Management**: Mod load order in launcher UI, no automatic versioning

**Distribution**: Nexus Mods, Steam Workshop

**No-Code Path**: Partial—XML mods can define items, troops, scenes; complex behavior requires C#

**Key Strengths**:
1. **Official Tools**: Bannerlord provides visual scene editor and asset tools
2. **SubModule.xml Standard**: Clear manifest format, well-documented
3. **XML + Code**: Separation of concerns (declarative data + imperative logic)
4. **Community Resources**: Good tutorials, module templates on Nexus

**Key Weaknesses**:
1. Load order management is manual (no dependency resolution)
2. C# required for anything beyond XML definitions
3. No versioning enforcement

**Relevance to DINOForge**:
- **Adopt**: SubModule.xml pattern is excellent—DINOForge's `pack.yaml` is equivalent
- **Adopt**: Separation of data (YAML) and behavior (C#) is core DINOForge principle
- **Improve**: Add semantic versioning to pack.yaml (Bannerlord doesn't have this)
- **Learn**: Official tools (scene editor) show value of GUI support

**Sources**:
- [Bannerlord Modding Documentation](https://docs.bannerlordmodding.com/)
- [SubModule.xml Documentation](https://docs.bannerlordmodding.com/_xmldocs/submodule.html)
- [Basic C# Mod Tutorial](https://docs.bannerlordmodding.com/_tutorials/basic-csharp-mod.html)

---

### 9. The Sims 4 - Python + Package Format

**Mod Format**: Python scripts + `.package` binary files
**File Structure**: Mod mods/ folder, .ts4script files

**Load System**:
- Game scans mods/ folder for `.ts4script` files
- Python scripts have direct game API access
- `.package` files contain tuning (game object definitions)
- Sims4Studio tool edits `.package` files

**Content Types**:
- Game objects (furniture, characters, interactions)
- Custom behaviors (Python scripts)
- Tuning files (XML-like data)
- Cheats and automation tools

**Dependency Management**: None (mods are independent)

**Distribution**: CurseForge, The Sims Resource, Nexus Mods

**No-Code Path**: No—even simple object additions require Python

**Key Strengths**:
1. **Python Scripting**: More accessible than C# for new modders
2. **Large Community**: 10K+ active mod creators
3. **Sims4Studio Tool**: Visual editor for `.package` files (reduces friction)

**Key Weaknesses**:
1. Binary `.package` format is not human-readable or git-friendly
2. Python API is poorly documented (reverse-engineered)
3. No dependency management (mods can break each other silently)
4. Decompiling required to understand game code (legal gray area)

**Relevance to DINOForge**:
- **Avoid**: Binary format is a pain point; DINOForge's YAML is better
- **Avoid**: Python for game modding adds complexity; C# is more integrated
- **Learn**: Large community shows value of accessible tooling (Sims4Studio)

**Sources**:
- [Lot 51 Modding Guide](https://lot51.cc/simdex/resources)
- [Sims 4 Modding Wiki](https://sims-4-modding.fandom.com/wiki/Python_Scripting)
- [CurseForge Sims 4 Mods](https://www.curseforge.com/sims4)

---

### 10. Cities: Skylines - C# Workshop Mods

**Mod Format**: C# source code (compiled to DLL, uploaded to Workshop)
**File Structure**: C# project, compiled to `.dll`, uploaded via Steam Workshop

**Load System**:
- Workshop mods auto-downloaded
- DLLs loaded into game runtime (Unity)
- ModTools provides debugging/inspection

**Content Types**:
- Buildings and assets (new city structures)
- Mechanics (traffic, economy, simulation)
- UI overlays

**Dependency Management**: Manual (no versioning)

**Distribution**: Steam Workshop only

**No-Code Path**: No—all mods are C#

**Key Strengths**:
1. **Direct Unity Access**: Mods have full access to game engine
2. **Workshop Integration**: Seamless Steam distribution
3. **ModTools**: In-game debugging tool for mod development

**Key Weaknesses**:
1. **No No-Code Path**: All mods require C# knowledge
2. **No Dependency Management**: Mods can break each other
3. **Compilation Barrier**: C# projects, build tools, knowledge required
4. **Workshop Only**: No alternative distribution channel

**Relevance to DINOForge**:
- **Avoid**: Requiring C# for all mods is high barrier
- **Learn**: Workshop integration is seamless (consider BepInEx mod browser)
- **Contrast**: DINOForge's YAML + optional C# is more inclusive

**Sources**:
- [Cities Skylines Modding Guide](https://citiesskylinesmoddingguide.readthedocs.io/en/latest/modding/Getting-Started/)
- [CSL Modding Info](https://cslmodding.info/)

---

### 11. Minecraft - Data Packs + Fabric/Forge

**Mod Format**: JSON data packs (declarative) or Fabric/Forge mods (Java)
**File Structure**: `data/{domain}/` (JSON), or Java mods with Gradle build

**Load System**:
- Data packs in world folder, auto-discovered
- Fabric/Forge loader handles mod detection and loading
- Modrinth/CurseForge integration

**Content Types**:
- Recipes (crafting, smelting)
- Loot tables (drops, chests)
- Advancements (achievements)
- Custom structures and functions
- Full mechanics via Fabric/Forge mods

**Dependency Management**: Fabric has metadata; many mods don't declare deps

**Distribution**: Modrinth (best), CurseForge, GitHub

**No-Code Path**: **Full** for data packs (JSON only, no code)

**Key Strengths**:
1. **Data Packs**: Pure JSON content without mods
2. **Modrinth**: Modern package manager with good UX
3. **Open Source Ecosystem**: Fabric/Forge are community-driven
4. **Two Tiers**: Beginners use data packs, advanced use Fabric/Forge
5. **Large Community**: Millions of players, thousands of mods

**Key Weaknesses**:
1. Java modding has high barrier (Gradle, JDK, Maven Central)
2. Data packs are JSON-heavy (not intuitive for non-programmers)
3. Fabric/Forge dominance means fragmented mod ecosystem (mods choose one, not compatible)
4. Dependency management is informal (many mods skip metadata)

**Relevance to DINOForge**:
- **Adopt**: Two-tier system (JSON datapacks for basics, C# for advanced) is ideal
- **Adopt**: Modrinth's UX (mod browser, dependency visualization) is excellent
- **Improve**: YAML is clearer than JSON for game configs
- **Learn**: Data packs show value of no-code content patches

**Sources**:
- [Fabric Wiki: Custom Resources](https://wiki.fabricmc.net/tutorial:custom_resources)
- [Forge Documentation: Data](https://docs.minecraftforge.net/en/1.16.x/concepts/data/)
- [Modrinth](https://modrinth.com/)

---

### 12. Skyrim / Fallout 4 - Creation Kit GUI

**Mod Format**: Binary (ESP/ESM files created by Creation Kit)
**File Structure**: `.esp` (plugin) or `.esm` (master) files, edited via GUI

**Load System**:
- Creation Kit GUI editor
- Plugins load on top of master files (ESM)
- Light Masters (ESL) reduce plugin slot limits

**Content Types**:
- Quests, NPCs, dialogue
- Items, weapons, armor
- Locations and interiors
- Scripts and behaviors

**Dependency Management**: Master/plugin hierarchy (ESM are masters, ESP are plugins)

**Distribution**: Nexus Mods (largest collection)

**No-Code Path**: Partial—Creation Kit is GUI-based, but editing still requires understanding game data

**Key Strengths**:
1. **Creation Kit GUI**: Official editor, no file format knowledge needed
2. **Mature Community**: 50K+ mods on Nexus, modding guides for everything
3. **Plugin System**: Clear dependency chain (ESM → ESP)
4. **Modding Culture**: Community thrives (Nexus Mods is gold standard)

**Key Weaknesses**:
1. **Binary Format**: Not git-friendly, hard to review changes
2. **Creation Kit Crashes**: GUI tool is notoriously unstable
3. **Plugin Limits**: 255 ESP limit (improved by ESL, but still constrained)
4. **Merge Complexity**: Combining mods requires merging binary files
5. **No Version Tracking**: Manual plugin management

**Relevance to DINOForge**:
- **Avoid**: Binary format is brittle; DINOForge's YAML is better
- **Learn Negatively**: Creation Kit stability issues show problems with GUI-only tools
- **Contrast**: DINOForge's text-based packs are mergeable and git-friendly
- **Target**: Don't replicate binary plugin system; YAML packs are simpler

**Sources**:
- [UESP Creation Kit Usage](https://en.uesp.net/wiki/Skyrim_Mod:Creation_Kit_Usage)
- [Fallout4 Creation Kit Wiki](https://falloutck.uesp.net/wiki/Data_File)
- [Nexus Mods Skyrim](https://www.nexusmods.com/skyrimspecialedition)

---

### 13. OpenTTD - NewGRF + NML

**Mod Format**: NewGRF binary (compiled from NML high-level language)
**File Structure**: `.grf` binary files, source is NML/M4nfo text

**Load System**:
- GRFs loaded from `newgrf/` folder
- Game applies graphics/content from each GRF in order
- NML compiler generates `.grf` from `.nml` source
- Grfcodec tool for lower-level editing

**Content Types**:
- Vehicles and trains
- Buildings and stations
- Cargo types and industries
- Railtypes and roadtypes

**Dependency Management**: Load order specified in game config, no versioning

**Distribution**: Community forums (TT-Forums), [Steam Workshop](https://steamcommunity.com/workshop/browse/?appid=1536610)

**No-Code Path**: No—requires NML or low-level NFO/M4nfo

**Key Strengths**:
1. **NML Language**: High-level language compiles to binary GRF (abstraction layer)
2. **Open Source**: [Grfcodec](https://github.com/OpenTTD/grfcodec) and [NML](https://github.com/OpenTTD/nml) are open-source (GPL)
3. **Community Resources**: TT-Forums #openttdcoop has extensive development zone
4. **Documentation**: Modding is well-documented despite binary format
5. **Mature Ecosystem**: 20+ year modding tradition

**Key Weaknesses**:
1. **Compilation Step**: Must compile NML → GRF (not live reload)
2. **Binary Format**: Final `.grf` is binary (not mergeable)
3. **Steep Learning Curve**: NML syntax is complex (template language)
4. **No Dependency Management**: Load order is manual

**Relevance to DINOForge**:
- **Adopt**: NML shows value of compiling from readable source to binary (but DINOForge should ship YAML, not compile)
- **Improve**: DINOForge's YAML packs should be directly readable (no compilation)
- **Learn**: TT-Forums community model shows engaged modding community
- **Avoid**: Don't require compilation; YAML is simpler

**Sources**:
- [OpenTTD NewGRF Manual](https://wiki.openttd.org/en/Manual/NewGRF)
- [OpenTTD Development Tools](https://wiki.openttd.org/en/Development/NewGRF/Development%20Tools)
- [NML GitHub](https://github.com/OpenTTD/nml)

---

### 14. Victoria 3 - Jomini Engine Scripting

**Mod Format**: PDX script (text-based, Jomini engine)
**File Structure**: `/common/` folders with `.txt` files

**Load System**:
- Mods in launcher, `.mod` descriptor files
- Steam Workshop integration
- PDX parser loads `.txt` files
- Jomini engine (evolved Clausewitz) handles rendering

**Content Types**:
- Game rules, events, decisions
- Character traits and cultures
- Buildings and technologies
- Map modifications

**Dependency Management**: Mod descriptor lists dependencies

**Distribution**: [Paradox Mods](https://mods.paradoxplaza.com/games/victoria3), [Steam Workshop](https://steamcommunity.com/app/529340/workshop/)

**No-Code Path**: Partial—new rules require PDX script

**Key Strengths**:
1. **Jomini Evolution**: Improved over Clausewitz (64-bit, better performance)
2. **Documentation**: Victoria 3 modding docs are better than CK3
3. **Model Modding**: Can replace 3D models (building meshes, etc.)
4. **Live Editing**: PDX script is relatively readable

**Key Weaknesses**:
1. Same as CK3: proprietary script, incomplete documentation
2. Jomini is complex; learning curve is steep
3. No code generation tools or validators

**Relevance to DINOForge**:
- **Avoid**: PDX script is proprietary
- **Learn**: Jomini shows evolution of game engines (important context, but not directly applicable)
- **Contrast**: DINOForge should use open standards (YAML, JSON) instead of proprietary formats

**Sources**:
- [Victoria 3 Modding Wiki](https://vic3.paradoxwikis.com/Modding)
- [Dev Diary #60: Modding](https://www.paradoxinteractive.com/games/victoria-3/news/victoria-3-dev-diary-60-modding)

---

### 15. Dwarf Fortress - RAW Text Modding

**Mod Format**: RAW text files (declarative object definitions)
**File Structure**: Text files in `/mods/` or `/data/vanilla/*/objects/`

**Load System**:
- Mods folder auto-discovered
- RAW files define creatures, items, materials, etc.
- SELECT/CUT tokens allow partial overrides (don't need full file)
- Game merges mods at startup

**Content Types**:
- Creatures (bodies, castes, interactions)
- Materials and properties
- Items and buildings
- Civilizations and entities
- Languages and translations

**Dependency Management**: None (all mods are independent)

**Distribution**: [Steam Workshop](https://steamcommunity.com/app/975370/community/), [DFFD (Dwarf Fortress File Depot)](https://dffd.bay12games.com/)

**No-Code Path**: **Full** — all modding is RAW text, no code required

**Key Strengths**:
1. **Pure RAW Modding**: Text files, no compilation, no code
2. **SELECT/CUT Tokens**: Mods only need to specify what they change (don't need full file copy)
3. **Declarative Format**: Intuitive structure (BODY_DETAIL_TEMPLATE blocks)
4. **Community Culture**: Strong modding tradition, 20+ year history
5. **OpenSource Engine**: Community maintains compatibility tools (PyDwarf, DF Tools, Material Helper)
6. **Zero Barrier to Entry**: No coding, no special tools needed

**Key Weaknesses**:
1. No dependency management or versioning
2. Mods can conflict silently (last definition wins)
3. RAW syntax is verbose (long files)
4. Limited to DF's design scope (can't add new creature properties without engine mods)

**Relevance to DINOForge**:
- **Adopt**: RAW's SELECT token is brilliant—DINOForge should allow partial overrides
- **Adopt**: Pure declarative format with no code barrier is ideal
- **Target**: `faction.yaml` + `units.yaml` for modders to only override what they need
- **Learn**: Dwarf Fortress modding shows that large games can thrive with text-based modding
- **Plan**: Community tools (like Material Helper) show value of validator/helper tools

**Sources**:
- [Dwarf Fortress Modding Wiki](https://dwarffortresswiki.org/Modding)
- [Steam Guide: Modding & Custom Workshops](https://steamcommunity.com/sharedfiles/filedetails/?id=2903013900)
- [PyDwarf Documentation](https://github.com/pineapplemachine/PyDwarf/blob/master/docs/modding.md)

---

## Key Insights for DINOForge

### 1. **No-Code Path is Essential**

Games with the strongest modding communities (Stardew Valley, RimWorld, Dwarf Fortress, Minecraft datapacks) all support no-code content creation:
- **Stardew Valley**: JSON Content Patcher (zero code)
- **RimWorld**: XML defs (zero code, optional C# patches)
- **Dwarf Fortress**: RAW text (zero code, pure declarative)
- **Minecraft**: JSON datapacks (zero code)

**DINOForge Action**:
- Core pack format (YAML) must be 100% code-free
- Must allow modders to create complete units, factions, buildings, doctrines without C#
- Optional C# for complex behaviors, but not required for balance/content

### 2. **Text-Based Formats Trump Binary**

Games stuck with binary formats (Total War Warhammer III `.pack`, Sims 4 `.package`, Skyrim `.esp`) suffer from:
- Non-mergeable conflicts (can't diff or review changes)
- Tool dependency (RPFM, Sims4Studio, Creation Kit)
- Git-unfriendly (can't version control easily)
- High friction for collaboration

Games with text formats (Dwarf Fortress RAW, Factorio Lua, RimWorld XML, Paradox PDX script) show:
- Community contributions are easier (edit `.txt` files, not binary)
- Version control is natural (git diff shows exact changes)
- Merge conflicts can be resolved manually

**DINOForge Action**:
- Stay with YAML/JSON (human-readable, git-friendly)
- Never ship binary pack format; always ship source
- Design packs to be mergeable (don't require complex tool workflow)

### 3. **Partial Overrides Beat Full Replacement**

Dwarf Fortress RAW's SELECT token is brilliant:
```
[SELECT:creature:dwarf]
[BODY:DWARF_BODY]  # only override body, inherit everything else
```

This beats "replace entire file" approaches because:
- Mods don't need to copy 90% of vanilla content
- Maintenance is easier (changes to vanilla don't break mods)
- Conflicts are fewer (mod only touches what it cares about)

RimWorld defs also support this via inheritance (child defs inherit from parent).

**DINOForge Action**:
- Design pack YAML to allow partial overrides
- Example: `units.yaml` can define a unit with `base: vanilla.archer` and just override `attack: 8`
- Don't require full unit definitions in every pack
- Use JSON Merge Patch (RFC 7386) or similar for merging

### 4. **Dependency Management Matters**

CKAN's semantic versioning is the gold standard:
- `depends_on: ["ecs-bridge>=1.0.0, <2.0.0"]`
- Automatic conflict detection
- Version lockfiles for reproducibility

RimWorld's load order is manual (brittle). Bannerlord has no versioning (fragile).

**DINOForge Action**:
- Require `pack.yaml` to include semantic versions
- Implement CKAN-style resolver (or wrap NuGet)
- Validate: a pack can't depend on conflicting versions
- Store lockfiles in git (reproducible builds)

### 5. **Multi-Stage Loading Prevents Conflicts**

Factorio's data lifecycle (data → updates → final-fixes) is genius:
1. Data stage: Define all new prototypes
2. Updates stage: Modify existing prototypes
3. Final-fixes stage: Last-minute balance tweaks

This prevents load-order spaghetti (Bannerlord, Total War suffer here).

**DINOForge Action**:
- Design pack loading in stages:
  1. Base content (units, buildings, factions)
  2. Balance overrides (stat adjustments)
  3. Conflict resolution (cross-pack rules)
- Document which stage each yaml file is loaded in
- Validators should check for stage violations

### 6. **Avoid the RTS Modding Pitfalls**

**Pitfall 1: Call to Arms / Men of War: Assault Squad 2**
- Binary mission files (not mergeable)
- Proprietary script language (undocumented)
- No dependency management
- High tool requirement (only one editor works)

**Pitfall 2: StarCraft 2 Arcade**
- Requires in-game editor (fragile, hard to use)
- No git integration
- Distribution is only via Blizzard (not open)

**Pitfall 3: Company of Heroes**
- XML modding but no clear content scope
- Mods often conflict with each other
- Load order is fragile

**DINOForge Avoidance**:
- Stay with text-based YAML (mergeable, git-friendly)
- Document exactly what can be modded (schema is law)
- Provide clear load stages (no spaghetti)
- Support distribution beyond BepInEx (Nexus, GitHub Releases, etc.)

### 7. **Community Distribution Matters**

Games with thriving mod communities use **multiple distribution channels**:
- Stardew Valley: Nexus Mods + CurseForge (not just official portal)
- RimWorld: Steam Workshop + Nexus (community chooses)
- Minecraft: Modrinth + CurseForge (multiple loaders, not vendor lock-in)

Games with single distribution (Cities Skylines Steam Workshop only) have less vibrant communities.

**DINOForge Action**:
- Support Nexus Mods as primary (best UX for modders)
- Add GitHub Releases support (direct download)
- Don't force BepInEx mod database-only
- Consider building simple mod browser (like SMAPI/Nexus do)

### 8. **Validation & Tooling Reduce Errors**

SMAPI's [smapi.io/json](https://smapi.io/json) validator is a game-changer:
- Validates manifest.json and content.json before load
- Catches typos, missing dependencies, version conflicts
- Prevents "mod won't load" silent failures

DINOForge should ship validators:
- `dinoforge validate pack.yaml` (checks schema, dependencies, references)
- Web validator (like SMAPI) for GitHub PR checks
- IDE support (VS Code extension with JSON Schema)

### 9. **Optional Beginner UI Tools Boost Adoption**

Games with GUI tools see more modders:
- Mount & Blade II: Official scene editor + SubModule wizard
- Skyrim: Creation Kit (despite being buggy, it lowers barrier)
- Sims 4: Sims4Studio (visual editor, non-programmers can mod)

DINOForge doesn't need this immediately, but consider:
- VS Code JSON Schema for YAML editing (already works)
- Web-based pack builder (future)
- In-game mod browser (future)

### 10. **Licensing & Open Source Matters**

OpenTTD, Minecraft (community tools), Dwarf Fortress all benefit from:
- Open-source modding tools (Grfcodec, Fabric, PyDwarf)
- Community wiki documentation (not proprietary)
- GPL/MIT licenses (mods are shared freely)

Paradox's proprietary PDX script limits documentation (leading to sparse guides).

**DINOForge Action**:
- Ship all tools as open-source (PackCompiler, DumpTools, CLI)
- Document YAML schema on wiki (not closed)
- Encourage community tool development (e.g., pack validators, web browser)
- MIT/GPL license ensures tools stay free

---

## Recommendations for DINOForge Implementation

### Short Term (Current)
1. **Keep YAML pack format** as-is (excellent design)
2. **Add partial override support** (allow `base: unit_archetype` in units.yaml)
3. **Implement CKAN-style dependency resolver** with semantic versioning
4. **Add validation tool** (`dinoforge validate packs/`)
5. **Schema as contract** (JSON Schema for pack.yaml, units.yaml, doctrines.yaml)

### Medium Term (M6)
1. **Multi-stage loading** (data → balance → conflict-resolution)
2. **Nexus Mods integration** (upload/download support)
3. **VS Code extension** with JSON Schema validation
4. **Web validator** (smapi.io-style)
5. **GitHub Actions template** for pack authors

### Long Term (M7+)
1. **Web-based pack builder** (no YAML needed for simple packs)
2. **In-game mod browser** (in BepInEx UI)
3. **Community tools** (diff viewer, conflict detector, balance calculator)
4. **Scripting guide** (when/why to use C# plugins)

---

## Conclusion

DINOForge's YAML-based, text-driven pack system is **already superior** to most industry approaches. The key is to:

1. **Emphasize the no-code path** (make YAML the primary way to mod)
2. **Support partial overrides** (don't force full file replacement)
3. **Add robust dependency management** (CKAN-style versioning)
4. **Validate early and often** (catch errors before game load)
5. **Avoid the RTS pitfalls** (binary formats, undocumented scripting, single distribution)
6. **Community distribution** (Nexus, GitHub, not just BepInEx)

If DINOForge executes on these principles, it will rival or exceed Factorio, Stardew Valley, and RimWorld in modding accessibility and community adoption.

---

## Sources

### Primary Research
- [Factorio Modding Wiki](https://wiki.factorio.com/Modding)
- [Factorio Lua API](https://lua-api.factorio.com/latest/)
- [Content Patcher Documentation](https://stardewvalleywiki.com/Modding:Content_Patcher)
- [SMAPI](https://smapi.io/)
- [RimWorld Modding Wiki](https://rimworldmodding.wiki.gg/)
- [CK3 Modding Wiki](https://ck3.paradoxwikis.com/Modding)
- [Stellaris Workshop](https://steamcommunity.com/workshop/browse/?appid=281990)
- [RPFM GitHub](https://github.com/Frodo45127/rpfm)
- [CKAN GitHub](https://github.com/KSP-CKAN/CKAN)
- [Bannerlord Modding Docs](https://docs.bannerlordmodding.com/)
- [Sims 4 Modding Wiki](https://sims-4-modding.fandom.com/)
- [Cities Skylines Modding](https://citiesskylinesmoddingguide.readthedocs.io/)
- [Minecraft Data Packs](https://wiki.fabricmc.net/tutorial:custom_resources)
- [Skyrim Creation Kit](https://en.uesp.net/wiki/Skyrim_Mod:Creation_Kit_Usage)
- [OpenTTD NewGRF](https://wiki.openttd.org/en/Manual/NewGRF)
- [Victoria 3 Modding](https://vic3.paradoxwikis.com/Modding)
- [Dwarf Fortress Modding](https://dwarffortresswiki.org/Modding)

### Comparative References
- [Factorio Lua Data Lifecycle](https://lua-api.factorio.com/latest/auxiliary/data-lifecycle.html)
- [Mount & Blade SubModule.xml](https://docs.bannerlordmodding.com/_xmldocs/submodule.html)
- [CKAN Metadata Spec](https://github.com/KSP-CKAN/CKAN/wiki)
- [Minecraft Datapacks (Forge)](https://docs.minecraftforge.net/en/1.16.x/concepts/data/)
- [NML GitHub](https://github.com/OpenTTD/nml)
- [PyDwarf Documentation](https://github.com/pineapplemachine/PyDwarf)

