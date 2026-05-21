# State of Headless / Sandbox / Automation Stack — Iter-142 Honest Snapshot

**Date**: 2026-05-18  
**Scope**: Truthful assessment of game sandboxing, isolation, parallel testing, and journey recording infra.  
**Owner**: Orchestrator  
**Status**: BASELINE SNAPSHOT — no implementation; design-phase status only

---

## Status Legend

- ✅ **WORKING** — implemented AND end-to-end verified in production
- ⚠️ **IMPLEMENTED-UNVERIFIED** — code exists; signatures work; behavior never proven end-to-end
- 🔬 **RESEARCH-ONLY** — design doc + spec; zero implementation
- 💀 **DEAD** — code exists but orphaned (never wired to live code paths)
- 🚫 **ASPIRATIONAL** — concept only; no code or spec

---

## 1. Sandbox (Security Isolation for Game Execution)

| Component | Status | Notes |
|-----------|--------|-------|
| Windows Sandbox (`.wsb` container) | 🚫 | Concept only; no config authored |
| Docker container (headless game) | 💀 | `DockerBackend` stub in `src/Tools/DinoforgeMcp/dinoforge_mcp/isolation_layer.py` line 556+; never imported by `server.py` |
| Codex sandbox (`--sandbox workspace-write`) | ✅ | Works for agent execution; NOT applicable to game process |
| Hyper-V VM | 🚫 | Not configured; $500-1000 licensing barrier |

**Verdict**: **NO sandbox available for game execution today**. Docker exists as design-only. Codex sandbox runs agent code, not game.

---

## 2. Hidden RDP / Hidden Desktop (Isolated Game Session, Same Machine)

| Component | Status | Notes |
|-----------|--------|-------|
| HiddenDesktopBackend (Win32 CreateDesktop) | 💀 | 314 LOC, lines 121-434 in `isolation_layer.py`; **never imported by `server.py`** (see audit: `docs/qa/hidden_desktop_wire_up_audit_iter142.md`) |
| `_launch_hidden()` PS1 helper | ⚠️ | Exists in `server.py:205–269`; creates hidden desktop via Win32 API; routing path **bypassed in iter-142 test** (fell back to plain `Start-Process`) |
| Win32 CreateDesktop per-call | ⚠️ | Works when invoked (creates isolated desktop); never proven with game rendering (no GPU on hidden desktop) |
| RDPWrap unofficial mod | 🚫 | EULA-risky; not pursued |
| Windows Server concurrent sessions | 🚫 | Requires $500+ license; not purchased |
| VDD (Virtual Display Driver) future | 🔬 | Tier 1 in roadmap; no code yet |

**Verdict**: **No working hidden desktop / RDP path in production**. CreateDesktop can be invoked but (a) game cannot render on hidden desktop (no GPU), (b) routing in `server.py` ignores `hidden=True` parameter, instead launches to primary desktop.

---

## 3. VMs (Cloud / Hypervisor Isolation)

| Component | Status | Notes |
|-----------|--------|-------|
| Hyper-V VM | 🚫 | Not configured |
| phenocompose (external dep) | 🔬 | Research-phase design doc; `KooshaPari/phenocompose` (external repo); no DINO integration |
| AWS g4dn (GPU compute) | 🔬 | Research-only; not provisioned |
| Azure NV series (GPU) | 🔬 | Research-only; not provisioned |

**Verdict**: **No VM-isolated game running today**. phenocompose is documented as future v0.24.0+ roadmap item but has zero integration code.

---

## 4. Automated Steam / Game Management

| Component | Status | Notes |
|-----------|--------|-------|
| MCP `game_launch` tool | ⚠️ | HTTP endpoint exists (line 454-475 in `server.py`); **returned 406 on iter-142 test**; fallback to `subprocess.Popen` (TIER 3) triggered instead |
| Direct `Start-Process` with Steam running | ✅ | WORKING: silent 5s exit if Steam not running; succeeds when Steam URL invoked via `steam://run/287210` |
| HandleConnect handshake (BepInEx IPC) | ⚠️ | Deployed 2026-05-18 18:55; **plugin never logged output** on iter-142 test; load may be failing silently |
| steamguard-cli + keychain auth | 🔬 | Research only (documented in `docs/proposals/headless_steam_drm_stack_iter142.md` Section 1) |
| Steamless DRM-strip (atom0s/Steamless) | 🔬 | RESEARCH-ONLY — 1-sprint estimate in feasibility doc; Option D recommended path; zero implementation |
| MockSteamworksNet plugin | ✅ | **COMPILED** at `src/Tools/MockSteamworksNet/bin/Release/net8.0/MockSteamworksNet.dll`; 5 Harmony patches ready (SteamAPI.Init, IsSteamRunning, GetSteamID, BIsSubscribedApp, GetPersonaName); **deploy chain NOT LANDED** (28-line MSBuild target in spec, unmerged) |

