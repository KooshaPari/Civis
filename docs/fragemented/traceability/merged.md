# Merged Fragmented Markdown

## Source: traceability/EVENT_TAXONOMY.md

# Event Taxonomy

**Status:** Living document — updated as namespaces are extended.
**Protocol:** JSON-RPC 2.0 notifications over WebSocket (`crates/protocol`).
**Storage:** Every event is appended to the DB audit log within the same tick it is emitted.

## Common Envelope

All events share this outer envelope regardless of namespace:

```json
{
  "event_id":   "<UUIDv7>",
  "event_type": "<namespace.name.v1>",
  "session_id": "<UUID>",
  "tick":       12345,
  "created_at": "2026-02-21T00:00:00.000Z",
  "payload":    { }
}
```

`event_id` is a UUIDv7 (time-ordered). `tick` is the engine tick counter (`u64`). `created_at` is RFC 3339 UTC. `payload` is event-specific and documented per event below.

Subscribers column key:
- **UI** — render/frontend clients
- **DB** — audit log writer (`crates/db`)
- **AI** — AI subsystem (`crates/ai`)
- **MOD** — mod host (`crates/engine/mod_loader`)
- **ALL** — every registered subscriber

---

## `run.*` — Simulation Lifecycle

Emitter: `crates/engine/src/tick.rs`, `crates/engine/src/integrity.rs`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `run.started.v1` | Engine has initialised world state and begun the tick loop. | `seed: u64`, `civ_count: u8`, `hex_count: u32`, `tick_budget_ms: u32` | engine/tick | ALL | once per run |
| `run.tick.completed.v1` | A single tick has completed; carries tick hash and wall-clock duration. | `tick: u64`, `hash: string (hex-64)`, `duration_ms: u32`, `event_count: u32` | engine/tick | ALL | every tick |
| `run.paused.v1` | Simulation has been paused (speed set to 0). | `tick: u64`, `reason: string` | engine/tick | UI, DB | on-action |
| `run.resumed.v1` | Simulation has resumed after a pause. | `tick: u64`, `speed_multiplier: f32` | engine/tick | UI, DB | on-action |
| `run.ended.v1` | Simulation has terminated (victory, defeat, or manual stop). | `tick: u64`, `termination_reason: string`, `winner_civ_id: uuid \| null` | engine/tick | ALL | once per run |
| `run.hash.mismatch.v1` | Replay diverged — tick hash does not match recorded hash. | `tick: u64`, `expected_hash: string`, `actual_hash: string` | engine/integrity | DB, UI | on-condition |
| `run.determinism.violation.v1` | Two identical runs produced different tick hashes; emitted in test harness. | `tick: u64`, `run_a_hash: string`, `run_b_hash: string`, `seed: u64` | engine/integrity | DB | on-condition |

**Example — `run.tick.completed.v1` payload:**
```json
{
  "tick": 42,
  "hash": "a3f1d...c9b2",
  "duration_ms": 67,
  "event_count": 14
}
```

---

## `session.*` — Session Management

Emitter: `crates/engine/src/session.rs`
Source spec: `CIV-0900`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `session.created.v1` | A new session record has been created in the DB. | `session_id: uuid`, `mode: string`, `player_count: u8` | engine/session | DB, UI | on-action |
| `session.started.v1` | The session is fully initialised and the tick loop is active. | `session_id: uuid`, `tick: u64` | engine/session | ALL | once per session |
| `session.ended.v1` | The session has concluded and is no longer accepting inputs. | `session_id: uuid`, `tick: u64`, `outcome: string` | engine/session | ALL | once per session |
| `session.turn.start.v1` | A hot-seat turn has begun for a given player. | `session_id: uuid`, `tick: u64`, `player_id: uuid`, `turn_number: u32` | engine/session | UI, DB | per turn |
| `session.turn.end.v1` | A hot-seat turn has ended; next player is queued. | `session_id: uuid`, `tick: u64`, `player_id: uuid`, `turn_number: u32` | engine/session | UI, DB | per turn |
| `session.speed_changed.v1` | Simulation speed multiplier has changed. | `session_id: uuid`, `tick: u64`, `old_speed: f32`, `new_speed: f32` | engine/session | UI, DB | on-action |
| `session.paused.v1` | Session-level pause (distinct from `run.paused.v1`; may carry UI context). | `session_id: uuid`, `tick: u64`, `initiated_by: string` | engine/session | UI, DB | on-action |
| `session.resumed.v1` | Session resumed after pause. | `session_id: uuid`, `tick: u64` | engine/session | UI, DB | on-action |

**Example — `session.turn.start.v1` payload:**
```json
{
  "session_id": "0192f3a1-...",
  "tick": 200,
  "player_id": "0192f3a2-...",
  "turn_number": 4
}
```

---

## `save.*` — Save / Load

Emitter: `crates/db/src/save.rs`, `crates/db/src/load.rs`
Source spec: `CIV-1000`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `session.saved.v1` | World state has been serialised and written to a save slot. | `session_id: uuid`, `tick: u64`, `slot: string`, `byte_size: u64`, `schema_version: u32` | db/save | UI, DB | on-action |
| `session.loaded.v1` | World state has been deserialised from a save slot and is ready. | `session_id: uuid`, `tick: u64`, `slot: string`, `schema_version: u32` | db/load | UI, DB | on-action |
| `session.save_failed.v1` | Save attempt failed; state was NOT written. | `session_id: uuid`, `tick: u64`, `slot: string`, `error: string` | db/save | UI, DB | on-condition |

**Example — `session.saved.v1` payload:**
```json
{
  "session_id": "0192f3a1-...",
  "tick": 500,
  "slot": "quicksave",
  "byte_size": 204800,
  "schema_version": 3
}
```

---

## `economy.*` — Economy Events

