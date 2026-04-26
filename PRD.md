# Product Requirements Document (PRD)
# DINOForge - Mod Platform for Diplomacy is Not an Option

**Version:** 2.0  
**Date:** 2026-04-05  
**Status:** Production Ready  
**Author:** DINOForge Development Team  
**Game:** Diplomacy is Not an Option (Unity ECS/DOTS)  

---

## 1. Executive Summary

### 1.1 Product Overview

DINOForge is a comprehensive mod operating system for the real-time strategy game *Diplomacy is Not an Option*. Unlike traditional single-purpose mods, DINOForge provides a complete framework, registry system, schema validation, and tooling ecosystem that enables players and developers to create, distribute, and manage any type of game modification—from simple balance tweaks to full total conversion packs.

**Mission Statement:**  
*Transform DINO into a moddable platform where creativity thrives through powerful tools, robust registries, and seamless content distribution.*

### 1.2 Key Capabilities

| Capability | Description | Status |
|------------|-------------|--------|
| Pack System | YAML-first declarative content packs | Production |
| Typed Registries | 10+ content types with layered overrides | Production |
| ECS Bridge | Unity ECS component mapping (30+ mappings) | Production |
| Asset Pipeline | Import → Validate → Optimize → LOD → Prefab | Production |
| Pack Manager | Git submodule-based pack management | Production |
| Desktop Companion | WinUI 3 visual pack manager | Production |
| MCP Server | 13+ game automation tools | Production |
| Hot Reload | Runtime pack reloading without restart | Production |
| Schema Validation | 24 JSON schemas catch errors early | Production |

### 1.3 Current Metrics

| Metric | Value | Target |
|--------|-------|--------|
| Test Count | 1,017+ | 1,500+ |
| Code Coverage | 78% | 85% |
| Active Packs | 8+ example packs | 20+ community packs |
| Downloads | TBD | 10,000+ |
| MCP Tools | 13 | 20+ |

### 1.4 Architecture Highlights

