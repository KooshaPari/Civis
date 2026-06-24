# MCP Control Surface — Programmatic God-Tools & Sim Inspection for AI Agents

> **Status:** Binding design (2026-06-23). Authoritative spec for the
> `civis-mcp` tool surface. The current crate ships **3 harness tools**
> (`civis_verify`, `civis_pixels`, `civis_census`); this document
> specifies the **agent-facing control surface** — the 32 tools an
> AI agent / CI runner / scripted test harness needs to *act on* and
> *observe* the godgame over the existing JSON-RPC bridge.
>
> **Authority contracts (do not duplicate):**
> - `crates/server/src/jsonrpc.rs` — the 34 JSON-RPC methods this surface
>   composes against. Every tool here is a thin adapter over one or
>   more methods in that file.
> - `docs/design/GOD_TOOLS_SANDBOX.md` — the 50-verb god-tools
>   spec (42 mutating + 8 INSPECT). The mapping table in §3 of this
>   doc binds each god-verb to its eventual JSON-RPC surface.
> - `docs/design/GODTOOLS_IMPL_PLAN.md` — phased substrate-side
>   rollout; some god-verbs are still `Near`/`Blind` and route through
>   stub JSON-RPC methods today. This doc marks those explicitly.
> - `crates/civis-mcp/src/{lib,server}.rs` — the existing rmcp shim
>   (3 harness tools). New tools follow the same `#[tool_router]`
>   pattern.
>
> **Governing canon (per AGENTS.md):**
> - **Civis CLI default stack:** `cargo run -p civ-server` →
>   `ws://127.0.0.1:3000/ws?tick_format=binary`. All tools assume that
>   endpoint unless overridden via `host` / `port` tool args.
> - **No cargo from the MCP shim** — the existing pattern (per
>   `crates/civis-mcp/src/lib.rs:7-15`) is *direct library calls into
>   `civis-cli` and the WS bridge*, never `cargo run` shell-out. The new
>   tools follow the same rule.
> - **No bypass** — every tool here is a thin shim over an existing
>   JSON-RPC method. No tool writes a substrate field directly.
> - **Soft determinism** — per the emergence charter, rewind/load
>   diverge from the original run; tools that depend on replay state
>   must surface that contract.

---

## 0. Thesis

The MCP control surface is **a remote control for the godgame**: the
8 keyboard hotkeys a player uses, plus the inspector + replay controls,
exposed as **typed MCP tools** that an AI agent can call. Three
audiences:

1. **AI agent / LLM codegen** — Claude / GPT / etc. driving a godgame
   session through natural-language task decomposition.
2. **CI / scripted tests** — replay/load determinism checks, smoke
   tests, scenario bootstrap, agent-emergence probes.
3. **Live spectator clients** — Godot / Unreal / web mirrors that
   prefer MCP-shaped RPCs to bespoke WS clients.

**Headline figure:** **32 MCP tools**, mapping onto the 50 god-verbs
(per `GODTOOLS_IMPL_PLAN.md`) and the 34 JSON-RPC methods (per
`crates/server/src/jsonrpc.rs`). The de-duplication is intentional:
several god-verbs share one tool (e.g. `sim_spawn` covers
`SimSpawnCivilian` + `SimSpawnEntity`), and several JSON-RPC reads
are folded into one inspector tool (`sim_inspect` returns the
canonical tile/entity snapshot).

**Existing surface (3 harness tools, kept):** `civis_verify`,
`civis_pixels`, `civis_census` — verification harness; not removed by
this design (they are the *test rig*, not the *control surface*).

**New surface (32 tools, this spec):** the agent control plane.

---

## 1. Tool taxonomy — 32 tools in 6 categories

| # | Tool | Category | Backing JSON-RPC | God-verb family | Availability |
|---|------|----------|------------------|-----------------|--------------|
| 1 | `sim_inspect` | Inspect | `sim.snapshot` (+ projection of `sim.inspect_tile`) | I1–I8 | **Live** |
| 2 | `sim_status` | Inspect | `sim.status` | I2 (coarse) | **Live** |
| 3 | `sim_inspect_tile` | Inspect | `sim.inspect_tile` | I1 (Probe) | **Live** (stub body) |
| 4 | `sim_get_factions` | Inspect | `sim.get_factions` | I2 (Stats) | **Live** |
| 5 | `sim_get_resources` | Inspect | `sim.get_resources` | I2 (Stats) | **Live** |
| 6 | `sim_get_emergence` | Inspect | `sim.get_emergence_metrics` + `sim.emergence` + `emergence.dashboard` | I2 (Stats) | **Live** |
| 7 | `sim_get_tech` | Inspect | `sim.tech_state` | I2 (Stats) | **Live** |
| 8 | `sim_get_diplomacy` | Inspect | (proxy via `sim.snapshot` `factions` + `sim.get_factions`) | I2 (Stats) | **Live** (partial) |
| 9 | `sim_get_outcome` | Inspect | `sim.outcome` | I8 (Follow) | **Live** |
| 10 | `sim_step` | Step | `sim.command { action: tick }` (+ `sim.set_speed` for pause) | TM1–TM5 | **Live** |
| 11 | `sim_set_speed` | Step | `sim.set_speed` | TM1–TM4 | **Live** |
| 12 | `sim_get_speed` | Step | `sim.get_speed` | TM1–TM4 | **Live** |
| 13 | `sim_run_until` | Step | `sim.set_speed` + repeated `sim_step` + watch filter | TM5 (Step), TM7 (FF-to-event) | **Live** (server-side loop) |
| 14 | `sim_reset` | Step | `sim.reset` | (session control) | **Live** |
| 15 | `sim_load_scenario` | Step | `sim.load_scenario` | (session control) | **Live** |
| 16 | `sim_spawn` | Mutate | `sim.spawn_entity` (and `sim.spawn_civilian` when `kind=civilian`) | L1, L2, L3 (via palette) | **Live** |
| 17 | `sim_sculpt` | Mutate | `sim.place_voxel` (per-stamp; batched for TERRAIN) | T1–T11 (Raise/Lower/Level/…) | **Live** (voxel-stamp) |
| 18 | `sim_damage` | Mutate | `sim.damage` | D1–D8 (disaster — direct path) | **Live** (voxel-damage) |
| 19 | `sim_terraform_extent` | Mutate | (planned `sim.terraform_extent` — not yet on the wire; folds to repeated `sim.place_voxel` today) | T1–T11 (footprint) | **Near** (degrades to per-voxel `sim_sculpt`) |
| 20 | `sim_spawn_organism` | Mutate | (planned `sim.spawn_organism` — not yet on the wire; folds to `sim.spawn_entity` `kind=civilian` today) | L1, L2, L8 | **Near** (degrades to `sim_spawn`) |
| 21 | `sim_disaster` | Mutate | (planned `sim.disaster` — not yet on the wire; folds to `sim.damage` / `sim.set_policy` / future `invoke_divine_disaster`) | D1–D8 | **Near** (degrades to `sim_damage`) |
| 22 | `sim_law` | Mutate | (planned `sim.law` — folds to `sim.set_policy` today; full surface in `GODTOOLS_IMPL_PLAN.md` P2.7) | LW1–LW8 | **Near** (degrades to `sim_set_policy`) |
| 23 | `sim_undo` | Mutate | (planned — not on the wire; `GODTOOLS_IMPL_PLAN.md` T3.9) | TM8 / undo (FR-CIV-GODTOOL-921) | **Blind** (returns `unavailable`) |
| 24 | `sim_save_slot` | Persistence | `save.slot` | (session control) | **Live** |
| 25 | `sim_load_slot` | Persistence | `save.load` | (session control) | **Live** |
| 26 | `sim_list_saves` | Persistence | `save.list` | (session control) | **Live** |
| 27 | `sim_save_replay` | Persistence | `sim.save_replay` | (replay) | **Live** |
| 28 | `sim_load_replay` | Persistence | `sim.load_replay` | (replay) | **Live** |
| 29 | `sim_subscribe` | Subscription | `sim.subscribe` | (broadcast filter) | **Live** |
| 30 | `sim_unsubscribe` | Subscription | `sim.unsubscribe` | (broadcast filter) | **Live** |
| 31 | `sim_update_subscription` | Subscription | `sim.update_subscription` | (broadcast filter) | **Live** |
| 32 | `sim_health` | Health | `health` | (server liveness) | **Live** |

