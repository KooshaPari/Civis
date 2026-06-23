# CIV Engineering Plan: Phases 0-6 with DAG Dependencies

**Project:** Civis (headless Rust civilization simulator + 3D substrate)
**Workspace:** 17 members in root `Cargo.toml` — 16 under `crates/` plus `clients/bevy-ref` (no `crates/climate`, `crates/actors`, `crates/social`, `crates/metrics`, `crates/policy`, `crates/geo`, `crates/spatial`)
**Methodology:** Test-first (TDD), spec-driven, determinism-first, L3 copilot agent workers
**Status key:** ✅ landed · 🔶 partial/stub · ❌ not started (per plan)
**Authoritative map:** `docs/IMPLEMENTATION_STATUS.md` · verify crates: `cargo metadata --no-deps --format-version 1`

---

## Current vs planned (workspace)

| Area | Planned (legacy PLAN / specs) | Actual (`Cargo.toml` + tree) | Status |
|------|-----------------------------|------------------------------|--------|
| Workspace size | ~8 domain crates | 17 members (`Cargo.toml`): 16× `crates/*` + `clients/bevy-ref` | ✅ layout |
| Core tick / RNG | `crates/engine` + FR-CIV-0001 tests | `civ-engine`: `Simulation::tick()`, `ChaCha8Rng`, inline `#[cfg(test)]` in `engine.rs` / `lib.rs` | ✅ |
| Replay / determinism | `tests/fr_determinism_replay.rs` | `ReplayLog`, `.civreplay` (`replay.rs`, `replay_format.rs`); determinism tests in `engine.rs` | 🔶 |
| Climate | `crates/climate` | `civ-planet` orbital climate; **no** `crates/climate` (CIV-0102 CO₂ model) | 🔶 |
| Economy | `crates/economy` market + joule | `civ-economy` **partial** (`EconomyState`, `drain_energy_budget`, `step`, `MarketState::step`); engine `phase_economy` syncs budget + market | 🔶 |
| Actors / social | `crates/actors`, `crates/social` | `civ-agents` + citizen ECS in `civ-engine`; **no** `actors`/`social` crates | 🔶 |
| Policy / diplomacy | `crates/policy` | `civ-laws` RON stubs; diplomacy not split out | ❌ |
| Metrics / research export | `crates/metrics` | `civ-engine` metrics + `civ-research` stubs; no `metrics` crate | 🔶 |
| Server / protocol | WebSocket + JSON tick stream | `civ-server` JSON-RPC + `ws_bridge`; `civ-protocol-3d`; `server/tests/ws_smoke.rs` | 🔶 |
| Phase 0 (foundation) | M0 complete | Engine tests green; replay beyond original FR harness | 🔶 |
| Phase 1 (economy) | M1 market + joule | Ledger + market stepped in engine; full CIV-0100 institution layer not implemented | 🔶 |

---

## Phase Diagram & Critical Path

```
Phase 0: Foundation (Core Tick Loop)                    [🔶 partial]
  └─> Phase 1: Economy Layer                          [🔶 partial]
        ├─> Phase 2: Actor + Social                   [❌ planned crates missing]
        │     └─> Phase 4: War + Diplomacy            [❌]
        └─> Phase 3: Client Protocol  (parallel)      [🔶 JSON-RPC / WS smoke]
              ├─> Phase 4: War + Diplomacy            [❌]
              └─> Phase 5: Research API               [🔶 scenario + replay landed in engine]
                    └─> Phase 6: Polish + Hardening   [❌]
```

**Critical Path:** Phase 0 → 1 → 2 → 4 → 5 → 6 (unchanged intent)
**Parallel Tracks:** Phase 3 can run alongside Phases 1-2
**Blocker Dependencies:** See "Depends On" column in each phase table below.

---

## Phase 0: Foundation — Core Tick Loop & Determinism Tests