```
┌─────────────────────────────────────────────────────────────────┐
│                     Content Packs Layer                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐            │
│  │ Star Wars│ │  Modern  │ │ Guerrilla│ │ Balance  │            │
│  │   Pack   │ │ Warfare  │ │ Warfare  │ │   Mods   │            │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘            │
│       └────────────┴────────────┴────────────┘                  │
│                          │                                       │
│                          ▼                                       │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │                   SDK Layer (Registries)                  │    │
│  │  Units │ Buildings │ Factions │ Weapons │ Doctrines     │    │
│  └────────────────────────┬───────────────────────────────────┘    │
│                           │                                       │
│                           ▼                                       │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │                  Runtime Layer (ECS Bridge)                 │    │
│  │           BepInEx Plugin + Unity ECS Integration          │    │
│  └──────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Problem Statement

### 2.1 Core Problems Addressed

#### Problem 1: Mod Fragmentation
The DINO modding community has been limited by single-purpose mods that conflict with each other and require manual installation, creating a poor user experience and limiting creative potential.

**Evidence:**
- Manual file replacement causes game corruption
- No standardized content format
- Mods cannot combine or layer safely
- Players fear installing multiple mods

#### Problem 2: Development Complexity
Creating DINO mods requires deep Unity knowledge, reverse-engineering expertise, and manual asset manipulation, creating a high barrier to entry.

**Evidence:**
- Binary asset modification required
- No official modding SDK
- C# programming required for simple changes
- Asset bundle complexity

#### Problem 3: Content Distribution
Mod distribution relies on manual downloads, forum posts, and scattered Discord links with no versioning, dependency management, or update mechanisms.

**Evidence:**
- No centralized mod repository
- Manual update checking
- No dependency resolution
- Version conflicts common

#### Problem 4: Runtime Integration
Mods operate outside the game's systems, leading to crashes, save corruption, and unpredictable behavior.

**Evidence:**
- Hardcoded content IDs break with game updates
- No validation before runtime
- Memory corruption from manual patching
- Debugging is nearly impossible

### 2.2 Target User Pain Points

| User Type | Pain Point | Impact |
|-----------|------------|--------|
| Players | Complex mod installation | Abandon mods, play vanilla |
| Players | Mod conflicts and crashes | Frustration, game abandonment |
| Modders | Steep learning curve | Fewer mod creators |
| Modders | Debugging difficulty | Longer development cycles |
| Server Admins | No mod management tools | Manual, error-prone processes |
| Content Teams | Asset pipeline complexity | Slow content production |

### 2.3 Market Gap

Existing solutions:
- **Manual modding:** High skill requirement, fragile
- **BepInEx plugins:** Low-level, requires C# expertise
- **Asset replacement:** No composition, breaks updates

DINOForge fills the gap by providing:
- Declarative YAML content (no coding required for basic mods)
- Composable packs with dependency resolution
- Runtime integration through ECS bridge
- Professional asset pipeline
- Desktop companion for easy management

---

## 3. Target Users

### 3.1 Primary User Personas

#### Persona 1: Player "Jordan"
- **Demographics:** 25-40 years old, casual to moderate gamer
- **Experience:** Plays 5-10 hours/week, enjoys strategy games
- **Goals:** Enhance gameplay with quality mods, easy installation
- **Pain Points:** Technical barriers, mod conflicts, update hassle
- **Usage Pattern:** Browses packs in Desktop Companion, subscribes to favorites

#### Persona 2: Aspiring Modder "Alex"
- **Demographics:** 20-35 years old, learning game development
- **Experience:** Basic programming, familiar with YAML/JSON
- **Goals:** Create and share mods without learning Unity/C#
- **Pain Points:** Learning curve, debugging, distribution
- **Usage Pattern:** Uses PackCompiler, follows tutorials, shares on Discord

#### Persona 3: Professional Modder "Maya"
- **Demographics:** 30-45 years old, professional game developer
- **Experience:** Deep Unity/C# expertise, shipping mods before
- **Goals:** Build complex total conversions, maintain quality standards
- **Pain Points:** Tooling limitations, asset pipeline, collaboration
- **Usage Pattern:** Creates packs with custom C#, uses full toolchain

#### Persona 4: Server Administrator "Sam"
- **Demographics:** 25-40 years old, manages game servers
- **Experience:** System administration, some scripting
- **Goals:** Maintain stable server with curated mods
- **Pain Points:** Mod version management, stability concerns
- **Usage Pattern:** CLI tools, automated deployment, monitoring

#### Persona 5: Content Creator "Taylor"
- **Demographics:** 20-35 years old, creates YouTube/Twitch content
- **Experience:** Moderate technical skills, content production expertise
- **Goals:** Showcase mods, create unique content experiences
- **Pain Points:** Finding quality mods, quick setup for videos
- **Usage Pattern:** Desktop Companion for quick browsing, F9/F10 hot reload

### 3.2 Secondary User Personas

#### Persona 6: AI Agent "Claude"
- **Type:** MCP-connected AI assistant
- **Goals:** Automate mod development, testing, and analysis
- **Capabilities:** Pack generation, validation, game interaction via MCP
- **Usage Pattern:** MCP server tools, automated workflows

### 3.3 User Needs Matrix

| Need | Jordan | Alex | Maya | Sam | Taylor | Claude |
|------|--------|------|------|-----|--------|--------|
| Easy Installation | Critical | Medium | Low | High | High | N/A |
| Visual Pack Browser | High | Medium | Low | Low | High | N/A |
| No-Code Modding | High | Critical | Medium | Medium | High | N/A |
| Advanced Scripting | Low | Medium | Critical | Low | Low | High |
| Hot Reload | Medium | High | Critical | Medium | Critical | High |
| Conflict Detection | High | Medium | High | Critical | Medium | High |
| Auto-Updates | High | Medium | Medium | Critical | Medium | High |
| MCP Integration | Low | Low | Low | Low | Low | Critical |

### 3.4 User Journey Maps

#### Player Journey: Installing First Mod
1. **Discovery:** Sees DINOForge mod showcase on YouTube
2. **Installation:** Runs PowerShell one-liner to install Desktop Companion
3. **Configuration:** Sets packs directory to game folder
4. **Browsing:** Opens Asset Browser, finds Star Wars pack
5. **Installation:** Click "Add to Game" - pack downloads and installs
6. **Launch:** Starts game through companion or manually
7. **Verification:** Presses F9 to confirm pack loaded
8. **Play:** Enjoys modded gameplay
9. **Updates:** Companion notifies of pack updates

#### Modder Journey: Creating First Pack
1. **Setup:** Installs .NET 8.0 SDK
2. **Template:** Runs `/new-pack my-first-mod` command
3. **Editing:** Opens YAML files in VS Code with schema validation
4. **Iteration:** Runs PackCompiler validate → fix → validate cycle
5. **Testing:** Places pack in game folder, hot reloads with F10
6. **Polish:** Adds thumbnail, description, version
7. **Distribution:** Creates GitHub repo, adds as submodule
8. **Sharing:** Posts to Discord, players subscribe
9. **Maintenance:** Receives feedback, iterates with hot reload

---

## 4. Functional Requirements

### 4.1 Pack System (FR-PACK-001 to FR-PACK-020)

#### FR-PACK-001: Pack Manifest
- YAML-based pack definition (pack.yaml)
- Required fields: id, name, version, author, type
- Optional: description, thumbnail, dependencies, conflicts
- Framework version compatibility declaration

**Acceptance Criteria:**
- [ ] Schema validates all required fields present
- [ ] Semantic versioning enforced
- [ ] Invalid manifests rejected with clear errors
- [ ] Multiple pack types supported (balance, content, total-conversion)

#### FR-PACK-002: Content Types
Support for all major game content types:
- Units (infantry, cavalry, siege, ranged, etc.)
- Buildings (production, defensive, economic)
- Factions (playable civilizations)
- Weapons and projectiles
- Doctrines (faction bonuses)
- Skills and abilities
- Waves (spawn compositions)
- Squads (unit groupings)
- Resources and economies
- Scenarios and campaigns

#### FR-PACK-003: Dependency Resolution
- Declarative dependency specification
- Version range support (semver)
- Topological load order calculation
- Circular dependency detection
- Missing dependency warnings

#### FR-PACK-004: Conflict Detection
- Explicit conflict declaration
- Automatic conflict detection (same content IDs)
- Conflict resolution strategies
- User notification of conflicts

#### FR-PACK-005: Layered Overrides
Priority-based content layering:
- Priority 0: Base game content
- Priority 1000: Framework defaults
- Priority 2000: Domain plugin defaults
- Priority 3000+: Pack content overrides

Higher priority values override lower priority values.

#### FR-PACK-006: Pack Validation
- Schema validation for all YAML files
- Content reference validation (units referenced by factions exist)
- Asset reference validation (textures, models exist)
- Balance calculation validation
- Circular dependency detection

#### FR-PACK-007: Hot Module Replacement
- Runtime pack reloading without game restart
- F9/F10 hotkey integration
- State preservation where possible
- Rollback on reload failure
- Change detection and incremental updates

#### FR-PACK-008: Pack Distribution
- Git submodule-based distribution
- Pack registry/subscription model
- Automatic update checking
- Version locking support

### 4.2 Registry System (FR-REG-001 to FR-REG-015)

#### FR-REG-001: Typed Registries
Type-safe registries for each content type:
- UnitRegistry: All unit definitions
- BuildingRegistry: Building definitions
- FactionRegistry: Faction configurations
- WeaponRegistry: Weapon and projectile data
- DoctrineRegistry: Combat doctrines
- WaveRegistry: Spawn wave definitions

#### FR-REG-002: Registration API
- Register content with ID, data, source, priority
- Override detection and resolution
- Get content by ID
- List all content of type
- Filter by source pack

#### FR-REG-003: Content Queries
- Query by attributes (faction, type, role)
- Filter by tags
- Sort by priority, name, cost
- Pagination support
- Search functionality

#### FR-REG-004: Registry Persistence
- Serialize registry state
- Export/import registry snapshots
- Migration support for version changes
- Audit trail of changes

### 4.3 ECS Bridge (FR-BRIDGE-001 to FR-BRIDGE-020)

#### FR-BRIDGE-001: Component Mapping
- Map mod content to Unity ECS components
- 30+ component type mappings
- Dynamic component composition
- Runtime entity creation

#### FR-BRIDGE-002: Entity Spawning
- Spawn units from registry definitions
- Position and orientation control
- Faction assignment
- Squad composition
- Wave spawning coordination

#### FR-BRIDGE-003: Stat Modification
- Health, armor, damage overrides
- Cost modifications
- Speed, range adjustments
- Build time changes
- Multiplicative and additive modifiers

#### FR-BRIDGE-004: System Integration
- Hook into game systems safely
- Event handling for game events
- Component update propagation
- Delta compression for network sync

#### FR-BRIDGE-005: Entity Dumper
- Dump entity state to files
- Component inspection
- Save file analysis
- Debug output formatting

### 4.4 Asset Pipeline (FR-ASSET-001 to FR-ASSET-020)

#### FR-ASSET-001: Import Pipeline
- Support for FBX, OBJ, glTF models
- Texture import (PNG, JPEG, DDS)
- Audio import (WAV, OGG)
- Metadata extraction

#### FR-ASSET-002: Validation
- Mesh validation (topology, UVs)
- Texture validation (size, format)
- Naming convention enforcement
- Reference integrity checks

#### FR-ASSET-003: Optimization
- Mesh optimization (decimation, welding)
- Texture compression (BCn formats)
- Audio compression
- Unused asset detection

#### FR-ASSET-004: LOD Generation
- Automatic LOD chain generation (100%, 60%, 30%)
- Distance-based LOD switching
- LOD bias configuration
- Manual LOD override support

#### FR-ASSET-005: Prefab Creation
- Unity prefab generation
- Component attachment
- Material assignment
- Addressables registration

#### FR-ASSET-006: Catalog Management
- 38 catalog entries supported
- Catalog versioning
- Asset dependency tracking
- Catalog diff tools

### 4.5 Desktop Companion (FR-COMP-001 to FR-COMP-015)

#### FR-COMP-001: Pack Browser
- Visual pack browser with thumbnails
- Category filtering
- Search functionality
- Sort by popularity, date, rating

#### FR-COMP-002: Pack Installation
- One-click pack installation
- Git submodule management
- Progress indication
- Error handling and rollback

#### FR-COMP-003: Conflict View
- Visual conflict detection
- Side-by-side comparison
- Resolution suggestions
- Manual override capability

#### FR-COMP-004: Update Management
- Automatic update checking
- Changelog display
- One-click updates
- Version pinning

#### FR-COMP-005: Game Integration
- Launch game from companion
- F9/F10 hotkey monitoring
- Live pack status display
- Screenshot capture

#### FR-COMP-006: Settings
- Packs directory configuration
- Game path configuration
- Update frequency settings
- Theme customization (Mica support)

### 4.6 MCP Server (FR-MCP-001 to FR-MCP-020)

#### FR-MCP-001: Game Control
- game_launch: Launch game and wait for bridge
- game_status: Check running state
- game_reload_packs: Hot reload packs
- game_wait_for_world: Wait for ECS world ready

#### FR-MCP-002: Entity Query
- game_query_entities: Query by component type
- game_get_stat: Read entity stat values
- game_apply_override: Apply stat overrides
- game_dump_state: Trigger entity dump

#### FR-MCP-003: Automation
- game_screenshot: Capture game window
- game_ui_automation: Automate UI interactions
- game_input: Inject keyboard/mouse input
- game_navigate_to: Navigate game states

#### FR-MCP-004: Analysis
- game_analyze_screen: UI element detection
- game_verify_mod: Verify mod loaded
- list_packs: List loaded packs
- get_registry: Dump registry contents

### 4.7 Development Tools (FR-DEV-001 to FR-DEV-015)

#### FR-DEV-001: PackCompiler CLI
- validate: Schema and reference validation
- build: Compile pack to distributable format
- assets: Process and optimize assets
- stats: Display pack statistics

#### FR-DEV-002: DumpTools
- Entity dump analysis
- Component diff
- Save file inspection
- Performance profiling

#### FR-DEV-003: Debug Overlay
- In-game F10 debug menu
- Entity inspection
- Registry browsing
- Hot reload trigger
- Pack information display

#### FR-DEV-004: Schema Tools
- JSON Schema generation
- Schema validation
- Auto-completion for editors
- Migration scripts

---

## 5. Non-Functional Requirements

### 5.1 Performance Requirements (NFR-PERF-001 to NFR-PERF-010)

#### NFR-PERF-001: Loading Performance
| Operation | Target | Max |
|-----------|--------|-----|
| Pack discovery | <100ms | <500ms |
| Pack validation (single) | <50ms | <200ms |
| Content loading | <500ms | <2s |
| Hot reload | <2s | <5s |
| Asset loading | <100ms per asset | <1s |

#### NFR-PERF-002: Runtime Performance
- Registry lookup: <1μs
- ECS component mapping: <10μs
- Entity spawn: <1ms
- Stat modification: <100μs
- Memory overhead per pack: <10MB

#### NFR-PERF-003: Desktop Companion Performance
- UI startup: <3s
- Pack list load: <1s for 100 packs
- Thumbnail display: <500ms
- Update check: <5s

#### NFR-PERF-004: MCP Server Performance
- Tool execution: <5s
- Screenshot capture: <1s
- Entity query: <100ms
- State dump: <2s

### 5.2 Reliability Requirements (NFR-REL-001 to NFR-REL-010)

#### NFR-REL-001: Stability
- No crashes from malformed pack content
- Graceful degradation on missing assets
- Recovery from hot reload failures
- Game stability with 20+ packs loaded

#### NFR-REL-002: Data Integrity
- No save corruption from mods
- Consistent state after hot reload
- Rollback capability for failed operations
- Backup creation before destructive operations

#### NFR-REL-003: Error Handling
- All errors catchable and loggable
- User-friendly error messages
- Automatic error reporting (optional)
- Recovery suggestions in errors

#### NFR-REL-004: Compatibility
- Support game version 1.0.x - 1.x.x
- Backward compatibility for packs
- Migration paths for breaking changes
- Version detection and warnings

### 5.3 Security Requirements (NFR-SEC-001 to NFR-SEC-010)

#### NFR-SEC-001: Code Safety
- No unsafe code in pack definitions (YAML only)
- Sandboxed C# execution (if supported)
- No file system access outside packs directory
- Network access restricted (no unauthorized calls)

#### NFR-SEC-002: Content Validation
- All YAML validated against schemas
- Asset file type verification
- Size limits on pack content
- No executable content in packs

#### NFR-SEC-003: Distribution Security
- Git submodule integrity verification
- Optional pack signing
- Update source verification
- Malware scanning integration (future)

### 5.4 Usability Requirements (NFR-USE-001 to NFR-USE-010)

#### NFR-USE-001: Installation Ease
- One-line PowerShell installation
- Automated dependency setup
- Clear error messages for setup issues
- First-run wizard for configuration

#### NFR-USE-002: Documentation Quality
- Tutorial for first pack creation
- API documentation for all public APIs
- Video tutorials for complex workflows
- Troubleshooting guides

#### NFR-USE-003: Visual Feedback
- Progress indicators for long operations
- Success/error notifications
- Visual pack thumbnails
- Conflict visualization

#### NFR-USE-004: Accessibility
- Keyboard navigation support
- Screen reader compatibility (companion)
- Color-blind friendly indicators
- Font size options

### 5.5 Maintainability Requirements (NFR-MAINT-001 to NFR-MAINT-010)

#### NFR-MAINT-001: Code Quality
- 1,017+ tests with 78% coverage
- All code reviewed before merge
- No compiler warnings
- Automated formatting enforcement

#### NFR-MAINT-002: Documentation
- XML documentation for all public APIs
- Architecture Decision Records (ADRs)
- Changelog maintenance
- Migration guides

#### NFR-MAINT-003: Testability
- Unit tests for all components
- Integration tests for pack loading
- Fuzz testing (FsCheck, SharpFuzz)
- Snapshot testing for UI

---

## 6. User Stories

### 6.1 Player Stories

#### US-PLY-001: Easy Mod Installation
**As a** player  
**I want** to install mods with one click  
**So that** I can enhance my game without technical knowledge

**Acceptance Criteria:**
- [ ] Run single PowerShell command to install companion
- [ ] Browse mods visually with thumbnails
- [ ] Install with single click
- [ ] Automatic game integration
- [ ] Update notifications

#### US-PLY-002: Conflict Awareness
**As a** player  
**I want** to know if mods conflict before installing  
**So that** I can avoid crashes and corrupted saves

**Acceptance Criteria:**
- [ ] Automatic conflict detection
- [ ] Visual conflict indicators
- [ ] Clear explanation of conflicts
- [ ] Suggested resolution steps

#### US-PLY-003: Mod Discovery
**As a** player  
**I want** to browse and discover new mods easily  
**So that** I can find content that matches my interests

**Acceptance Criteria:**
- [ ] Category filtering
- [ ] Search functionality
- [ ] Sort by popularity/date
- [ ] Preview screenshots
- [ ] Rating and reviews

### 6.2 Modder Stories

#### US-MOD-001: No-Code Modding
**As an** aspiring modder  
**I want** to create mods without learning C#  
**So that** I can focus on game design not programming

**Acceptance Criteria:**
- [ ] YAML-only pack creation for basic mods
- [ ] Schema validation in editor
- [ ] Template packs for common patterns
- [ ] Tutorial documentation

#### US-MOD-002: Fast Iteration
**As a** modder  
**I want** to see my changes immediately  
**So that** I can iterate quickly on my designs

**Acceptance Criteria:**
- [ ] Hot reload with F10 key
- [ ] Changes visible in <5 seconds
- [ ] No game restart required
- [ ] State preservation where possible

#### US-MOD-003: Distribution Ease
**As a** modder  
**I want** to share my mods easily  
**So that** others can enjoy my creations

**Acceptance Criteria:**
- [ ] GitHub integration for hosting
- [ ] Automatic pack packaging
- [ ] Version management
- [ ] Update distribution

#### US-MOD-004: Debugging Support
**As a** modder  
**I want** clear error messages when something breaks  
**So that** I can fix issues quickly

**Acceptance Criteria:**
- [ ] Schema validation with line numbers
- [ ] Entity dump for runtime inspection
- [ ] Debug overlay in-game
- [ ] Error log with context

### 6.3 Power User Stories

#### US-PWR-001: Advanced Scripting
**As a** power modder  
**I want** to write custom C# code for complex mods  
**So that** I can create total conversions

**Acceptance Criteria:**
- [ ] C# plugin support
- [ ] API documentation
- [ ] Debugging integration
- [ ] Sample projects

#### US-PWR-002: Asset Pipeline
**As a** content creator  
**I want** professional asset import and optimization  
**So that** my models and textures work in-game

**Acceptance Criteria:**
- [ ] FBX/OBJ/glTF import
- [ ] Automatic LOD generation
- [ ] Texture compression
- [ ] Prefab creation

### 6.4 Developer Stories

#### US-DEV-001: MCP Integration
**As an** AI-assisted developer  
**I want** programmatic access to game state  
**So that** I can automate testing and analysis

**Acceptance Criteria:**
- [ ] 13+ MCP tools available
- [ ] Entity querying
- [ ] Screenshot capture
- [ ] UI automation
- [ ] State dumps

#### US-DEV-002: Testing Automation
**As a** QA engineer  
**I want** automated testing capabilities  
**So that** I can verify mod compatibility

**Acceptance Criteria:**
- [ ] Pack validation in CI
- [ ] Integration test framework
- [ ] Fuzz testing support
- [ ] Snapshot testing for UI

---

## 7. Features

### 7.1 Core Features

#### F-CORE-001: Pack System
**Description:** YAML-first declarative content packs

**User Value:**
- No programming required for basic mods
- Human-readable format
- Version control friendly
- Easy to share and collaborate

**Technical Implementation:**
- pack.yaml manifest format
- Schema validation
- Dependency resolution
- Conflict detection

**Example:**
```yaml
id: my-star-wars-pack
name: Star Wars Faction
version: 1.0.0
author: "CloneCommander"
type: content
framework_version: ">=0.1.0"
dependencies:
  - dinoforge-warfare: ">=0.1.0"
