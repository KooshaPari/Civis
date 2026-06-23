# Coverage Gap Audit — 2026-06-15

Source: CI `Test Numbers` coverage job (llvm-cov `--summary-only`), run for
commit `a5f1537d` on `feat/sim-emergence-batch`. VERIFIED numbers, not estimates.

## Workspace total (VERIFIED)

| Metric | Covered | Total | % |
|--------|---------|-------|---|
| Regions | 47601 | 53588 | **88.83%** |
| Functions | 3185 | 3714 | **85.76%** |
| Lines | 32093 | 36475 | **87.99%** |

Trend: 87.78% → 87.99% line. Above the project's 90% target on regions-adjacent
but the line/function metrics have headroom, concentrated in one crate.

## Lowest line-coverage files (test-lane targets, ascending)

| Line % | File | Notes / lane |
|--------|------|--------------|
| 0.00 | `crates/watch/src/main.rs` | binary entrypoint — likely needs an integration smoke or `#[cfg(test)]` arg-parse split; low ROI to unit-test directly |
| 23.36 | `crates/watch/src/server.rs` | **HIGH ROI** — WS/HTTP server wiring; add route/handler tests |
| 33.14 | `crates/watch/src/sim_worker.rs` | **HIGH ROI** — live-sim tick worker; add step/snapshot tests |
| 55.92 | `crates/watch/src/saves_api.rs` | save/load endpoints; add round-trip tests |
| 58.67 | `crates/watch/src/app.rs` | app/router assembly; add builder tests |
| 61.83 | `crates/watch/src/snapshot.rs` | snapshot serialization; add field-presence tests |
| 69.52 | `crates/watch/src/mods_api.rs` | mod listing endpoints |
| 80.00 | `crates/voxel/src/material.rs` | material table; cheap enum/table tests |
| 80.88 | `crates/watch/src/control_routes.rs` | sim control endpoints |
| 86.16 | `crates/server/src/ws_bridge.rs` | WS JSON-RPC bridge |

## Finding

`crates/watch` (the live-sim WS worker/server) is the dominant coverage gap —
7 of the 10 lowest files. It is exactly the surface the emergent-systems work
feeds (sim_worker drives the tick loop; snapshot/server expose market_prices,
civ_pins, etc. that the ws_smoke tests exercise). Raising `server.rs` (23%) and
`sim_worker.rs` (33%) is the highest-leverage path to the 90% line target and
also hardens the live-sim API the new couplings depend on.

## Converted lanes (refill DAG)

1. `watch/sim_worker.rs` tick + snapshot tests → target 33% → 70%+
2. `watch/server.rs` route/handler tests → target 23% → 60%+
3. `watch/snapshot.rs` field-presence tests (mirror ws_smoke assertions in-crate)
4. `voxel/material.rs` cheap table tests (80% → 95%, low effort)

Each lane is additive, in-crate `#[cfg(test)]`, and CI-validated via Test Numbers.
