# DINOForge - Product Requirements Document

**Version**: 0.6.0
**Status**: Active
**Created**: 2026-03-09
**Updated**: 2026-03-14
**Author**: kooshapari + Agent Org

---

## 0. Document Purpose

This document defines the product and architecture foundation for **DINOForge**, a general-purpose mod platform and agent-oriented development scaffold for **Diplomacy is Not an Option (DINO)**.

It is intentionally broader than a single warfare mod. The goal is to create a reusable framework that can support:

- Warfare total conversions (Star Wars, Modern, Futuristic)
- Balance mods
- Content packs
- Wave/AI mods
- Scenario mods
- UI/QoL mods
- Debugging and introspection tools
- Future domain-specific mod extensions

---

## 1. Executive Summary

### 1.1 Product Name

**DINOForge**

### 1.2 Product Definition

DINOForge is a **general mod runtime, SDK, pack system, and tooling platform** for DINO, built for **agent-driven development** first.

It is not just a mod. It is a **mod operating system** for this game.

### 1.3 Why This Exists

DINO modding requires nontrivial reverse engineering and ECS-aware patching. There is no polished official mod SDK. Ad hoc mod development is brittle, repetitive, and poor for long-term iteration.

The intended development model is **agent-driven**: the human acts as product owner, tester, design director, and failure reporter. Agents handle all coding.

This means the system must be designed for:
- Rigid abstractions
- Declarative schemas
- Machine-checkable contracts
- Validation pipelines
- Debug surfaces
- Deterministic extension points

### 1.4 Strategic Outcome

The first major use case is a **warfare domain plugin** supporting modern and Star Wars themed conversions. The platform itself supports broader mod classes.

**Target end state:**
> New DINO mods can be created primarily by defining validated pack manifests, schemas, mappings, and assets through the DINOForge SDK and toolchain, with minimal or no fresh reverse engineering.

---

## 2. Vision

### 2.1 Vision Statement

Create the canonical mod platform for DINO that transforms brittle one-off reverse-engineered hacks into a structured, extensible, testable, agent-operable ecosystem.

### 2.2 Product Principles

1. **Wrap, don't handroll** - Use established libraries/tools and wrap them. Never build from scratch what a proven package already solves. This is a vibecoding-only environment: agents produce more reliable output integrating proven code than generating novel implementations. Every handrolled component is a liability; every wrapped dependency is borrowed reliability.
2. **Framework before content** - The first product is the platform, not the themed mod.
3. **Declarative before imperative** - Prefer pack manifests, schemas, mappings, and registries over custom patch code.
4. **Stable abstraction over unstable internals** - Low-level engine glue must be isolated from mod authoring surfaces.
5. **Agent-first repository design** - The codebase and docs must optimize for autonomous agent development.
6. **Observability is a first-class feature** - Runtime must explain itself through logs, overlays, reports, validators.
7. **Domain extensibility** - Warfare is the first domain plugin, not the only one.
8. **Compatibility-aware packaging** - Mods must be packs with explicit dependencies, conflicts, versions.
9. **Graceful degradation** - Missing assets/broken mappings fail loudly with fallbacks where safe.

---

## 3. Users

### Primary User
- The mod platform owner / product director using agent-driven development

### Secondary Users
- Autonomous coding agents
- Validation/diagnosis agents
- Content-authoring agents
- Future technical contributors

### Tertiary Users
- End-users who install packs or total conversions built on DINOForge

---

## 4. User Needs

### Product Owner Needs
- Request features in natural language
- Avoid reading source code
- Receive clear diagnostics when things fail
- Iterate on gameplay, balance, and theming quickly
- Add new mod concepts without fresh reverse engineering each time

### Agent Needs
- Clear public APIs
- Typed schemas
- Examples and templates
- Deterministic build/test flows
- Bounded ownership areas
- Machine-readable contracts
- Debugging tools and reports

### End-User Needs
- Install packs safely
- Understand compatibility and conflicts
- Get stable gameplay behavior
- Receive understandable errors when packs fail

---

## 5. Goals

