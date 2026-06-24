# civ-watch snapshot/debug run

Date: 2026-05-28

## What I ran

- Started `cargo run -p civ-watch --release` in the background.
- Waited 5 seconds.
- Probed `http://localhost:9090/snapshot`.
- Probed `http://localhost:9090/events`.
- Checked `http://localhost:9090/api/snapshot` and `http://localhost:9090/api/events` as a prefix sanity check.

## Live results

- `GET /snapshot` returned `404 page not found`.
- `GET /events` returned `404 page not found`.
- `GET /api/snapshot` returned `404 page not found`.
- `GET /api/events` returned `404 page not found`.
- `GET /` returned `302 Found` redirecting to `/graph`.

## What that means

- The live process on port `9090` was `civ-watch.exe`, but it was not exposing the documented API routes at the time of the probe.
- Because the snapshot endpoint itself returned 404, I could not inspect the JSON payload for civilians, buildings, or factions.
- SSE could not be verified for the same reason. The failure is at route exposure, not at JSON content.

## Code-path check

Relevant code paths in the repo indicate the intended behavior is:

- `crates/watch/src/server.rs` registers `GET /events` and `GET /snapshot`.
- `crates/watch/src/sse.rs` serves `snapshot_handler` from `state.latest`.
- `crates/watch/src/sim_worker.rs` calls `sim.tick()` inside the background loop before building and publishing a snapshot.
- `crates/engine/src/engine.rs` `Simulation::new()` already spawns initial entities, faction civilians, and attaches citizen data.

So the engine/worker path does not look like the source of an empty snapshot. The observable problem is that the running watch server was not serving the expected routes.

## Notes

- I killed the `civ-watch` process after the probe.
- If the current binary is expected to expose these endpoints, the next step is to inspect why the active listener on `9090` is serving `/graph` but not the documented API routes.
