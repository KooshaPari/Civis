# WebSocket JSON-RPC surface (`civ-server`)

**Source of truth:** [`crates/server/src/jsonrpc.rs`](../../crates/server/src/jsonrpc.rs) (`JsonRpcMethod`, `dispatch_request`, param parsers).

**Transport:** connect to `ws://<bind>/ws`, send JSON-RPC 2.0 requests as WebSocket **text** frames. Tick pushes (`Frame3d`) are separate broadcasts, not JSON-RPC responses.

**Role gate:** when `WsBridgeConfig::require_role` is true (env `CIVIS_REQUIRE_ROLE=1`), privileged methods require effective role `"operator"` from, in order: `params.role` on the request, then connection role from the `x-civis-role` WebSocket header or first-message `params.role`. Error: `-32003` (`FORBIDDEN`) with `data.required_role: "operator"`.

---

<<<<<<< HEAD
## Method catalog (21)
=======
## Method catalog (14)
>>>>>>> 2c9bf0da (add save-db coverage tests)

| Method | Role (when `require_role`) | Params | Success result (dispatch; bridge may enrich) | `ws_smoke` integration test |
|--------|----------------------------|--------|---------------------------------------------|------------------------------|
| `health` | — | `{}` or omit | `{ "tick": <u64> }` | [`ws_jsonrpc_health_returns_tick`](../../crates/server/tests/ws_smoke.rs) |
| `sim.status` | — | `{}` or omit | `{ "tick": <u64> }`; adds `"population"` when bridge has sim | [`ws_jsonrpc_sim_status_returns_tick_and_population`](../../crates/server/tests/ws_smoke.rs) |
| `sim.snapshot` | — | `{}` or omit | Full snapshot when sim available (see [Snapshot result](#simsnapshot-result)); else `{ "tick", "speed_multiplier" }` | [`ws_jsonrpc_sim_snapshot_returns_snapshot_fields`](../../crates/server/tests/ws_smoke.rs) |
| `sim.emergence` | — | `{}` or omit | Latest emergence sample when available; else `{ "tick", "sample": null }` | Unit: `sim_emergence_*` in `jsonrpc.rs` |
| `sim.subscribe` | — | `{ "frame_kinds"? \| "filter"? \| "filter_types"?, "tick_stride"?, "max_framerate_hz"?, "subscription_id"? }` | WebSocket only: `{ "subscribed": true, "subscription_id", "filter_active", "frame_kinds", "tick_stride", "current_tick" }`; plain dispatch returns `-32603` | [`ws_sim_subscribe_limits_tick_broadcast_frames`](../../crates/server/tests/ws_smoke.rs) |
| `sim.update_subscription` | — | Same as `sim.subscribe` | WebSocket only: replaces the per-connection filter and returns the same shape as `sim.subscribe`; plain dispatch returns `-32603` | Unit: `handle_sim_update_subscription_*` in `ws_bridge.rs` |
| `sim.unsubscribe` | — | `{}` or omit | WebSocket only: `{ "unsubscribed": true }`; plain dispatch returns `-32603` | Unit: `handle_sim_unsubscribe_*` in `ws_bridge.rs` |
| `sim.command` | `noop`: —; `tick`: **operator** | `{ "action": "noop" \| "tick", "role"? }` | `noop`: `{ "accepted": true }`; `tick`: `{ "accepted": true, "tick": <u64> }` (tick updated after advance) | `tick`: [`ws_jsonrpc_sim_command_tick_advances_tick`](../../crates/server/tests/ws_smoke.rs), [`ws_jsonrpc_sim_command_tick_rejects_missing_role_when_required`](../../crates/server/tests/ws_smoke.rs), [`ws_jsonrpc_sim_command_tick_accepts_x_civis_role_header`](../../crates/server/tests/ws_smoke.rs); F3D0 broadcast: `ws_sim_command_tick_broadcasts_f3d0_*` |
| `sim.save_replay` | — | `{ "path": <non-empty string> }` | `{ "saved": true, "path": <string> }` | [`ws_jsonrpc_sim_save_and_load_replay_roundtrip`](../../crates/server/tests/ws_smoke.rs) |
| `sim.load_replay` | — | `{ "path": <non-empty string> }` | `{ "loaded": true, "tick": <u64> }` | [`ws_jsonrpc_sim_save_and_load_replay_roundtrip`](../../crates/server/tests/ws_smoke.rs) |
| `sim.reset` | — | `{ "seed": <u64> }` **required** | `{ "seed": <u64>, "tick": 0 }` | [`ws_jsonrpc_sim_reset_replaces_simulation_and_zeroes_tick`](../../crates/server/tests/ws_smoke.rs) |
| `sim.set_policy` | — | `{ "scarcity_multiplier": <f64≥0>, "base_consumption_joules"? }` | `{ "updated": true, "scarcity_multiplier": <f64> }`; bridge adds `base_consumption_joules` after apply | [`ws_jsonrpc_sim_set_policy_rejects_nan_scarcity`](../../crates/server/tests/ws_smoke.rs), [`ws_jsonrpc_sim_set_policy_zero_scarcity_tick_preserves_energy_budget`](../../crates/server/tests/ws_smoke.rs) |
| `sim.set_speed` | — | `{ "multiplier": 0 \| 1 \| 2 \| 4 \| 8 }` | `{ "accepted": true, "multiplier": <u32> }` | [`ws_jsonrpc_sim_set_speed_accepts_valid_multiplier`](../../crates/server/tests/ws_smoke.rs), [`ws_jsonrpc_sim_set_speed_rejects_invalid_multiplier`](../../crates/server/tests/ws_smoke.rs) |
| `sim.get_speed` | — | omit or `{}` | `{ "multiplier": <u32> }` | [`ws_jsonrpc_sim_set_speed_accepts_valid_multiplier`](../../crates/server/tests/ws_smoke.rs) (paired with set) |
| `sim.spawn_civilian` | **operator** | `{ "x", "y": <f64 normalized 0–1>, "faction"? }` default faction `0` | Dispatch: `{ "accepted": true }`; bridge: `{ "accepted", "ok", "entity_id" }` | [`ws_jsonrpc_sim_spawn_civilian_returns_entity_id`](../../crates/server/tests/ws_smoke.rs), [`ws_jsonrpc_spawn_civilian_pin_appears_in_snapshot`](../../crates/server/tests/ws_smoke.rs) |
| `sim.spawn_entity` | **operator** | `{ "kind", "x", "y", "faction"? }` — `kind`: `civilian` \| `vehicle` \| `airport` \| `port` \| `hangar` | Dispatch: `{ "accepted": true, "kind": <wire label> }`; bridge adds `ok`, `entity_id` | [`ws_jsonrpc_sim_spawn_entity_vehicle_returns_entity_id`](../../crates/server/tests/ws_smoke.rs) |
| `sim.place_voxel` | **operator** | `{ "x", "y", "z": <i64 world>, "material"? }` default `0` | Dispatch: `{ "accepted": true }`; bridge: `{ "accepted", "ok": true }` | — (unit: `parse_place_voxel_params_reads_coords` in `jsonrpc.rs`) |
| `sim.damage` | **operator** | `{ "x", "y", "z": <i64>, "radius"? }` default `8` clamped 1–32, `"energy"?` default `1000` | Dispatch: `{ "accepted": true }`; bridge: `{ "accepted", "ok", "queued": true }` (applied next tick) | [`ws_jsonrpc_sim_damage_accepts_event`](../../crates/server/tests/ws_smoke.rs) |

**Invalid `sim.command` action:** `-32601` `Method not found` (not `-32602`).

**Parse / protocol errors (not methods):** invalid JSON → `-32700`; bad request shape / batch → `-32600`; unknown method name → `-32601`. Covered by [`ws_jsonrpc_invalid_json_returns_parse_error`](../../crates/server/tests/ws_smoke.rs) and `jsonrpc.rs` unit tests.

---

## `sim.snapshot` result

When the bridge supplies live `SnapshotFields`, the result may include:

| Field | Notes |
|-------|--------|
| `tick`, `population`, `building_count` | Always |
| `market_prices` | `BTreeMap` good → cents |
| `speed_multiplier` | Bridge multiplier |
| `energy_budget`, `hash_chain_root` | Omitted when unset |
| `civ_pins`, `factions`, `buildings`, `is_day` | From `SpectatorView` when present |
| `institutions` | `{ id, kind, balance_joules }[]` when non-empty |
| `military_units` | Pin rows (`unit_type` e.g. `Vehicle` for knights) |
| `damage_events`, `damage_events_count`, `voxel_damage_removed_this_tick` | Tactical damage telemetry |

---

## HTTP routes (not JSON-RPC)

Documented alongside WS in root [`README.md`](../../README.md): `GET /healthz`, `GET /replay/export`, `POST /replay/import`. Integration tests: `healthz_returns_ok_with_tick`, `replay_*` in [`ws_smoke.rs`](../../crates/server/tests/ws_smoke.rs).

---

## Drift watchlist

| Location | Issue |
|----------|--------|
| [`README.md`](../../README.md) | Lists `sim.spawn_entity` kinds as civilian \| vehicle \| airport only; code also supports `port`, `hangar`. |
| [`README.md`](../../README.md) | `sim.snapshot` fallback described as `{ "tick" }` only; code also returns `speed_multiplier`. |
| [`docs/api/index.md`](index.md) | Still marks WebSocket JSON-RPC as **Planned**; implementation is live in `civ-server`. |

Regenerate this page when adding variants to `JsonRpcMethod` in `jsonrpc.rs`.

---

## Related

- Spec: [`docs/specs/CIV-0200-client-protocol.md`](../specs/CIV-0200-client-protocol.md)
- Maturity: [`docs/development-guide/fr-ax-dx-ux-maturity-audit.md`](../development-guide/fr-ax-dx-ux-maturity-audit.md) (DX-03)
- Run WS tests: `cargo test -p civ-server --test ws_smoke --quiet`
