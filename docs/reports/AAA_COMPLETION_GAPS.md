# AAA Godgame — Completion Gap Audit

> **Type:** Honest read-only gap analysis toward a finished AAA godgame.
> **Date:** 2026-06-24.
> **Branch:** `research/aaa-completion-gap-audit`.
> **Scope:** `crates/` (32 crates), `docs/design/` (48 docs), `docs/adr/` (25 ADRs), plus `docs/specs/`, `clients/`, `web/`, `scenarios/`, `mods/`.
> **Method:** Read-only static review. No `cargo`. No branch checkout. Verified against the working tree at `C:/Users/koosh/Dev/Civis/.worktrees/wt-gap`.

---

## 0. TL;DR

The repo is a **rich, layered substrate that has not yet closed the loop to a playable AAA godgame**. Of the 32 Rust crates, ~14 carry real product-facing behaviour; the rest are stubs, schema-only files, or supporting libraries. Of the 48 design docs, **the god-tool pipeline is the single largest DESIGNED-BUT-NOT-IMPLEMENTED gap** (44/50 verbs registered as `Near` not `Live`), and the **PHASE_ORDER ↔ `Simulation::tick` mismatch is the single largest wired-but-wrong-bug** that today would fail the engine's own determinism tests.

**Headline numbers:**

| Surface | Designed | Implemented | Wired | Coverage |
|---|---|---|---|---|
| God-tool verbs (`civ-powers`) | 50 | 6 `Live` + 44 `Near` | 6 on substrate | **12% real, 100% lit** |
| JSON-RPC methods (`civ-server`) | 34+ (catalog) / 32 (impl) | 32 | 32 on wire | **100%** |
| MCP tools (`civis-mcp`) | 32 (per spec) | 3 harness | 3 harness | **9%** |
| Emergence phases (`Simulation::tick`) | 24 call sites | 24 call sites | **only 12 listed in `PHASE_ORDER`** | **sync bug** |
| Charter-layer design docs | 16 (physics/godtools/content/saveload/HUD/rendering/MCP/disaster/economy/diplomacy/language/religion/psyche/culture/markets/law/sound/tech) | n/a | mostly stubs | **design ceiling, impl floor** |
| Renderers | 3 (Bevy, Godot, Unreal) + web Babylon/Three | all 4 attached | partial (Bevy full, others 16³ mesh when dense) | **partial–good** |
| Modding (CIV-0700) | full spec | 25+ tests, `.civmod`, signing, mod browser | partial | **v3 partial–good** |

The single most important takeaway: **the substrate is finished enough to tick deterministically; the substrate-to-player loop is not.** A player today can run `cargo run -p civ-server` and read the JSON snapshot, but cannot click a button in the Bevy/Godot/Unreal clients and see a god-tool fire on the world. **Bridging that last mile is what "playable AAA" means.**

---

## 1. The four gap categories

### 1.1 DESIGNED-BUT-NOT-IMPLEMENTED (the "blueprint on the shelf")

A design doc exists at authoritative spec level, but no code in `crates/` actually delivers the behaviour.