loads:
  units:
    - units/
  factions:
    - factions/
```

---

#### F-CORE-002: Typed Registries
**Description:** Type-safe content registries with layered overrides

**User Value:**
- Content composition without conflicts
- Clear override semantics
- Type-safe content access
- Performance optimized lookups

**Technical Implementation:**
- Generic TypedRegistry<T>
- Priority-based layering
- Conflict detection
- Fast hash-based lookup

---

#### F-CORE-003: ECS Bridge
**Description:** Unity ECS component mapping at runtime

**User Value:**
- Seamless game integration
- No manual patching required
- Dynamic content injection
- Safe runtime modification

**Technical Implementation:**
- ComponentMap for type mapping
- EntityQueries for filtering
- StatModifierSystem for overrides
- Safe Unity ECS interop

---

### 7.2 Tooling Features

#### F-TOOL-001: PackCompiler CLI
**Description:** Command-line tool for pack validation and building

**Capabilities:**
- Validate pack structure and content
- Build distributable packages
- Process and optimize assets
- Generate statistics

**Commands:**
```bash
dotnet run --project PackCompiler -- validate packs/my-pack
dotnet run --project PackCompiler -- build packs/my-pack --output dist/
dotnet run --project PackCompiler -- assets packs/my-pack --optimize
dotnet run --project PackCompiler -- stats packs/my-pack
```

---

#### F-TOOL-002: Desktop Companion
**Description:** WinUI 3 application for visual pack management

**Pages:**
- Browse: Visual pack browser with thumbnails
- Installed: Manage installed packs
- Updates: Update management
- Conflicts: Conflict resolution
- Settings: Configuration

**Features:**
- Mica material design
- Git submodule management
- One-click install/update
- In-game overlay mirror

---

#### F-TOOL-003: MCP Server
**Description:** HTTP-based MCP server for game automation

**Tools (13 total):**
1. game_launch - Launch game and wait
2. game_status - Check game state
3. game_query_entities - Query ECS entities
4. game_get_stat - Read entity stats
5. game_apply_override - Apply stat override
6. game_reload_packs - Hot reload
7. game_dump_state - Trigger dump
8. game_screenshot - Capture screen
9. game_verify_mod - Verify mod loaded
10. game_wait_for_world - Wait for ECS
11. game_ui_automation - UI automation
12. game_input - Input injection
13. game_analyze_screen - Screen analysis

---

### 7.3 Content Features

#### F-CONTENT-001: Warfare Domain
**Description:** Complete warfare simulation domain

**Components:**
- Faction archetypes (Order, Industrial Swarm, Asymmetric)
- Doctrines with strategic bonuses
- Unit roles (tank, DPS, support, etc.)
- Wave composition scripting
- Balance calculation formulas

---

#### F-CONTENT-002: Star Wars Pack
**Description:** Example total conversion pack

**Content:**
- 28 units (Republic + CIS)
- 10 buildings
- 2 playable factions
- Visual assets and prefabs
- Addressables catalog entries

---

## 8. Metrics and Success Criteria

### 8.1 Adoption Metrics

| Metric | Current | Target 6mo | Target 12mo |
|--------|---------|------------|-------------|
| Active installations | TBD | 5,000 | 20,000 |
| Published packs | 8 | 50 | 200 |
| Community members | TBD | 1,000 | 5,000 |
| Average packs per user | TBD | 3 | 5 |

### 8.2 Quality Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Test count | 1,017+ | 1,500+ |
| Code coverage | 78% | 85% |
| Crash reports | Minimal | <0.1% of sessions |
| Pack validation pass rate | 95% | 98% |
| User satisfaction | TBD | >4.0/5.0 |

### 8.3 Performance Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Pack load time | <2s | Manual timing |
| Hot reload time | <5s | Manual timing |
| Memory overhead | <50MB | Memory profiler |
| ECS update latency | <16ms | Frame timing |
| MCP tool response | <5s | API timing |

### 8.4 Development Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Build time | <5 min | <3 min |
| Test execution | <2 min | <1 min |
| Release frequency | Monthly | Bi-weekly |
| Open issues | <20 | <10 |
| PR merge time | <3 days | <2 days |

---

## 9. Release Criteria

### 9.1 Milestone Completion

All 14 milestones must be complete:
- [x] M0: Reverse-Engineering Harness
- [x] M1: Runtime Scaffold
- [x] M2: Generic Mod SDK
- [x] M3: Dev Tooling
- [x] M4: Warfare Domain
- [x] M5: Example Packs
- [x] M6: In-Game Mod Menu + HMR
- [x] M7: Installer + Universe Bible
- [x] M8: Runtime Integration
- [x] M9: Desktop Companion
- [x] M10: Fuzzing
- [x] M11: Test Coverage (1,017+ tests)
- [x] M12: Pack Submodule Management
- [x] M13: Asset Browser + Mod Manager
- [x] M14: Asset Library & Catalog

### 9.2 Pre-Release Checklist

#### Code Quality
- [ ] All 1,017+ tests passing
- [ ] Code coverage >= 78% (target 85%)
- [ ] No clippy warnings
- [ ] No security advisories (cargo audit)
- [ ] All public APIs documented
- [ ] CHANGELOG.md updated

#### Pack Validation
- [ ] All example packs validate
- [ ] Star Wars pack loads and runs
- [ ] No pack conflicts detected
- [ ] Asset pipeline tested

#### Desktop Companion
- [ ] Installs via PowerShell one-liner
- [ ] Browse page loads packs
- [ ] Install/uninstall works
- [ ] Update checking functional
- [ ] Settings persist

#### MCP Server
- [ ] All 13 tools functional
- [ ] game_launch works
- [ ] game_screenshot captures
- [ ] game_query_entities returns data
- [ ] game_reload_packs triggers HMR

#### Documentation
- [ ] Installation guide complete
- [ ] Pack creation tutorial done
- [ ] API documentation generated
- [ ] Troubleshooting guide available
- [ ] Video tutorials (optional)

### 9.3 Release Gates

| Gate | Criteria | Owner |
|------|----------|-------|
| CI/CD | All GitHub Actions green | Automated |
| QA | Manual test pass | QA Lead |
| Performance | Benchmarks within targets | Performance Lead |
| Security | Security review complete | Security Lead |
| Docs | Documentation review complete | Docs Lead |
| Final | Product Owner approval | Product Owner |

### 9.4 Post-Release Validation

- [ ] Download and test from clean VM
- [ ] Verify companion installation
- [ ] Test pack installation flow
- [ ] Verify hot reload works
- [ ] Check MCP server connectivity
- [ ] Monitor crash reports

### 9.5 Rollback Criteria

Release must be rolled back if:
- Critical security vulnerability
- Game crashes with common configurations
- Save file corruption reported
- Performance regression >50%
- >5% of users cannot install

---

## 10. Appendix

### 10.1 Glossary

| Term | Definition |
|------|------------|
| Pack | Declarative content bundle |
| Registry | Type-safe content storage |
| ECS | Entity Component System (Unity DOTS) |
| BepInEx | Unity plugin framework |
| HMR | Hot Module Replacement |
| MCP | Model Context Protocol |
| LOD | Level of Detail |
| YAML | YAML Ain't Markup Language |
| Prefab | Unity reusable object template |
| Addressables | Unity asset management system |

### 10.2 Architecture Diagrams

#### Pack Loading Pipeline
```
PackCompiler → ContentLoader → SchemaValidator → DependencyResolver → Registry
     ↓              ↓                 ↓                  ↓              ↓
   YAML files   Discover packs   Validate JSON    Topological    Type-safe
   pack.yaml    in directory     schemas          sort           storage
```

#### Runtime Architecture
```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Packs     │────▶│  Registries │────▶│  ECS Bridge │
│  (YAML)     │     │  (Memory)   │     │  (Unity)    │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       ▼                   ▼                   ▼
  Validation          Query API          Component
  Hot Reload          Override           Mapping
```

### 10.3 Related Documents

- README.md - Project overview
- SECURITY.md - Security policies
- SUPPORT.md - Support channels
- FUZZING.md - Fuzzing documentation
- CONTRIBUTING.md - Contribution guidelines
- RELEASING.md - Release process
- CHANGELOG.md - Version history
- docs/ - Full documentation site

---

**Document Control**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2024-01-20 | DINO Team | Initial release |
| 1.5 | 2024-06-15 | DINO Team | Added MCP server |
| 2.0 | 2026-04-05 | DINO Team | Production ready |

**Review Schedule:** Monthly during active development, quarterly post-release  
**Next Review:** 2026-05-05  
**Approvals Required:** Tech Lead, Product Owner, QA Lead
