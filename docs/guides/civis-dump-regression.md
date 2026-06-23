# CIVIS_DUMP render-frame regression

Machine-level verification for the Bevy reference client using the **`CIVIS_DUMP`** JSON document emitted by `clients/bevy-ref/src/scene_dump.rs`. This gate inspects scene graph + sim counters — not pixels — so it catches floating actors, collapsed terrain meshes, and empty pre-spawn scenes without a GPU read.

## Producer (Bevy client)

Set environment variables before launching the standalone client:

| Variable | Purpose |
|----------|---------|
| `CIVIS_DUMP=<path>` | Write scene+sim JSON after warmup, then exit (unless keep-alive) |
| `CIVIS_DUMP_WARMUP=<seconds>` | Warmup before dump (default `6.0`) |
| `CIVIS_DUMP_KEEP_ALIVE=1` | Windowed run: dump without exiting (for animation checks) |

Example (headless one-shot):

```powershell
$env:CIVIS_DUMP = "target/civis-dump/scene.json"
$env:CIVIS_DUMP_WARMUP = "6"
$env:BEVY_ASSET_ROOT = "clients/bevy-ref"
cargo run -p civ-bevy-ref --features bevy,egui --bin civ-standalone
```

The client also prints marker-wrapped JSON on stdout:

```text
=== CIVIS_DUMP BEGIN ===
{ ... }
=== CIVIS_DUMP END ===
```

## Consumer (`civis-dump`)

The harness lives in `crates/civis-cli` (`civis-dump` bin + [`dump`](../../crates/civis-cli/src/dump.rs) library module). It does **not** link Bevy — CI validates fixtures with `cargo test -p civis-cli`.

### Policy check (invariants)

```powershell
cargo run -p civis-cli --bin civis-dump -- check target/civis-dump/scene.json
cargo run -p civis-cli --bin civis-dump -- check --policy headful target/civis-dump/scene.json
cargo run -p civis-cli --bin civis-dump -- check --from-markers capture.log
```

| Policy | Use when |
|--------|----------|
| `headless` (default) | `CIVIS_DUMP` one-shot without window / GLTF scenes |
| `headful` | Windowed run with `CIVIS_DUMP_KEEP_ALIVE=1`; requires `animation.players > 0` when actors exist |

Override via `CIV_DUMP_POLICY=headless|headful`.

### Baseline diff (regression)

Commit a golden dump under `crates/civis-cli/fixtures/dump/` (or your scenario folder) and diff:

```powershell
cargo run -p civis-cli --bin civis-dump -- diff `
  crates/civis-cli/fixtures/dump/good-headless.json `
  target/civis-dump/scene.json
```

Tolerances: `--resource-tol`, `--placement-tol`, `--count-delta`.

### Extract markers

```powershell
Get-Content capture.log | cargo run -p civis-cli --bin civis-dump -- extract > scene.json
```

## JSON schema (authoritative fields)

Top-level keys written by `scene_dump.rs`:

| Section | Proves |
|---------|--------|
| `sim` | Tick, population, citizen/building counts, resources |
| `voxel` | Grid dims, non-air census, water %, surface height samples |
| `meshes` | Chunk mesh count + origin AABB spread (dissolved terrain) |
| `actors` / `buildings` | Count, floating tally, sample placements vs `surface_y` |
| `animation` | AnimationPlayer census (headless may read `0` — not authoritative there) |
| `total_entities` | ECS sanity |

**Headless caveat:** GLTF `SceneRoot` hierarchies do not instantiate without a window, so `animation.players == 0` is expected in headless dumps even when windowed runs would show clips playing. Use `headless` policy in CI; reserve `headful` for manual/windowed regression.

## Gates enforced by `headless` policy

| Gate | Failure means |
|------|----------------|
| `sim.present` | No live sim bridge attached |
| `voxel.present` / `voxel.non_air_cells` | Empty or missing voxel layer |
| `meshes.count` / `meshes.spread` | No terrain meshes or collapsed world extent |
| `actors.floating` / `buildings.floating` | Entities more than 1 m off terrain surface |
| `total_entities` | Pre-spawn / empty scene dumped too early |

## CI

Workflow [`.github/workflows/civis-dump-regression.yml`](../../.github/workflows/civis-dump-regression.yml) runs `cargo test -p civis-cli` (fixture-backed unit tests). Full client capture remains a local/agent step: produce `CIVIS_DUMP`, then `civis-dump check` or `diff`.

## Agent evidence

When claiming render/scene correctness, cite **`civis-dump` JSON output** (pass/fail + violation gates) alongside pixel stats from `civis-pixels` where applicable. Do not narrate screenshots without machine numbers.