Emitter: `crates/economy/src/district.rs`, `crates/economy/src/trade.rs`, `crates/economy/src/subsistence.rs`
Source specs: `CIV-0100`, `CIV-0107`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `economy.joule.deficit.v1` | A district's Joule balance fell below zero this tick. | `civ_id: uuid`, `district_id: uuid`, `deficit_kj: i64`, `tick: u64` | economy/district | UI, DB, AI | on-condition |
| `economy.trade.executed.v1` | A bilateral trade transfer completed between two civilizations. | `from_civ: uuid`, `to_civ: uuid`, `joules_kj: i64`, `credits_mc: i64`, `treaty_id: uuid` | economy/trade | DB, UI | per trade agreement per tick |
| `economy.district.collapsed.v1` | A district has been in Joule deficit for 3 consecutive ticks and has collapsed. | `civ_id: uuid`, `district_id: uuid`, `deficit_ticks: u8` | economy/district | ALL | on-condition |
| `economy.subsistence.triggered.v1` | A civilization's total Joule balance dropped below subsistence threshold. | `civ_id: uuid`, `total_balance_kj: i64`, `threshold_kj: i64` | economy/subsistence | ALL | on-condition |

**Example — `economy.district.collapsed.v1` payload:**
```json
{
  "civ_id":       "0192-...",
  "district_id":  "0193-...",
  "deficit_ticks": 3
}
```

---

## `climate.*` — Climate Events

Emitter: `crates/climate/src/events.rs`
Source spec: `CIV-0102`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `climate.threshold.crossed.v1` | Global mean temperature crossed a named threshold level (upward or downward). | `threshold_name: string`, `temp_celsius: f32`, `direction: "up" \| "down"`, `tick: u64` | climate/events | ALL | on-condition |
| `climate.damage.applied.v1` | Climate damage has reduced one or more districts' production capacity this tick. | `affected_districts: u32`, `total_damage_kj: i64`, `tick: u64` | climate/damage | DB, UI, AI | per-tick when active |
| `climate.tipping_point.v1` | A tipping-point cascade has been triggered (e.g. ice-albedo feedback). | `cascade_name: string`, `trigger_temp: f32`, `projected_delta: f32`, `tick: u64` | climate/tipping | ALL | on-condition |

**Example — `climate.threshold.crossed.v1` payload:**
```json
{
  "threshold_name": "critical_2c",
  "temp_celsius": 2.01,
  "direction": "up",
  "tick": 800
}
```

---

## `institution.*` — Institutional Events

Emitter: `crates/institutions/src/events.rs`
Source spec: `CIV-0103`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `institution.type.changed.v1` | A civilization's governance type has transitioned. | `civ_id: uuid`, `old_type: string`, `new_type: string`, `tick: u64` | institutions/governance | ALL | on-condition |
| `institution.capture.threshold.v1` | Institutional capture score crossed 0.75. | `civ_id: uuid`, `capture_score: f32`, `tick: u64` | institutions/capture | DB, UI, AI | on-condition |
| `institution.collapse.v1` | Institution has collapsed; governance transition is queued. | `civ_id: uuid`, `institution_id: uuid`, `cause: string`, `tick: u64` | institutions/collapse | ALL | on-condition |

**Example — `institution.type.changed.v1` payload:**
```json
{
  "civ_id":   "0192-...",
  "old_type": "democracy",
  "new_type": "autocracy",
  "tick": 350
}
```

---

## `citizen.*` — Citizen Lifecycle

Emitter: `crates/citizens/src/lifecycle.rs`
Source spec: `CIV-0103`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `citizen.born.v1` | A cohort of citizens has been born in a district this tick. | `civ_id: uuid`, `district_id: uuid`, `count: u32`, `tick: u64` | citizens/lifecycle | DB, UI | per-tick when births > 0 |
| `citizen.died.v1` | A cohort of citizens has died (disease, famine, or war). | `civ_id: uuid`, `district_id: uuid`, `count: u32`, `cause: string`, `tick: u64` | citizens/lifecycle | DB, UI | on-condition |
| `citizen.migrated.v1` | Citizens have moved between districts or civilizations. | `from_district: uuid`, `to_district: uuid`, `count: u32`, `reason: string`, `tick: u64` | citizens/lifecycle | DB, UI | on-condition |
| `citizen.stress.critical.v1` | Aggregate citizen stress in a district exceeded the critical threshold. | `civ_id: uuid`, `district_id: uuid`, `stress_score: f32`, `tick: u64` | citizens/lifecycle | ALL | on-condition |

**Example — `citizen.stress.critical.v1` payload:**
```json
{
  "civ_id":      "0192-...",
  "district_id": "0194-...",
  "stress_score": 0.91,
  "tick": 612
}
```

---

## `diplomacy.*` — Diplomatic Events

Emitter: `crates/diplomacy/src/war.rs`, `crates/diplomacy/src/peace.rs`, `crates/diplomacy/src/treaty.rs`, `crates/diplomacy/src/espionage.rs`
Source spec: `CIV-0105`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `diplomacy.war.declared.v1` | A civilization has declared war on another. | `aggressor_civ: uuid`, `target_civ: uuid`, `tick: u64` | diplomacy/war | ALL | on-action |
| `diplomacy.peace.signed.v1` | A peace agreement has been finalised. | `civ_a: uuid`, `civ_b: uuid`, `terms_summary: string`, `tick: u64` | diplomacy/peace | ALL | on-action |
| `diplomacy.treaty.formed.v1` | A new treaty has been created and signed. | `treaty_id: uuid`, `civ_a: uuid`, `civ_b: uuid`, `treaty_type: string`, `tick: u64` | diplomacy/treaty | DB, UI, AI | on-action |
| `diplomacy.treaty.broken.v1` | A civilization has violated a treaty; reputation penalty applied. | `treaty_id: uuid`, `breaching_civ: uuid`, `rep_delta: f32`, `tick: u64` | diplomacy/treaty | ALL | on-condition |
| `diplomacy.espionage.detected.v1` | An espionage operation was detected. | `target_civ: uuid`, `suspected_civ: uuid \| null`, `operation_type: string`, `tick: u64` | diplomacy/espionage | DB, UI | on-condition |