| ID | Goal |
|----|------|
| G1 | General mod framework - reusable runtime + SDK + pack system |
| G2 | Agent-safe development surface - declarative definitions + stable SDK |
| G3 | Runtime observability - logs, overlays, inspectors, validation reports |
| G4 | Domain plugins - modular extensions (warfare, economy, UI, scenarios) |
| G5 | Pack-based delivery - installable, composable packs with metadata |
| G6 | Low-level change isolation - engine/ECS breakage confined to runtime layer |

---

## 6. Non-Goals

| ID | Non-Goal |
|----|----------|
| NG1 | Perfect official-grade SDK parity |
| NG2 | Full automation of every conceivable mod on day one |
| NG3 | Full custom asset pipeline at v1 |
| NG4 | Human-code-review-first workflows |

---

## 7. Core Functional Requirements

### FR1. Runtime Bootstrap
Initialize inside DINO and establish version-aware runtime integration via BepInEx + `ecs_plugins` loader.

### FR2. Runtime Introspection
Provide access to discovered systems, entities, components, resources, or equivalent game structures.

### FR3. Stable SDK Layer
Expose high-level extension interfaces for defining mods without requiring direct low-level runtime patching for common cases.

### FR4. Registry System
Support registries for core extensible domains:
- Units, Buildings, Weapons, Projectiles, Effects
- Audio Packs, UI Skins, Doctrines, Factions
- Wave Templates, Behaviors, Scenario Scripts
- Tech Nodes, Localization Bundles

### FR5. Pack Manifest System
Support installable pack manifests with:
- id, version, dependencies, conflicts
- framework_version compatibility range
- load order hints
- content declarations

### FR6. Schema Validation
Validate pack schemas, dependency graphs, asset references, and ECS registration conflicts before game boot.

### FR7. Pack Compiler
Build tool that validates schema, resolves references, checks missing assets, checks circular deps, builds pack artifacts, emits compatibility metadata.

### FR8. Domain Plugin Architecture
Support modular domain extensions that add domain-specific registries, schemas, and behaviors without modifying core.

### FR9. Debug and Diagnostics
Provide runtime logs, in-game debug overlay, entity dump tools, and validation reports.

### FR10. Content Override Model
Support layered content overrides: base game -> framework defaults -> domain plugin -> pack overrides.

---

## 8. Supported Mod Classes

| Category | Description |
|----------|-------------|
| Content | Units, buildings, projectiles, effects, icons, names, localization, audio, tech tree |
| Balance | Stats, costs, spawn sizes, upgrade tuning, HP/armor/accuracy/fire rate, economy rates |
| Ruleset | Research requirements, build prereqs, wave timings, victory/loss, population, factions |
| AI/Wave | Enemy composition, attack priorities, target selection, escalation logic, event scripting |
| UI/UX | HUD elements, tooltips, overlays, debug inspectors, minimap, faction themes |
| Scenario | Mission scripting, map conditions, scripted events, starting states, faction matchups |
| Utility | Profiler, entity inspector, hot reload, content diff, compatibility checker, save analyzer |

---

## 9. Three-Product Architecture

### Product A - Runtime / Hook Layer
Low-level engine glue. Most brittle, fewest agents should touch.
- Boot into DINO
- Locate systems/entities/components/assets
- Expose safe patch points
- Handle version checks and rollback/fallback
- Expose debug and diagnostics

### Product B - Mod API / Domain SDK
The real scaffold. Where trivial modding becomes possible.
- High-level mod definition interfaces
- Hide engine internals
- Provide schemas, registries, validators, pack loaders
- Support multiple mod classes

### Product C - Mod Packs / Content Packs
Where actual mods live. Mostly declarative and content-driven.
- Warfare modern pack, warfare Star Wars pack
- Balance packs, economy packs
- QoL/UI packs, debug packs

---

## 10. First-Use-Case: Warfare Domain

### Faction Archetypes (3 mechanical families)

| Archetype | Traits | Used By |
|-----------|--------|---------|
| Order | Strong line infantry, reliable DPS, better defenses, higher unit cost | Republic, West |
| Industrial Swarm | Larger numbers, cheaper core, expendable, strong siege | CIS, Classic West Enemy |
| Asymmetric | Light units, mobility, ambush, raid pressure, structure harassment | Guerrilla West Enemy |