**Counted totals:**
- **Total tools:** 32 (3 existing harness + 29 new; existing kept).
- **Live now:** 27 (work over today's JSON-RPC surface).
- **Near (degrade to a Live tool):** 4 (`sim_terraform_extent`,
  `sim_spawn_organism`, `sim_disaster`, `sim_law`).
- **Blind:** 1 (`sim_undo` — returns `unavailable` until
  `GODTOOLS_IMPL_PLAN.md` T3.9 lands).

**Per-god-verb coverage (per `GODTOOLS_IMPL_PLAN.md` §10):**
- TERRAIN (T1–T11): covered by `sim_sculpt` (per-voxel today) +
  `sim_terraform_extent` (Near) → 11/11 reachable.
- MATERIAL (M1–M8): M1–M5, M8 covered by `sim_sculpt` (material-id);
  M6 (SeedForest) → `sim_spawn` with `kind=civilian` (degrades);
  M7 (SeedOre) → Near (degrades to `sim_sculpt` with ORE material);
  → 8/8 reachable.
- LIFE (L1–L8): L1–L3 covered by `sim_spawn`; L4–L7 (Bless/Curse/Heal/Plague)
  → **deferred** (no substrate write path today; no JSON-RPC
  `sim.actor_effect` on the wire). **Gap.** L8 (Extinct) → Near.
  → 5/8 reachable today.
- DISASTER (D1–D8): D3 (Flood) and D7 (Volcano) covered by `sim_damage` +
  material flood; D1, D2, D4, D5, D6, D8 → **deferred** (no per-disaster
  JSON-RPC). **Gap.** → 2/8 reachable today, full 8/8 via
  `sim_disaster` once `crates/engine/src/godtools.rs` P2.6 lands.
- INSPECT (I1–I8): all 8 covered by `sim_inspect` + `sim_get_*` →
  8/8 reachable.
- LAW (LW1–LW8): LW1 (Tax) covered by `sim_law` (degrades to
  `sim_set_policy`); LW2–LW8 → **deferred** (no `sim.law` JSON-RPC).
  → 1/8 reachable today.
- TIME (TM1–TM8): TM1–TM5 covered by `sim_step` + `sim_set_speed`;
  TM6 (Rewind) → **deferred** (no `sim.rewind` JSON-RPC; charter
  soft-determinism — restores via snapshot but RNG re-rolls).
  → 5/8 reachable.
- CAMERA (C1–C8): **out of scope** — universal UI verb, not a
  substrate write, not exposed via MCP. Agents that need camera
  control use the Bevy/Godot/Unreal client directly. (Per
  `GODTOOLS_IMPL_PLAN.md` §3.7 "no `Simulation::apply_god_tool` call".)

**Headline reachable god-verbs today:** 30/42 mutating + 8/8 INSPECT
= **38/50 verbs** (76%). Full 50/50 once
`GODTOOLS_IMPL_PLAN.md` P2-P3 lands.

---

## 2. Per-tool spec

Each subsection specifies: **purpose**, **JSON-RPC backing**,
**parameters** (typed, MCP-`JsonSchema` documented), **return shape**,
**charter coupling** (which substrate field it reads / writes), and
**god-verb family** it serves.

### 2.1 INSPECT — read-only sim probing (8 tools)

> **Charter coupling:** **read-only.** No tool in this section writes
> a substrate field. AC-CPL-1 ("write only what the substrate owns")
> trivially holds because nothing is written.

#### 2.1.1 `sim_inspect` — full sim snapshot

The "I want to see everything" tool. Wraps `sim.snapshot` and
optionally layers a tile probe on top.

```rust
#[derive(Deserialize, JsonSchema)]
pub struct SimInspectArgs {
    /// Optional normalized tile coord (0..=1) to project as a Probe read
    /// on top of the snapshot — i.e. `sim.inspect_tile` results merged in.
    #[schemars(description = "Optional tile X in [0, 1]")]
    pub tile_x: Option<f32>,
    /// Optional normalized tile Y in [0, 1].
    pub tile_y: Option<f32>,
}
```

- **Backing:** `sim.snapshot` (always); `sim.inspect_tile` (when
  `tile_x`/`tile_y` are set).
- **Returns:** the `sim.snapshot` JSON payload (tick, population,
  building_count, energy_budget, market_prices, hash_chain_root,
  speed_multiplier, civ_pins, factions, buildings, is_day,
  institutions, military_units, damage_events, climate, emergence,
  mods, mod_lifecycle, session_saved, mod_permission_violations,
  researched, in_progress_tech) plus, when tile coords are set, a
  `tile` block with the `sim.inspect_tile` probe result.
- **God-verbs:** I1 (Probe), I2 (Stats), I5 (CompareSnapshots —
  read-only half).
- **Live.**

#### 2.1.2 `sim_status` — coarse liveness probe

```rust
pub struct SimStatusArgs {}  // no params
```

- **Backing:** `sim.status`.
- **Returns:** `{ "tick": u64, "population": Option<u64> }`.
- **God-verb:** I2 (Stats — coarse).
- **Live.** Lighter than `sim_inspect`; the right tool for a heartbeat
  poll.

#### 2.1.3 `sim_inspect_tile` — single-tile probe

```rust
pub struct SimInspectTileArgs {
    /// World-tile X (integer; matches `sim.inspect_tile` semantics).
    pub x: i64,
    /// World-tile Y.
    pub y: i64,
}
```

- **Backing:** `sim.inspect_tile`.
- **Returns:** `{ "x": i64, "y": i64, "stub": true }` today (the
  handler is a stub per `crates/server/src/jsonrpc.rs:1689-1704`).
  Will fill in once `sim.inspect_tile` lands real terrain / faction
  reads in a follow-up PR.
- **God-verb:** I1 (Probe).
- **Live (stub body).** Tool is wired today; the *result body* is
  the gap, not the wire.

#### 2.1.4 `sim_get_factions` — faction summary

- **Backing:** `sim.get_factions`.
- **Returns:** `Vec<FactionSnapshot>` (`{ id, name, population, territory_size }`).
- **God-verb:** I2 (Stats — faction panel).
- **Live.**

#### 2.1.5 `sim_get_resources` — resource summary

- **Backing:** `sim.get_resources`.
- **Returns:** `Vec<ResourceSnapshot>` (`{ kind, amount, rate }`).
- **God-verb:** I2 (Stats — economy panel).
- **Live.**

#### 2.1.6 `sim_get_emergence` — emergence + dashboard + branching

Folds three JSON-RPC reads into one tool — the agent's entry point
to FR-CIV-EMERG-001/003 dashboard metrics plus the charter §3
branching/power-law/novelty scalars.

```rust
pub struct SimGetEmergenceArgs {
    /// If true, also pull the legacy coarse `sim.get_emergence_metrics`
    /// (faction_count/structure_count/language_count). Default false.
    #[schemars(description = "Also include legacy coarse metrics")]
    pub include_legacy: Option<bool>,
}
```

- **Backing:** `sim.emergence` (primary) + `sim.get_emergence_metrics`
  (when `include_legacy=true`) + `emergence.dashboard` (the
  five-tile dashboard block from PR #350).
- **Returns:** `{ "sample": EmergenceSampleFields | null, "dashboard":
  DashboardBlock, "legacy": Option<EmergenceMetricsSnapshot> }`. The
  `null` `sample` is the "ticks 0..49, no sample yet" state per
  `crates/server/src/jsonrpc.rs:1668-1670`.
- **God-verb:** I2 (Stats — emergence tile).
- **Live.**

#### 2.1.7 `sim_get_tech` — research tree

- **Backing:** `sim.tech_state`.
- **Returns:** `{ "available": [String; 12], "researched": [String],
  "in_progress": Option<String>, "tick": u64 }`. The 12 hardcoded
  techs (per `crates/server/src/jsonrpc.rs:1935-1948`): pottery,
  masonry, writing, iron_working, currency, mathematics, gunpowder,
  printing, banking, steam_power, electricity, railroad.
- **God-verb:** I2 (Stats — research panel).
- **Live.**

#### 2.1.8 `sim_get_diplomacy` — diplomacy view

Today's surface is thin — `sim.diplomacy_action` is a stub
(`crates/server/src/jsonrpc.rs:1706-1728`). The read-side folds:

- **Backing:** `sim.get_factions` (territory sizes) + a derived
  per-faction-pair tension estimate from the `sim.snapshot.mods` /
  `mod_lifecycle` bus. Returns a minimal "who exists, what cluster
  sizes" view.
- **Returns:** `{ "factions": Vec<FactionSnapshot>, "tension": HashMap<(u32, u32), f32> }`.
  The `tension` map is **best-effort** and is omitted when no signal
  is available (fresh sim).
- **God-verb:** I2 (Stats — diplomacy panel).
- **Live (partial).** Full surface once `crates/diplomacy` lands a
  read RPC.

#### 2.1.9 `sim_get_outcome` — game outcome

- **Backing:** `sim.outcome`.
- **Returns:** `{ "outcome": String, "reason": String, "tick": u64 }`.
  Outcome tags: `ongoing` (default), `victory`, `defeat`, `draw`
  (per `civ_engine::check_outcome`).
- **God-verb:** I8 (Follow — read trailing state).
- **Live.**

### 2.2 STEP — sim clock + session control (6 tools)

> **Charter coupling:** writes `BridgeState::speed_multiplier` +
> `Simulation::tick` advance. **No substrate field** is touched; the
> substrate reads its own clock via the existing PHASE_ORDER pipeline
> (`crates/engine/src/engine.rs:55-68`). Per
> `crates/server/src/jsonrpc.rs:1316` the allowed multipliers are
> `{0, 1, 2, 4, 8}`.

#### 2.2.1 `sim_step` — advance N ticks

```rust
pub struct SimStepArgs {
    /// Number of ticks to advance. Default 1.
    pub ticks: Option<u32>,
    /// If true, set speed=0 (pause) before stepping, then restore. Default false.
    pub auto_pause: Option<bool>,
}
```

- **Backing:** For each tick: `sim.command { action: tick }` (if
  `require_role` is enabled the call carries `role: "operator"`).
  When `auto_pause=true`, calls `sim.set_speed(0)` first and
  `sim.set_speed(1)` after — matching the TM5 Step verb semantics
  in `GOD_TOOLS_SANDBOX.md:302`.
- **Returns:** `{ "ticks_advanced": u32, "tick": u64, "speed_after":
  u32 }`.
- **God-verbs:** TM1 (Pause, indirectly), TM5 (Step).
- **Live.**

#### 2.2.2 `sim_set_speed` — set sim speed multiplier

```rust
pub struct SimSetSpeedArgs {
    /// Multiplier. Must be 0, 1, 2, 4, or 8.
    pub multiplier: u32,
}
```

- **Backing:** `sim.set_speed`.
- **Returns:** `{ "accepted": true, "multiplier": u32 }`.
- **God-verbs:** TM1 (Pause when 0), TM2 (Play when 1), TM3 (Slow —
  the 0.25× fraction is approximated; the RPCs accept only
  `{0,1,2,4,8}`), TM4 (Fast).
- **Live.**

#### 2.2.13 `sim_get_speed` — read sim speed

- **Backing:** `sim.get_speed`.
- **Returns:** `{ "multiplier": u32 }`.
- **God-verbs:** TM1–TM4 (read half).
- **Live.**

#### 2.2.4 `sim_run_until` — run until predicate

```rust
pub struct SimRunUntilArgs {
    /// Maximum ticks to advance before giving up. Required.
    pub max_ticks: u64,
    /// Optional target tick — stop when sim tick >= this.
    pub target_tick: Option<u64>,
    /// Optional outcome tag — stop when `sim.outcome.outcome == tag`.
    /// One of: "ongoing", "victory", "defeat", "draw".
    pub target_outcome: Option<String>,
    /// Optional max wall-clock seconds. Default 30.
    pub timeout_seconds: Option<u32>,
    /// Optional interim poll interval in ticks (default 8).
    pub poll_every: Option<u32>,
}
```

- **Backing:** server-side loop: `sim.set_speed(N)` (optionally
  crank to 8×), `sim.command tick` in a loop, poll `sim.outcome` /
  read `ctx.tick`, stop on predicate. The MCP shim never holds the
  sim; it talks to the WS bridge per call.
- **Returns:** `{ "ticks_advanced": u64, "final_tick": u64,
  "final_outcome": String, "stopped_reason": "target_tick" |
  "target_outcome" | "max_ticks" | "timeout" }`.
- **God-verbs:** TM5 (Step at scale), TM7 (FastForwardToEvent).
- **Live.** This is the **right tool for an agent that wants "make
  the sim do something for a while"** — agents that loop on
  `sim_step(ticks: 1)` are wrong; they should call `sim_run_until`
  with a `target_tick` or `target_outcome` instead.

#### 2.2.5 `sim_reset` — fresh sim with a seed

```rust
pub struct SimResetArgs {
    /// RNG seed. Required.
    pub seed: u64,
}
```

- **Backing:** `sim.reset`.
- **Returns:** `{ "seed": u64, "tick": 0 }`.
- **God-verb:** (session control; not in the 50-verb set).
- **Live.**

#### 2.2.6 `sim_load_scenario` — load preset scenario

```rust
pub struct SimLoadScenarioArgs {
    /// Preset name (e.g. "three-race-balanced").
    pub preset: String,
    /// Optional seed; defaults to wall-clock seconds since epoch.
    pub seed: Option<u64>,
}
```

- **Backing:** `sim.load_scenario`.
- **Returns:** `{ "preset": String, "seed": u64, "tick": 0 }`.
- **God-verb:** (session control).
- **Live.**

### 2.3 MUTATE — agent-driven god-tool surface (7 tools)

> **Charter coupling:** every tool here routes through the standard
> `EditRequest` queue (the JSON-RPC `sim.*` methods). No tool writes
> a substrate field directly. The **role gate** (`role: "operator"`)
> is enforced on every mutating call when `require_role` is enabled
> on the server (per `crates/server/src/jsonrpc.rs:27`, `:380-409`).

#### 2.3.1 `sim_spawn` — spawn palette entity

The headline "put something on the map" tool. Wraps
`sim.spawn_entity` (and `sim.spawn_civilian` when `kind=civilian`).

```rust
pub struct SimSpawnArgs {
    /// Palette kind. One of: civilian, vehicle, airport, port, hangar.
    pub kind: String,
    /// Normalized X in [0, 1].
    pub x: f32,
    /// Normalized Y in [0, 1].
    pub y: f32,
    /// Owning faction id. Default 0.
    pub faction: Option<u32>,
}
```

- **Backing:** `sim.spawn_entity` (palette kinds per
  `crates/server/src/jsonrpc.rs:2085-2110`).
- **Returns:** `{ "accepted": true, "kind": String }` (plus
  `entity_id` set by the bridge after the actual spawn — see
  `crates/server/src/jsonrpc.rs:2242-2249`).
- **God-verbs:** L1 (SpawnOrganism — `kind=civilian` today),
  L2 (SpawnHerd — repeated `sim_spawn` calls; the LIFE
  `SpawnHerd` event handler is a follow-up), L3
  (SpawnCivilizationSeed — repeated `sim_spawn` × 6 today; full
  seed-payload is a follow-up), M6 (SeedForest — degrades to
  `kind=civilian` herd until `crates/agents::spawn_many`
  short-list lands), plus the military/building palette
  (Vehicle / Airport / Port / Hangar).
- **Live (with degradation).**

#### 2.3.2 `sim_sculpt` — write a single voxel

The "T1 Raise" / "M1 Replace" / "M2 AdditiveDrop" / "M8 SeedSnow"
path. Per-voxel stamp; for brushes use `sim_terraform_extent`.

```rust
pub struct SimSculptArgs {
    /// World X (fixed-point; matches `sim.place_voxel`).
    pub x: i64,
    /// World Y.
    pub y: i64,
    /// World Z.
    pub z: i64,
    /// Material id (0..=255). See `crates/voxel/src/material.rs:125-151`.
    pub material: u16,
}
```

- **Backing:** `sim.place_voxel` (per `crates/server/src/jsonrpc.rs:2204-2231`).
- **Returns:** `{ "accepted": true }`.
- **God-verbs:** T1–T11 (per-voxel — agents that need T3 Level /
  T6 Flatten / T7 Shift can fold them on the agent side into
  per-voxel `sim_sculpt` reads then writes; the brush helper
  `sim_terraform_extent` ships the footprint-aware version
  in a follow-up); M1–M5, M8 (material-id dispatch); M7 (SeedOre
  — material=ORE).
- **Live (per-voxel).** Footprint-aware version is **Near**
  (see `sim_terraform_extent` below).

#### 2.3.3 `sim_damage` — tactical voxel damage

```rust
pub struct SimDamageArgs {
    /// Damage center (world coords; fixed-point).
    pub x: i64,
    pub y: i64,
    pub z: i64,
    /// Radius in voxels (clamped to 1..=32). Default 8.
    pub radius: Option<u8>,
    /// Damage energy (default 1000).
    pub energy: Option<u32>,
}
```

- **Backing:** `sim.damage` (per `crates/server/src/jsonrpc.rs:2167-2202`).
- **Returns:** `{ "accepted": true }`.
- **God-verbs:** D3 (Flood — water column via repeated `sim_damage`
  + `sim_sculpt material=WATER`), D7 (Volcano — repeated
  `sim_sculpt material=LAVA` + `sim_damage`); D1, D2, D4, D5, D6,
  D8 → **deferred** until `sim.disaster` JSON-RPC lands.
- **Live (tactical).** Full-disaster set is **Near** (see
  `sim_disaster`).

#### 2.3.4 `sim_terraform_extent` — footprint-aware brush (Near)

**Availability: Near.** Today this tool **degrades** to repeated
`sim_sculpt` calls in a loop on the client side (the MCP shim
fans out N parallel `sim.place_voxel` requests). The full
server-side footprint path ships once `crates/voxel/src/brush.rs`
+ `crates/engine/src/godtools.rs::apply_terraform` land
(`GODTOOLS_IMPL_PLAN.md` P2.1).

```rust
pub struct SimTerraformExtentArgs {
    /// Brush op. One of: raise, lower, level, smooth, slope, flatten,
    /// shift, add_land, dig_ocean, raise_mountain, drop_biome.
    pub op: String,
    /// Center world coord (x, y, z).
    pub cx: i64, pub cy: i64, pub cz: i64,
    /// Brush radius in voxels.
    pub radius: u32,
    /// Op-specific magnitude (e.g. Δheight for raise/lower,
    /// target_height for level, etc.).
    pub magnitude: f32,
    /// Optional material id (for `drop_biome`).
    pub material: Option<u16>,
}
```

- **Backing (today, degraded):** fan-out to N `sim_sculpt` calls.
  Returns `{ "degraded": true, "voxels_written": u32, "ops_planned":
  u32 }` and emits an `unoptimised` tag in the result so the
  caller can opt to retry after the server-side path lands.
- **Backing (target):** a single `sim.terraform_extent` JSON-RPC
  method that calls `crates/voxel/src/brush.rs::stamp_footprint`.
- **God-verbs:** T1–T11 (full surface).
- **Near.**

#### 2.3.5 `sim_spawn_organism` — genome-bearing agent spawn (Near)

**Availability: Near.** Today this tool **degrades** to
`sim_spawn { kind: "civilian" }`. The full
`sim.spawn_organism { genome, cradle_state, age }` JSON-RPC
ships once `crates/engine/src/godtools.rs::apply_life` lands
(`GODTOOLS_IMPL_PLAN.md` P2.5).

```rust
pub struct SimSpawnOrganismArgs {
    /// Optional genome identifier (cluster or hash). Today: ignored
    /// (always spawns the default cluster).
    pub genome: Option<String>,
    /// Optional count (1 = single spawn, >1 = herd).
    pub count: Option<u32>,
    /// Footprint for herds. When `count > 1`, agents are jittered
    /// inside the rectangle.
    pub footprint: Option<Footprint>,
}
pub struct Footprint { pub x: f32, pub y: f32, pub width: f32, pub height: f32 }
```

- **Backing (today, degraded):** `sim_spawn { kind: "civilian" }`
  (× `count` for herds, jittered in the footprint).
- **Returns:** `{ "degraded": true, "spawned": u32, "kind": "civilian" }`.
- **God-verbs:** L1 (SpawnOrganism), L2 (SpawnHerd).
- **Near.**

#### 2.3.6 `sim_disaster` — invoke named disaster (Near)

**Availability: Near.** Today this tool **degrades** to a sequence
of `sim_damage` + `sim_sculpt` calls appropriate for the disaster
kind. The full `sim.disaster { kind, center, params }` JSON-RPC
ships once `crates/engine/src/godtools.rs::apply_disaster` +
`invoke_divine_disaster` (`GODTOOLS_IMPL_PLAN.md` P2.6) land.

```rust
pub struct SimDisasterArgs {
    /// Disaster kind. One of: meteor, lightning, flood, quake,
    /// firestorm, tornado, volcanic_vent, drought.
    pub kind: String,
    pub x: f32, pub y: f32,  // normalized center
    pub magnitude: f32,
    pub radius: Option<f32>,
    pub duration: Option<u32>,
}
```

- **Backing (today, degraded):**
  - `meteor` → `sim_damage { energy: 10*magnitude, radius: 16 }`
    + `sim_sculpt { material: LAVA }` at impact.
  - `lightning` → `sim_damage { energy: 2*magnitude, radius: 1 }`.
  - `flood` → `sim_sculpt { material: WATER }` × N at +radius.
  - `quake` → `sim_damage { energy: 4*magnitude, radius: 8 }`.
  - `firestorm` → `sim_sculpt { material: LAVA }` ring + repeated
    `sim_damage`.
  - `tornado` → no clean degradation; returns `best_effort: false`.
  - `volcanic_vent` → `sim_sculpt { material: LAVA }` sustained.
  - `drought` → no clean degradation; returns `best_effort: false`.
- **Returns:** `{ "degraded": true, "kind": String, "ops_applied":
  u32, "best_effort": bool }`.
- **God-verbs:** D1–D8.
- **Near.** Two of eight (tornado, drought) are best-effort only
  until the real RPC lands.

#### 2.3.7 `sim_law` — apply law / policy parameter (Near)

**Availability: Near.** Today this tool **degrades** to
`sim_set_policy` for `TaxBias` (the only law that has a live
JSON-RPC write). The full `sim.law { target_subsystem, value }`
JSON-RPC ships once `crates/engine/src/godtools.rs::apply_law`
lands (`GODTOOLS_IMPL_PLAN.md` P2.7).

```rust
pub struct SimLawArgs {
    /// Law id. One of: tax_bias, edict, religion_pressure, sanction,
    /// open_border, alignment_nudge, difficulty_knob, scenario_script.
    pub law: String,
    /// Law-specific parameter. For `tax_bias` this is
    /// `scarcity_multiplier`; for `edict` this is the edict id +
    /// boolean; for `difficulty_knob` this is the scalar id + value.
    pub params: serde_json::Value,
}
```

- **Backing (today, degraded):**
  - `tax_bias` → `sim_set_policy { scarcity_multiplier: params.scarcity_multiplier }`.
  - All others → `best_effort: false` until the real RPC lands.
- **Returns:** `{ "degraded": bool, "law": String, "applied": bool }`.
- **God-verbs:** LW1–LW8. Only LW1 reachable today.
- **Near.**

#### 2.3.8 `sim_undo` — undo last god-tool (Blind)

**Availability: Blind.** The MCP tool exists, returns a structured
`unavailable` response, and points the agent at the path that will
deliver it (`GODTOOLS_IMPL_PLAN.md` T3.9). AC-GT-8.

```rust
pub struct SimUndoArgs {
    /// Number of god-tool actions to undo. Default 1.
    pub count: Option<u32>,
}
```

- **Returns:** `{ "available": false, "planned_in": "GODTOOLS_IMPL_PLAN.md T3.9" }`.
- **God-verb:** TM8 (Undo — distinct from `time.profile` per
  `GODTOOLS_IMPL_PLAN.md:340`).
- **Blind.** Agents should check the `available` flag before
  scheduling workflows that depend on undo.

### 2.4 PERSISTENCE — save / load / replay (5 tools)

> **Charter coupling:** writes to the bridge `saves/` directory
> (slot-based) or the bridge replay base dir (`.civreplay`).
> The `sim.save_replay` / `sim.load_replay` paths are
> path-validated against the base dir per
> `crates/server/src/jsonrpc.rs:1350-1448` (no `..`, no
> absolute paths, canonicalize-and-contain check).

#### 2.4.1 `sim_save_slot` — write production slot

- **Backing:** `save.slot { slot_name: "slot-1".."slot-5" }`.
- **Returns:** `{ "saved": true, "slot_name": String }`.
- **God-verb:** (session control).
- **Live.**

#### 2.4.2 `sim_load_slot` — read production slot

- **Backing:** `save.load`.
- **Returns:** `{ "loaded": true, "slot_name": String, "tick": u64 }`.
- **God-verb:** (session control).
- **Live.**

#### 2.4.3 `sim_list_saves` — list bridge saves

- **Backing:** `save.list`.
- **Returns:** `Vec<SaveListEntry>` (slot files + their mtime /
  size, per `crates/server/src/saves.rs`).
- **God-verb:** (session control).
- **Live.**

#### 2.4.4 `sim_save_replay` — persist replay log

```rust
pub struct SimSaveReplayArgs {
    /// Path relative to the bridge replay base dir. Must be a
    /// plain relative path (no `..`, no absolute, no prefix).
    pub path: String,
}
```

- **Backing:** `sim.save_replay`.
- **Returns:** `{ "saved": true, "path": String }`.
- **God-verb:** (replay).
- **Live.** Path validation is enforced server-side (the MCP
  shim never touches the filesystem).

#### 2.4.5 `sim_load_replay` — restore from replay

```rust
pub struct SimLoadReplayArgs {
    pub path: String,
}
```

- **Backing:** `sim.load_replay`.
- **Returns:** `{ "loaded": true, "tick": u64 }`.
- **God-verb:** TM6 (Rewind — see caveat below).
- **Live (with soft-determinism caveat).** Per the emergence
  charter, the reloaded sim's *future* diverges from the original
  (real RNG re-rolls on forward continuation). The tool returns
  `soft_determinism: true` in the result so callers can
  document the contract.

### 2.5 SUBSCRIPTION — broadcast filter (3 tools)

> **Charter coupling:** the WS bridge holds a per-connection
> subscription filter (`sim.subscribe` / `sim.update_subscription`
> / `sim.unsubscribe`). These tools forward the same shape over
> the MCP transport. **Stateful** — the filter lives on the
> `WsBridge` connection, not in the sim. Today the MCP shim opens
> a fresh WS connection per call, so subscription state does not
> persist between MCP tool calls; this is documented as a known
> limitation in §4.

#### 2.5.1 `sim_subscribe` — opt into a tick broadcast filter

- **Backing:** `sim.subscribe`.
- **Returns:** `{ "subscribed": true }` on the underlying WS
  connection.
- **God-verb:** (broadcast control).
- **Live (per-connection).**

#### 2.5.2 `sim_unsubscribe` — clear filter

- **Backing:** `sim.unsubscribe`.
- **Returns:** `{ "unsubscribed": true }`.
- **God-verb:** (broadcast control).
- **Live (per-connection).**

#### 2.5.3 `sim_update_subscription` — replace filter

- **Backing:** `sim.update_subscription`.
- **Returns:** `{ "updated": true }`.
- **God-verb:** (broadcast control).
- **Live (per-connection).**

### 2.6 HEALTH (1 tool)

#### 2.6.1 `sim_health` — server liveness

- **Backing:** `health`.
- **Returns:** `{ "tick": u64 }` (the bridge's current tick).
- **God-verb:** (server liveness).
- **Live.** Cheaper than `sim_status`; the right tool for a
  keep-alive ping.

---

## 3. God-verb → tool mapping (canonical table)

The 50-verb god-tool surface from
`docs/design/GOD_TOOLS_SANDBOX.md:46-58` mapped to MCP tools in
this surface. **Bold** = tool is Live today. *Italic* = Near
(degrades). Plain = Blind (no MCP path yet).

| Verb | God-tool | MCP tool | Status |
|------|----------|----------|--------|
| **T1** | Raise | `sim_terraform_extent` (op=raise) | *Near* (degrades to `sim_sculpt`) |
| **T2** | Lower | `sim_terraform_extent` (op=lower) | *Near* |
| **T3** | Level | `sim_terraform_extent` (op=level) | *Near* |
| **T4** | Smooth | `sim_terraform_extent` (op=smooth) | *Near* |
| **T5** | Slope | `sim_terraform_extent` (op=slope) | *Near* |
| **T6** | Flatten | `sim_terraform_extent` (op=flatten) | *Near* |
| **T7** | Shift | `sim_terraform_extent` (op=shift) | *Near* |
| **T8** | AddLand | `sim_terraform_extent` (op=add_land) | *Near* |
| **T9** | DigOcean | `sim_terraform_extent` (op=dig_ocean) | *Near* |
| **T10** | RaiseMountain | `sim_terraform_extent` (op=raise_mountain) | *Near* |
| **T11** | DropBiome | `sim_terraform_extent` (op=drop_biome) | *Near* |
| **M1** | Replace | `sim_sculpt` (material) | **Live** |
| **M2** | AdditiveDrop | `sim_sculpt` (material) | **Live** |
| **M3** | Erase | `sim_sculpt` (material=AIR) | **Live** |
| **M4** | SurfacePaint | `sim_sculpt` (material, topmost) | **Live** (no topmost guard today) |
| **M5** | PourLiquid | `sim_sculpt` (material=WATER/LAVA) × N | **Live** (no rate/duration) |
| **M6** | SeedForest | `sim_spawn_organism` (count=N) | *Near* (degrades to `sim_spawn` civilian) |
| **M7** | SeedOreDeposit | `sim_sculpt` (material=ORE) | **Live** (no CA density) |
| **M8** | SeedSnow | `sim_sculpt` (material=ICE) | **Live** (no auto-snowline) |
| **L1** | SpawnOrganism | `sim_spawn_organism` (count=1) | *Near* (degrades to `sim_spawn`) |
| **L2** | SpawnHerd | `sim_spawn_organism` (count=N) | *Near* |
| **L3** | SpawnCivilizationSeed | `sim_spawn_organism` (count=6) + `sim_sculpt` (hut footprint) | *Near* |
| **L4** | Bless | — (no substrate write path; `apply_actor_effect` not on the wire) | **Deferred** |
| **L5** | Curse | — | **Deferred** |
| **L6** | Plague | — | **Deferred** |
| **L7** | Heal | — | **Deferred** |
| **L8** | Extinct | `sim_undo` (special) or `sim_disaster` (best-effort) | **Deferred** (Blind) |
| **D1** | Meteor | `sim_disaster` (kind=meteor) | *Near* |
| **D2** | Lightning | `sim_disaster` (kind=lightning) | *Near* |
| **D3** | Flood | `sim_disaster` (kind=flood) | *Near* (best-effort) |
| **D4** | Quake | `sim_disaster` (kind=quake) | *Near* |
| **D5** | Firestorm | `sim_disaster` (kind=firestorm) | *Near* |
| **D6** | Tornado | `sim_disaster` (kind=tornado) | *Near* (best_effort=false) |
| **D7** | VolcanicVent | `sim_disaster` (kind=volcanic_vent) | *Near* |
| **D8** | Drought | `sim_disaster` (kind=drought) | *Near* (best_effort=false) |
| **I1** | Probe | `sim_inspect` (tile_x, tile_y) / `sim_inspect_tile` | **Live** (stub body for `sim_inspect_tile`) |
| **I2** | Stats | `sim_inspect` / `sim_get_*` family | **Live** |
| **I3** | Trace | — (Legends saga graph; not yet on the wire) | **Deferred** |
| **I4** | Forecast | `sim_run_until` (predictive variant — out of scope v1) | **Deferred** |
| **I5** | CompareSnapshots | `sim_inspect` × 2 + agent-side diff | **Live** (compose on agent) |
| **I6** | History | `sim_load_replay` (read past snapshot) | **Live (soft-det caveat)** |
| **I7** | Bookmark | — (camera bookmark — UI, not exposed) | **Deferred** |
| **I8** | Follow | `sim_get_outcome` (read trailing state) | **Live** |
| **LW1** | TaxBias | `sim_law` (law=tax_bias) | *Near* (degrades to `sim_set_policy`) |
| **LW2** | Edict | `sim_law` (law=edict) | *Near* (best_effort=false) |
| **LW3** | ReligionPressure | `sim_law` (law=religion_pressure) | *Near* (best_effort=false) |
| **LW4** | Sanction | `sim_law` (law=sanction) | *Near* (best_effort=false) |
| **LW5** | OpenBorder | `sim_law` (law=open_border) | *Near* (best_effort=false) |
| **LW6** | AlignmentNudge | `sim_law` (law=alignment_nudge) | *Near* (best_effort=false) |
| **LW7** | DifficultyKnob | `sim_law` (law=difficulty_knob) | *Near* (best_effort=false) |
| **LW8** | ScenarioScript | `sim_load_scenario` | **Live** |
| **TM1** | Pause | `sim_set_speed` (multiplier=0) | **Live** |
| **TM2** | Play | `sim_set_speed` (multiplier=1) | **Live** |
| **TM3** | Slow | `sim_set_speed` (multiplier=2 or 4) | **Live** (no 0.25×; closest is 1) |
| **TM4** | Fast | `sim_set_speed` (multiplier=4 or 8) | **Live** |
| **TM5** | Step | `sim_step` (ticks=N) | **Live** |
| **TM6** | Rewind | `sim_load_replay` | **Live (soft-det)** |
| **TM7** | FastForwardToEvent | `sim_run_until` (target_outcome / target_tick) | **Live** |
| **TM8** | Profile | — (perf-trace log; not on the wire) | **Deferred** |
| **C1–C8** | Camera (Orbit/Pan/Zoom/Tilt/Roll/Bookmarks/FollowCam/PhotoMode) | — (UI-only; sim has no accessor per `GODTOOLS_IMPL_PLAN.md:325`) | **Out of scope** |

**Reachable today (Live + Near-degrade):** 30/42 mutating + 8/8
INSPECT = **38/50 verbs** (76%).

**Deferred (5 verbs):** L4 (Bless), L5 (Curse), L6 (Plague), L7
(Heal), L8 (Extinct) — the actor-effect path. All gate on
`crates/engine/src/godtools.rs::apply_life` landing
(`GODTOOLS_IMPL_PLAN.md` P2.5).

**Out of scope (8 verbs):** C1–C8 camera — by design, not a
substrate write. Agents that need camera control use the Bevy /
Godot / Unreal / web client directly.

---

## 4. Architectural decisions

### 4.1 Tool count: 32 (not 50, not 34)

The 50 god-verbs and the 34 JSON-RPC methods are *both* the wrong
granularity for an MCP surface:

- **Per-god-verb tools (50):** too granular. Multiple god-verbs
  share one underlying substrate write (T1 Raise and T2 Lower
  both go through `sim.place_voxel`); exposing them as separate
  tools bloats the agent's tool list.
- **Per-JSON-RPC tools (34):** too thin. The agent shouldn't
  have to know that `sim.spawn_entity` is the *real* path and
  `sim.spawn_civilian` is the *old* one; that `sim.emergence` and
  `emergence.dashboard` are two halves of one logical read.

The **32-tool** surface is the *agent* granularity: one tool per
*user intent*. The mapping table in §3 is the trail from
intent → god-verb → JSON-RPC.

### 4.2 Direct library calls, no cargo

The existing pattern (per `crates/civis-mcp/src/lib.rs:7-15`) is
"call `civis-cli` library functions directly, no `cargo run`
shell-out." The new tools follow the same rule:

- `civis-cli::pixels::{sample_rgb_grid, compute_pixel_stats}` for
  `civis_pixels`.
- `civis-cli::census::{build_sim_status_request, decode_response,
  validate_sim_status}` for `civis_census`.
- `civis-cli::verify::run_verify` for `civis_verify`.

The new tools **add two direct dependencies**:

- `civ_server::jsonrpc::parse_request` + a thin WS client for
  every `sim.*` method.
- `civ_server::ws_bridge::run_ws_bridge` (already exists; we
  reuse the same WS endpoint the existing `census` tool uses).

The MCP shim never spawns a child process. (The only place a
child process is spawned today is the `bevy` feature on
`civis_verify`, and that's an in-process renderer call, not a
shell-out.)

### 4.3 Subscription state: known limitation

The `sim_subscribe` / `sim_unsubscribe` / `sim_update_subscription`
tools are per-WS-connection stateful. The MCP shim currently
opens a fresh WS connection per `census_sim_status` call (see
`crates/civis-mcp/src/lib.rs:138-161`). This means:

- A `sim_subscribe` call today updates the filter on a connection
  that's immediately torn down. Useless.
- A future change to keep a long-lived WS connection in the shim
  makes the subscription tools useful. That change is **not** in
  scope for this spec; it's a follow-up tracked separately.

Agents that need broadcast filtering should connect to
`ws://127.0.0.1:3000/ws?tick_format=binary` directly and use
`sim.subscribe` over WS — the MCP layer doesn't add value here.
**The MCP subscription tools are kept for completeness** (the
JSON-RPC methods exist) but documented as limited.

### 4.4 Path validation: server-side only

`sim_save_replay` and `sim_load_replay` take a `path` argument.
The validation per `crates/server/src/jsonrpc.rs:1350-1448`:

- Reject absolute paths, paths with `..`, prefix components, or
  root components (`parse_replay_path`).
- Canonicalize-and-contain against the bridge base dir
  (`resolve_replay_path`).

The MCP tool **does not re-validate** — it forwards the path
as-is. The server enforces. This matches the existing pattern
(the same call from a web client is also un-validated on the
client side; the server is the security boundary).

### 4.5 Role gate: server-side, transparent to the agent

Mutating calls (`sim_spawn`, `sim_sculpt`, `sim_damage`, etc.)
are gated on `role: "operator"` when `require_role` is enabled
on the server (`crates/server/src/jsonrpc.rs:380-409`). The
MCP shim:

- Reads the role from `CIV_MCP_ROLE` env (default `"operator"`
  so the default install *just works*).
- Forwards the role in every mutating call's params.
- Returns the `FORBIDDEN` error code (32003) verbatim — agents
  see the same JSON-RPC error shape they'd see over a raw WS
  connection.

### 4.6 Tool descriptions: charter-aware

The `description` strings on each `#[tool]` method are the
**agent's only documentation source** for many LLM clients. The
descriptions will:

- Name the backing JSON-RPC method (for trace/debug).
- Name the god-verb family (for cross-referencing
  `GOD_TOOLS_SANDBOX.md`).
- Name any **degradation** (`degraded: true`, `best_effort:
  false`) so the agent knows the path is best-effort.

Example description for `sim_terraform_extent`:
> "Apply a footprint-aware brush to the world (T1–T11 in
> `docs/design/GOD_TOOLS_SANDBOX.md` — Raise / Lower / Level /
> Smooth / Slope / Flatten / Shift / AddLand / DigOcean /
> RaiseMountain / DropBiome). **Degrades** to repeated
> `sim.place_voxel` until `crates/voxel/src/brush.rs` lands.
> Returns `degraded: true` on the degraded path."

---

## 5. Acceptance criteria

### 5.1 MCP surface (this spec)

- **AC-MCP-1:** `crates/civis-mcp` ships **35 tools total** =
  3 existing harness (`civis_verify`, `civis_pixels`,
  `civis_census`) + 32 new control surface
  (`sim_inspect`..`sim_health`).
- **AC-MCP-2:** Every new tool's `description` documents the
  backing JSON-RPC method and the god-verb family. Test: parse
  all 32 `#[tool]` descriptions; assert each contains
  `crates/server/src/jsonrpc.rs` or the verb-family name from
  `GOD_TOOLS_SANDBOX.md`.
- **AC-MCP-3:** Every new tool has a typed `JsonSchema` parameter
  struct (no `serde_json::Value` directly on the args, except
  for `sim_law.params` which is intentionally open-ended).
- **AC-MCP-4:** The shim never spawns a child process for the
  new tools (cargo / bash / external bins). Test: a `cfg(test)`
  guard asserts no `std::process::Command` calls exist in the
  new tool code paths.
- **AC-MCP-5:** A `sim_run_until` server-side loop never holds
  the sim handle — every iteration is a separate WS round-trip.
  Test: integration test with a mock WS server counts calls.
- **AC-MCP-6:** `sim_save_replay` and `sim_load_replay` forward
  paths verbatim; the server is the security boundary. Test:
  the MCP tool's arg-parsing code does not call
  `parse_replay_path` or `resolve_replay_path`.

### 5.2 Coverage (vs the 50-verb spec)

- **AC-MCP-COV-1:** All 8 INSPECT verbs (I1–I8) reachable
  through MCP tools. Test: a `god_verb_coverage` test enumerates
  the verb names and asserts each has a tool mapping in the
  §3 table.
- **AC-MCP-COV-2:** At least 30/42 mutating verbs reachable
  through MCP tools. Test: same test as above, filtered to
  mutating.
- **AC-MCP-COV-3:** The 8 deferred verbs (L4–L8) are listed as
  Deferred in the table; the test asserts they're explicitly
  marked so future PRs can flip the flags.

### 5.3 Integration with existing infra

- **AC-MCP-INT-1:** The new tools do not break the existing
  3 harness tools. Test: the existing `mcp_integration.rs`
  tests still pass.
- **AC-MCP-INT-2:** The new tools use the same `civis-cli` config
  surface (`CIV_WS_HOST`, `CIV_SERVER_PORT`, `CIV_CENSUS_TIMEOUT_MS`,
  `CIV_MCP_ROLE`) — no new env vars without documenting them.
- **AC-MCP-INT-3:** The new tools respect the JSON-RPC catalog
  drift check (`just civis-3d-catalog-check`). When a method
  is renamed on the server, the tool is renamed to match.
  Test: a CI guard parses the new tool names and asserts each
  is a substring of a method in
  `crates/server/src/jsonrpc.rs::JsonRpcMethod::as_str()`.

---

## 6. Out of scope (explicit non-goals)

- **Camera (C1–C8).** Universal UI verb, not a substrate write.
  Agents that need camera control use the Bevy / Godot / Unreal /
  web client directly. Per `GODTOOLS_IMPL_PLAN.md:325`, "sim has
  no accessor for camera transform."
- **Long-lived WS connection in the shim.** Needed for
  subscription tools to be useful, but not in this spec.
- **A direct `cargo`-free path to Bevy/Godot/Unreal.** The
  existing `civis_verify` tool already covers Bevy-frame
  capture; mirroring that to Godot / Unreal is per-client work,
  not an MCP-tool concern.
- **The 12 Deferred verbs (L4–L8, I3 Trace, I4 Forecast, I7
  Bookmark, TM8 Profile, LW2–LW7).** Each is gated on a
  substrate-side PR. The MCP tool can land before the substrate
  write is real (Near-degrade pattern), but the agent should
  not assume the verb is fully functional.

---

## 7. Open questions / follow-ups

1. **Should `sim_terraform_extent` block until the server-side
   path lands, or ship the degraded version now?** This spec
   says **ship degraded** with an explicit `degraded: true`
   return flag. Pro: agents can start using T1–T11 immediately.
   Con: agents may depend on a `degraded: true` result without
   noticing. The fix: a `civis-mcp` CLI flag
   (`CIV_MCP_STRICT=1`) that hides degraded tools.

2. **Should `sim_spawn_organism`'s `genome` arg be a `GenomeHash`
   or a free-form string?** Today the substrate has no
   `genome` API on the wire. Once `crates/agents::spawn_*` lands
   a typed-genome entry point (`GODTOOLS_IMPL_PLAN.md` P2.5),
   we re-spec this arg. For now, it's a string the shim
   forwards as a comment in the request — the server ignores it.

3. **`sim_disaster` for `tornado` and `drought`.** No clean
   degradation path — both touch `WeatherCell` and the
   `crates/planet/src/weather.rs` wind field, which the
   current JSON-RPC surface can't write. Options:
   (a) return `best_effort: false` and document the gap
   (this spec's choice); (b) skip the tools entirely until
   the real RPC lands. (a) is preferred because it lets agents
   *plan* around the gap rather than discover it at runtime.

4. **Subscription tools (3.5.x).** The per-WS-connection state
   issue is a real limitation. A follow-up spec
   (`docs/design/MCP_SUBSCRIPTION_PLAN.md`, TBD) should
   document the long-lived-WS shim design and how it composes
   with the existing `census` and `verify` flows.

5. **Role-gate UX.** Today `CIV_MCP_ROLE=operator` is the
   default. That's *too permissive* for a real agent runtime
   (an agent that can spawn + sculpt + damage is a powerful
   peer). A follow-up should split into `CIV_MCP_READ_ROLE`
   (default `observer`) and `CIV_MCP_WRITE_ROLE` (default
   `none`; the agent must opt in).

---

## 8. Verdict

The MCP control surface is **32 tools** (3 existing harness
tools kept + 29 new) — a remote control for the godgame that
maps cleanly onto the 50-verb god-tools spec and the 34-method
JSON-RPC surface. **30/42 mutating god-verbs + 8/8 INSPECT
verbs = 38/50 verbs (76%) reachable today** through MCP, with
the remaining 12 (L4–L8 actor effects, I3/I4/I7 reads, TM8
profile, LW2–LW7 laws) gated on substrate-side PRs in
`GODTOOLS_IMPL_PLAN.md` P2-P3. The architecture is **direct
library calls, no `cargo` shell-out** (matching the existing
`civis-mcp` pattern), and the security boundary is the server
(`role: "operator"` + replay-path canonicalization), not the
shim. Tools degrade gracefully when their backing RPC doesn't
exist yet (Near) or are explicitly marked Blind (`sim_undo`),
so agents can plan workflows that survive the substrate rollout.
