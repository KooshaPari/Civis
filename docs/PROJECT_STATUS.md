# DINOForge Project Status

**Last updated:** 2026-03-28
**Owner:** kooshapari (Koosha Paridehpour)
**Repository:** github.com/KooshaPari/Dino

---

## Milestones Overview

All milestones progress M0 through M13 with completion status and next actions.

### Completed Milestones

| Milestone | Title | Status | Completion Date | Key Deliverables |
|-----------|-------|--------|-----------------|------------------|
| M0 | Reverse-Engineering Harness | ✅ DONE | 2026-01-15 | Entity/component dumps (45K entities), ECS reflection layer |
| M1 | Runtime Scaffold | ✅ DONE | 2026-02-01 | BepInEx plugin, ECS detection, PlayerLoop resurrection, debug overlay |
| M2 | Generic Mod SDK | ✅ DONE | 2026-02-15 | Registries, 10 JSON schemas, ContentLoader pipeline, 46 tests |
| M3 | Dev Tooling | ✅ DONE | 2026-02-28 | PackCompiler, DumpTools, DebugOverlay, CLI commands |
| M4 | Warfare Domain Plugin | ✅ DONE | 2026-03-10 | Archetypes, doctrines, roles, waves, balance system, 31 tests |
| ECS Bridge | Component Mapping Layer | ✅ DONE | 2026-03-15 | ComponentMap (30+ mappings), EntityQueries, StatModifier, VanillaCatalog, AssetSwap skeleton |
| Asset Pipeline | AssetsTools.NET Integration | ✅ DONE | 2026-03-18 | Bundle reading/writing, Addressables catalog, asset import/export |

### In-Progress Milestones

| Milestone | Title | Status | Current Phase | Next Checkpoint |
|-----------|-------|--------|----------------|-----------------|
| M5 | Example Content Packs | 🔄 IN PROGRESS | Asset sourcing & import | Pack validation + gameplay integration |
| Docs | VitePress Documentation Site | 🔄 IN PROGRESS | ADR indexing & architecture diagrams | Specs completion + deployment |
| CI/QA | GitHub Actions + Testing | 🔄 IN PROGRESS | Workflow templates + quality gates | Test coverage target 130+ |

### Upcoming Milestones

| Milestone | Title | Target | Scope |
|-----------|-------|--------|-------|
| M6 | Economy Domain Plugin | Q2 2026 | Rates, trade, balance, economy registry |
| M7 | Scenario Domain Plugin | Q2 2026 | Scripting, conditions, validation, triggers |
| M8 | UI Domain Plugin | Q2 2026 | HUD injection, menu management, layout system |
| M9 | Agent Tooling Evolution | Q2 2026 | Autonomous CLI commands, subagent delegation |
| M10 | Pack Management UI | Q3 2026 | Desktop companion for pack installation/configuration |
| M11 | Multiplayer Support | Q3 2026 | Network sync layer, cross-player state |
| M12 | Performance Optimization | Q3 2026 | ECS profiling, asset bundle optimization |
| M13 | Release & Launch | Q4 2026 | Installer, documentation, GitHub release |

---

## Architecture Decisions (ADRs)

All Architectural Decision Records tracked at `/docs/adr/`.

| ADR | Title | Status | Decision | Impact |
|-----|-------|--------|----------|--------|
| ADR-001 | Agent-Driven Development | ✅ Accepted | DINOForge optimized for autonomous agent development (vibecoding) | Governance model, CLI patterns, test automation |
| ADR-002 | Declarative-First Architecture | ✅ Accepted | YAML/JSON manifests over imperative C# patches | Pack system, content model, validation layers |
| ADR-003 | Pack System Design | ✅ Accepted | Modular content via pack manifests with explicit dependencies | Extensibility, conflict detection, version compatibility |
| ADR-004 | Registry Model | ✅ Accepted | Typed registries with conflict detection for all domains | Units, buildings, factions, weapons, doctrines, etc. |
| ADR-005 | ECS Integration Strategy | ✅ Accepted | Stable Bridge layer isolating ECS internals from mod API | ComponentMap, entity queries, stat modifiers, asset swaps |
| ADR-006 | Domain Plugin Architecture | ✅ Accepted | Plugin-based domains (Warfare first, then Economy/Scenario/UI) | Extensibility, isolation, pluggable feature sets |
| ADR-007 | Observability First | ✅ Accepted (v1) | Logs, overlays, reports, validators as first-class | Debug visibility, runtime monitoring, error reporting |
| ADR-008 | Wrap, Don't Handroll | ✅ Accepted | Always prefer existing libraries/tools over custom implementations | Reduced defect rate, borrowed reliability, agent efficiency |
| ADR-009 | Runtime Orchestration | ✅ Accepted | PlayerLoop resurrection, Harmony hooks for system injection | Execution model certainty, frame timing, background threads |
| ADR-010 | Asset Intake Pipeline | ✅ Accepted | Unified PackCompiler asset workflow (import → validate → optimize → generate) | Asset governance, repeatable processing, content versioning |
| ADR-011 | Desktop Companion | ✅ Accepted | WinUI 3 desktop UI for pack management & monitoring | User-friendly pack control, settings persistence, status dashboard |
| ADR-012 | Fuzzing Strategy | ✅ Accepted | Property-based testing for balance/combat models | Edge case discovery, regression prevention, statistical validation |
| ADR-013 | Duplicate Instance Detection Bypass | ✅ Accepted | Harmony prefix on `Awake()` to detect/suppress BepInEx plugin duplicates | Runtime stability, single-plugin guarantee, clean initialization |

