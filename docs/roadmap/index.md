# Roadmap

DINOForge follows a milestone-based development plan, building from low-level runtime to high-level content packs.

## Milestones

### M0 — Reverse-Engineering Harness :white_check_mark:

**Status**: Complete

Install BepInEx, confirm plugin loading, test plugin, dump entities.

- BepInEx 5.4.23.5 installed with modified Doorstop pre-loader
- Runtime builds to `BepInEx/ecs_plugins/DINOForge.Runtime.dll`
- Entity Dumper, System Enumerator, Debug Overlay scaffolded
- Initial test suite passing (xUnit + FluentAssertions)

---

### M1 — Runtime Scaffold :white_check_mark:

**Status**: Complete

Bootstrap plugin, version detection, logging, ECS introspection, debug overlay.

- BepInEx plugin bootstrap with version-aware initialization
- ECS system and component discovery
- Runtime logging infrastructure
- Debug overlay foundation (F9 key)

---

### M2 — Generic Mod SDK :white_check_mark:

**Status**: Complete

Pack manifest format, registry system, schema validation, override model, dependency resolver.

- Pack manifest model with YAML loader (YamlDotNet)
- Generic `Registry&lt;T&gt;` system with priority layers and conflict detection
- Schema validation pipeline (NJsonSchema)
- Content override model with 4-layer priority (BaseGame -> Framework -> DomainPlugin -> Pack)
- Dependency resolver with cycle detection
- 17 JSON/YAML schemas covering all content types
- Content loader for YAML pack ingestion
- Asset service with addressables catalog support

---

### M3 — Dev Tooling :white_check_mark:

**Status**: Complete

Pack compiler CLI, validator CLI, test harness, diff tools, diagnostics.

- PackCompiler CLI (validate, build, assets commands)
- DumpTools CLI with Spectre.Console interactive terminal UI
- `dinoforge` CLI with 11 commands (status, query, override, reload, watch, dump, resources, screenshot, verify, component-map, etc.)
- MCP Server with 13 game interaction tools for Claude Code integration

---

### M3.5 — QA Harness :white_check_mark:

**Status**: Complete

Bridge Protocol/Client for IPC, integration test suite, CI-runnable tests.

- Bridge Protocol: JSON-RPC message types, IGameBridge interface, typed result models
- Bridge Client: GameClient for out-of-process game communication with connection state management
- Integration test project (`DINOForge.Tests.Integration`) with assertions, fixtures, and test data
- 14 integration tests covering content loading, bridge communication, end-to-end pack application
- CI solution (`DINOForge.CI.sln`) excluding Runtime for headless builds

---

### M4 — Warfare Domain Plugin :white_check_mark:

**Status**: Complete

Factions, doctrines, unit classes, weapons, waves, defenses.

- Faction archetype system (Order, Industrial Swarm, Asymmetric)
- 5 factions: Republic, CIS, West, Classic West Enemy, Guerrilla
- Unit role matrix with 13 shared slots across all factions
- Weapon classes (12 types) and defense tags (8 types)
- Wave composition templates
- Doctrine modifiers
- Balance validation

---

### M5 — First Example Packs :white_check_mark:

**Status**: Complete

Full set of example packs demonstrating all content types.

- `warfare-modern` — West vs Classic West Enemy (Modern Warfare theme)
- `warfare-starwars` — Republic vs CIS (Star Wars Clone Wars theme)
- `warfare-guerrilla` — Asymmetric warfare (Guerrilla faction)
- `example-balance` — Simple stat override balance mod
- `economy-balanced` — Economy balance profiles and trade rules
- `scenario-tutorial` — Tutorial scenario with scripted conditions

---

### M6 — In-Game Mod Menu & HMR :white_check_mark:

**Status**: Complete

F10 mod menu overlay, hot module replacement for YAML packs.

- Mod menu overlay (F10) with pack listing, enable/disable, manual reload
- Mod settings panel for per-pack configuration
- Pack file watcher (HMR) for live YAML reload during gameplay
- Hot reload bridge wiring changes to re-validation and re-application

---

### M7 — Installer & Universe Bible :white_check_mark:

**Status**: Complete

