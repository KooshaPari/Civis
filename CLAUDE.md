# DINOForge - CLAUDE.md

## Project Overview

**DINOForge** is a general-purpose mod platform and agent-oriented development scaffold for **Diplomacy is Not an Option (DINO)**. It is a **mod operating system**, not a single mod.

- **Game**: Diplomacy is Not an Option (Unity 2021.3.45f2 ECS/DOTS, Mono runtime, BepInEx 5.4.23.5)
- **Architecture**: Polyrepo-hexagonal, declarative-first, agent-driven
- **Language**: C# (.NET), YAML/JSON schemas, CLI tooling
- **Layer stack**: DINO Game → Runtime (plugin + ECS bridge) → SDK (registries/validators/ContentLoader) → Domain Plugins (Warfare/Economy/Scenario/UI) → Packs → User Mods

## .NET Version Policy (MANDATORY — DO NOT CHANGE WITHOUT CHECKING)

**This repo uses .NET 11 preview** (`11.0.100-preview.2.26159.112`, pinned in `global.json` with `latestMajor` rollforward). This is intentional — .NET 11 exists (https://dotnet.microsoft.com/download/dotnet/11.0).

| Project type | TFM |
|---|---|
| Tool/app projects (CLI, PackCompiler, McpServer, Installer) | `net11.0` |
| Core SDK/domain libs (SDK, Bridge.Protocol, Bridge.Client) | `net8.0` + netstandard2.0 compat where consumed by Runtime |
| Runtime BepInEx plugin (DINOForge.Runtime) | `netstandard2.0` ONLY — `net8.0` silently breaks `Plugin.Awake()` (BepInEx ships Mono CLR 4.0; iter-142 incident, Pattern #233) |

- CI installs .NET 11 preview via `include-prerelease: true` in `setup-dotnet`.
- **NEVER downgrade `net11.0` TFMs to net9.0/net8.0.** If CI fails on SDK version, fix the CI workflow to install .NET 11.
- **NEVER target net8.0+ in DINOForge.Runtime.csproj** — use `netstandard2.0`.
- BepInEx plugin consumed as a project ref by net8.0+ tests MUST multi-target `<TargetFrameworks>netstandard2.0;net8.0</TargetFrameworks>` (Pattern #233).

## Agent Operational Rules (MANDATORY)

### Claude (Orchestrator) Constraints
The top-level Claude instance may ONLY read/write documentation files and spawn subagents. **Delegate everything else to subagents** (per `~/.claude` memory: prefer Codex `gpt-5.4-mini`, else haiku): all Bash/shell, file reads beyond first 3 per task or >500 lines, all code edits/writes, builds, deployments, test runs, git ops, log analysis, game launch/kill.

### Tooling Evolution Rule
Continuously collapse multi-step workflows into single optimal CLI/MCP calls. Prefer updating an existing `.claude/commands/` skill or MCP tool over creating new ones. Keep agent plugins, skills, CLI, and MCP server reflecting the shortest path to any common operation.

### Game Launch Protocol (via subagent)
Never assume a launch succeeded. (1) Kill all: `Stop-Process -Name 'Diplomacy is Not an Option' -Force -ErrorAction SilentlyContinue`. (2) Wait 3s, verify none remain. (3) `Start-Process -FilePath '<exe>' -WorkingDirectory '<gamedir>'`. (4) Wait 5s, check `MainWindowTitle` — "Fatal error" / "another instance" = FAILED. (5) Proceed only if title is empty/game. NOTE: per memory, the 8s+MainWindowTitle check alone is insufficient for "blank gray" hangs — augment with pipe-ready/log-mtime/health-loop checks.

### File Deletion Protocol (MANDATORY)
NEVER use `rm`, `del`, `Remove-Item`, or any permanent delete. ALWAYS send to Recycle Bin:
```powershell
powershell -c "Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile('<abs-path>','OnlyErrorDialogs','SendToRecycleBin')"
# directories: ::DeleteDirectory('<abs-path>','OnlyErrorDialogs','SendToRecycleBin')
```

### File Governance (MANDATORY)
- **Desktop contamination**: NEVER write to `C:\Users\koosh\Desktop\` or any user Desktop path.
- All agent output goes to repo dirs: scripts → `scripts/game/`; screenshots/PNG → `docs/screenshots/`; logs/TXT → `docs/sessions/`; research md → `docs/sessions/`; temp capture → `$env:TEMP\DINOForge\` (used by `game_screenshot`).
- **Script lifecycle**: temp scripts go to `scripts/game/` or `docs/scripts/`, are deleted (via Recycle Bin) when the task is done, and are never left as artifacts unless promoted to a named slash command.

### Git Rules (from `~/.claude` memory)
- NEVER `git stash` (user-forbidden). Use `git diff HEAD` / `git status --short`. Under pressure needing a clean tree, auto-route to a dated branch + push.
- NEVER bypass hooks (`--no-verify`, `--no-gpg-sign`, `-c commit.gpgsign=false`, `LEFTHOOK=0`/`LEFTHOOK_EXCLUDE`). Fix the root cause + retry. Branch naming: `safety/iter<N>-snapshot-DATE`, `stash/auto-DATE-HHmm-reason`.
- No git checkout/branch/rebase/merge/reset/stash from orchestrator while agents do background git work — use worktrees. Delegation prompts MUST use SHA-anchored branch creation or assert branch before any non-read git op.
- Long pushes (lefthook pre-push 25-35min): run as background Bash from orchestrator; pre-kill stale testhosts.

## Build Commands

```bash
dotnet build src/DINOForge.sln                                  # Build
dotnet test src/DINOForge.sln --verbosity normal                # Test
dotnet format src/DINOForge.sln --verify-no-changes             # Lint
dotnet run --project src/Tools/PackCompiler -- validate packs/  # Validate packs
dotnet run --project src/Tools/PackCompiler -- build packs/<pack-name>  # Package a pack
```
NEVER mark work done without `dotnet build src/DINOForge.sln -c Release` exit 0. Build-green + log-present are NOT proof of correctness — assume broken until externally verified.

## Repository Structure

```
src/
  Runtime/        BepInEx plugin: bootstrap, ECS detection, debug overlay
    Bridge/       ECS bridge: ComponentMap, StatModifier, EntityQueries, VanillaCatalog
    HotReload/  UI/   Hot module reload; in-game mod menu (F10) + settings
  SDK/            Public mod API: Registry/, Validation/ (NJsonSchema), Assets/, Models/,
                  Dependencies/ (resolver w/ cycle detection), HotReload/, Universe/
  Bridge/Protocol Bridge/Client    JSON-RPC types + IGameBridge; GameClient
  Domains/        Warfare / Economy / Scenario / UI  (domain plugins)
  Tools/          Cli (dinoforge, 22 cmds), DinoforgeMcp (FastMCP), PackCompiler,
                  DumpTools, Installer, McpServer (DEPRECATED)
  Tests/          xUnit + FluentAssertions; Integration/, ParameterizedTests/ (FsCheck)
packs/            example-balance, warfare-modern/-starwars/-guerrilla, economy-balanced, scenario-tutorial
schemas/          Canonical JSON/YAML schemas (29)
docs/  manifests/   Documentation; system contracts, ownership maps, extension points
```

## Agent Governance

**Agents MUST**: work through manifests/registries; use generators/templates for new content; update docs/contracts when changing public surfaces; add tests for new public APIs; log failure modes; keep features pack-based; run `dotnet test` before considering work complete; update CHANGELOG.md / README.md / VERSION as applicable; use Mermaid diagrams in docs.

**Agents MUST NOT**: handroll what a library solves (search packages first); patch runtime internals unless assigned runtime work; invent registry patterns; duplicate schemas; bypass validators; hardcode content IDs in engine glue; add undocumented extension points; skip tests; merge without compatibility checks.

**Legal Move Classes** (reduce all work to one): create schema · extend registry · add content pack · patch mapping · write validator · add test fixture · add debug view · add migration · add compatibility rule · add documentation manifest.

## Code Style

C# 12+ with nullable reference types. `async/await` over raw Tasks. XML doc comments on all public APIs. Immutable data models preferred. Registry pattern for all extensible domains. No `var` for non-obvious types. Meaningful names over comments.

## Key Design Principles

1. **Wrap, don't handroll** — use established libraries/tools, thin wrappers over custom code. Vibecoding-only environment: maximize coverage, minimize risk by standing on existing shoulders.
2. Framework before content. 3. Declarative (YAML/JSON) before imperative (C# patches). 4. Stable abstraction over unstable internals (isolate ECS glue). 5. Agent-first repo design. 6. Observability is first-class. 7. Domain extensibility. 8. Compatibility-aware packaging (explicit deps/conflicts/versions). 9. Graceful degradation (fail loudly with fallbacks).

### Build vs Wrap Decision Rule
**ALWAYS prefer** (in order): (1) direct use of existing lib/tool; (2) thin wrapper/adapter; (3) composition of libs; (4) modified fork (last resort). **ONLY handroll when**: no existing solution covers the need (e.g. DINO-specific ECS glue), wrapping would be more complex, or scope is tiny (<50 lines).

| Need | DO | DON'T |
|------|----|-------|
| Schema validation | JsonSchema.Net / NJsonSchema | custom validator |
| Dependency resolution | NuGet resolver / Semver.NET | custom semver solver |
| Logging | Serilog / NLog via BepInEx | custom logger |
| CLI | System.CommandLine / Spectre.Console | custom arg parser |
| Config | BepInEx ConfigurationManager | custom config |
| ECS introspection | wrap Unity.Entities reflection | custom reflection |
| File watch / hot reload | FileSystemWatcher | custom polling |
| Serialization | YamlDotNet + System.Text.Json | custom parsers |
| Diffing | DiffPlex | custom diff engine |
| Testing | xUnit + FluentAssertions + Moq | custom test framework |

## Pack System

Every mod is a pack with a `pack.yaml` manifest:
```yaml
id: example-pack
name: Example Pack
version: 0.1.0
framework_version: ">=0.1.0 <1.0.0"
type: content  # content | balance | ruleset | scenario | total_conversion | utility
depends_on: []
conflicts_with: []
loads: { factions: [], units: [], buildings: [] }
```

**Load sequence**: Discovery → YAML parse/deserialize → `IValidatable.Validate()` (fail → skip pack) → compatibility check (deps exist, conflicts absent; fail → deactivate) → Registry insert → Runtime active.

## Testing Philosophy

BDD-first (behavior specs define acceptance criteria), SDD (Spec-Driven Development), TDD (unit tests for all public API surfaces), property-based tests (FsCheck) for balance/combat, pack validation tests, integration tests against mock ECS runtime. Methodologies: SDD, BDD, TDD, DDD, ADD (agent-driven), CDD (contract-driven).

## Asset Pipeline (summary — full governance in `docs/asset-pipeline-governance.md`)

All asset operations (3D/textures/VFX) MUST go through **PackCompiler commands**, never fragmented/legacy tools. Every pack with assets defines `asset_pipeline.yaml` (schema: `schemas/asset_pipeline.schema.json`). Mandatory order: Define → `sync download` → `assets import` → `assets validate` → `assets optimize` → `assets generate` → `assets build` → commit. Bundles MUST be built with Unity 2021.3.45f2 (others fail silently). Bundle filename = `visual_asset` key = Addressable key. Do NOT manually edit definitions, skip steps, create ad-hoc asset dirs, or hardcode polycount/LOD in C#.

## Game Automation & Testing

### Game Install Path
```
G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\
  Diplomacy is Not an Option.exe   ← launch directly (not via Steam) for 2nd instance
  BepInEx\plugins\DINOForge.Runtime.dll   ← deployed by: dotnet build -p:DeployToGame=true
  BepInEx\dinoforge_packs\                ← deployed packs (auto-copied on build)
  BepInEx\dinoforge_debug.log             ← DINOForge Runtime log (swap/entity info)
  BepInEx\LogOutput.log                   ← BepInEx log (scene changes, plugin load errors)
```

### Deploying Fixes
```bash
# MAIN instance (overwrites main save — safe for CI/CD):
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true -p:TargetFramework=netstandard2.0
# TEST instance (isolated, no save impact — use during active dev):
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true -p:TargetFramework=netstandard2.0 -p:GameInstallPath="G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST"
# Check after ~12s (600-frame delay):
tail -50 "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log"
```
- **DeployToGame copies ALL `DINOForge.*.dll`** (Runtime + SDK + Bridge.Protocol + Bridge.Client + Domains), not just `DINOForge.Runtime.dll` — the SDK/Protocol deploy gap was fixed (#942). Verify a deploy by file hash/timestamp, NOT by build exit 0 (MSBuild can silently no-op — Pattern #530; always pass `-p:TargetFramework=netstandard2.0`).
- After any TFM change: `Remove-Item obj/, bin/ -Recurse; dotnet clean; dotnet build --no-incremental` and test actual runtime behavior, not reflection metadata (Pattern #233).

### Test Instance (Second Concurrent Game)
Unity's native mutex (UnityPlayer.dll) blocks a 2nd instance from the same dir. Test instance: `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST\` (independent BepInEx). MCP `game_launch_test(hidden=True)` launches it on a hidden Win32 desktop. Path config: `.dino_test_instance_path` in repo root (auto-detected by MCP server).

### MCP Bridge
Canonical: **FastMCP Python** server, HTTP on `http://127.0.0.1:8765`, managed by `scripts/start-mcp.ps1` (`-Action start -Detached` keeps it alive across sessions). The C# `src/Tools/McpServer/` is **DEPRECATED** (in-tree for reference only). All new MCP tool work lands in the Python server (`src/Tools/DinoforgeMcp/dinoforge_mcp/server.py`).

Key tools: `game_launch` / `game_launch_test` (`hidden=True` uses CreateDesktop), `game_status`, `game_query_entities`, `game_get_stat`, `game_apply_override`, `game_reload_packs`, `game_dump_state`, `game_screenshot`, `game_verify_mod`, `game_wait_for_world`, `game_ui_automation`, `game_analyze_screen` (OmniParser UI detection), `game_input` (Win32 SendInput, no focus needed), `game_wait_and_screenshot`, `game_navigate_to`. ~43 tools total (asset/catalog/logging/pack surfaces). Note: `asset_import`/`asset_optimize` advertise Rust accel but run the Python pass-through (#595); 2 duplicates pending consolidation (#555/#595). Isolation/playCUA backend detail in `docs/playcua-backends.md`.

### Agent Slash Commands for Game Work
`/launch-game` (2nd instance) · `/test-swap` (build→deploy→launch→verify) · `/check-game` (read debug log) · `/game-test` (MCP suite) · `/game-test-task <task>` (TITAN coverage-driven) · `/game-coverage` · `/entity-dump` (archetype analysis) · `/pack-deploy` · `/asset-create`.

### Critical ECS Facts (DO NOT FORGET)
- **ALL DINO entities are ECS Prefab entities** — every `EntityQuery` MUST use `EntityQueryOptions.IncludePrefab` or it returns 0 results.
- `World.Systems` returns `NoAllocReadOnlyCollection` — index access only, NO IEnumerable cast.
- `MonoBehaviour.Update()`/`OnGUI()`/PlayerLoop callbacks NEVER run (DINO replaces Unity's PlayerLoop) — use `SystemBase.OnUpdate()`. Only reliable: `Awake()`/`OnEnable()`/`OnDestroy()`, `SceneManager.activeSceneChanged`, Win32 bg thread.
- F9/F10 work via Win32 `GetAsyncKeyState` bg thread → `KeyInputSystem` (SetActive from bg thread OK in Mono 2021.3 for DontDestroyOnLoad).
- `Resources.FindObjectsOfTypeAll` from a bg thread DEADLOCKS during asset loading — never call off main thread.
- DINO system groups (`Systems.ComponentSystemGroups.*`) only fire during active gameplay, not main menu.
- Asset swap Phase 2 (live entity swap) is the primary visual mechanism; Phase 1 (catalog disk patch) is best-effort. Addressables catalog uses custom address keys, NOT Unity asset paths, for unit prefabs.
- No explicit Faction component — factions implicit via Enemy tag + unit type markers. ~45,776 entities in Default World, 6 worlds. See `project_dino_runtime_execution_model.md` for the full execution model.

### AgilePlus PM Dashboard
**AgilePlus** (kooshapari/agileplus) at `C:\Users\koosh\agileplus` is the spec-driven PM engine. Launch: `cd C:\Users\koosh\agileplus && bun run dev`. Specs in `docs/specs/` map to AgilePlus stories.

## Pattern Catalog (active CI gates)

Full prose (Smell / Why-bad / Detection / Governance) per pattern lives in **`docs/qa/pattern-catalog.md`**. The table below is the index — follow the pointer for detail and allowlist paths. Suppression markers are inline `// <pattern-marker>: <reason>` plus per-pattern allowlist files under `docs/qa/`.

| # | Name | Gate | Threshold / notes |
|---|------|------|-------------------|
| 99 | Unprotected `Dictionary<string,T>` w/o StringComparer | CI script | HIGH > 10 |
| 100 | Direct DateTime in SDK API surface | CI script | TimeProvider required |
| 101 | Stringly-typed enum discriminator | CI script | use JsonStringEnumConverter |
| 102 | Orphan Process handle leakage | CI script | wrap in `using` |
| 103 | Local-time logging drift | CI script | log UTC |
| 104 | Catch-swallow-default erasure | CI script (pending #303) | surface/log/rethrow |
| 105 | Event-subscription lifecycle asymmetry | CI script (pending #308) | pair += / -= |
| 106 | Implicit `File.ReadAllText` encoding | CI script (pending #313) | UTF-8 explicit |
| 107 | `BuildServiceProvider` w/o ValidateOnBuild | CI script (pending #316) | ValidateOnBuild=true |
| 108 | Sleep-based test sync | CI script (pending #322) | use TestWait.UntilAsync |
| 109 | Inline `JsonSerializerOptions` construction | CI script | one static holder/project |
| 110 | Open-ended count assertion | CI script | HIGH > 50 |
| 111 | Silent exception swallowing (bare `catch {}`) | CI script | DANGEROUS > 50 |
| 112 | Unadjustable time source (direct DateTime) | CI script | fail > 87 (~82) |
| 113 | Blocking poll w/ hardcoded sleep | CI script (pending #340) | fail > 8 (~12) |
| 114 | CancellationToken accepted but not threaded | CI script (pending #345) | warn > 5 HIGH |
| 115 | HttpClient per-call/per-ctor | CI script (#352) | static readonly / DI |
| 116 | Sync-over-async (`.Result`/`.Wait()`) | CI script (pending #356) | HIGH > 5; CRITICAL in GameBridgeServer |
| 117 | StringBuilder capacity not pre-sized | CI script | HIGH > 5 |
| 120 | `JsonSerializer.Deserialize` w/o options | CI script | HIGH > 5 |
| 121 | Unnecessary LINQ terminal allocation | CI script (pending #375) | prefer IEnumerable/AsReadOnly |
| 123 | Public collection mutability in DTOs | CI script | NuGet HIGH > 5; use IReadOnlyList init |
| 124 | Unsealed public classes in NuGet assemblies | CI script (pattern-gates.yml) | default `sealed` |
| 125 | Service interfaces w/o test doubles | CI script (pending) | ≥3 prod refs → add Mock/Fake |
| 220 | Unsealed concrete class w/ mutable state | Roslyn **DF1013** (Info) | canonicalized into #124 detector |
| 221 | Hardcoded numeric thresholds (≥100) | Roslyn **DF1014** (Info) | extract to const/readonly |
| 222 | Method body > 60 lines | Roslyn **DF1015** (Info) | decompose; dispatcher exempt |
| 231 | Static init with I/O side effect | CI script | NuGet HIGH = 0; use Lazy<T> |
| 232 | Unbounded append-only logging | RETIRED (iter-143 w2) | rotate at ~100MB |
| 233 | Stale `obj/` cache during TFM migration | manual / future hash-check | clean obj/+bin/ on TFM change |
| 234 | Test fixture IDs leaking into packs | CLOSED (MSBuild exclude); detector pending | fixtures in `src/Tests/Fixtures/` |
| 235 | GraphicRaycaster w/o EventSystem guard | grep check | ensure EventSystem.current first |
| 530 | MSBuild deploy target silent no-op (multi-TFM) | manual review / future gate | pair w/ DF0530 warning guardrail |

## Infra Reality Notes (do not build on vaporware)

- **PhenoCompose** — vaporware/research-only as of iter-144; repo does NOT exist under KooshaPari/. Do not author code/MCP tools/roadmap assuming it exists. See git history.
- **Steamless / MockSteamworks** — vaporware as of iter-144; NO source in repo. DINO launches via Steam directly (or test-instance path). Do not predicate features/CI gates/sandbox on them.
- HiddenDesktopBackend is the current stable isolation tier; playCUA is unexercised. Full backend detail: `docs/playcua-backends.md`.

## Reference Pointers

- Full pattern catalog prose → `docs/qa/pattern-catalog.md`
- Asset pipeline governance (full) → `docs/asset-pipeline-governance.md`
- Isolation/playCUA backends → `docs/playcua-backends.md`
- v0.23.0 release notes (historical) → `docs/releases/v0.23.0-release-notes.md`
- DINO runtime execution model → `project_dino_runtime_execution_model.md` (memory)
- Time-provider governance → `docs/qa/pattern-112-time-provider.md`