**Example — `diplomacy.war.declared.v1` payload:**
```json
{
  "aggressor_civ": "0192-...",
  "target_civ":   "0195-...",
  "tick": 780
}
```

---

## `social.*` — Social Events

Emitter: `crates/social/src/insurgency.rs`, `crates/social/src/ideology.rs`, `crates/social/src/health.rs`
Source spec: `CIV-0106`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `social.insurgency.started.v1` | Insurgency has begun in a civilization after stress threshold crossed. | `civ_id: uuid`, `trigger_district: uuid`, `stress_score: f32`, `tick: u64` | social/insurgency | ALL | on-condition |
| `social.insurgency.ended.v1` | Insurgency has been suppressed or resolved. | `civ_id: uuid`, `duration_ticks: u32`, `resolution: string`, `tick: u64` | social/insurgency | ALL | on-condition |
| `social.ideology.shift.v1` | A cohort's dominant ideology has shifted beyond a threshold. | `civ_id: uuid`, `cohort_id: uuid`, `old_ideology: string`, `new_ideology: string`, `tick: u64` | social/ideology | DB, UI, AI | on-condition |
| `social.health.crisis.v1` | Health index dropped below crisis threshold; labor productivity reduced. | `civ_id: uuid`, `health_index: f32`, `productivity_delta: f32`, `tick: u64` | social/health | ALL | on-condition |

**Example — `social.insurgency.started.v1` payload:**
```json
{
  "civ_id":          "0192-...",
  "trigger_district": "0194-...",
  "stress_score": 0.88,
  "tick": 430
}
```

---

## `ai.*` — AI Decision Events

Emitter: `crates/ai/src/events.rs`
Source spec: `CIV-0400`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `ai.decision.v1` | An AI civilization has selected an action this tick. | `civ_id: uuid`, `action_type: string`, `utility_score: f32`, `tick: u64` | ai/events | DB, MOD | every AI tick |
| `ai.personality.drift.v1` | An AI leader's personality weights have drifted stochastically. | `civ_id: uuid`, `trait: string`, `old_weight: f32`, `new_weight: f32`, `tick: u64` | ai/personality | DB | on-condition |
| `ai.mcts.computed.v1` | MCTS lookahead has completed; records depth and best action. | `civ_id: uuid`, `depth: u8`, `iterations: u32`, `best_action: string`, `duration_ms: u32` | ai/mcts | DB | on-action |
| `ai.performance.v1` | AI subsystem budget usage for this tick. | `civ_id: uuid`, `compute_ms: u32`, `budget_ms: u32`, `overrun: bool`, `tick: u64` | ai/mcts | DB | every AI tick |

**Example — `ai.mcts.computed.v1` payload:**
```json
{
  "civ_id":      "0196-...",
  "depth": 4,
  "iterations": 256,
  "best_action": "build_solar_array",
  "duration_ms": 18
}
```

---

## `policy.*` — Intervention Events

Emitter: `crates/engine/src/policy.rs` (formerly `crates/policy`)
Source spec: `docs/models/civ-sim/TECHNICAL_SPEC.md`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `policy.intervention.applied.v1` | A policy intervention has been applied to the simulation state. | `civ_id: uuid`, `policy_id: string`, `lever: string`, `delta: f32`, `tick: u64` | engine/policy | ALL | on-action |
| `policy.intervention.rejected.v1` | A proposed intervention was rejected (constraint violation or invalid state). | `civ_id: uuid`, `policy_id: string`, `rejection_reason: string`, `tick: u64` | engine/policy | UI, DB | on-action |
| `policy.bundle.activated.v1` | A named policy bundle (group of interventions) has been activated. | `civ_id: uuid`, `bundle_name: string`, `intervention_count: u8`, `tick: u64` | engine/policy | ALL | on-action |

**Example — `policy.intervention.applied.v1` payload:**
```json
{
  "civ_id":    "0192-...",
  "policy_id": "carbon_tax_v2",
  "lever":     "co2_emission_rate",
  "delta":     -0.15,
  "tick": 300
}
```

---

## `metrics.*` — Derived Metrics Snapshots

Emitter: `crates/engine/src/metrics.rs` (or `crates/metrics`)
Source spec: `docs/models/civ-sim/DATA_MODEL_DB_SPEC.md`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `metrics.snapshot.v1` | Per-tick summary of all civilizations' key indicators. | `tick: u64`, `civs: [{civ_id, gdp_mc, joule_balance_kj, pop, temp_c, capture_score}]` | engine/metrics | DB, UI, AI | every tick |
| `metrics.threshold.warning.v1` | A derived metric has crossed a warning threshold (50–75 % of critical). | `metric: string`, `civ_id: uuid \| null`, `value: f32`, `warning_threshold: f32`, `tick: u64` | engine/metrics | UI, DB | on-condition |
| `metrics.threshold.critical.v1` | A derived metric has crossed the critical threshold (≥ 75 % of max). | `metric: string`, `civ_id: uuid \| null`, `value: f32`, `critical_threshold: f32`, `tick: u64` | engine/metrics | ALL | on-condition |

**Example — `metrics.snapshot.v1` payload (abbreviated):**
```json
{
  "tick": 100,
  "civs": [
    { "civ_id": "0192-...", "gdp_mc": 4200000, "joule_balance_kj": 8800, "pop": 15400, "temp_c": 1.3, "capture_score": 0.42 }
  ]
}
```

---

## `mod.*` — Mod System Events

