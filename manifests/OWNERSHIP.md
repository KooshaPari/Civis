# Module Ownership Map

This document defines ownership boundaries for agent-driven development.
Each module has a single owner type and clear dependency rules.

---

## Runtime Layer (HIGH SENSITIVITY)

**Owner**: Runtime agents only
**Files**: `src/Runtime/`
**Submodules**:
- `src/Runtime/Bridge/` — ECS bridge (component mapping, stat modifiers, entity queries, override applicator)
- `src/Runtime/HotReload/` — Hot reload bridge
- `src/Runtime/UI/` — Mod menu overlay (F10), settings panel
**Touches**: Raw ECS, BepInEx bootstrap, game version detection, hook points
**Dependencies**: Unity.Entities, BepInEx, HarmonyX (sparingly), SDK, Bridge.Protocol
**Rule**: Fewest agents should touch this. Changes require version-gate testing.

---

## SDK Layer (MEDIUM SENSITIVITY)

**Owner**: Architect agents + SDK agents
**Files**: `src/SDK/`
**Submodules**:
- `src/SDK/Assets/` — Asset service, addressables catalog, bundle info
- `src/SDK/Dependencies/` — Pack dependency resolver with cycle detection
- `src/SDK/HotReload/` — Pack file watcher for live YAML reload
- `src/SDK/Models/` — Content data models (units, factions, buildings, weapons, etc.)
- `src/SDK/Registry/` — Generic `Registry<T>` with priority layers and conflict detection
- `src/SDK/Universe/` — Universe Bible system (taxonomies, naming, style guides, pack generator)
- `src/SDK/Validation/` — Schema validation via NJsonSchema
**Touches**: Public mod API, registries, schema validation, pack loading
**Dependencies**: System.Text.Json, YamlDotNet, NJsonSchema, Semver
**Rule**: All public API changes require docs + tests + schema updates.

---

## Bridge Layer (MEDIUM SENSITIVITY)

**Owner**: Bridge agents
**Files**: `src/Bridge/`
**Submodules**:
- `src/Bridge/Protocol/` — JSON-RPC message types, IGameBridge interface, typed result models
- `src/Bridge/Client/` — GameClient for out-of-process communication, connection state management
**Touches**: IPC protocol, game process management
**Dependencies**: System.Text.Json
**Rule**: Protocol changes require both client and server updates.

---

## Domain Plugins (MEDIUM SENSITIVITY)

**Owner**: Domain agents (per-domain)
**Files**: `src/Domains/<DomainName>/`
**Active Domains**:
- `src/Domains/Warfare/` — Factions, doctrines, archetypes, roles, waves, balance
- `src/Domains/Economy/` — Resource rates, trade, balance profiles, validation
- `src/Domains/Scenario/` — Scripted scenarios, conditions, balance, validation
- `src/Domains/UI/` — HUD injection, menu management
**Dependencies**: SDK only (never other domains, never Runtime directly)
**Rule**: Each domain is independent. No cross-domain imports.

---

## Packs (LOW SENSITIVITY)

**Owner**: Pack agents / content agents
**Files**: `packs/<pack-name>/`
**Active Packs**:
- `packs/example-balance/` — Simple stat override example
- `packs/warfare-modern/` — Modern warfare theme
- `packs/warfare-starwars/` — Star Wars Clone Wars theme
- `packs/warfare-guerrilla/` — Asymmetric guerrilla warfare
- `packs/economy-balanced/` — Economy balance profiles
- `packs/scenario-tutorial/` — Tutorial scenario
**Dependencies**: Schemas only
**Rule**: Must pass pack compiler validation. Mostly declarative.

---

## Tools (MEDIUM SENSITIVITY)

**Owner**: Tooling agents
**Files**: `src/Tools/`
**Active Tools**:
- `src/Tools/Cli/` — `dinoforge` CLI with 11 commands
- `src/Tools/McpServer/` — MCP server with 13 game interaction tools for Claude Code
- `src/Tools/PackCompiler/` — Pack validation, build, and asset management
- `src/Tools/DumpTools/` — Entity/component dump analysis (Spectre.Console)
- `src/Tools/Installer/` — PowerShell/Bash installer (+ InstallerLib)
**Dependencies**: SDK, Bridge.Client
**Rule**: Tools must be deterministic and produce machine-parseable output.

---

## Tests (REQUIRED FOR ALL)

**Owner**: All agents (each maintains tests for their module)
**Files**: `src/Tests/` (unit tests), `src/Tests/Integration/` (integration tests)
**Coverage**: 342 tests (328 unit + 14 integration)
**Rule**: Every public API change requires corresponding test update.

---

## Schemas (MEDIUM SENSITIVITY)

**Owner**: Architect agents
**Files**: `schemas/`
**Count**: 17 schemas (JSON and YAML formats)
**Rule**: Schema changes require validator updates, model updates, and test updates.

---

## Docs (ALL AGENTS)

**Owner**: All agents update docs for their changes
**Files**: `docs/`
**Rule**: ADRs for architectural changes. WORKLOG for session summaries. CHANGELOG for releases.