**Goal:** Deterministic simulation core passing all foundational tests (M0)
**Duration:** 3-5 days (original estimate; core landed, replay harness differs from plan)
**Status:** 🔶 Partial — tick loop, seeded RNG, and replay log exist; separate `crates/engine/tests/fr_*` harness not used
**Success Metric:** `cargo test -p civ-engine` passes (determinism + replay tests in `crates/engine/src/engine.rs`, `replay_format.rs`, `lib.rs`)

### Phase 0 Worktree Setup

```bash
git worktree add ../civ-wt-phase0-foundation main
cd ../civ-wt-phase0-foundation
```

### Phase 0 — actual code map

| Concern | Planned path | Actual path |
|---------|--------------|-------------|
| Tick loop | `crates/engine/src/simulation.rs` | `crates/engine/src/engine.rs` — `Simulation::tick()` |
| Determinism tests | `crates/engine/tests/fr_*.rs` | `crates/engine/tests/determinism_proptest.rs` + inline `#[cfg(test)]` in `engine.rs`, `lib.rs`, `replay_format.rs` |
| Replay | `record_state` / `verify_replay` | `crates/engine/src/replay.rs`, `replay_format.rs` — `ReplayLog`, `.civreplay` save/load |
| RNG | `Simulation::rng_seed()` contract tests | `SimRng = ChaCha8Rng` in `engine.rs` / `lib.rs` |
| Invariants | — (not in original P0 table) | `crates/engine/src/invariants.rs` — tick/replay alignment, energy ≥ 0 |
| Scenario loader | Phase 5 | **Landed early:** `crates/engine/src/scenario.rs`, `scenarios/baseline.yaml` |

Legacy `fr_*.rs` harness files are not used; extend `determinism_proptest.rs` or module tests instead of adding `fr_core_tick_loop.rs` unless deliberately splitting layout.

### Phase 0 Tasks

| Task ID | Description | Depends On | Status | Acceptance criteria (actual) |
|---------|-------------|------------|--------|------------------------------|
| P0.1 | Core tick loop `Simulation::tick(&mut self)` | — | ✅ | `cargo test -p civ-engine` — tick advances `state.tick` |
| P0.2 | Tick phases (voxel, planet, economy sync, replay record) | P0.1 | ✅ | `determinism_holds_with_all_phases_enabled`, `tick_invariants_hold_across_many_ticks` |
| P0.3 | Determinism replay harness | P0.2 | 🔶 | Tests use `ReplayLog` / voxel replay, not standalone `fr_determinism_replay.rs` |
| P0.4 | Replay implementation | P0.3 | 🔶 | `replay_reproduces_final_voxel_chunk_count_and_tick`, `civreplay_save_load_restores_tick_after_ticks` |
| P0.5 | RNG seeding contract | P0.4 | ✅ | `test_determinism`, `determinism_same_seed_same_output` in `lib.rs` |
| P0.6 | Seeded RNG across dependent crates | P0.5 | 🔶 | Engine + `civ-agents` use `ChaCha8Rng`; audit remaining crates as they grow |

---

## Phase 1: Economy Layer — Market + Joule Allocators

**Goal:** Full economy system per CIV-0100 / CIV-0107 (M1) — ledger, markets, allocation
**Duration:** 5-7 days (not started beyond stub)
**Depends On:** Phase 0 core tick loop (🔶 sufficient to proceed)
**Status:** 🔶 Partial — ledger drain/step + deterministic `MarketState::step` wired in `phase_economy`; full allocation / institution layer not started
**Success Metric:** `cargo test -p civ-economy` covers market + joule + proptest invariants; `phase_economy` delegates to `civ_economy::step` with real state mutation ✅

### Phase 1 Worktree Setup

```bash
git worktree add ../civ-wt-phase1-economy main
cd ../civ-wt-phase1-economy
```

### Phase 1 — actual code map

