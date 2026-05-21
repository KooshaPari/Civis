# Iter-144 Infrastructure Status Audit (2026-05-20)

Read-only audit of DINOForge supporting infrastructure. File paths absolute. Status legend: **WORKING** (proven in last 7d) / **STUB** (compiles, partial) / **VAPORWARE** (referenced in docs only) / **DEPRECATED**.

## 1. Headless launch (RDP / sandbox / hidden-desktop)

| System | Path/file | Status | Notes |
|---|---|---|---|
| `IsolationBackend` ABC | `src/Tools/DinoforgeMcp/dinoforge_mcp/isolation_layer.py:68` | WORKING | Abstract base, auto-detect chain (playcua → hidden_desktop) |
| `HiddenDesktopBackend` | `isolation_layer.py:121` (Win32 CreateDesktopW) | **STUB/BROKEN** | Code exists (~679 LOC inline) but routing broken; "game can't render on hidden desktop anyway (no GPU)" per `docs/sessions/iter-142-state-of-infrastructure-stack.md:123`. Memory: known BROKEN. |
| `PlayCUAClient` + `PlayCUABackend` | `isolation_layer.py:800`, `:920` | STUB | JSON-RPC adapter to playCUA on port 9000. Code present (~370 LOC inline). Not yet exercised against DINO end-to-end. |
| `DockerBackend` | `src/Tools/DinoforgeMcp/dinoforge_mcp/docker_backend.py` (329 LOC) | **STUB** | Header line 4-5: "are stubbed beyond the v0.23.0 skeleton but are not yet end-to-end functional. Marked NotImplementedError where the inner integration is still TBD." Multiple `raise NotImplementedError` at lines 231, 272. iter-144 wave-1 in progress per memory. |
| Separate `hidden_desktop_backend.py` / `playcua_backend.py` modules | (referenced by user) | **DOES NOT EXIST** | Both backends are embedded *inside* `isolation_layer.py` as classes, not separate files. |
| `IsolationContextManager` / `get_isolation_context()` | `isolation_layer.py:1115,1164` | WORKING | Backend selection logic working; defaults to "auto". |
| `game_launch_test(hidden=True)` MCP tool | `src/Tools/DinoforgeMcp/dinoforge_mcp/server.py` (1599 LOC, 43 `@tool(`) | PARTIAL | Tool registered; underlying hidden-desktop path broken. |

**Headline**: hidden-desktop isolation is the long-standing blocker. Without GPU passthrough no game renders headlessly. PlayCUA path is plumbed but unproven against DINO.

## 2. Steamless / Steamguard

| Item | Status | Evidence |
|---|---|---|
| Steamless unpacker | **VAPORWARE** | "Steamless doesn't exist (DINOBox is Steam-tolerant)" — MEMORY.md Infra reality 2026-04-25. iter-142 stack doc line 274: "30% — Unblock via Steamless unpacking + MockSteamworks (Wave 1)". |
| MockSteamworks / Goldberg emulator | **VAPORWARE** | Line 296: "Headless launch: Broken (MockSteamworks undeployed, Steamless unimplemented)". |
| Steamguard CLI helpers | Not found | No source under `src/`. Only doc references in `docs/sessions/steamworks-goldberg-investigation.md`. |

**Status**: 100% vaporware in code; all references are in `docs/sessions/*.md` investigation notes only.

## 3. Game bridge + agents (playCUA)