### Theme Packs

| Pack | Factions | Theme |
|------|----------|-------|
| Republic vs CIS | Galactic Republic, CIS | Star Wars Clone Wars |
| West vs Enemies | West, Classic West Enemy, Guerrilla West Enemy | Modern Warfare |

### Implementation Order
1. West vs Classic West Enemy (easiest, proves framework)
2. Republic vs CIS (harder art/audio, proves theme-skin abstraction)
3. West vs Guerrilla Enemy (asymmetry, hardest balance)

---

## 11. Milestone Roadmap

| # | Milestone | Description | Status |
|---|-----------|-------------|--------|
| M0 | Reverse-Engineering Harness | Install BepInEx, confirm plugin loading, test plugin, dump entities | Done |
| M1 | Runtime Scaffold | Bootstrap plugin, version detection, logging, ECS introspection, debug overlay | Done |
| M2 | Generic Mod SDK | Pack manifest format, registry system, schema validation, override model, dependency resolver | Done |
| M3 | Dev Tooling | Pack compiler CLI, validator CLI, test harness, diff tools, diagnostics | Done |
| M3.5 | QA Harness | Bridge Protocol/Client, integration tests, CI-runnable test suite | Done |
| M4 | Warfare Domain Plugin | Factions, doctrines, unit classes, weapons, waves, defenses | Done |
| M5 | First Example Packs | West vs Classic Enemy, Republic vs CIS, Guerrilla, economy, scenario | Done |
| M6 | In-Game Mod Menu & HMR | F10 mod menu overlay, hot module replacement for YAML packs | Done |
| M7 | Installer & Universe Bible | PowerShell/Bash installer, Universe Bible system, pack generator | Done |
| M8 | Runtime Integration | ModPlatform orchestrator wiring SDK to Bridge to UI to HMR, end-to-end pack application | Done |
| M9 | Unit Spawner + ECS Integration | PackUnitSpawner, FactionSystem, WaveInjector, VanillaArchetypeMapper | Done |
| M10 | Pack Registry + Discovery | PackRegistryClient, registry.json, compat.json | Done |
| M11 | YAML-only Mods | YAML arrays, stat overrides, content loader | Done |
| M12 | Polyrepo + Submodule Support | dinoforge-packs repo, PackSubmoduleManager, packs.lock | Done |
| M13 | Total Conversion Framework | TotalConversionPlugin, AssetReplacementEngine, VanillaCatalog | Done |

### Current Test Coverage

- **416 tests passing** (402 unit + 14 integration)
- Tests cover: pack loading, registry system, dependency resolution, schema validation, model serialization, warfare domain, economy domain, scenario domain, hot reload, installer, bridge client, content loader, asset service, mod menu, universe bible, skills/waves/squads, unit spawner, faction system, wave injection

### Current Project Count

The solution (`src/DINOForge.sln`) contains **20+ projects**:
- Runtime, SDK, Bridge.Protocol, Bridge.Client
- Domains: Warfare, Economy, Scenario, UI
- Tools: Cli, McpServer, PackCompiler, DumpTools, Installer, Templates
- Tests, Tests.Integration
- Infrastructure: CI configuration projects (MinVer, NetArchTest, CycloneDX, Scorecard integration)

### Current Release Status

**Version**: 0.5.0 (released 2026-03-11)
- M9-M11 complete: Unit Spawner, Pack Registry, YAML-only Mods
- Full CI/QA infrastructure: MinVer versioning, NetArchTest validation, CycloneDX SBOM, Scorecard security analysis, Thunderstore distribution support
- 416+ tests with 60%+ code coverage enforcement
- NuGet distribution: DINOForge.SDK published to nuget.org
- GitHub Actions: automated versioning, release pipeline, dependency scanning

---

## 12. Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| ECS patch fragility | Game updates break hooks | Isolate hook points, version gate, component dumps |
| Performance collapse | Harmony path costs perf | ECS-native modding, batch stat application, cache queries |
| Asset pipeline pain | Model import blocked | Treat models as optional, build around text/icons/VFX/stats first |
| Asymmetry imbalance | Guerrilla faction broken | Build peer warfare first, add insurgent after baseline balance |
| No official API docs | Discovery is manual | Build own introspection tools, maintain component dumps |

