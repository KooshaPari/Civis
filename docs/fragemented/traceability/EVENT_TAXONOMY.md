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
| `metrics.threshold.critical.v1` | A derived metric has crossed the critical threshold (&gt; 75 % of max). | `metric: string`, `civ_id: uuid \| null`, `value: f32`, `critical_threshold: f32`, `tick: u64` | engine/metrics | ALL | on-condition |

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