See `/docs/adr/index.md` for full details on each decision.

---

## Open Issues

| Issue ID | Title | Status | Priority | Assigned | Notes |
|----------|-------|--------|----------|----------|-------|
| ISSUE-042 | Duplicate Instance Fatal Error | 🔄 IN PROGRESS | 🔴 CRITICAL | @agent | Harmony prefix fix deployed; testing in progress |
| ISSUE-043 | Video Captures Wrong Window | 🔄 IN PROGRESS | 🔴 CRITICAL | @agent | gdigrab offset targeting needs refinement for multi-monitor |

---

## Current Work In Progress

### 1. M5 Example Content Packs

**Status:** Asset sourcing and integration phase

**Packs in development:**
- `warfare-starwars`: Clone Wars theme (CIS + Republic factions)
  - 25 unit/building assets designed, 3 normalized (Clone Trooper, B2 Droid, AAT Walker)
  - Phase 2C manifest complete with 58 missing unit gap analysis
  - Ready for Phase 2D (bulk model download)

- `warfare-modern`: West vs Classic Enemy theme
  - Framework complete, content sourcing in progress

- `warfare-guerrilla`: Asymmetric faction design
  - Doctrine system validated, unit archetypes ready

**Next:** Phase 2D asset batch download + import workflow (see `/packs/warfare-starwars/PHASE_2C_CIS_SOURCING.md`)

### 2. Agent Tooling Evolution

**Status:** Command autonomy and subagent delegation

**Custom commands (`.claude/commands/`):**
- `launch-game.md` — Direct EXE launch (bypasses Steam mutex)
- `prove-features.md` — Autonomous video proof with TTS voiceover
- `check-game.md` — Game status validation
- `game-test.md` — Gameplay scenario testing
- `entity-dump.md` — Entity analysis and dump
- `asset-create.md` — Asset creation workflow
- `pack-deploy.md` — Pack deployment + validation
- `test-swap.md` — Asset swap integration test

**Evolution goals:**
- Each command runs fully autonomously (no user interaction required)
- Subagent delegation for all work items
- Video proof standard: recording + annotations + neural TTS + window-targeted capture
- Blacklist manual workflows: "user launch", "click button", "test yourself"

**Recent additions:**
- `/prove-features` skill command (autonomous feature proof videos)
- Edge-TTS neural voice integration (Microsoft Aria voice)
- Game window capture via gdigrab with offset targeting

### 3. Desktop Companion UI

**Status:** Stabilization phase

**Recent fixes:**
- WinUI 3 Symbol icon (Repair instead of Code)
- Settings button deduplication (NavigationView built-in + custom footer)
- BoolToVisibilityConverter cast errors resolved
- ConfigureAwait(false) → ConfigureAwait(true) for UI thread safety
- Pack list crash fixed via ObservableObject pattern

**Next:** Feature completeness + integration with main DINOForge runtime

---

## Test Coverage

| Layer | Target | Current | Status |
|-------|--------|---------|--------|
| Unit Tests | 130+ | 80 | 🔄 Expanding (target M5-M6) |
| Integration Tests | 20+ | 12 | 🔄 In progress |
| Pack Validation | All packs | example-balance only | 🔄 M5 focus |
| Asset Pipeline | 15+ scenarios | 8 | 🔄 Growing with M5 assets |

---

## Key Reference Documents

- **CLAUDE.md** — Agent governance, build commands, repository structure
- **CHANGELOG.md** — Complete versioned changelog (Keep a Changelog format)
- **MEMORY.md** — Auto-persisted project memory (architecture, conventions, facts)
- **MASTER_SYNTHESIS.md** — Complete synthesis of all user prompts and decisions from both conversation threads (57+ unique prompts, all milestone plans M0-M13)
- **project_dino_runtime_execution_model.md** — Definitive map of execution contexts in DINO (critical for Runtime layer work)
- **windows_hang_investigation_final.md** — Root cause analysis of Windows CLI hang (.NET runtime limitation, not code defect)
- **windows_build_resolution.md** — Workarounds for Windows users (GitHub Actions, WSL2, Docker)

---

## Quick Links

- **Docs home:** `/docs/index.md`
- **ADRs:** `/docs/adr/` (14 decisions documented)
- **Specs:** `/docs/specs/` (6 technical specs)
- **Issues:** `/docs/issues/` (2 critical issues tracked)
- **Milestones:** `/docs/milestones/` (M5 progress docs)
- **Plans:** `/docs/plans/` (agent tooling evolution strategy)

---

## Next Priorities (by urgency)

1. **M5 Pack Validation** — Deploy warfare-starwars + warfare-modern packs to playable state
2. **Video Proof Fixes** — Resolve multi-monitor window capture (ISSUE-043)
3. **Subagent Delegation** — Implement M9 agent tooling evolution framework
4. **Domain Plugins** — Begin M6 Economy domain plugin design
5. **Documentation** — Complete VitePress site deployment

---

**Generated:** 2026-03-24
**Maintained by:** Planning Agent (Haiku 4.5)
