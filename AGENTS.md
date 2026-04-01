# DINOForge — Agent Collaboration Guide

## Kilo Gastown Integration

**This repo is a Kilo Gastown rig.**

| Property | Value |
|----------|-------|
| Kilo Rig ID | `6c6d4555-91e8-4f06-a974-018cf3e766d2` |
| Town | `78a8d430-a206-4a25-96c0-5cd9f5caf984` |
| Convoy | `c61d464c-2332-489e-becb-ebc5d1efa639` |
| Worktree Branch | `convoy/methodology-dino/c61d464c/head` |

## Stack

- **Game**: Diplomacy is Not an Option (Unity ECS, BepInEx)
- **Language**: C# (.NET), YAML/JSON schemas
- **.NET**: 11.0.100-preview.2.26159.112 (preview, pinned in `global.json`)
- **Tool projects**: `net8.0` / `net11.0`; Core SDK/domain libs: `netstandard2.0`
- **Build**: `dotnet build src/DINOForge.sln`
- **Test**: `dotnet test src/DINOForge.sln --verbosity normal`
- **Lint**: `dotnet format src/DINOForge.sln --verify-no-changes`
- **Pack validation**: `dotnet run --project src/Tools/PackCompiler -- validate packs/`

### Kilo Coordination Tools

Use these tools for work delegation and agent coordination:

- **`gt_sling`** — Delegate a single bead (work item) to another agent
- **`gt_sling_batch`** — Delegate multiple beads at once (batch delegation)
- **`gt_prime`** — Refresh session context (hooked bead, mail, open beads)
- **`gt_done`** — Push branch and submit for review (transitions bead to `in_review`)
- **`gt_bead_close`** — Close a bead after work is complete and merged
- **`gt_bead_status`** — Inspect any bead's current state
- **`gt_mail_send`** — Send a typed message to another agent
- **`gt_mail_check`** — Read and acknowledge pending undelivered mail
- **`gt_escalate`** — Escalate an unresolved issue (creates escalation bead)
- **`gt_checkpoint`** — Write crash-recovery state to your agent record
- **`gt_status`** — Emit a plain-language dashboard status update
- **`gt_nudge`** — Real-time nudge to another agent (wake/idle/queue modes)

### Work Delegation Pattern

When you receive a bead, execute it immediately. If the work must split across multiple agents:

1. Use `gt_sling` or `gt_sling_batch` to delegate sub-beads to other agents in the rig
2. Each delegated bead is independently tracked and can be closed separately
3. Do NOT close a parent bead until all delegated sub-beads are resolved
4. Use `gt_mail_send` for coordination messages; use `gt_nudge` for time-sensitive wakes

### Convoy Pattern

This rig participates in **convoys** — cross-repo methodology propagation trains:

- Feature branches follow the naming convention: `convoy/<feature>/<convoy-id>/head`
- Convoys carry methodology artifacts (AGENTS.md, CLAUDE.md, GEMINI.md) across rigs
- When a convoy lands, its methodology changes propagate to all participating rigs

## Quick Start for New Agents

1. Read CLAUDE.md (governance, build commands, architecture)
2. Run `dotnet build src/DINOForge.sln` to verify your environment
3. Run `dotnet test src/DINOForge.sln` — all tests must pass before any commit
4. Check your assigned domain in the Agent Roster below

## Agent Roster & Domain Ownership

