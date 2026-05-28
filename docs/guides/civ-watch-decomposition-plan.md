# civ-watch decomposition plan

## Goal

Split `crates/watch/src/main.rs` into focused modules without changing behavior:

- `crates/watch/src/server.rs`
- `crates/watch/src/sim_worker.rs`
- `crates/watch/src/snapshot.rs`
- `crates/watch/src/control_routes.rs`
- `crates/watch/src/sse.rs`
- `crates/watch/src/mods_api.rs`
- `crates/watch/src/saves_api.rs`
- `crates/watch/src/main.rs`

## Current risk profile

`crates/watch/src/main.rs` is a full monolith that currently contains:

- process bootstrap and `main()`
- shared app state
- terrain cache
- simulation worker
- snapshot synthesis
- SSE endpoint
- control routes
- saves APIs
- mods APIs
- a large test suite

That makes an all-at-once manual split high risk for regressions in route behavior, snapshot shape, and state ownership.

## Proposed decomposition order

1. Create a new shared module for common types and state.
   - Keep `AppState`, `Snapshot`, request/response DTOs, terrain cache, and shared constants in one place.
   - Expose only what the route modules need.

2. Extract `snapshot.rs`.
   - Move `make_snapshot` and the snapshot helper functions it owns.
   - Keep it pure: `Simulation` + current state in, `Snapshot` out.

3. Extract `sim_worker.rs`.
   - Move the 10 Hz background loop.
   - Leave all simulation-side mutations and snapshot publishing there.

4. Extract `sse.rs`.
   - Move `GET /events` handling and stream construction.

5. Extract `saves_api.rs`.
   - Move `GET /control/saves` and slot/save/load handler logic.
   - Keep save-db access and filesystem listing behavior intact.

6. Extract `mods_api.rs`.
   - Move the mod catalog, published mods, remote mods, upload, fetch, install, unload, reload endpoints.

7. Extract `control_routes.rs`.
   - Move the remaining `/control/*` handlers: voxel placement, spawn, damage, speed, reset/pause-style controls.

8. Extract `server.rs`.
   - Build the Axum router.
   - Own server bootstrap, CORS/fallback/static hosting, and listener bind/serve.

9. Reduce `main.rs` to a thin entrypoint.
   - Initialize tracing.
   - Build app state.
   - Call `server::run(state).await`.

## Implementation notes

- Prefer `pub(crate)` visibility, not `pub`, unless a module boundary requires broader access.
- Keep shared helpers single-sourced. Do not duplicate:
  - `resolve_data_dir`
  - `resolve_session_id`
  - `TerrainCache`
  - `AppState`
  - route DTOs
  - `REMOTE_*` constants
- Preserve the current route table exactly unless a regression fix is explicitly required.
- Keep the worker tick rate at `10 Hz`.
- Do not change snapshot JSON field names or SSE event payloads.

## Validation gates

Run in this order after each extraction step:

1. `cargo check -p civ-watch`
2. `cargo test -p civ-watch` if the crate-specific check is green

If the split is completed, re-run the scoped compile check from the repo root:

```powershell
cargo check -p civ-watch
```

## Stop condition

If a step introduces unresolved compile errors that require broad rework of shared state or route ownership, stop the extraction and keep `main.rs` untouched for that step. Resume from the last green boundary rather than forcing the entire split in one edit.