| Concern | Planned path | Actual path |
|---------|--------------|-------------|
| Economy crate | `crates/economy` with `market.rs`, `joule.rs` | `lib.rs` + `market.rs` — `EconomyState`, `drain_energy_budget`, `step`, `MarketState::step` |
| Economy tests | `crates/economy/tests/fr_econ_*.rs` | `#[cfg(test)]` in `lib.rs` + `market.rs` (ledger + market + proptest) |
| Engine integration | `economy::tick()` from `Simulation::tick` | `engine.rs::phase_economy` — syncs `WorldState::energy_budget_joules` ↔ `EconomyState`, policy drain via `effective_consumption` |
| Spec reference | FR-CIV-ECON-* | `docs/specs/CIV-0100-economy-v1.md` |

No `crates/climate` or separate `crates/metrics` — climate inputs come from `civ-planet` inside engine phases.

### Phase 1 Tasks

| Task ID | Description | Depends On | Status | Acceptance criteria (target) |
|---------|-------------|------------|--------|------------------------------|
| P1.1 | Market struct and price tracking | P0.6 | ✅ | `MarketState` in `market.rs`; unit tests for per-tick price updates |
| P1.2 | Market implementation | P1.1 | ✅ | `cargo test -p civ-economy` — deterministic price update invariants + edge cases |
| P1.3 | Joule allocator harness | P1.2 | 🔶 | `drain_energy_budget` + `step` exist; full `JouleAllocator` / actor splits ❌ |
| P1.4 | Joule allocator implementation | P1.3 | ❌ | Conservation: allocated joules ≤ budget |
| P1.5 | Property-based economy tests | P1.4 | ✅ | `proptest` in `market.rs` (determinism + positive prices) |
| P1.6 | Engine integration | P1.5 | ✅ | `phase_economy_*` tests — budget sync, `civ_economy::step`, `MarketState::step` |

---

## Phase 2: Actor + Social — Citizen Lifecycle & Institutions

**Goal:** Full actor + social systems passing all FR-CIV-ACTOR and FR-CIV-SOCIAL tests (M2)
**Duration:** 6-8 days
**Depends On:** Phase 1 (economy layer must be stable)
**Success Metric:** All `FR-CIV-ACTOR-*` and `FR-CIV-SOCIAL-*` tests pass with 85%+ coverage

### Phase 2 Worktree Setup

```bash
git worktree add ../civ-wt-phase2-actors main
cd ../civ-wt-phase2-actors
```

### Phase 2 Tasks

| Task ID | Description | Depends On | Owner | L3 Copilot Dispatch | Acceptance Criteria |
|---------|-------------|-----------|-------|-------------------|---------------------|
| P2.1 | Citizen lifecycle state machine | P1.6 | copilot-L3-2a | `copilot -p "Implement FR-CIV-ACTOR-001-LIFECYCLE: Citizen lifecycle. Write failing test in crates/actors/tests/fr_citizen_lifecycle.rs. Test must verify state transitions: Born -> Employed -> Retired -> Dead. Define Citizen struct with age, status, health. Commit: 'test(actors): FR-CIV-ACTOR-001 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P2.2 | Citizen lifecycle implementation | P2.1 | copilot-L3-2b | `copilot -p "Implement FR-CIV-ACTOR-001-LIFECYCLE: Citizen lifecycle in crates/actors/src/citizen.rs. Implement Citizen struct with state transitions in tick(). Each tick: age++, check employment, check mortality. Commit: 'feat(actors): FR-CIV-ACTOR-001 lifecycle'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass |
| P2.3 | Institution system harness | P2.2 | copilot-L3-2c | `copilot -p "Implement FR-CIV-SOCIAL-001-INSTITUTIONS: Institution system. Write failing test in crates/social/tests/fr_institutions.rs. Test must verify Institution can hold policies and citizens. Define Institution struct. Commit: 'test(social): FR-CIV-SOCIAL-001 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P2.4 | Institution implementation | P2.3 | copilot-L3-2d | `copilot -p "Implement FR-CIV-SOCIAL-001-INSTITUTIONS: Institution struct in crates/social/src/institution.rs. Implement Institution { policies, members, budget, approval_rating }. Add methods: add_member(), remove_member(), update_policy(). Commit: 'feat(social): FR-CIV-SOCIAL-001 institutions'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass |
| P2.5 | Social ideologies & preferences | P2.4 | copilot-L3-2e | `copilot -p "Implement FR-CIV-SOCIAL-002-IDEOLOGY: Ideologies. Write failing test in crates/social/tests/fr_ideology.rs. Test must verify Citizen { ideology: f32 } ranges [-1.0, 1.0] from lib to auth. Commit: 'test(social): FR-CIV-SOCIAL-002 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P2.6 | Ideology implementation | P2.5 | copilot-L3-2f | `copilot -p "Implement FR-CIV-SOCIAL-002-IDEOLOGY: Ideology in crates/social/src/ideology.rs. Add ideology field to Citizen, implement ideology_shift() based on institution policy drift. Commit: 'feat(social): FR-CIV-SOCIAL-002 ideology'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass |
| P2.7 | Time-series metrics | P2.6 | copilot-L3-2g | `copilot -p "Implement FR-CIV-METRICS-001-TIMESERIES: Time-series metrics. Write failing test in crates/metrics/tests/fr_timeseries.rs. Test must verify TimeSeries { data: Vec<(tick, value)> } and query_range(). Commit: 'test(metrics): FR-CIV-METRICS-001 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P2.8 | Time-series implementation | P2.7 | copilot-L3-2h | `copilot -p "Implement FR-CIV-METRICS-001-TIMESERIES: TimeSeries in crates/metrics/src/timeseries.rs. Implement TimeSeries struct with append(), query_range(), downsample(). Commit: 'feat(metrics): FR-CIV-METRICS-001 timeseries'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass |

---

## Phase 3: Client Protocol — WebSocket Server + Bevy Reference Client

**Goal:** First WebSocket client-server connection passing integration tests (M3)
**Duration:** 5-7 days
**Depends On:** Phase 0 (core tick loop), can run **parallel** to Phase 1-2
**Success Metric:** WebSocket server starts, accepts client, sends tick updates, integrates with engine

### Phase 3 Worktree Setup

```bash
git worktree add ../civ-wt-phase3-client main
cd ../civ-wt-phase3-client
```

### Phase 3 Tasks

| Task ID | Description | Depends On | Owner | L3 Copilot Dispatch | Acceptance Criteria |
|---------|-------------|-----------|-------|-------------------|---------------------|
| P3.1 | WebSocket server struct | P0.6 | copilot-L3-3a | `copilot -p "Implement FR-CIV-SERVER-001-WS: WebSocket server. Write failing test in crates/server/tests/fr_websocket_server.rs. Test must create SimServer, bind to 0.0.0.0:8080, accept one connection. Commit: 'test(server): FR-CIV-SERVER-001 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P3.2 | WebSocket server implementation | P3.1 | copilot-L3-3b | `copilot -p "Implement FR-CIV-SERVER-001-WS: WebSocket server in crates/server/src/websocket.rs. Use tokio-tungstenite. Implement SimServer { listener: TcpListener } and accept_client() loop. Commit: 'feat(server): FR-CIV-SERVER-001 websocket'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass, clippy clean |
| P3.3 | Protocol message encoding (JSON) | P3.2 | copilot-L3-3c | `copilot -p "Implement FR-CIV-SERVER-002-PROTO: Protocol messages. Write failing test in crates/server/tests/fr_protocol_messages.rs. Define ClientMessage { command: String, payload: serde_json::Value } and ServerMessage { tick: u64, state: SimSnapshot }. Commit: 'test(server): FR-CIV-SERVER-002 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P3.4 | Protocol implementation | P3.3 | copilot-L3-3d | `copilot -p "Implement FR-CIV-SERVER-002-PROTO: Protocol in crates/server/src/protocol.rs. Implement serde-based JSON encoding for ClientMessage and ServerMessage. Commit: 'feat(server): FR-CIV-SERVER-002 protocol'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass |
| P3.5 | Engine-server integration | P3.4 | copilot-L3-3e | `copilot -p "Integrate server with engine. Write failing test in crates/server/tests/fr_server_engine.rs. Test must: (1) create Simulation, (2) create SimServer, (3) call server.tick_and_broadcast(). Commit: 'test(server): FR-CIV-SERVER-ENGINE failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P3.6 | Server-engine tick loop | P3.5 | copilot-L3-3f | `copilot -p "Implement FR-CIV-SERVER-ENGINE: Server ticks engine and broadcasts. In crates/server/src/main.rs, implement: let mut sim = Simulation::new(); loop { sim.tick(); server.broadcast(sim.snapshot()); }. Commit: 'feat(server): FR-CIV-SERVER-ENGINE integration'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Integration test passes |
| P3.7 | Bevy reference client stub | P3.6 | copilot-L3-3g | `copilot -p "Stub Bevy reference client. Create web/bevy-client/src/main.rs. Write failing test in web/bevy-client/tests/integration_test.rs that verifies client connects to server and receives one tick update. Use tokio-tungstenite as client. Commit: 'test(web): bevy-client stub'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P3.8 | Client-server roundtrip test | P3.7 | copilot-L3-3h | `copilot -p "Implement FR-CIV-CLIENT-ROUNDTRIP: Full roundtrip test. In web/bevy-client/tests/integration_test.rs: spawn SimServer in background, create client, send 'tick' command, receive ServerMessage with tick number. Verify state mutates. Commit: 'feat(web): bevy-client roundtrip'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Roundtrip test passes |