Emitter: `crates/engine/src/mod_loader.rs`
Source spec: `CIV-0700`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `mod.loaded.v1` | A WASM mod has been loaded and registered successfully. | `mod_id: string`, `mod_name: string`, `version: string`, `tick: u64` | engine/mod_loader | DB, UI | on-action |
| `mod.unloaded.v1` | A mod has been unloaded (user request or session end). | `mod_id: string`, `reason: string`, `tick: u64` | engine/mod_loader | DB, UI | on-action |
| `mod.error.v1` | A mod encountered a runtime error (trap, OOM, constraint violation). | `mod_id: string`, `error_type: string`, `detail: string`, `tick: u64` | engine/mod_loader | ALL | on-condition |
| `mod.state.saved.v1` | A mod's internal state has been serialised as part of a save operation. | `mod_id: string`, `slot: string`, `byte_size: u32`, `tick: u64` | engine/mod_state | DB | on-action |

**Example — `mod.error.v1` payload:**
```json
{
  "mod_id":     "resource_pack_v3",
  "error_type": "wasm_trap",
  "detail":     "unreachable executed at offset 0x1a4",
  "tick": 512
}
```

---

## `challenge.*` — Async Challenge Mode

Emitter: `crates/engine/src/challenge.rs`
Source spec: `CIV-0900`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `challenge.submitted.v1` | A player has submitted a civilization seed for async challenge scoring. | `challenge_id: uuid`, `player_id: uuid`, `seed: u64`, `submitted_at: timestamp` | engine/challenge | DB | on-action |
| `challenge.completed.v1` | The challenge run has finished executing. | `challenge_id: uuid`, `tick_count: u64`, `final_hash: string`, `duration_ms: u64` | engine/challenge | DB, UI | on-condition |
| `challenge.scored.v1` | A completed challenge has been scored and ranked. | `challenge_id: uuid`, `score: f64`, `rank: u32`, `percentile: f32` | engine/challenge | DB, UI | on-condition |

**Example — `challenge.scored.v1` payload:**
```json
{
  "challenge_id": "0197-...",
  "score": 84231.5,
  "rank": 12,
  "percentile": 93.7
}
```

---

## `asset.*` — Asset Pipeline Events

Emitter: `crates/render/src/atlas.rs`
Source spec: `CIV-0600`

| Event Type | Description | Key Payload Fields | Emitter | Subscribers | Frequency |
|---|---|---|---|---|---|
| `asset.generated.v1` | A single asset (sprite, icon, texture) has been generated from source SVG. | `asset_id: string`, `lod: string`, `source_svg: string`, `output_path: string` | render/atlas | DB | on-action (build time) |
| `asset.atlas.built.v1` | The full texture atlas has been packed and written to disk. | `lod: string`, `tile_count: u32`, `atlas_path: string`, `byte_size: u64` | render/atlas | DB, UI | on-action (build time) |
| `asset.generation.failed.v1` | An asset generation step failed (bad SVG, missing source, tool error). | `asset_id: string`, `error: string`, `source_svg: string` | render/atlas | DB | on-condition (build time) |

**Example — `asset.atlas.built.v1` payload:**
```json
{
  "lod":        "operational",
  "tile_count": 256,
  "atlas_path": "assets/atlas_operational.png",
  "byte_size":  2097152
}
```

---

## Versioning Policy

Event types carry a `v1` suffix. When a breaking payload change is required:

1. Introduce a new `v2` event type alongside `v1`.
2. Emit both during a transition period.
3. Remove `v1` only after all subscribers are updated and confirmed.

Additive payload fields (new optional keys) do NOT require a version bump; subscribers MUST ignore unknown fields.

---

## Namespace Summary

| Namespace | Count | Primary Emitter Crate | Spec |
|---|---|---|---|
| `run.*` | 7 | `crates/engine` | CIV-0001 |
| `session.*` | 8 | `crates/engine` | CIV-0900 |
| `save.*` | 3 | `crates/db` | CIV-1000 |
| `economy.*` | 4 | `crates/economy` | CIV-0100 / CIV-0107 |
| `climate.*` | 3 | `crates/climate` | CIV-0102 |
| `institution.*` | 3 | `crates/institutions` | CIV-0103 |
| `citizen.*` | 4 | `crates/citizens` | CIV-0103 |
| `diplomacy.*` | 5 | `crates/diplomacy` | CIV-0105 |
| `social.*` | 4 | `crates/social` | CIV-0106 |
| `ai.*` | 4 | `crates/ai` | CIV-0400 |
| `policy.*` | 3 | `crates/engine` | TECHNICAL_SPEC |
| `metrics.*` | 3 | `crates/engine` / `crates/metrics` | DATA_MODEL_DB_SPEC |
| `mod.*` | 4 | `crates/engine` | CIV-0700 |
| `challenge.*` | 3 | `crates/engine` | CIV-0900 |
| `asset.*` | 3 | `crates/render` | CIV-0600 |
| **Total** | **61** | | |

---

*Last updated: 2026-02-21. When adding a new event: add a row to the relevant namespace table, provide a payload example, and update the namespace summary count.*


---

## Source: traceability/TRACEABILITY_MATRIX.md

# Traceability Matrix

**Status:** Living document — updated as FRs are assigned, implemented, and tested.
**Format:** FR ID | Requirement Summary (SHALL) | Spec Doc | Crate / Source Path | Test Name Pattern | Status

Status values: `planned` | `in_progress` | `implemented`

---

## Core Engine (FR-CORE-*)