| System | Path/version | Status | Notes |
|---|---|---|---|
| `GameBridgeServer.cs` (named-pipe ECS bridge) | `src/Runtime/Bridge/GameBridgeServer.cs` (2502 LOC) | **WORKING (just fixed)** | Latest commits: `974e78e4` (gray-freeze root cause: async pipe accept + force-cancel OnDestroy), `9a9e8772` (PumpIsAlive + bounded timeouts #535), `d03530de` (HandleConnect handshake). iter-143 wave-2 verified runtime. |
| `bare-cua-native.exe` (external) | `C:\Users\koosh\playcua_ci_test\target\release\bare-cua-native.exe` | WORKING | Binary present (debug symbols `.d` alongside). Repo `KooshaPari/PlayCua` (public, Rust+Python, updated 2026-05-13). |
| playCUA JSON-RPC server | port 9000, started via `cargo run` | UNVERIFIED | No `start-playcua.ps1` found in `scripts/`. Server boot is manual. |
| Steam/Doorstop integration | `KooshaPari/DINOForge-UnityDoorstop` (fork) | WORKING | Standard BepInEx 5.4.23.5 plus Doorstop fork referenced. |
| Named-pipe protocol (JSON-RPC) | `src/Bridge/Protocol/JsonRpcMessage.cs` | WORKING | Recently hardened (DTOs migrated to properties for NuGet binary-compat per memory). |

## 4. MCP CLI + skills

| Item | Path | Status |
|---|---|---|
| FastMCP server | `src/Tools/DinoforgeMcp/dinoforge_mcp/server.py` (1599 LOC) | WORKING — 43 `@tool(` registrations (CLAUDE.md says 21; **count drifted upward** — likely 21 game_* + 22 asset/catalog/log/proof). |
| Slash commands (`.claude/commands/`) | 27 `.md` files + `lib/game-check.sh` | WORKING — full set: launch-game, test-swap, check-game, game-test, game-test-task, game-coverage, entity-dump, pack-deploy, asset-create, eval-*, prove-*, status, release. |
| `prove-features-gate.ps1` | `.claude/commands/prove-features-gate.ps1` | WORKING |
| Companion modules | `bridge_receipt_aggregator.py`, `external_judge.py`, `journey_keyframe_tagger.py`, `merkle.py`, `native_dep_resolver.py`, `proof_policy.py`, `proof_signing.py`, `vision.py`, `capture_wgc.py` | MIXED — most WORKING; `capture_wgc.py` iter-143 #537 landed; `external_judge.py` Fireworks-Kimi wired but not gated as receipt. |

## 5. Nanovms / PhenoCompose

| Item | Path/repo | Status |
|---|---|---|
| `KooshaPari/nanovms` | github.com/KooshaPari/nanovms (public, Go CLI) | EXISTS (updated 2026-05-14). 3-tier: WASM/gVisor/Firecracker. NOT YET INTEGRATED into Dino. |
| `KooshaPari/phenocompose` | **DOES NOT EXIST as a GitHub repo** | CLAUDE.md PhenoCompose Integration section is **VAPORWARE**: no `phenocompose` repo under KooshaPari (only `nanovms` + `pheno` monorepo). Local dir `C:\Users\koosh\phenocompose` also absent. |
| `C:\Users\koosh\agileplus` (local) | EXISTS — Rust workspace (Cargo.toml + Dockerfile.rust). Repo `KooshaPari/AgilePlus` public. | WORKING (PM dashboard per CLAUDE.md). |
| Docs references | `docs/sessions/phenocompose_integration_technical.md`, `phenocompose_nvms_investigation.md`, `2026-04-25-sandbox-isolation-audit.md` | VAPORWARE — investigation notes only. |
| `pheno` monorepo (170K LOC) | `KooshaPari/pheno` | EXISTS — 11 workspace members. Not yet pulled into Dino. |

## 6. Upstream repos / org-level projects

KooshaPari org (top ~30 by recent activity):

| Repo | Purpose | Relevance to Dino |
|---|---|---|
| **Dino** | This repo | — |
| **PlayCua** | Bare-metal CUA Rust+Python | DIRECT dependency (bare-cua-native.exe) |
| **nanovms** | Three-tier headless VM Go CLI | FUTURE roadmap (v0.25.0+ per CLAUDE.md) |
| **DINOForge-UnityDoorstop** | Doorstop fork | Indirect (BepInEx ecosystem) |
| **pheno** | Rust monorepo, AgilePlus + PhenoLibs | Future infra |
| **AgilePlus** | Spec-driven PM | Dev workflow |
| **phenoShared** | Org shared crates | Reused by release-drafter workflow (per recent commit f222cd3) |
| **PhenoMCP**, **McpKit** | MCP framework | Could replace bespoke FastMCP server |
| **phenotype-journeys** | Rust CLI + Vue + Playwright journey harness | DIRECTLY relevant to "journey records UX" memory note |
| **phenodocs** | VitePress docs system | DINOForge docs already pattern-match |
| **PhenoSpecs** | Spec registry | Could host DINOForge ADRs |
| **TestingKit, AuthKit, ObservabilityKit, FocalPoint, BytePort** | SDK kits | Future consolidation candidates |
| **hwLedger, Tracera, GDK, MCPForge, thegent, Httpora, Parpoura, Httpora, heliosApp, QuadSGM, chatta, WorldSphereMod, PhenoProject, phenotype-dep-guard, phenotype-tooling, phenotype-auth-ts, agileplus-landing, PhenoKits** | Other org projects | Adjacent only |
| **PhenoRuntime** | ARCHIVED | — |

## Gaps & recommendations

1. **`phenocompose` is vaporware** — CLAUDE.md cites it as v0.24.0-26.0 roadmap with detailed 3-tier story, but the repo does not exist. Action: either rename references to `nanovms` (the actual repo) + `pheno` monorepo, or stand up a `phenocompose` repo before v0.25.0 release notes ship.
2. **`HiddenDesktopBackend` is the persistent blocker for headless proof** — broken since iter-142. Without GPU passthrough on hidden desktop, *nothing* downstream (external judge receipts, scaled fleet, CI proof) works. Action: pivot to `nanovms` Firecracker+VFIO path as Tier-1, formally deprecate the CreateDesktopW backend.
3. **Steamless / MockSteamworks is 100% vaporware** — listed at "30% complete" in iter-142 docs but no code lands. Action: drop from roadmap or open a real spike issue; current investigation docs imply progress that doesn't exist.
4. **MCP tool count drift** — CLAUDE.md says 21 tools, server.py has 43 `@tool(` registrations. Action: regenerate the table from `server.py` in CLAUDE.md next pass.
5. **Two MCP servers live in repo** — FastMCP Python (canonical, 1599 LOC) vs. C# McpServer (unregistered, drift risk). Action: explicitly DEPRECATE the C# McpServer or merge into FastMCP via `PhenoMCP`/`McpKit`.

**Bonus wins working well**: GameBridgeServer hang fix (974e78e4), bare-cua-native binary present and indexable, 27-strong `.claude/commands/` library, full 43-tool MCP surface, recently-fixed JSON-RPC handshake, robust ECS bridge after iter-143 wave-2.