---

## Phase 4: War + Diplomacy — Military & Political Systems

**Goal:** Military, diplomacy, and shadow networks passing all FR-CIV-WAR and FR-CIV-DIPLO tests (M4)
**Duration:** 7-9 days
**Depends On:** Phase 1 (economy stable), Phase 2 (actors + institutions)
**Success Metric:** All `FR-CIV-WAR-*` and `FR-CIV-DIPLO-*` tests pass with 85%+ coverage

### Phase 4 Worktree Setup

```bash
git worktree add ../civ-wt-phase4-war main
cd ../civ-wt-phase4-war
```

### Phase 4 Tasks

| Task ID | Description | Depends On | Owner | L3 Copilot Dispatch | Acceptance Criteria |
|---------|-------------|-----------|-------|-------------------|---------------------|
| P4.1 | Military unit struct | P2.8 | copilot-L3-4a | `copilot -p "Implement FR-CIV-WAR-001-UNITS: Military units. Write failing test in crates/actors/tests/fr_military_units.rs. Define MilitaryUnit { unit_type, strength, position, faction } and verify combat_strength() calculation. Commit: 'test(actors): FR-CIV-WAR-001 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P4.2 | Military units implementation | P4.1 | copilot-L3-4b | `copilot -p "Implement FR-CIV-WAR-001-UNITS: Military units in crates/actors/src/military.rs. Implement MilitaryUnit { strength, morale, fatigue } and combat_strength() = strength * morale / (1 + fatigue). Commit: 'feat(actors): FR-CIV-WAR-001 units'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass |
| P4.3 | Combat resolution harness | P4.2 | copilot-L3-4c | `copilot -p "Implement FR-CIV-WAR-002-COMBAT: Combat resolution. Write failing test in crates/actors/tests/fr_combat.rs. Test must define resolve_combat(attacker: &MilitaryUnit, defender: &MilitaryUnit) -> BattleResult. Commit: 'test(actors): FR-CIV-WAR-002 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P4.4 | Combat implementation | P4.3 | copilot-L3-4d | `copilot -p "Implement FR-CIV-WAR-002-COMBAT: Combat in crates/actors/src/combat.rs. Implement resolve_combat() with deterministic outcome based on strength, terrain, rng_seed. Return BattleResult { victor, casualties, morale_loss }. Commit: 'feat(actors): FR-CIV-WAR-002 combat'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass, deterministic with replay |
| P4.5 | Diplomacy relation tracking | P4.4 | copilot-L3-4e | `copilot -p "Implement FR-CIV-DIPLO-001-RELATIONS: Diplomacy relations. Write failing test in crates/policy/tests/fr_diplomacy_relations.rs. Define DiplomaticRelation { faction_a, faction_b, sentiment: f32, pact_type } with sentiment in [-1.0, 1.0]. Commit: 'test(policy): FR-CIV-DIPLO-001 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P4.6 | Diplomacy implementation | P4.5 | copilot-L3-4f | `copilot -p "Implement FR-CIV-DIPLO-001-RELATIONS: Diplomacy in crates/policy/src/diplomacy.rs. Implement DiplomaticRelation struct with update_sentiment(delta) clamped to [-1, 1]. Implement treaty negotiation. Commit: 'feat(policy): FR-CIV-DIPLO-001 diplomacy'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass |
| P4.7 | Shadow networks harness | P4.6 | copilot-L3-4g | `copilot -p "Implement FR-CIV-DIPLO-002-SHADOW: Shadow networks. Write failing test in crates/policy/tests/fr_shadow_networks.rs. Define ShadowNetwork { members, influence, covert_actions }. Test verify_undetected() must track detection risk. Commit: 'test(policy): FR-CIV-DIPLO-002 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P4.8 | Shadow networks implementation | P4.7 | copilot-L3-4h | `copilot -p "Implement FR-CIV-DIPLO-002-SHADOW: Shadow networks in crates/policy/src/shadow_networks.rs. Implement ShadowNetwork { members, influence, detection_risk } and execute_action() which mutates detection_risk. Commit: 'feat(policy): FR-CIV-DIPLO-002 shadow networks'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass |

---

## Phase 5: Research API — Scenario Runner & Metrics Export

**Goal:** Full scenario runner and metrics export passing all FR-CIV-RESEARCH tests (M5)
**Duration:** 5-7 days
**Depends On:** Phase 4 (war+diplomacy stable), Phase 3 (server stable)
**Success Metric:** All `FR-CIV-RESEARCH-*` and `FR-CIV-METRICS-*` tests pass, scenario YAML format defined

### Phase 5 Worktree Setup

```bash
git worktree add ../civ-wt-phase5-research main
cd ../civ-wt-phase5-research
```

### Phase 5 Tasks

| Task ID | Description | Depends On | Owner | L3 Copilot Dispatch | Acceptance Criteria |
|---------|-------------|-----------|-------|-------------------|---------------------|
| P5.1 | Scenario YAML format spec | P4.8 | copilot-L3-5a | `copilot -p "Define scenario YAML format. Create docs/SCENARIO_FORMAT.md. Spec must define: initial_citizens, institutions, terrain_map, initial_policies, simulation_params. Include 3 example scenarios. Commit: 'docs: scenario YAML format spec'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Spec doc exists with 3 examples |
| P5.2 | Scenario loader harness | P5.1 | copilot-L3-5b | `copilot -p "Implement FR-CIV-RESEARCH-001-SCENARIO: Scenario loader. Write failing test in crates/engine/tests/fr_scenario_loader.rs. Test must load YAML from docs/scenarios/test.yaml and create Simulation. Commit: 'test(engine): FR-CIV-RESEARCH-001 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P5.3 | Scenario loader implementation | P5.2 | copilot-L3-5c | `copilot -p "Implement FR-CIV-RESEARCH-001-SCENARIO: Scenario loader in crates/engine/src/scenario.rs. Use serde_yaml to deserialize, then populate Simulation with initial state. Commit: 'feat(engine): FR-CIV-RESEARCH-001 scenario loader'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass |
| P5.4 | Metrics snapshot harness | P5.3 | copilot-L3-5d | `copilot -p "Implement FR-CIV-RESEARCH-002-SNAPSHOT: Metrics snapshot. Write failing test in crates/metrics/tests/fr_snapshot.rs. Test must verify Simulation::snapshot() returns MetricsSnapshot { tick, gdp, population, avg_ideology, ... }. Commit: 'test(metrics): FR-CIV-RESEARCH-002 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P5.5 | Metrics snapshot implementation | P5.4 | copilot-L3-5e | `copilot -p "Implement FR-CIV-RESEARCH-002-SNAPSHOT: Metrics snapshot in crates/metrics/src/snapshot.rs. Implement MetricsSnapshot { tick, population, gdp, avg_ideology, health_index, conflict_score }. Add Simulation::snapshot() method. Commit: 'feat(metrics): FR-CIV-RESEARCH-002 snapshot'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass |
| P5.6 | CSV export harness | P5.5 | copilot-L3-5f | `copilot -p "Implement FR-CIV-RESEARCH-003-EXPORT: CSV export. Write failing test in crates/metrics/tests/fr_export_csv.rs. Test must run 100-tick scenario and export to CSV with headers: tick, population, gdp, avg_ideology, health_index. Commit: 'test(metrics): FR-CIV-RESEARCH-003 failing test'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Test file exists, test fails |
| P5.7 | CSV export implementation | P5.6 | copilot-L3-5g | `copilot -p "Implement FR-CIV-RESEARCH-003-EXPORT: CSV export in crates/metrics/src/export.rs. Use csv crate to write MetricsSnapshot records. Implement export_csv(path: &str, snapshots: &[MetricsSnapshot]). Commit: 'feat(metrics): FR-CIV-RESEARCH-003 csv export'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All tests pass, CSV valid |
| P5.8 | Replay format spec & validation | P5.7 | copilot-L3-5h | `copilot -p "Implement FR-CIV-RESEARCH-004-REPLAY: Replay format. Create docs/REPLAY_FORMAT.md and implement crates/engine/src/replay.rs. Spec: SimulationReplay { seed, tick_count, events: Vec<(tick, event_type, data)> }. Commit: 'feat(engine): FR-CIV-RESEARCH-004 replay format'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Spec doc + implementation, tests pass |

---

## Phase 6: Polish + Hardening — Coverage, Performance, Documentation

**Goal:** 100% engine coverage, 80%+ overall coverage, benchmarks, full docs (M6)
**Duration:** 7-10 days
**Depends On:** Phase 5 (all systems complete), Phase 3 (server stable)
**Success Metric:** `cargo test` 100%, `cargo tarpaulin --out Html` shows 100% engine, 80% overall, benchmarks published

### Phase 6 Worktree Setup

```bash
git worktree add ../civ-wt-phase6-polish main
cd ../civ-wt-phase6-polish
```

### Phase 6 Tasks

| Task ID | Description | Depends On | Owner | L3 Copilot Dispatch | Acceptance Criteria |
|---------|-------------|-----------|-------|-------------------|---------------------|
| P6.1 | Coverage audit | P5.8 | copilot-L3-6a | `copilot -p "Audit coverage. Run cargo tarpaulin --out Html --exclude-files tests/ and identify all uncovered lines in engine/ (target: 100%), economy/, actors/, social/ (target: 80%). Create docs/COVERAGE_GAPS.md listing all uncovered functions. Commit: 'docs: coverage audit baseline'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Coverage report written, gaps documented |
| P6.2 | Fill engine coverage gaps | P6.1 | copilot-L3-6b | `copilot -p "Fill engine coverage gaps. For each uncovered line in crates/engine/src/, write a test in crates/engine/tests/fr_coverage_*.rs. Each test must exercise the uncovered code path. Target: 100% coverage. Commit: 'test(engine): comprehensive coverage tests'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Tarpaulin shows 100% engine coverage |
| P6.3 | Fill other crate coverage gaps | P6.2 | copilot-L3-6c | `copilot -p "Fill other crate coverage gaps. For economy/, actors/, social/, policy/ crates, write tests to reach 80% coverage on each. Commit: 'test: comprehensive coverage for other crates'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Tarpaulin shows 80%+ on non-engine crates |
| P6.4 | Edge case testing | P6.3 | copilot-L3-6d | `copilot -p "Edge case testing. Write edge-case tests in crates/*/tests/fr_edge_cases_*.rs: (1) zero agents (2) negative balances clamped (3) market with single good (4) institution with 0 members (5) 1000-tick determinism replay. Commit: 'test: edge cases comprehensive'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All edge case tests pass |
| P6.5 | Performance benchmarks | P6.4 | copilot-L3-6e | `copilot -p "Performance benchmarks. Create benches/sim_benchmarks.rs with criterion benchmarks for: (1) sim.tick() N=1000 (2) market.price_update() N=10000 (3) citizen.lifecycle_tick() N=100000. Run: cargo bench --bench sim_benchmarks. Document results in docs/PERFORMANCE_BASELINE.md. Commit: 'perf: benchmark baseline'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Benchmarks run, results documented |
| P6.6 | API documentation | P6.5 | copilot-L3-6f | `copilot -p "API documentation. Add comprehensive /// rustdoc comments to all pub items in crates/engine/src/lib.rs, crates/economy/src/lib.rs, etc. Run: cargo doc --no-deps --open. All modules must have module-level docs. Commit: 'docs: comprehensive rustdoc'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Rustdoc builds cleanly, all pub items documented |
| P6.7 | Integration guide | P6.6 | copilot-L3-6g | `copilot -p "Write integration guide. Create docs/guides/INTEGRATION_GUIDE.md with: (1) quick start (2) scenario YAML examples (3) metrics export walkthrough (4) determinism replay validation (5) WebSocket client example. Include code snippets. Commit: 'docs: integration guide'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | Guide complete with 5+ examples |
| P6.8 | Final quality gate | P6.7 | copilot-L3-6h | `copilot -p "Final quality gate. Run: (1) cargo test --all (2) cargo clippy --all -- -D warnings (3) cargo fmt --all -- --check (4) cargo tarpaulin --out Html (5) task spec:validate (6) task traceability:check. Fix any failures. Commit: 'final: quality gate pass'" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &` | All quality gates pass |