Source spec: `docs/specs/CIV-0001-core-simulation-loop.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-CORE-001 | The engine SHALL advance simulation state by exactly one tick per `Engine::step()` invocation. | CIV-0001 | `crates/engine/src/tick.rs` | `tick::step_advances_tick` | implemented |
| FR-CORE-002 | The engine SHALL produce identical output for identical seed and input sequence (determinism). | CIV-0001 | `crates/engine/src/lib.rs` | `determinism::double_run_identical` | implemented |
| FR-CORE-003 | The engine SHALL use ChaCha20Rng seeded per-run; no global mutable RNG state. | CIV-0001 | `crates/engine/src/rng.rs` | `rng::chacha20_seeded_isolated` | implemented |
| FR-CORE-004 | Each tick SHALL complete within 100 ms wall-clock on the reference hardware profile. | CIV-0001 | `crates/engine/src/tick.rs` | `perf::tick_under_100ms` | in_progress |
| FR-CORE-005 | The engine SHALL emit a BLAKE3 hash of full world state at the end of every tick. | CIV-0001 | `crates/engine/src/hash_chain.rs` | `hash_chain::tick_hash_emitted` | implemented |
| FR-CORE-006 | Consecutive tick hashes SHALL form an append-only chain (each hash includes prior hash). | CIV-0001 | `crates/engine/src/hash_chain.rs` | `hash_chain::chain_includes_prior` | implemented |
| FR-CORE-007 | The engine SHALL surface a `run.hash.mismatch.v1` event when replayed state diverges. | CIV-0001 | `crates/engine/src/integrity.rs` | `integrity::mismatch_event_emitted` | in_progress |
| FR-CORE-008 | World state SHALL be modelled as bevy_ecs 0.18.x `World`; no global singletons. | CIV-0001 | `crates/engine/src/world.rs` | `world::no_global_resources` | planned |
| FR-CORE-009 | Hex grid SHALL use `hexx` 0.21.x axial coordinates throughout engine and render crates. | CIV-0001 | `crates/engine/src/grid.rs` | `grid::axial_roundtrip` | planned |
| FR-CORE-010 | All integer quantities SHALL use fixed-point types (`FixedI32<U16>`, `i64` KiloJoules, `i64` MilliCredits). | CIV-0001 | `crates/engine/src/numerics.rs` | `numerics::no_float_in_state` | planned |

---

## Economy (FR-ECON-*)

Source specs: `docs/specs/CIV-0100-economy.md`, `docs/specs/CIV-0107-joule-economy.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-ECON-001 | Each district SHALL produce Joules each tick according to its resource type and capacity. | CIV-0100 | `crates/economy/src/production.rs` | `production::district_produces_joules` | implemented |
| FR-ECON-002 | Joule consumption SHALL be deducted from district reserves before regional distribution. | CIV-0107 | `crates/economy/src/consumption.rs` | `consumption::deducted_before_distribution` | implemented |
| FR-ECON-003 | Joule consumption per tick SHALL never be negative (consumption_non_negative invariant). | CIV-0107 | `crates/economy/src/consumption.rs` | `consumption::consumption_non_negative` | implemented |
| FR-ECON-004 | Surplus Joules SHALL flow to adjacent districts via the distribution graph each tick. | CIV-0100 | `crates/economy/src/distribution.rs` | `distribution::surplus_flows_adjacent` | in_progress |
| FR-ECON-005 | Waste heat SHALL be computed as a percentage of total Joules consumed per tick. | CIV-0107 | `crates/economy/src/waste.rs` | `waste::heat_computed_from_consumption` | planned |
| FR-ECON-006 | GDP SHALL be derived from sum of regional Joule throughput converted at a fixed exchange rate. | CIV-0100 | `crates/economy/src/gdp.rs` | `gdp::sum_of_regional_joules` | planned |
| FR-ECON-007 | Trade agreements SHALL transfer Joules and MilliCredits between civilizations each tick. | CIV-0100 | `crates/economy/src/trade.rs` | `trade::bilateral_transfer_balanced` | planned |
| FR-ECON-008 | A district in Joule deficit for 3 consecutive ticks SHALL emit `economy.district.collapsed.v1`. | CIV-0100 | `crates/economy/src/district.rs` | `district::collapse_after_deficit_ticks` | planned |
| FR-ECON-009 | Subsistence mode SHALL activate when a civilization's total Joule balance drops below threshold. | CIV-0107 | `crates/economy/src/subsistence.rs` | `subsistence::activates_below_threshold` | planned |
| FR-ECON-010 | Treasury balance SHALL be tracked in MilliCredits (`i64`) with no floating-point accumulation. | CIV-0100 | `crates/economy/src/treasury.rs` | `treasury::milliCredits_no_float` | planned |

---

## Level of Detail (FR-LOD-*)

Source spec: `docs/specs/CIV-0101-lod.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-LOD-001 | The engine SHALL support two zoom levels: strategic (region) and operational (district/hex). | CIV-0101 | `crates/engine/src/lod.rs` | `lod::two_levels_defined` | planned |
| FR-LOD-002 | Strategic view SHALL aggregate district data into region summaries each tick. | CIV-0101 | `crates/engine/src/lod.rs` | `lod::strategic_aggregation` | planned |
| FR-LOD-003 | LOD transitions SHALL not alter simulation state, only view projection. | CIV-0101 | `crates/engine/src/lod.rs` | `lod::transition_no_state_mutation` | planned |
| FR-LOD-004 | Operational view SHALL expose individual hex-cell resource and population data. | CIV-0101 | `crates/engine/src/lod.rs` | `lod::operational_hex_data_visible` | planned |

---

## Climate (FR-CLIM-*)

Source spec: `docs/specs/CIV-0102-climate.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-CLIM-001 | Atmospheric CO2 SHALL accumulate each tick based on industrial Joule consumption. | CIV-0102 | `crates/climate/src/co2.rs` | `co2::accumulates_with_consumption` | planned |
| FR-CLIM-002 | Global mean temperature SHALL be derived from CO2 concentration via parameterised formula. | CIV-0102 | `crates/climate/src/temperature.rs` | `temperature::derived_from_co2` | planned |
| FR-CLIM-003 | The engine SHALL emit `climate.threshold.crossed.v1` when temperature crosses a defined level. | CIV-0102 | `crates/climate/src/events.rs` | `climate_events::threshold_event_emitted` | planned |
| FR-CLIM-004 | Climate damage SHALL reduce district Joule production capacity when temperature exceeds threshold. | CIV-0102 | `crates/climate/src/damage.rs` | `damage::reduces_production_above_threshold` | planned |
| FR-CLIM-005 | The engine SHALL model at least one tipping-point cascade (e.g. ice-albedo) above critical temperature. | CIV-0102 | `crates/climate/src/tipping.rs` | `tipping::cascade_triggered` | planned |
| FR-CLIM-006 | Civilizations SHALL be able to invest MilliCredits into adaptation to reduce climate damage. | CIV-0102 | `crates/climate/src/adaptation.rs` | `adaptation::investment_reduces_damage` | planned |