PowerShell/Bash installer, Universe Bible system, pack generator.

- `Install-DINOForge.ps1` (Windows) and `install.sh` (Linux/macOS) installer scripts
- Installer library with programmatic installation support
- Universe Bible system: faction taxonomies, naming guides, style guides, crosswalk dictionaries
- Universe loader for parsing Bible documents
- Pack generator for scaffolding total conversion packs from a Universe Bible

**Completed 2026-03-15**: Universe Bible system working, installers ready.

---

### M8 — Runtime Integration & Video Proof :white_check_mark:

**Status**: Complete

ModPlatform orchestrator wiring SDK to Bridge to UI to HMR + autonomous video proof pipeline.

- `ModPlatform` class owns entire mod platform lifecycle (ADR-009)
- Lifecycle phases: Initialize -> OnWorldReady -> LoadPacks -> StartHotReload -> Shutdown
- Plugin.cs stays thin (bootstrap only, forwards Unity callbacks)
- ECS bridge with component mapping, stat modifiers, entity queries, override applicator
- F9/F10 overlay regression fixed (2026-03-20): Removed Harmony patches, restored HideAndDontSave + DontDestroyOnLoad
- **prove-features v2 pipeline (2026-03-27)**: Remotion video rendering + Edge-TTS voiceover + OmniParser VLM validation
  - 4 MP4 proof videos rendered with captions and transitions
  - 5 MP3 audio tracks with neural voice (Microsoft Aria)
  - OmniParser-based UI element detection for automatic screenshot validation
  - Evidence bundle: `docs/proof-of-features/20260327_213851/`

**Completed 2026-03-27**: End-to-end validation verified, F9/F10 working, video proof pipeline mature.

---

### M9 — Desktop Companion :hourglass:

**Status**: In Progress

WinUI 3 / WindowsAppSDK 1.6 desktop GUI that mirrors F9/F10 in-game panels for out-of-game evaluation.

- WinUI 3 + Mica background + NavigationView shell
- Pack list view (mirrors F10 mod menu) with enable/disable toggle
- Debug panel view (mirrors F9 panel) with 5 collapsible sections
- `disabled_packs.json` read/write parity with game
- No game launch required for pack configuration evaluation
- ADR-011

---

### M10 — Fuzzing Infrastructure :clock3:

**Status**: Planned

Comprehensive property-based and coverage-guided fuzzing across all serialization paths.

- FsCheck expanded from 4 domains / 14 properties to 10+ domains / 30+ properties
- SharpFuzz coverage-guided fuzzing on YAML, JSON, semver, and PackManifest parsers
- Persistent fuzz corpus (`src/Tests/FuzzCorpus/`)
- Nightly CI fuzz gate (`.github/workflows/fuzz.yml`)
- ADR-012

---

### M11 — Test Coverage + Code Completion :clock3:

**Status**: Planned

Fill test gaps and resolve build-excluded incomplete code items.

- `AssetSwapRegistryTests`, `FileSystemPackRootResolverTests`, `PackFileWatcherIntegrationTests` added
- Excluded test re-enablement (UnitSpawner, FactionSystem behind mock ECS)
- Target: 130+ passing tests (up from ~80)
- `ContextualModMenuHost` unblocked (NativeMainMenuModMenu implemented)
- `PackUnitSpawner.OnUpdate()` completed (spawn queue + entity instantiation)
- `HotReloadBridge` entity update logic completed
- Aviation namespace: decision and resolution

---

## Current Stats (v0.12.0+)

- **18 projects** in the solution
- **100+ tests passing** (unit + integration, target 130+)
- **17 schemas** covering all content types
- **6 example packs** complete and playable (Modern, Star Wars, Guerrilla, Economy, Scenario, Balance)
- **4 domain plugins**: Warfare (✅ DONE), Economy, Scenario, UI
- **21 MCP tools** for Claude Code game automation
- **8 CLI commands** via `dinoforge` CLI + custom `.claude/commands/`
- **Autonomous video proof** via Remotion + TTS + OmniParser

---

## Success Criteria

> A new mod can be created mostly by editing validated pack files and running the toolchain, without new reverse engineering.

If every mod still needs runtime surgery, the framework failed.