**Verdict**: **Manual Steam launch works; full headless automation NOT proven end-to-end**. MockSteamworksNet is production-ready but deploy chain missing. Steamless is recommended research-backed path but unimplemented.

---

## 5. Generalization for Other Game Projects

| Component | Status | Notes |
|-----------|--------|-------|
| Reusable MCP server pattern | ✅ | FastMCP (Python) server is generic; can be forked for other games |
| Reusable BepInEx plugin scaffold | ✅ | Plugin structure is game-agnostic; entry points documented |
| Asset swap pipeline | ⚠️ | Addressables-specific; works for any Unity game with Addressables |
| JSON-RPC bridge | ✅ | Protocol generic; can be reused |
| Cross-project skill / plugin in `~/.claude/` | 🚫 | Not abstracted; each piece is DINO-specific today |
| Rust / Go cross-project CLI tool | 🚫 | ASPIRATIONAL per `feedback_tool_construction_lang_pref` (prefers Rust/Go for infra, not C#) |

**Verdict**: **Nothing abstracted yet; all pieces are DINO-specific**. Patterns are reusable but require manual porting per game. No shared CLI or plugin registry exists.

---

## 6. Demo Windows + Journey Recordings

| Component | Status | Notes |
|-----------|--------|-------|
| Remotion video pipeline | ⚠️ | 4 renders exist at `docs/proof-of-features/dinoforge_proof_20260328_214844/` (reel=3.2MB); never tested post-iter-101; renders are ~3min clips (static, not interactive) |
| VitePress docs site | ✅ | WORKING; builds clean; all sidebar sections live |
| Screenshot capture (`game_screenshot` MCP) | ⚠️ | Worked in past; untested this session; wired to `GameControlCli` C# via named pipes |
| Journey records UI viewer (manifest-driven) | 🚫 | ASPIRATIONAL — VitePress component that reads `docs/journeys/<id>/manifest.yaml` + renders video/keyframes/SVG; user flagged as UX need (see `project_journey_records_ux_need.md`) but not authorized |
| HWLedger prior art | ⚠️ | Sibling project has journey-record pattern (user notes "not fully there yet either"); lives at `C:\Users\koosh\HWLedger\` |

**Verdict**: **Raw artifacts can be captured (screenshots, video renders); rich-UI viewer that compiles them per journey is missing**. Remotion pipelines exist but are untested post-iter-101. Manifest-driven viewer is user-flagged need but design-only.

---

## What's the SHORTEST PATH to User-Facing Demo Windows + Journey Recordings?

### Top 3 Next Sprints (Ranked by Impact / Effort)

| Priority | Sprint | Component | Effort | Impact | Unblocks |
|----------|--------|-----------|--------|--------|----------|
| 1️⃣ | **Spike #1** | MockSteamworksNet deploy chain + Steamless unpacking | 15 min XML + 1h CI = **2h** | Enables headless game without Steam client; foundation for all automation | #98, #101, #103, #425 |
| 2️⃣ | **Spike #2** | Journey records UI viewer (VitePress component) | **1 sprint (30h)** | User-facing proof compilation; critical for external judge credibility | #103 (Kimi proof), user UX |
| 3️⃣ | **Spike #3** | Steam URL launch via MCP + CreateDesktop wiring | **4h** (fix 406 endpoint + import isolation_layer) | Hidden-launch on same machine; enables parallel test fleet on primary box | #98 (pack-swap proof) |

**Critical dependency**: Spike #1 is the blocker for #2 and #3. Do Spike #1 first.

---

## What Does the User NOT Have Today That They Want?

From `feedback_self_judging_proof_is_not_proof.md` + `project_journey_records_ux_need.md`:

1. **Hidden desktop launch that ACTUALLY works** — Createdesktop code exists but routing is broken; game can't render on hidden desktop anyway (no GPU)
2. **Sandbox isolation for game execution** — Docker exists as design-only; Windows Sandbox is concept-only
3. **VM isolation for parallel testing** — phenocompose is research; Hyper-V not configured
4. **Parallel test fleet (100+ instances)** — phenocompose roadmap; zero implementation
5. **Journey video output viewer** — Remotion captures exist but no rich UI to navigate them
6. **Generalized framework for cross-project reuse** — all patterns are DINO-specific; no abstraction
7. **External judge receipts with real game proof** — Kimi integration blocked because headless launch is broken

---

## Recommended Sprint Sequence for v0.26.0+

### Wave 1: Foundation (Headless + Mock Steam) — **Iter-143 (1 sprint)**

**Goal**: Unblock real-game proof artifacts (fix critical path #98 #101 #103 #425)

1. **Spike 1a**: Unpack DINO with Steamless (local, Windows 1-shot, 30 min)
   - Run atom0s/Steamless on `Diplomacy is Not an Option.exe` from Steam install
   - Output: `DINO_unpacked.exe` (DRM-free binary)
   - Validate locally: `.\DINO_unpacked.exe -batchmode -nographics` (should start)
   - **Gate**: Real `BepInEx/dinoforge_debug.log` entries from unpacked binary (not error)

2. **Spike 1b**: Deploy MockSteamworksNet (1h merge + 4h CI)
   - Land 28-line MSBuild target from spec into `src/Runtime/DINOForge.Runtime.csproj`
   - Add CI step: copy DLL to test instance `BepInEx/plugins/`
   - Verify no crashes on SteamAPI patch load
   - **Gate**: CI job completes, logs show "MockSteamworks: SteamAPI.Init() patched"

3. **Spike 1c**: Wire CI headless-launch job (4h)
   - Update `.github/workflows/game-launch.yml` (new headless-launch target)
   - Cache unpacked binary (GitHub Actions artifact cache, 5-day TTL)
   - Launch: `.\DINO_unpacked.exe -nographics -batchmode`
   - Wait 10s; poll `dinoforge_debug.log` for "DINOForge initialized"
   - Capture screenshot via MCP bridge
   - Kill process; upload artifacts
   - **Gate**: Workflow completes without timeout; screenshot captured; log shows pack load

**Effort**: 1 sprint (36-40h total)  
**Blockers**: None (all components exist)  
**Fallback**: If Steamless fails on SteamStub v3, spike to Option A (steamcmd auth) — 1 day regression

---

### Wave 2: Journey Viewer (Rich Media) — **Iter-144 (1 sprint)**

**Goal**: Surface proof artifacts to external judges; enable Kimi integration (#103)

1. **Viewer 2a**: Design manifest schema (4h)
   - YAML structure: `docs/journeys/<journey-id>/manifest.yaml`
   - List video, keyframes, screenshots, SVG, metadata
   - Reference HWLedger pattern; extend if needed

2. **Viewer 2b**: Implement VitePress component (16h)
   - Vue 3 component: `docs/.vitepress/components/JourneyViewer.vue`
   - Read manifest YAML; render video player (shaka-player or native `<video>`)
   - Keyframe carousel (swiper.js or vanilla arrows)
   - Screenshot lightbox (lightbox2 or fancybox)
   - SVG inline-embed + optional pan/zoom

3. **Viewer 2c**: Compile step (4h)
   - Pre-build script (`docs/scripts/compile-journeys.js`) reads all manifests
   - Generate per-journey pages (`docs/journeys/<id>/index.md`)
   - Wire into VitePress sidebar + routing
   - Test locally: `npm run docs:dev`

4. **Viewer 2d**: Integrate Remotion outputs (4h)
   - Copy existing 4 renders from `docs/proof-of-features/dinoforge_proof_20260328_214844/` into journeys
   - Wire keyframe extraction from video (ffprobe + NodeJS spike)
   - Validate lossless playback + scrubbing

**Effort**: 1 sprint (28-32h total)  
**Blockers**: Spike #1 must complete first (need real game proof artifacts to demo viewer)  
**Deliverable**: User can navigate `https://kooshapari.github.io/Dino/journeys/iter-143-headless-launch/` and see video + keyframes + proofs

---

### Wave 3: Cross-Project Framework — **Iter-145 (1-2 sprints)**

**Goal**: Generalize DINO's patterns for WorldBox + other games; reusable CLI in Rust/Go

1. **Framework 3a**: Extract reusable patterns (8h)
   - MCP server template (FastMCP boilerplate for any game)
   - BepInEx plugin scaffold (Entry point + logging template)
   - JSON-RPC protocol (language-agnostic bridge spec)

2. **Framework 3b**: Implement Rust CLI tool (16h)
   - Single-binary: `dinoforge-multiproject` (or shorter name)
   - Supports multiple games (DINO, WorldBox, etc.)
   - Commands: `launch-game`, `screenshot`, `query-state`, `inject-input`
   - Reads config: `~/.dinoforge/games.toml` (per-game exe paths, mod plugin paths)
   - **Pairs with**: Rust preference from `feedback_tool_construction_lang_pref`

3. **Framework 3c**: Publish to GitHub + itch (4h)
   - Release binary as standalone download
   - Document for other mod authors
   - Pair with CLAUDE.md / RUNBOOK for other projects

**Effort**: 1-2 sprints (28-40h total)  
**Blockers**: Spike #1 + Wave #2 must complete first (pattern maturity threshold)  
**Deliverable**: WorldBox session can use same `dinoforge-multiproject` CLI to launch, screenshot, verify mods

---

### Wave 4: Cloud Scale-Out (Parallel VM Fleet) — **v0.27.0+ (2-3 sprints)**

**Goal**: Parallel test fleet (100+ concurrent instances); performance regression testing at scale

1. **Cloud 4a**: phenocompose integration (8h design + 16h impl)
   - Evaluate `KooshaPari/phenocompose` nanovms tier for DINO snapshot + spawn
   - Hook into CI: spawn 10 parallel test instances from baseline snapshot
   - Collect results; merge coverage + crash logs
   - Document in `docs/phenocompose_dino_integration.md`

2. **Cloud 4b**: Tier 1 (VDD) fallback for Tier 3 fallback (8h spec + 12h impl)
   - If phenocompose delayed, implement Windows VDD (Indirect Display Driver) as Tier 1 in isolation_layer.py
   - Lower latency than HiddenDesktop; supports GPU passthrough
   - Deprecate HiddenDesktop (Tier 2); DeleteCandidate in code review

3. **Cloud 4c**: RDP pool management (8h)
   - Script to spawn RDP-enabled VMs (AWS / Azure) on demand
   - Login + launch game; capture visuals via RDP
   - Auto-cleanup after test run
   - Cost dashboard (warn if > $20/day in Azure charges)

**Effort**: 2-3 sprints (40-56h total)  
**Blockers**: Wave #1 + #2 must complete first  
**Deliverable**: `game-test-parallel` command spins up 100 instances, collects proofs, shuts down

---

## Roadmap Timeline

```
v0.26.0 (iter-143-145, 6-8 weeks):
├── Wave 1: MockSteamworksNet + Steamless headless launch      [1 sprint]
├── Wave 2: Journey records UI viewer                          [1 sprint]
└── Wave 3: Cross-project Rust CLI tool                        [1-2 sprints]

v0.27.0 (iter-146-150, following release):
├── phenocompose integration (parallel VM fleet)               [2-3 sprints]
└── RDP pool management + cost tracking                        [1 sprint]
```

---

## Infrastructure Stack Health Report

| Layer | Status | Confidence | Action |
|-------|--------|------------|--------|
| **Capture** (screenshots, video) | ⚠️ IMPL-UNVERIFIED | 70% | Re-test `game_screenshot` + Remotion post-iter-142 |
| **Isolation** (sandbox, hidden, VM) | 💀💀💀 | 5% | DELETE isolation_layer.py; pursue Steamless (Wave 1) |
| **Auth** (Steam, game launch) | ⚠️ BROKEN | 30% | Unblock via Steamless unpacking + MockSteamworks (Wave 1) |
| **Proof** (journey viewer, receipts) | 🚫 ASPIR | 40% | Build UI (Wave 2); gate on external judge integration |
| **Generalization** (Rust CLI) | 🚫 ASPIR | 50% | Design (Wave 3) after Wave 1 patterns stabilize |
| **Scale** (parallel fleet) | 🚫 ASPIR | 40% | Scope post-v0.26.0 release (Wave 4) |

---

## Critical Files to Read

For deep dives:

- **`docs/qa/hidden_desktop_wire_up_audit_iter142.md`** — Why HiddenDesktopBackend is orphaned
- **`docs/qa/isolation_layer_dead_code_inventory_iter142.md`** — Full code path audit (814 LOC dead)
- **`docs/proposals/headless_steam_drm_stack_iter142.md`** — Steamless + MockSteamworks rationale (Option D recommended)
- **`project_journey_records_ux_need.md`** — User UX need for rich journey viewer

---

## Verdict Summary

**The user wants 7 things; they have 1 of them (VitePress site).**

- **Headless launch**: Broken (MockSteamworks undeployed, Steamless unimplemented)
- **Sandbox isolation**: Doesn't exist for game execution
- **Hidden desktop**: Code exists but routing bypassed; game can't render on hidden desktop anyway
- **VM fleet**: phenocompose research-only
- **Journey viewer**: Design-only (user flagged UX need; no implementation authorized)
- **Cross-project tool**: Nothing abstracted
- **External judge proof**: Blocked by all of the above

**Shortest unblock sequence**:
1. Merge 28-line MSBuild target (MockSteamworksNet deploy) — **2h** — unblocks headless
2. Steamless unpack (local, 1-shot) + CI integration — **2h + 4h** — real game testing starts
3. VitePress journey viewer component — **30h** — proof becomes user-viewable

**Recommended authorization**: Approve Wave 1 (iter-143, 1 sprint). All blockers clear after that.

---

**Document Status**: Baseline snapshot complete. Ready for sprint planning.

**Next Steps (If User Authorizes)**:
1. Schedule iter-143 spike: Steamless unpack + MockSteamworks deploy
2. Assign Wave 2 (journey viewer) to follow
3. Gate v0.26.0 release on completion of both