---

## Institutions (FR-INST-*)

Source spec: `docs/specs/CIV-0103-institutions.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-INST-001 | Each civilization SHALL have an institutional type (democracy, autocracy, technocracy, etc.). | CIV-0103 | `crates/institutions/src/governance.rs` | `governance::type_assigned_at_init` | planned |
| FR-INST-002 | Institutional capture score SHALL accumulate each tick based on resource concentration. | CIV-0103 | `crates/institutions/src/capture.rs` | `capture::accumulates_with_concentration` | planned |
| FR-INST-003 | The engine SHALL emit `institution.capture.threshold.v1` when capture crosses 0.75. | CIV-0103 | `crates/institutions/src/events.rs` | `inst_events::capture_threshold_event` | planned |
| FR-INST-004 | Institutional collapse SHALL trigger a governance type transition. | CIV-0103 | `crates/institutions/src/collapse.rs` | `collapse::triggers_type_transition` | planned |
| FR-INST-005 | Institution time-series data SHALL be stored in the metrics DB for post-run analysis. | CIV-0103 | `crates/db/src/institution_series.rs` | `db::institution_series_stored` | planned |
| FR-INST-006 | Citizen lifecycle (birth, migration, death) SHALL be driven by institutional and economic state. | CIV-0103 | `crates/citizens/src/lifecycle.rs` | `lifecycle::driven_by_inst_economy` | planned |

---

## Theorem / Invariants (FR-THRY-*)

Source spec: `docs/specs/CIV-0104-theorem.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-THRY-001 | Total Joule energy in a closed system SHALL be conserved each tick (production - consumption - waste = 0). | CIV-0104 | `crates/engine/src/invariants.rs` | `invariants::joule_conservation` | planned |
| FR-THRY-002 | Total MilliCredit supply SHALL remain constant absent explicit treasury mint/burn operations. | CIV-0104 | `crates/engine/src/invariants.rs` | `invariants::credit_supply_conserved` | planned |
| FR-THRY-003 | Population delta per tick SHALL equal births minus deaths minus emigration plus immigration. | CIV-0104 | `crates/engine/src/invariants.rs` | `invariants::population_delta_balanced` | planned |
| FR-THRY-004 | The invariant checker SHALL run every tick and panic in debug builds on violation. | CIV-0104 | `crates/engine/src/invariants.rs` | `invariants::checker_panics_on_violation` | planned |

---

## Diplomacy (FR-DIPL-*)

Source spec: `docs/specs/CIV-0105-war-diplomacy.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-DIPL-001 | Civilizations SHALL be able to declare war, producing `diplomacy.war.declared.v1`. | CIV-0105 | `crates/diplomacy/src/war.rs` | `war::declare_emits_event` | planned |
| FR-DIPL-002 | Peace SHALL be negotiated via signed treaty, producing `diplomacy.peace.signed.v1`. | CIV-0105 | `crates/diplomacy/src/peace.rs` | `peace::signed_emits_event` | planned |
| FR-DIPL-003 | Treaties SHALL encode terms (trade ratios, non-aggression, alliance) as structured data. | CIV-0105 | `crates/diplomacy/src/treaty.rs` | `treaty::terms_structured` | planned |
| FR-DIPL-004 | Treaty breach SHALL emit `diplomacy.treaty.broken.v1` and apply reputation penalty. | CIV-0105 | `crates/diplomacy/src/treaty.rs` | `treaty::breach_emits_event_and_penalty` | planned |
| FR-DIPL-005 | Espionage operations SHALL have a configurable detection probability per tick. | CIV-0105 | `crates/diplomacy/src/espionage.rs` | `espionage::detection_probability_applied` | planned |
| FR-DIPL-006 | Detected espionage SHALL emit `diplomacy.espionage.detected.v1`. | CIV-0105 | `crates/diplomacy/src/espionage.rs` | `espionage::detected_emits_event` | planned |
| FR-DIPL-007 | Shadow networks SHALL model covert influence as a hidden resource accumulating per tick. | CIV-0105 | `crates/diplomacy/src/shadow.rs` | `shadow::influence_accumulates` | planned |

---

## Social (FR-SOCI-*)

Source spec: `docs/specs/CIV-0106-social.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-SOCI-001 | Ideological alignment SHALL be tracked per-citizen cohort as a continuous score. | CIV-0106 | `crates/social/src/ideology.rs` | `ideology::per_cohort_continuous` | planned |
| FR-SOCI-002 | Citizen stress SHALL accumulate when Joule access falls below subsistence level. | CIV-0106 | `crates/social/src/stress.rs` | `stress::accumulates_below_subsistence` | planned |
| FR-SOCI-003 | Insurgency SHALL start when aggregate stress exceeds the configured threshold. | CIV-0106 | `crates/social/src/insurgency.rs` | `insurgency::starts_above_threshold` | planned |
| FR-SOCI-004 | The engine SHALL emit `social.insurgency.started.v1` and `social.insurgency.ended.v1`. | CIV-0106 | `crates/social/src/events.rs` | `social_events::insurgency_lifecycle_events` | planned |
| FR-SOCI-005 | Health index SHALL be computed from food Joules, clean water, and medical infrastructure. | CIV-0106 | `crates/social/src/health.rs` | `health::computed_from_inputs` | planned |
| FR-SOCI-006 | A health crisis SHALL emit `social.health.crisis.v1` and reduce labor productivity. | CIV-0106 | `crates/social/src/health.rs` | `health::crisis_emits_event_reduces_labor` | planned |