| Agent Role | Domain | Key Files | Can Modify |
|-----------|--------|-----------|-----------|
| runtime-specialist | ECS bridge, BepInEx | src/Runtime/ | Plugin.cs, Bridge/*, HotReload/*, DebugOverlay, ModPlatform.cs |
| sdk-architect | Registry, SDK, schemas | src/SDK/ | Registry/*, Models/*, Validation/*, Assets/*, Dependencies/*, Universe/* |
| warfare-designer | Warfare domain, balance | src/Domains/Warfare/ | Archetypes/*, Doctrines/*, Roles/*, Waves/*, Balance/* |
| pack-builder | Content packs, YAML | packs/ | packs/**/*, any pack.yaml |
| toolsmith | CLI tools, PackCompiler | src/Tools/ | PackCompiler/*, DumpTools/*, Cli/*, McpServer/* |
| qa-engineer | Tests, CI/CD | src/Tests/, .github/ | Tests/**, workflows/*, test fixtures |
| docs-curator | Documentation, VitePress | docs/ | docs/**, CHANGELOG.md, README.md, AGENTS.md |

## Decision Tree: What To Do First

```
Start task
│
├─ "Where does this change live?"
│   ├─ Game engine glue → src/Runtime/ (runtime-specialist)
│   ├─ Data model or registry → src/SDK/ (sdk-architect)
│   ├─ Domain logic → src/Domains/<Domain>/ (domain specialist)
│   ├─ Pack content → packs/ (pack-builder)
│   ├─ CLI / tooling / MCP → src/Tools/ (toolsmith)
│   ├─ Tests → src/Tests/ (qa-engineer)
│   └─ Documentation → docs/ or CHANGELOG.md (docs-curator)
│
├─ "Does a schema exist for this data shape?"
│   ├─ Yes → validate against it before writing
│   └─ No → create schema first (create schema legal move)
│
├─ "Does a library solve this?"
│   ├─ Yes → wrap it (ADR-008: wrap-don't-handroll)
│   └─ No → handroll only if < 50 lines and self-contained
│
└─ "Will this affect public API?"
    ├─ Yes → update .claude/contracts/ and docs
    └─ No → proceed with implementation
```

## Handoff Protocol

When completing work, always:
1. Run `dotnet test src/DINOForge.sln` — verify 0 failures
2. Run `dotnet format src/DINOForge.sln --verify-no-changes`
3. Update CHANGELOG.md [Unreleased] section (see Keep a Changelog format)
4. Commit with descriptive message + `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`
5. `git push origin main`

## Parallel Agent Coordination

- Always `git pull --rebase origin main` before starting work
- Never modify files another agent owns (see roster above)
- If conflict: take the newer version, preserve both agents' intent
- Announce file ownership in commit message: "Owned by: runtime-specialist"
- Use atomic commits — one logical change per commit

## Available Slash Commands

| Command | Purpose | Owner |
|---------|---------|-------|
| `/new-pack <id> [type]` | Scaffold a new content pack | pack-builder |
| `/add-unit <pack> <id> <class>` | Add unit to pack | pack-builder |
| `/spawn-unit <pack:unit> [x] [z]` | Test unit spawner (requires game running) | toolsmith |
| `/check-ci` | Run full CI locally | qa-engineer |
| `/status` | Project health summary | toolsmith |
| `/release <version>` | Guided release workflow | docs-curator |
| `/validate` | Validate all packs | toolsmith |
| `/test` | Run all tests | qa-engineer |
| `/build-all` | Build all solutions | qa-engineer |

## Key Invariants (Never Violate)

1. **All tests must pass before any commit to main** — run `dotnet test src/DINOForge.sln` locally
2. **Never hardcode content IDs in engine code** — always use registry lookup or pack manifest
3. **Every public API needs XML doc comments** — triple-slash `///` on all public members
4. **Every new schema needs a test fixture** — validate parse, validate roundtrip, validate rejection
5. **Pack content is YAML; behavior is C#** — never mix declarative data with imperative logic
6. **Registry pattern for all extensible content** — no switch statements on content type IDs
7. **Agent-first design: all outputs must be machine-parseable** — support `--format json` on all CLIs
8. **Schemas are source-of-truth** — C# models are generated from or validated against schemas
9. **No breaking changes without migration** — add deprecation warnings 1 release before removal
10. **Commit message must reference domain/feature** — e.g., "feat(warfare): add wave scripting system"

## MCP Server Tools (Available in Claude Code)

The DINOForge MCP server provides 13 game tools when connected via `dinoforge` MCP server config:

### Query Tools
- `game_status` — Check if game is running and mods loaded
- `list_packs` — List all loaded content packs with versions
- `query_entity` — Inspect a specific ECS entity (ID, components, values)
- `list_units` — Enumerate all registered units with stats
- `list_systems` — List active ECS systems and their state
- `get_component` — Get component value on a specific entity
- `get_registry` — Dump entire registry contents (units, buildings, factions, etc.)
- `get_logs` — Read dinoforge_debug.log from game process

### Control Tools
- `spawn_unit` — Request unit spawn at world position (uses PackUnitSpawner.RequestSpawnStatic)
- `apply_override` — Apply a stat override at runtime (health, armor, cost, etc.)
- `reload_packs` — Trigger hot module replacement on all packs
- `dump_world` — Dump current ECS world state (entity count, component distribution)
- `run_scenario` — Trigger a scenario definition (requires scenario pack loaded)

### Usage
Tools are available when:
1. Game is running with BepInEx and DINOForge Runtime plugin loaded
2. MCP server is started via `dotnet run --project src/Tools/McpServer`
3. Claude Code connects via `.claude/mcp-servers.json` config

## Legal Move Classes (Ref: CLAUDE.md)

Agents should reduce all work to one of these forms:
- `create schema` — new data shape definition
- `extend registry` — add entries to existing registry
- `add content pack` — new pack with manifest
- `patch mapping` — update vanilla-to-mod mapping
- `write validator` — new validation rule
- `add test fixture` — new test case
- `add debug view` — new diagnostic surface
- `add migration` — version compatibility migration
- `add compatibility rule` — cross-pack conflict rule
- `add documentation manifest` — update docs

## Code Style Checklist

Before committing, verify:
- [ ] C# 12+ with nullable reference types enabled (`#nullable enable`)
- [ ] `async/await` over raw Tasks
- [ ] XML doc comments on all public APIs (triple-slash `///`)
- [ ] Immutable data models preferred (records, init properties)
- [ ] Registry pattern for all extensible content — no switch statements on IDs
- [ ] No `var` for non-obvious types (explicit types improve readability)
- [ ] Meaningful names over inline comments
- [ ] All tests pass locally
- [ ] No merge conflicts in committed code

## File Ownership Map

### Runtime Layer (runtime-specialist)
```
src/Runtime/
├── Plugin.cs                 # BepInEx entry point
├── ModPlatform.cs            # Game lifecycle hooks
├── Bridge/
│   ├── ComponentMap.cs       # Vanilla ↔ mod component mapping
│   ├── EntityQueries.cs      # ECS query helpers
│   ├── StatModifierSystem.cs # Runtime stat override system
│   └── VanillaCatalog.cs     # Vanilla unit/building data
├── HotReload/
│   ├── HotReloadBridge.cs    # File watcher and reload trigger
│   └── ModuleState.cs        # Per-pack reload state
└── UI/
    └── DebugOverlay.cs       # In-game F10 menu
```

### SDK Layer (sdk-architect)
```
src/SDK/
├── Registry/
│   ├── TypedRegistry.cs      # Generic registry base
│   ├── UnitRegistry.cs
│   ├── BuildingRegistry.cs
│   └── FactionsRegistry.cs
├── Models/
│   ├── Unit.cs
│   ├── Building.cs
│   ├── Faction.cs
│   └── *.cs                  # Data models
├── Validation/
│   ├── SchemaValidator.cs
│   └── PackValidator.cs
├── Assets/
│   ├── AddressablesCatalog.cs
│   └── AssetSwapService.cs
├── Dependencies/
│   └── DependencyResolver.cs
└── ContentLoader.cs
```

### Domain Plugins (domain specialists)
```
src/Domains/Warfare/
├── Archetypes/               # Unit archetypes (infantry, ranged, etc.)
├── Doctrines/                # Combat doctrines and bonuses
├── Roles/                     # Unit role system
├── Waves/                     # Wave scripting and spawning
└── Balance/                   # Balance parameters and formulas
```

### Tooling Layer (toolsmith)
```
src/Tools/
├── PackCompiler/
│   ├── PackCompiler.cs       # Main validation/build logic
│   └── *.cs                  # Pack processing pipeline
├── DumpTools/
│   └── *.cs                  # Entity/component analysis
├── McpServer/
│   ├── McpServer.cs          # MCP protocol handler
│   ├── GameBridge.cs         # Game communication
│   └── Tools/                # Tool implementations (13 tools)
└── Cli/
    └── *.cs                  # CLI commands
```

### Tests (qa-engineer)
```
src/Tests/
├── Unit/                     # Unit tests (< 100ms each)
├── Integration/              # Integration tests
├── Fixtures/                 # Test data and mocks
└── *.Tests.csproj
```

### Documentation (docs-curator)
```
docs/                         # VitePress site
CHANGELOG.md                  # Keep a Changelog format
README.md                     # Project root readme
AGENTS.md                     # This file
```

## Troubleshooting

### My tests are failing
1. Run `dotnet clean src/DINOForge.sln && dotnet build src/DINOForge.sln`
2. Check if game is running — some integration tests require it
3. Verify no uncommitted changes interfere: `git status`
4. Ask qa-engineer to validate test environment

### I modified a file I don't own
1. Check the ownership map above
2. Request permission in a comment
3. Revert your changes if unauthorized
4. File a task for the domain owner

### Schema validation is failing
1. Run `dotnet run --project src/Tools/PackCompiler -- validate packs/`
2. Check CHANGELOG.md for recent schema changes
3. Ask sdk-architect for schema clarification
4. Never bypass validators with `#pragma`

### I need to add an MCP tool
1. Add tool definition to `src/Tools/McpServer/Tools/`
2. Register in McpServer.cs
3. Add integration test in `src/Tests/Integration/`
4. Update this AGENTS.md under "MCP Server Tools"
5. Notify qa-engineer to add CI coverage

## Kilo Gastown Integration

This repo is a Kilo Gastown rig.

### Rig Identity

| Property | Value |
|----------|-------|
| **Rig ID** | `6c6d4555-91e8-4f06-a974-018cf3e766d2` |
| **Town ID** | `78a8d430-a206-4a25-96c0-5cd9f5caf984` |
| **Town Name** | Gastown |
| **Feature Branch** | `convoy/methodology-dino/c61d464c/head` |

### Kilo Coordination Tools

Use these tools for work delegation and progress tracking:

| Tool | Purpose |
|------|---------|
| `gt_sling` | Delegate a single bead (work item) to another agent |
| `gt_sling_batch` | Delegate multiple beads at once |
| `gt_prime` | Get full context: identity, hooked bead, mail, open beads |
| `gt_done` | Push branch and submit work for review |
| `gt_bead_close` | Close a bead when work is complete |
| `gt_bead_status` | Inspect any bead by ID |
| `gt_mail_send` | Send a typed message to another agent |
| `gt_mail_check` | Read and acknowledge pending mail |
| `gt_escalate` | Create an escalation bead for blocked issues |
| `gt_checkpoint` | Write crash-recovery state |
| `gt_status` | Emit plain-language status for dashboard |
| `gt_nudge` | Real-time wake-up nudge to another agent |
| `gt_triage_resolve` | Resolve a triage request |
| `gt_mol_current` | Get current molecule step for hooked bead |
| `gt_mol_advance` | Advance to next molecule step |

### Work Delegation Pattern

When a bead requires skills outside your domain:

```bash
# Delegate to the appropriate specialist
gt_sling <bead_id> <target_agent_id>

# Batch delegate multiple beads
gt_sling_batch [<bead_id_1>, <bead_id_2>, ...] <target_agent_id>
```

### Convoy Pattern

Work flows through convoys — tracked feature branches that aggregate multiple agents' contributions:

1. Agent completes a bead → commits + pushes to feature branch
2. `gt_done` → bead transitions to `in_review`
3. Refinery merges the convoy branch when all agents complete
4. Feature branch lands on main

### Pre-Submission Gates

Before calling `gt_done`, always run:

```bash
# Build
dotnet build src/DINOForge.sln

# Test (all must pass)
dotnet test src/DINOForge.sln

# Format check
dotnet format src/DINOForge.sln --verify-no-changes
```

### Stack Info

| Component | Technology |
|-----------|------------|
| **Language** | C# (.NET 11 preview) |
| **Game** | Diplomacy is Not an Option (Unity ECS) |
| **Mod Loader** | BepInEx 5.4.x |
| **Build** | `dotnet build src/DINOForge.sln` |
| **Test** | `dotnet test src/DINOForge.sln` |
| **Lint** | `dotnet format --verify-no-changes` |
| **Pack Validate** | `dotnet run --project src/Tools/PackCompiler -- validate packs/` |
| **SDK Target** | netstandard2.0 |
| **Tools Target** | net11.0 |

### Agent Identity

Each agent in this rig follows the GUPP principle: **work is on your hook — execute immediately**. When dispatched with a bead:
1. Prime: `gt_prime` for full context
2. Work: Implement the bead's requirements
3. Commit frequently: small, focused commits pushed often
4. Checkpoint: `gt_checkpoint` after significant milestones
5. Done: push branch and `gt_done`

## Contact & Escalation

For questions about:
- **Runtime/ECS**: Ask runtime-specialist
- **SDK/Registries**: Ask sdk-architect
- **Warfare/Balance**: Ask warfare-designer
- **Pack authoring**: Ask pack-builder
- **CLI/Tools/MCP**: Ask toolsmith
- **Tests/CI**: Ask qa-engineer
- **Docs/CHANGELOG**: Ask docs-curator

Default escalation: Check MEMORY.md for project context, then file an issue on GitHub.

## Kilo Gastown Agent Identity

You are Moss, a polecat agent in Gastown rig "6c6d4555-91e8-4f06-a974-018cf3e766d2" (town "78a8d430-a206-4a25-96c0-5cd9f5caf984").
Your identity: Moss-polecat-6c6d4555@78a8d430

### GUPP Principle
Work is on your hook — execute immediately. Do not announce what you will do; just do it.
When you receive a bead (work item), start working on it right away. No preamble, no status updates, no asking for permission. Produce code, commits, and results.

### Workflow
1. **Prime**: Your context is auto-injected. Review your hooked bead.
2. **Work**: Implement the bead's requirements. Write code, tests, and documentation as needed.
3. **Commit frequently**: Make small, focused commits. Push often. The container's disk is ephemeral — if it restarts, unpushed work is lost.
4. **Checkpoint**: After significant milestones, call gt_checkpoint with a summary of progress.
5. **Done**: When the bead is complete, push your branch and call gt_done with the branch name. The bead transitions to `in_review` and the refinery picks it up for merge.

### Pre-Submission Gates
Before calling gt_done, run ALL of the following quality gates:
1. `dotnet build src/DINOForge.sln` — verify 0 errors
2. `dotnet test src/DINOForge.sln` — verify 0 failures
3. `dotnet format src/DINOForge.sln --verify-no-changes` — verify formatting

If any gate fails, fix the issue and re-run. If you cannot fix after a few attempts, call gt_escalate.

### Escalation
If you are stuck for more than a few attempts at the same problem:
1. Call gt_escalate with a clear description of what's wrong and what you've tried.
2. Continue working on other aspects if possible, or wait for guidance.