| # | Doc | Designed for | Crate where it should live | Status |
|---|-----|---|---|---|
| 1 | `docs/design/MCP_CONTROL_SURFACE.md` (1095 lines, status: **Binding**) | 32 MCP tools | `crates/civis-mcp/src/server.rs` | **3 of 32 implemented** (`civis_verify`, `civis_pixels`, `civis_census`). 29 missing. The spec explicitly says: *"A `civis-mcp` shim opens a fresh WS connection per call, so subscription state does not persist between MCP tool calls"* — long-lived WS shim is deferred. |
| 2 | `docs/design/GODTOOLS_IMPL_PLAN.md` | 50 god-tool verbs with substrate handlers | `crates/engine/src/godtools.rs` | **6 of 50 verbs have substrate writes**: `terrain.raise`/`lower`/`level`/`life.spawn_organism`/`disaster.meteor`/`inspect.probe`. The other 44 are registered in `civ-powers` as `Near` (lit-but-inert). |
| 3 | `docs/design/SOUND_MUSIC_EMERGENCE.md` | Adaptive ambient, mood→music coupling, SFX triggers | `crates/audio/src/` (8 .rs files exist) | The crate exists with `bus.rs` / `mix.rs` / `ducking.rs` / `ambient.rs` / `mood.rs` / `sfx.rs` / `triggers.rs` / `ui_sound.rs`, but is **not wired into any client** and not invoked from `Simulation::tick`. |
| 4 | `docs/design/LANGUAGE_EMERGENCE.md` + `ADR-014` | Phoneme drift, lingua franca emergence, script atlas | `crates/engine/src/language.rs` | A file exists at `crates/engine/src/language.rs` (per `find crates -name "*.rs"`), but is **not in `PHASE_ORDER`** and not referenced in `Simulation::tick` — i.e. the language phase is implemented in isolation, never run. |
| 5 | `docs/design/SENTIENCE_PSYCHE.md` + ADR-016 | Psyche vector, drives/temperament, mood, beliefs | `crates/genetics/src/sentience.rs` | File exists; not wired into `Simulation::tick` PHASE_ORDER; no read API surfaced in JSON-RPC. |
| 6 | `docs/design/DIPLOMACY_EMERGENCE.md` + ADR-015 | Faction emergence via k-means, tension, treaty graph | `crates/diplomacy/` | Crate is in the workspace and `phase_diplomacy` is in tick, but **stub JSON-RPC** (`sim.diplomacy_action` is a stub per `crates/server/src/jsonrpc.rs:1706-1728`). The substrate phase reads events but no read RPC exists. |
| 7 | `docs/design/DISASTER_EMERGENCE.md` | 8 disaster kinds (meteor, lightning, flood, quake, firestorm, tornado, volcanic vent, drought) | `crates/engine/src/godtools.rs::DisasterRequest` | Enum has 1 variant (`Meteor`). The other 7 disasters are **not in the enum**. |
| 8 | `docs/design/civ-culture-emergent.md` | Cluster cultures, language_distance, CultureProfile | `crates/agents/src/culture.rs` | File referenced (`use civ_agents::culture::{cultural_distance, language_distance, CultureProfile}` in engine.rs line 11), but `phase_emergence` is the only phase that *would* consume it — and the orchestrator is at the tail of the tick and out of sync with PHASE_ORDER (see §1.2 #1). |
| 9 | `docs/design/civ-economy-emergent-markets.md` | CapitalistAllocator, market prices, scarcity multiplier, numeraire | `crates/economy/` | Mostly implemented (`CapitalistAllocator`, `MarketState::step`). **Not surfaced in JSON-RPC**: `sim.get_resources` returns `ResourceSnapshot[]`, not market prices. The `market_prices` field exists in `sim.snapshot` but is never populated. |
| 10 | `docs/design/SAVELOAD_HUD_PLAN.md` §3 (HUD overlays) | Tile inspector, save/load modal, top-bar chips, KeycapPalette HUD | `crates/hud/src/` | **Token + palette data layer exists**. **No renderer wiring**: the `bevy-ref`/`godot-ref`/`unreal-show` clients don't `use civ_hud` — they reimplement chip rendering. The HUD crate is a substrate contract waiting for an integration PR. |
| 11 | `docs/design/RENDERING_MIGRATION_PLAN.md` | Phenotype-gfx Vulkan backend, Bevy primary | `crates/protocol-3d/` | **F3D0 frame codec implemented** (`protocol-3d/src/lib.rs`). The "phenotype-gfx primary Vulkan backend" the user named as "build unblocked" is referenced in `ADR-bevy-vulkan-primary-backend.md` but the actual gfx crate is not in `crates/` — there is no `crates/phenotype-gfx` workspace member. |
| 12 | `docs/design/TECH_PROGRESSION.md` + ADR-006 | 12 hardcoded techs, research queue, `ResearchCache` | `crates/research/` + `crates/server/src/jsonrpc.rs` | **Server has `sim.tech_state` and `sim.queue_research`** (lines 1935-1948 of jsonrpc.rs), but `ResearchCache` lives in `engine.rs` as a stub struct with no `tick_research`/`tick_tech` implementer that mutates the engine — only the `phase_research` + `phase_tech` calls (lines 1204-1205) are wired, and those methods are not in `PHASE_ORDER`. |

### 1.2 IMPLEMENTED-BUT-NOT-WIRED (the "code exists, no wire")

Code compiles, passes unit tests, but is not reachable from any entry-point the player or agent can touch.

| # | Code | Where it should be wired | Symptom |
|---|------|--------------------------|---------|
| 1 | **11 of 24 `Simulation::tick` phases are missing from `PHASE_ORDER`** | `crates/engine/src/engine.rs:55-68` | `PHASE_ORDER` lists 12 phases; `Simulation::tick` calls **24** (`production`, `citizen_lifecycle`, `military`, `policy`, `economy`, `planet`, `diplomacy`, `tactics`, `voxel`, `compact`, `buildings`, `life`, `research`, `tech`, `belief`, `unrest`, `cohesion`, `social_mood`, `economic_focus_pre`, `stratification`, `institutions`, `economic_focus`, `emergence`, `diffusion`). The test `phase_order_includes_emergence` at line 3399 *panics* today (it does `expect("PHASE_ORDER must include 'emergence'")` against a list that lacks `emergence`). The test `phase_order_matches_tick_sequence` at line 3373 asserts the 12-entry literal — it would fail the moment any of the missing phases is added, or is the test that masks the bug today. **Either PHASE_ORDER is wrong or the test is wrong — pick one and fix it.** |
| 2 | **`sim.inspect_tile` returns stub body** | `crates/server/src/jsonrpc.rs:1689-1704` (per `MCP_CONTROL_SURFACE.md` §2.1.3) | The RPC exists and is dispatched; the body returns `{ "x": .., "y": .., "stub": true }`. Every agent that calls it gets nothing. |
| 3 | **`sim.diplomacy_action` is a stub** | `crates/server/src/jsonrpc.rs:1706-1728` | Propose_treaty / declare_war / offer_trade all return "stub" responses. Diplomacy crate exists, `phase_diplomacy` is wired in `tick`, but **no agent-driven or player-driven diplomacy action reaches the substrate**. |
| 4 | **MCP subscription tools are per-connection stateful** | `crates/civis-mcp/src/server.rs` | `sim_subscribe` / `sim_unsubscribe` / `sim_update_subscription` are defined in the design spec, but the MCP shim opens a fresh WS connection per call — the subscription state is on a connection that's immediately torn down. **The MCP spec itself documents this as a known limitation** (§4.3 of `MCP_CONTROL_SURFACE.md`). |
| 5 | **Mod `civ-mod-host` is wired to one phase** | `crates/engine/src/engine.rs:1248-1255` (`ingest_mod_phase_lines`) | Only one phase's mod output is ingested; the other tick phases don't surface mod output to the replay bus. `mod.loaded.v1` works on scenario load but per-tick mod events are partial. |
| 6 | **Save/Load HUD is chrome-only** | `crates/hud/src/lib.rs:115-118` | `HudState.save_panel: SurfaceFlag` exists; the Bevy/Godot/Unreal/web clients **don't render the panel** — they hand-roll their own save/load dialog. |
| 7 | **Audio crate not invoked from any tick phase** | `crates/audio/src/` | 8 source files compile. No `phase_audio` exists in `Simulation::tick`. No substrate coupling to `mood.rs` despite `SOUND_MUSIC_EMERGENCE.md` saying mood → music. |
| 8 | **`diffusion` crate (28 KB) is wired but its output is opaque** | `crates/diffusion/src/lib.rs` + `crates/engine/src/engine.rs:1215` (`phase_diffusion`) | It's the **last phase in PHASE_ORDER**. It runs, but no JSON-RPC method surfaces diffusion state — agents cannot read or steer cultural/religious diffusion fields. |
| 9 | **Civ-pin spectator data not surfaced for agents** | `crates/engine/src/spectator.rs` | `civ_pins` are populated for `sim.snapshot`, but `inspect_tile` (stub, see #2) doesn't return the nearest agent alignment that `inspect.probe` is documented to return. |
| 10 | **`ResearchCache` is stored but not driven** | `crates/engine/src/engine.rs:70-77` + `phase_research`/`phase_tech` (lines 1204-1205) | `ResearchCache` is `pub struct` with `researched`/`queued`/`in_progress`. `sim.tech_state` returns the cache on read; **but `phase_research` and `phase_tech` are not in `PHASE_ORDER`** — and the read RPC returns a cache that is never mutated by `tick`. |
| 11 | **Godot reference only has 1 scene** | `clients/godot-ref/scenes/` | `main.tscn` is the only scene. No `game.tscn`, `editor.tscn`, `sandbox.tscn`, `inspect_panel.tscn`. The Godot attach matrix says "default `attach_mode=server`" but there is no playable Godot project scene — the `main.tscn` is a placeholder. |
| 12 | **Web dashboard is a Babylon/Three viewer, not a godgame** | `web/dashboard/src/` | 14 `.mjs` modules, no in-world interaction. No god-tool palette UI, no keycap palette, no in-world click → god-tool handler. |

### 1.3 MISSING ENTIRELY (no design, no code)

Things the player/agent expects in a finished AAA godgame that **do not appear in any design doc or any code path**.

| # | Missing | Why it matters |
|---|---------|----------------|
| 1 | **No in-world god-tool palette UI in any client** | The 50-verb catalog is data (`civ-powers`). The Bevy client has `god_panel.rs` (`clients/bevy-ref/src/god_panel.rs`) but it is **not in the active render pipeline** — only `godtools.rs` in the engine is wired. Godot/Unreal/web have **no equivalent** of `god_panel.rs`. The player cannot pick a tool and click on the world. |
| 2 | **No undo / redo path** | `GODTOOLS_IMPL_PLAN.md` mentions FR-CIV-GODTOOL-921 (undo). The MCP spec marks `sim_undo` as `Blind` (returns `unavailable`). No `GodToolHistory` exists in `civ-engine`. The HUD's `palette chip` for "data not yet surfaced" is literally the verbatim documentation string. |
| 3 | **No save/load UI in any client** | Save-db exists (`crates/save-db`, 568 lines, full SQLite metadata index). HTTP routes exist (`crates/watch/src/saves_api.rs`). The 4 clients (Bevy, Godot, Unreal, web) **do not render a save panel**. Players have to use the JSON-RPC bridge directly. |
| 4 | **No loadable scenario beyond `baseline.yaml`** | `scenarios/` has `baseline.yaml` + `canonical_seeds.ron` + `presets/`. The presets directory exists but is empty (per `ls`). `sim.load_scenario` will accept a string but the validator rejects anything other than `baseline` today. |
| 5 | **No notion of "campaign" or game-flow state machine** | `MCP_CONTROL_SURFACE.md` lists `sim.outcome` with tags `ongoing/victory/defeat/draw`. The `ongoing` is the default. **Nothing in the codebase transitions to `victory` or `defeat`** — there's no victory-condition evaluator, no defeat-condition evaluator, no end-game UI. |
| 6 | **No 3D actor / building art assets** | Quixel/Megascans is "local-only" per `AGENTS.md`. The `bevy-ref` and `godot-ref` clients render **capsule fallbacks** (`ActorVisualKind::Humanoid` → capsule). The 16³ procedural mesh in Godot/Unreal is only present when `voxels` is dense. |
| 7 | **No terrain LOD / streaming for world sizes > 256³** | `crates/voxel/src/` has adaptive voxel + dirty events, but the "20-mile streaming window" from `master-roadmap.md` S4.W1 is **not implemented** — S4 is in the future stages. |
| 8 | **No audio playback path** | The `civ-audio` crate compiles and has bus/mix/ducking/mood/sfx/triggers/ui_sound, but **no client plays sound**. No spatial audio, no music layer, no SFX on god-tool receipt. |
| 9 | **No tutorial / onboarding path** | `docs/design/onboarding-qol.md` exists. **No code under that doc exists in `crates/`**. The bevy-ref client has `tutorial.rs` but it's a stub. |
| 10 | **No multiplayer / hot-seat / shared session** | The watch server has one `sim` per bridge. No concept of multiplayer sync, no client-of-clients, no observer/spectator protocol beyond the per-connection subscription filter. |
| 11 | **No localization / native-language UI surface** | The HUD spec (§1.2 of `emergent-systems-spec.md`) calls for "native lexicon + script atlas toggles" — none of this ships. The 4 clients render English-only. |
| 12 | **No "world settings" / new-game flow** | `sim.reset` exists; `sim.load_scenario` exists; but **no client has a new-game flow** (pick seed, pick biome, pick starting era). Players launch into `baseline.yaml` or nothing. |

### 1.4 Perceptual & playability gaps (the "can a player actually launch?")

These are the things that make a godgame *feel* like a godgame. Today:

| # | Question | Answer | Evidence |
|---|----------|--------|----------|
| 1 | Can a player **launch**? | Yes, via `cargo run -p civ-server` + `cargo run -p civ-watch` + click into one of 4 clients. No installer, no packaged binary, no Steam hook. | `AGENTS.md` "verify before you claim done" table |
| 2 | Can a player **see a world**? | Yes in Bevy (full mesh), partial in Godot/Unreal (16³ mesh when dense), partial in web (Babylon/Three viewer). | `fr-ax-dx-ux-maturity-audit.md` UX-03 |
| 3 | Can a player **use god-tools**? | No in any client UI. The substrate accepts them via JSON-RPC; no client renders the palette in an actionable way. | `clients/bevy-ref/src/god_panel.rs` exists but isn't wired to input events |
| 4 | Can a player **watch emergence**? | The substrate runs emergence phases (12 of 24 in PHASE_ORDER — see §1.2 #1). The dashboards exist in `crates/server/src/jsonrpc.rs::EmergenceDashboard` + `sim.emergence`. The Bevy client has `emergence_dashboard.rs`. **No HUD wire in Godot/Unreal/web.** | `crates/server/src/jsonrpc.rs:80-83` |
| 5 | Can a player **save / load**? | Backend: yes (`save-db`, `save.slot`/`save.load` RPCs, `saves_api.rs`). UI: **no** — no save panel in any client. | `crates/save-db/src/lib.rs`, `crates/watch/src/saves_api.rs` |
| 6 | Can a player **hear anything**? | No. The audio crate compiles but is not invoked. | `crates/audio/src/` (8 files, no callers) |
| 7 | Can a player **read the lore** (legends)? | `crates/legends/src/` has 8 files (`graph.rs`, `model.rs`, `query.rs`, `rumor.rs`, `worker.rs`, `config.rs`, `ids.rs`). `sim.emergence` exposes some data. **No "legends browser" UI in any client.** | `crates/legends/src/` |
| 8 | Can a player **manage their faction**? | `sim.get_factions` returns faction summaries. No faction-management UI in any client. | `crates/server/src/jsonrpc.rs` `GetFactions` |
| 9 | Can a player **research tech**? | `sim.queue_research` exists. **The phase that should advance the queue (`phase_research`) is not in PHASE_ORDER.** UI: no tech-tree UI in any client except Bevy (`tech_tree_ui.rs` stub). | `crates/engine/src/engine.rs:1204`, `clients/bevy-ref/src/tech_tree_ui.rs` |
| 10 | Can a player **declare war**? | `sim.diplomacy_action` is a stub (see §1.2 #3). | `crates/server/src/jsonrpc.rs:1706-1728` |
| 11 | Can a player **get modded content**? | Yes — `just civis-3d-mod-sign` + `just civis-3d-mod-package-all` produce `example-policy.civmod` and `example-economic.civmod`. Mod browser on `sim.snapshot`. Web **Mods** panel. Godot **Mods** label. | `AGENTS.md` maturity section |
| 12 | Can a player **replay / rewind**? | `sim.save_replay` / `sim.load_replay` work; soft-determinism caveat (RNG re-rolls on forward continuation). No replay scrubber UI in any client. | `crates/server/src/jsonrpc.rs:1350-1448` |
| 13 | **Day/night cycle visible?** | Bevy: yes (lighting curve). Others: not in default state. | `clients/bevy-ref/src/lighting_gi.rs`, `crates/watch/src/snapshot.rs:80-81` |
| 14 | **Weather visible?** | `crates/watch/src/snapshot.rs:82` populates `weather_snapshot`; Bevy client doesn't render precipitation. | snapshot shape vs Bevy render code |

---

## 2. Top 15 highest-leverage gaps to reach playable-AAA

Each item: **what it is, why it unblocks playability, the [design|impl|wire|polish] tag, and the crate(s) it touches.**

| Rank | Gap | Why it unblocks | Tag | Crate(s) |
|------|-----|-----------------|-----|----------|
| 1 | **Fix `PHASE_ORDER` ↔ `Simulation::tick` sync** — extend `PHASE_ORDER` to all 24 phases in tick order; remove or relax the `phase_order_matches_tick_sequence` literal test (or update it); ensure `phase_emergence` is the final entry as `phase_order_includes_emergence` requires. | Without this, the engine's own determinism test (`tick_invokes_emergence_phase`) fails, the README claim of "11 emergence phases WIRED" is half-true, and every downstream feature that reads emergence state (legends, saga, psyche overlay, dashboard) sees stale data. | `[wire]` | `crates/engine/src/engine.rs:55-68, 1183-1223, 3373-3422` |
| 2 | **Implement MCP control surface (29 missing tools)** — add `sim_inspect`, `sim_status`, `sim_inspect_tile`, `sim_get_factions`, `sim_get_resources`, `sim_get_emergence`, `sim_get_tech`, `sim_get_diplomacy`, `sim_get_outcome`, `sim_step`, `sim_set_speed`, `sim_get_speed`, `sim_run_until`, `sim_reset`, `sim_load_scenario`, `sim_spawn`, `sim_sculpt`, `sim_damage`, `sim_terraform_extent`, `sim_spawn_organism`, `sim_disaster`, `sim_law`, `sim_undo`, `sim_save_slot`, `sim_load_slot`, `sim_list_saves`, `sim_save_replay`, `sim_load_replay`, `sim_subscribe`, `sim_unsubscribe`, `sim_update_subscription`, `sim_health`. Per `MCP_CONTROL_SURFACE.md` §2. | **Highest single-leverage gap.** The MCP surface is the spec binding target for godgame-as-a-service. 29 of 32 tools missing = agents can't operate the godgame. Today, MCP only exposes 3 verification harnesses. This is also the path to "AI agent plays the godgame" the marketing pillars rely on. | `[impl]` | `crates/civis-mcp/src/server.rs` (+ `crates/civis-mcp/src/lib.rs` for the WS-shim pooling fix from §4.3 of the spec) |
| 3 | **Wire `civ-powers` substrate handlers for the 44 `Near` verbs** — extend `TerraformOp` (8 missing), `LifeRequest` (7 missing), `DisasterRequest` (7 missing), `InspectRequest` (7 missing), add `LawRequest` enum (8 missing), `MaterialRequest` (8 missing). Each handler routes through the existing substrate API (push_voxel_write, invoke_divine_disaster, spawn_civilian_at). | Without these, the 50-verb god-tools panel is **purely cosmetic**. The data-driven catalog (`civ-powers`) is fine; the substrate write paths are missing. The player's god-game agency is ~12% of spec. | `[impl]` | `crates/engine/src/godtools.rs:104-178` (extend the four enums), `crates/powers/src/lib.rs` (flip `Near` → `Live` per handler landing) |
| 4 | **Add a unified god-tool palette UI to one client** — wire `clients/bevy-ref/src/god_panel.rs` to mouse / key events so a player can click a verb, click the world, and fire the substrate handler. Use `crates/hud/src/key_palette.rs` + `tile_inspector.rs` as the design template (KeycapPalette design system). | **This is the single perceptual gap that turns Civis from "engine library" into "playable game".** Without it, no player can wield a god-tool. Today, a player can launch Bevy and *watch* — but cannot *do*. | `[impl]` | `clients/bevy-ref/src/god_panel.rs` (existing stub), `crates/hud/src/key_palette.rs` (data layer, already shipped) |
| 5 | **Replace `sim.inspect_tile` stub body with real read** — fetch `civ-engine::Simulation::inspect_tile(x, y)` and return `{ material, faction_id, terrain_height, nearest_agent_alignment }`. Per `crates/server/src/jsonrpc.rs:1689-1704`. | The MCP `sim_inspect_tile` and the in-game `inspect.probe` god-tool both depend on this. Until it returns real data, no client can show "what's under my cursor" — which is the #1 information-views requirement (per `info-views.md` / `master-roadmap.md` "legibility root"). | `[wire]` | `crates/server/src/jsonrpc.rs:1689-1704`, `crates/engine/src/` (add `Simulation::inspect_tile` if missing) |
| 6 | **Implement `sim.diplomacy_action` body** — accept `propose_treaty` / `declare_war` / `offer_trade` and route to `civ-diplomacy`. Per `crates/server/src/jsonrpc.rs:1706-1728`. | Stub today; diplomacy crate exists, `phase_diplomacy` runs in `tick`, but no agent can ever actually *do* a diplomatic action. The MCP `sim_get_diplomacy` tool depends on this returning real data. | `[impl]` | `crates/server/src/jsonrpc.rs:1706-1728`, `crates/diplomacy/src/` |
| 7 | **Build out `sim_terraform_extent` substrate brush (T1–T11)** — the footprint-aware god-tool that today degrades to per-voxel `sim.place_voxel`. Per `GODTOOLS_IMPL_PLAN.md` P2.1 + `brush-tool-system.md`. Touches `crates/voxel/src/brush.rs` (new) + `crates/engine/src/godtools.rs::apply_terraform`. | 11 of the 50 god-verbs are TERRAIN ops. Without a brush primitive, every brush op (Raise/Lower/Level/Smooth/Slope/Flatten/Shift/AddLand/DigOcean/RaiseMountain/DropBiome) becomes a per-voxel loop in the agent. This is the highest-throughput gap. | `[impl]` | `crates/voxel/src/brush.rs` (new), `crates/engine/src/godtools.rs` (apply_terraform) |
| 8 | **Wire `phase_language` / `phase_religion` / `phase_psyche` into tick + JSON-RPC** — language.rs / religion.rs files exist in `crates/engine/src/` but are not in `PHASE_ORDER`. Add read RPCs `sim.language_state`, `sim.religion_state`, `sim.psyche_state` returning cluster-level aggregate. | The language/religion/psyche phases are designed in 3 charter-layer docs (`LANGUAGE_EMERGENCE.md`, `SENTIENCE_PSYCHE.md`, `CULTURE_IDEOLOGY.md` / `civ-culture-emergent.md`) but the substrate phases that should produce emergent data **never run**. The "emergence" the project markets is invisible until these run. | `[wire]` | `crates/engine/src/{language,religion}.rs`, `crates/server/src/jsonrpc.rs` (new RPCs) |
| 9 | **Add save/load UI to one client** — render `crates/hud/src/save_panel` (already defined as `SurfaceFlag`) in Bevy/Godot. Wire it to `save.slot` / `save.load` / `save.list`. The 4 clients can each do their own port after Bevy lands. | Save-db is finished. The HTTP API is finished. **The UI is the missing piece**. Players today have to `curl` the bridge to save a game. | `[impl]` | `clients/bevy-ref/src/save_load_ui.rs` (existing stub), `crates/hud/src/lib.rs` (data), `crates/save-db/src/lib.rs` (engine) |
| 10 | **Add a quixel/Megascans placeholder asset slot + bridge import step** — define `Content/Megascans/` slot conventions, document the bridge import flow, add `glb` stub loaders for hero assets. Even capsule-only is acceptable as a placeholder if explicitly labeled. | The Bevy/Godot clients render capsule fallbacks today. The "phenotype-gfx build unblocked" line in the task description refers to *the render pipeline being buildable*, not the *assets being present*. Until assets land, all renders look like prototype. | `[polish]` | `clients/bevy-ref/assets/`, `clients/godot-ref/assets/`, `clients/unreal-show/Content/` (asset slots), `crates/protocol-3d/` (asset codec) |
| 11 | **Add audio playback path in `civ-watch`** — pipe `crates/audio/src/{ambient,mood,sfx,triggers}.rs` output to a real audio sink (rodio or similar). Today the audio crate compiles but produces no sound. | `SOUND_MUSIC_EMERGENCE.md` is 332 lines of design and the crate exists. **No audio reaches a speaker.** Audio is the cheapest "AAA-feel" uplift available — adaptive ambient + mood → music is the "ten-second uplift" that makes a prototype feel shipped. | `[wire]` | `crates/audio/src/`, `crates/watch/src/app.rs` (audio spawn), `crates/engine/src/engine.rs` (`phase_audio` in PHASE_ORDER) |
| 12 | **Land `sim_run_until` server-side loop helper** — a `civ-server` utility that takes `target_tick` / `target_outcome` / `timeout` and drives the sim to that point. Per `MCP_CONTROL_SURFACE.md` §2.2.4. Without it, AI agents loop on `sim_step(1)`. | Without a long-running tick driver, every AI agent that wants to "fast-forward to outcome" hammers the server with single-tick requests. This is also a key UX win for human players who hit `time.fast_forward_to_event` and expect it to actually pause at the event. | `[impl]` | `crates/server/src/lib.rs` (new helper), `crates/civis-mcp/src/server.rs` (wire `sim_run_until`) |
| 13 | **Add legends browser UI to one client** — render `crates/legends/src/{graph,query,model}.rs` data as a clickable saga tree. Bevy has `clients/bevy-ref/src/event_feed.rs` (close enough to start). | `legends-engine.md` is the "#1 depth-moat" per `master-roadmap.md` S2.W2. Without UI, the saga graph is invisible to the player. **The depth that makes Civis a *godgame* instead of a *godtoy* is legends.** | `[impl]` | `crates/legends/src/`, `clients/bevy-ref/src/event_feed.rs` (extend), `crates/hud/src/` (legend token styles) |
| 14 | **Document and wire the long-lived WS MCP shim** — per `MCP_CONTROL_SURFACE.md` §4.3, the shim currently opens a fresh WS per call. Replace with a connection pool keyed by `civis_cli::attach_config`. | Without this, `sim_subscribe` / `sim_update_subscription` / `sim_unsubscribe` are no-ops. The MCP subscription surface is documented but useless — that breaks the spec's own AC-MCP-INT-1 acceptance criterion. | `[wire]` | `crates/civis-mcp/src/lib.rs:138-161`, `crates/server/src/ws_bridge.rs` (connection lifecycle) |
| 15 | **Resolve the design-vs-impl gap on outcome / victory / defeat** — `sim.outcome` returns `ongoing` forever today. Add a `civ-engine::check_outcome` evaluator that reads `institutions`, `population`, `era`, and transitions to `victory` / `defeat` / `draw`. Wire MCP `sim_get_outcome` to surface real transitions. Add a "campaign ended" UI screen to one client. | A godgame without an end-state is a sandbox. The MCP spec promises `ongoing / victory / defeat / draw`. The substrate has `ongoing` baked in. **Without an outcome evaluator, the game cannot ship.** | `[design]` (the doc needs a "victory condition spec" addition) + `[impl]` (the evaluator) + `[polish]` (end-game UI) | `crates/engine/src/engine.rs` (new `phase_outcome` or `Simulation::check_outcome`), `crates/server/src/jsonrpc.rs:102` (`SimOutcome`), `crates/hud/src/` (outcome overlay token), `clients/bevy-ref/src/outcome_overlay.rs` (existing stub) |

---

## 3. Top 5 gaps, one-line each

1. **Fix `PHASE_ORDER` to match the 24 phases `Simulation::tick` actually calls** — the engine's own determinism test (`tick_invokes_emergence_phase`) panics today, and 11 emergence phases silently don't run. `[wire]` · `crates/engine/src/engine.rs:55-68`
2. **Ship the 29 missing MCP control-surface tools** — only 3 of the 32 designed tools exist; agents cannot operate the godgame. `[impl]` · `crates/civis-mcp/src/server.rs`
3. **Implement substrate handlers for the 44 `Near` god-tool verbs** — 88% of the god-tools panel is lit-but-inert; substrate writes exist for only 6 of 50. `[impl]` · `crates/engine/src/godtools.rs:104-178`
4. **Wire a unified god-tool palette UI in one client** — without a clickable palette, no player can wield a god-tool; the engine is a viewer, not a game. `[impl]` · `clients/bevy-ref/src/god_panel.rs`
5. **Add the `sim_terraform_extent` footprint brush primitive** — 11 TERRAIN ops degrade to per-voxel `sim.place_voxel`; this is the highest-throughput substrate gap. `[impl]` · `crates/voxel/src/brush.rs` (new) + `crates/engine/src/godtools.rs::apply_terraform`

---

## 4. What the audit did NOT find

Honest negative findings — places where the substrate is **already** in playable shape and the report does not need to call them out as gaps:

- **Determinism + replay** are real: `BLAKE3 hash_chain`, `.civreplay` round-trip, `verify_hash_chain`, snapshot save/load all work and are tested.
- **JSON-RPC** is finished: 32 methods on the wire, `just civis-3d-catalog-check` gates drift, 28+ `ws_smoke` tests pass.
- **Modding v3** is shipped end-to-end for the headline figure: WASM ticks, `.civmod`, signing, mod browser on `sim.snapshot`, web **Mods** panel, `mod.loaded.v1` replay-bus JSON, 25+ `civ-mod-host` tests.
- **Build is unblocked** (`phenotype-gfx` is buildable per task description).
- **Three emergent phases ARE in tick + PHASE_ORDER** (production, citizen_lifecycle, military, policy, economy, planet, diplomacy, tactics, voxel, compact, buildings, diffusion).
- **Web dashboard** has Babylon + Three renderers and a 16³ mesh viewer — partial but real.
- **Per-client minimap click-to-focus** works in Bevy / Godot / web / Unreal.
- **Save-db** is finished (SQLite metadata index, archive bytes, restore path).
- **Spec backlog** (`docs/specs/backlog.md`) is well-curated; the gap is execution, not direction.

---

## 5. Recommended execution order (next 4 PRs)

Based on the leverage ranking above, the smallest set of changes that turn Civis from "engine library" into "playable prototype":

1. **PR-A (impl): Fix PHASE_ORDER** — `crates/engine/src/engine.rs:55-68` + matching test fix. Unblocks every downstream emergence consumer.
2. **PR-B (impl): Wire god-tool palette UI in Bevy client** — `clients/bevy-ref/src/god_panel.rs` + input handlers + 6 `Live` verb substrate paths. First playable click-to-fire loop.
3. **PR-C (impl): Replace `sim.inspect_tile` stub + `sim.diplomacy_action` stub** — both are 30-line handlers in `crates/server/src/jsonrpc.rs`. Unlocks MCP `sim_inspect_tile` + `sim_get_diplomacy`.
4. **PR-D (impl): Land first 10 MCP control-surface tools** — `sim_inspect`, `sim_status`, `sim_step`, `sim_set_speed`, `sim_get_speed`, `sim_spawn`, `sim_sculpt`, `sim_health`, `sim_outcome`, `sim_reset`. The "AI can play" milestone.

After these four, the project has a Bevy client where you can launch, see a world, pick a god-tool, click the world, fire it, see emergence in the dashboard, and save/replay. **That's a playable AAA godgame prototype.** Everything else is breadth.

---

*End of audit.*