---

## AI (FR-AI-*)

Source spec: `docs/specs/CIV-0400-ai.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-AI-001 | AI civilizations SHALL select actions using a utility scoring function over available moves. | CIV-0400 | `crates/ai/src/utility.rs` | `utility::scores_all_moves` | planned |
| FR-AI-002 | MCTS SHALL be used for multi-step lookahead planning beyond depth 1. | CIV-0400 | `crates/ai/src/mcts.rs` | `mcts::lookahead_depth_gt_1` | planned |
| FR-AI-003 | Each AI leader SHALL have a personality profile affecting utility weights. | CIV-0400 | `crates/ai/src/personality.rs` | `personality::weights_differ_per_profile` | planned |
| FR-AI-004 | Personality drift SHALL accumulate stochastically each N ticks. | CIV-0400 | `crates/ai/src/personality.rs` | `personality::drift_accumulates_stochastically` | planned |
| FR-AI-005 | AI SHALL never exceed a configurable MilliCredit/Joule expenditure per tick (fair-play cap). | CIV-0400 | `crates/ai/src/fair_play.rs` | `fair_play::cap_enforced_per_tick` | planned |
| FR-AI-006 | AI decision events SHALL be emitted for post-run analysis and replay. | CIV-0400 | `crates/ai/src/events.rs` | `ai_events::decision_emitted` | planned |
| FR-AI-007 | MCTS computation time SHALL be capped at a fraction of the 100 ms tick budget. | CIV-0400 | `crates/ai/src/mcts.rs` | `mcts::time_capped_within_budget` | planned |

---

## Protocol (FR-PROT-*)

Source spec: `docs/specs/CIV-0200-protocol.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-PROT-001 | The engine SHALL expose a JSON-RPC 2.0 API over WebSocket. | CIV-0200 | `crates/protocol/src/server.rs` | `protocol::jsonrpc_handshake` | planned |
| FR-PROT-002 | All events SHALL be emitted as JSON-RPC notifications with a common envelope. | CIV-0200 | `crates/protocol/src/events.rs` | `protocol::event_envelope_valid` | planned |
| FR-PROT-003 | Event envelope SHALL contain `event_id` (UUIDv7), `event_type`, `session_id`, `tick`, `created_at`, `payload`. | CIV-0200 | `crates/protocol/src/events.rs` | `protocol::envelope_fields_present` | planned |
| FR-PROT-004 | The server SHALL persist all emitted events to the DB audit log within the same tick. | CIV-0200 | `crates/db/src/audit_log.rs` | `db::events_persisted_same_tick` | planned |
| FR-PROT-005 | Client connections SHALL authenticate before receiving any session events. | CIV-0200 | `crates/protocol/src/auth.rs` | `protocol::unauthenticated_rejected` | planned |
| FR-PROT-006 | The protocol SHALL support at least 10 concurrent client connections per session. | CIV-0200 | `crates/protocol/src/server.rs` | `protocol::concurrent_clients_10` | planned |

---

## UI/UX (FR-UX-*)

Source spec: `docs/specs/CIV-0300-ui-ux.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-UX-001 | The UI SHALL render the hex map using the `crates/render` crate at 60 fps target. | CIV-0300 | `crates/render/src/hex_map.rs` | `render::hex_map_60fps` | planned |
| FR-UX-002 | The UI SHALL support RTS-style camera pan, zoom, and unit selection. | CIV-0300 | `crates/render/src/camera.rs` | `render::rts_camera_controls` | planned |
| FR-UX-003 | A timeline scrubber SHALL display tick history and allow rewind to any stored tick. | CIV-0300 | `crates/render/src/timeline.rs` | `render::timeline_scrubber_rewind` | planned |
| FR-UX-004 | LOD transitions SHALL be visually seamless within one rendered frame. | CIV-0300 | `crates/render/src/lod.rs` | `render::lod_seamless_transition` | planned |
| FR-UX-005 | All UI state changes SHALL derive from events; no direct engine state polling. | CIV-0300 | `crates/render/src/state.rs` | `render::state_from_events_only` | planned |

---

## Assets (FR-ASSET-*)

Source specs: `docs/specs/CIV-0600-2d-assets.md`, `docs/specs/CIV-0601-3d-assets.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-ASSET-001 | All 2D tile sprites SHALL be derived from SVG sources and rasterised at build time. | CIV-0600 | `crates/render/src/atlas.rs` | `asset::svg_rasterised_at_build` | planned |
| FR-ASSET-002 | The asset pipeline SHALL pack all tile sprites into a single texture atlas per LOD level. | CIV-0600 | `crates/render/src/atlas.rs` | `asset::atlas_packed_per_lod` | planned |
| FR-ASSET-003 | Atlas build SHALL emit `asset.atlas.built.v1` on success or `asset.generation.failed.v1` on error. | CIV-0600 | `crates/render/src/atlas.rs` | `asset::atlas_build_events` | planned |
| FR-ASSET-004 | 3D assets SHALL be stored as glTF 2.0 and loaded lazily on demand. | CIV-0601 | `crates/render/src/gltf.rs` | `asset::gltf_lazy_loaded` | planned |