---

## 13. Success Criteria

> A new mod can be created mostly by editing validated pack files and running the toolchain, without new reverse engineering.

If every mod still needs runtime surgery, the framework failed.

---

## 14. New Requirements (v0.6.0, 2026-03-14)

### 14.1 Desktop Companion (M9)

**User Requirements**:
- As a mod author, I can evaluate pack list and pack status without launching DINO, so iteration time drops from ~2min to <5s
- As a developer, I can toggle packs enabled/disabled from the companion and the game will respect the change on next launch
- As a developer, I can see the F9 debug panel sections (entity counts, system state, errors) populated from a file dump without an active game session
- As a developer, I can preview F9/F10 UI component layout changes in the companion before testing in-game

**Technical Requirements**:
- Target framework: `net8.0-windows`, unpackaged WinUI 3 app
- No Unity runtime dependency — `PackViewModel` DTO replaces `PackDisplayInfo`
- `disabled_packs.json` round-trips identically between companion and game
- SDK (`DINOForge.SDK.dll`, `netstandard2.0`) directly referenceable from companion
- Pack file watcher reloads companion state on YAML change (500ms debounce, via SDK `PackFileWatcher`)
- Mica material background, NavigationView shell, dark colour tokens matching `DinoForgeStyle.cs`

**Business Requirements**:
- Reduces game-launch friction for mod authors, directly increasing mod ecosystem growth
- Provides a standalone pack manager UI that can be distributed independently of game modding knowledge
- ADR: ADR-011

---

### 14.2 Fuzzing Infrastructure (M10)

**User Requirements**:
- As a pack author, I am guaranteed that a malformed pack.yaml will produce a clear error, not a crash or silent data corruption
- As a developer, I can run `dotnet test --filter Category=Property` in the PR gate with <30s runtime
- As a developer, I can run the nightly fuzz job and get a report of any newly-discovered crash inputs

**Technical Requirements**:
- FsCheck coverage: 30+ properties across 10+ domains (up from 14 properties / 4 domains)
- SharpFuzz targets for: YAML deserialization, JSON schema validation, semver parsing, PackManifest round-trip
- Persistent fuzz corpus: `src/Tests/FuzzCorpus/` committed to git
- CI gate: nightly `.github/workflows/fuzz.yml` running SharpFuzz targets on Linux runner
- All crash-inducing inputs added as regression fixtures before PR merge
- ADR: ADR-012

**Business Requirements**:
- Platform stability is a prerequisite for third-party pack adoption
- A crash from a bad pack.yaml before mod community grows would damage DINOForge reputation

---

### 14.3 Code Completion (M11)

**User Requirements**:
- As a developer, I can build the full solution with zero excluded files for known features (no Compile Remove for non-WIP items)
- As a pack author with aerial units, my `warfare-aerial` pack's units spawn correctly via `PackUnitSpawner`

**Technical Requirements**:
- `ContextualModMenuHost.cs` re-included in build (requires `NativeMainMenuModMenu` implementation)
- `PackUnitSpawner.OnUpdate()` implements spawn queue and entity instantiation
- `HotReloadBridge.cs:127` implements affected-entity lookup + component update
- Aviation namespace: `AerialUnitComponent` defined and 8 Aviation files re-included OR formally deferred to M12 in roadmap
- Test coverage target: 130+ passing tests

---

## 15. Reference Models

Best modding DX/UX to emulate:

| System | What to Copy |
|--------|-------------|
| Factorio | API shape, manifests, dependency/version handling, distribution |
| RimWorld | Split between declarative content and imperative code patches |
| Satisfactory/BepInEx | Mod loaders, plugin bootstrap, community tooling |
| Minecraft Bedrock | Pack schemas, folder conventions, generation/validation |
| UEFN/Roblox | End-to-end creation pipeline concept |

**What NOT to copy:**
- Raw BepInEx "drop DLL in plugins and pray"
- Undocumented patch soup
- Hidden load order rules
- Discord-as-documentation