---

## Critical Path Summary

**Minimum Critical Path Duration:** ~28-35 days wall-clock
**Parallelizable:** Phase 3 (client) can run alongside Phases 1-2
**Blockers:**
- Phase 0 blocks everything (core tick loop required)
- Phase 1 (economy) blocks Phase 2 (actors need economy working)
- Phase 2 (actors) blocks Phase 4 (war/diplomacy need institutions)
- Phase 4 blocks Phase 5 (research API depends on all systems)

**Recommended Dispatch Strategy:**
1. Start Phase 0 first (5 copilots, sequential tasks)
2. Upon Phase 0 completion, start Phase 1 + Phase 3 in parallel (10 copilots)
3. Upon Phase 1 completion, start Phase 2 (8 copilots)
4. Upon Phase 2 completion, start Phase 4 (8 copilots)
5. Upon Phase 4 completion, start Phase 5 (8 copilots)
6. Upon Phase 5 completion, start Phase 6 (8 copilots)

**Total L3 Agents:** ~50+ concurrent copilot-L3 agents across all phases
**Per-Phase Agents:** 4-8 agents per phase, working in dedicated git worktrees

---

## Git Worktree Merge Strategy

After each phase completes:

```bash
# In phase worktree
git log --oneline  # verify commits
cargo test --all   # final verification

# Back on main
cd C:/Users/koosh/Dev/Civis
git merge ../civ-wt-phase{N}-{name}
git worktree remove ../civ-wt-phase{N}-{name}
task quality     # final gate before next phase
```

---

## Rollback & Failure Handling

If a phase fails:
1. Keep worktree intact (don't delete)
2. Create issue in `docs/PHASE_FAILURES.md` with root cause
3. Dispatch new copilot-L3 agent with fix
4. Retry gate before merging
5. If repeated failures, escalate to main codebase review (human)