---

## Modding (FR-MOD-*)

Source spec: `docs/specs/CIV-0700-modding.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-MOD-001 | Mods SHALL be loaded from WASM binaries compiled against the published SDK. | CIV-0700 | `crates/engine/src/mod_loader.rs` | `modding::wasm_mod_loaded` | planned |
| FR-MOD-002 | Mod execution SHALL be sandboxed; mods SHALL NOT access host file system or network. | CIV-0700 | `crates/engine/src/mod_sandbox.rs` | `modding::sandbox_no_host_access` | planned |
| FR-MOD-003 | Mod state SHALL be persisted and restored as part of save/load (CIV-1000). | CIV-0700 | `crates/engine/src/mod_state.rs` | `modding::state_persisted_restored` | planned |
| FR-MOD-004 | The engine SHALL emit `mod.loaded.v1`, `mod.unloaded.v1`, and `mod.error.v1` events. | CIV-0700 | `crates/engine/src/mod_events.rs` | `modding::lifecycle_events_emitted` | planned |
| FR-MOD-005 | Mods SHALL be able to register new resource types, policy levers, and event handlers. | CIV-0700 | `crates/engine/src/mod_registry.rs` | `modding::can_register_resources` | planned |

---

## Audio (FR-AUD-*)

Source spec: `docs/specs/CIV-0800-audio.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-AUD-001 | Background music SHALL be driven by Kira and adapt to game state each tick. | CIV-0800 | `crates/render/src/audio.rs` | `audio::kira_initialized` | planned |
| FR-AUD-002 | Music layers SHALL fade in/out based on tension, prosperity, and war state. | CIV-0800 | `crates/render/src/audio.rs` | `audio::layers_respond_to_state` | planned |
| FR-AUD-003 | SFX SHALL be triggered by specific events (war declared, district collapsed, etc.). | CIV-0800 | `crates/render/src/audio.rs` | `audio::sfx_triggered_by_events` | planned |

---

## Session (FR-SESS-*)

Source spec: `docs/specs/CIV-0900-pve-session.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-SESS-001 | The engine SHALL support PvE (human vs AI) sessions. | CIV-0900 | `crates/engine/src/session.rs` | `session::pve_mode_supported` | planned |
| FR-SESS-002 | Hot-seat multiplayer SHALL allow multiple human players per session. | CIV-0900 | `crates/engine/src/session.rs` | `session::hotseat_multi_human` | planned |
| FR-SESS-003 | Observer mode SHALL allow read-only session access without influencing simulation. | CIV-0900 | `crates/engine/src/session.rs` | `session::observer_read_only` | planned |
| FR-SESS-004 | Challenge mode SHALL allow async submission of a civilization seed for scoring. | CIV-0900 | `crates/engine/src/challenge.rs` | `challenge::async_submission_accepted` | planned |
| FR-SESS-005 | Session speed SHALL be configurable (1x, 2x, 4x, paused) and emit `session.speed_changed.v1`. | CIV-0900 | `crates/engine/src/session.rs` | `session::speed_change_emits_event` | planned |
| FR-SESS-006 | Turn boundaries in hot-seat mode SHALL emit `session.turn.start.v1` and `session.turn.end.v1`. | CIV-0900 | `crates/engine/src/session.rs` | `session::turn_events_emitted` | planned |

---

## Save / Load (FR-SAVE-*)

Source spec: `docs/specs/CIV-1000-save-load.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-SAVE-001 | Quicksave SHALL serialize full world state to a named slot within 500 ms. | CIV-1000 | `crates/db/src/save.rs` | `save::quicksave_under_500ms` | planned |
| FR-SAVE-002 | Save SHALL emit `session.saved.v1` on success or `session.save_failed.v1` on error. | CIV-1000 | `crates/db/src/save.rs` | `save::save_events_emitted` | planned |
| FR-SAVE-003 | Load SHALL restore world state to byte-identical engine state (determinism guarantee). | CIV-1000 | `crates/db/src/load.rs` | `save::load_restores_identical_state` | planned |
| FR-SAVE-004 | Autosave SHALL trigger every N ticks (configurable, default 100). | CIV-1000 | `crates/db/src/autosave.rs` | `save::autosave_every_n_ticks` | planned |
| FR-SAVE-005 | Save format SHALL include a schema version; older saves SHALL be rejected with an error (no silent migration). | CIV-1000 | `crates/db/src/schema.rs` | `save::old_schema_rejected_explicitly` | planned |

---

## Performance (FR-PERF-*)

Source spec: `docs/specs/CIV-0500-performance.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-PERF-001 | The engine SHALL sustain 100 ms/tick (10 ticks/s) with 8 civilizations and 1,000 hex cells. | CIV-0500 | `crates/engine/src/tick.rs` | `perf::sustained_10_ticks_per_sec` | planned |
| FR-PERF-002 | Engine heap allocation per tick SHALL not exceed 1 MiB outside of initial world setup. | CIV-0500 | `crates/engine/src/tick.rs` | `perf::heap_under_1mib_per_tick` | planned |
| FR-PERF-003 | The render crate SHALL maintain 60 fps at 1080p on the reference GPU profile. | CIV-0500 | `crates/render/src/frame.rs` | `perf::render_60fps_1080p` | planned |
| FR-PERF-004 | DB write throughput SHALL not become a bottleneck for tick latency (async writes). | CIV-0500 | `crates/db/src/writer.rs` | `perf::db_writes_async_nonblocking` | planned |
| FR-PERF-005 | JSON-RPC serialization SHALL complete within 5 ms per event batch. | CIV-0500 | `crates/protocol/src/serializer.rs` | `perf::serialization_under_5ms` | planned |

---

*Last updated: 2026-02-21. Maintainer: add new FRs as specs are finalized; update Status as tests are written and pass CI.*


---
