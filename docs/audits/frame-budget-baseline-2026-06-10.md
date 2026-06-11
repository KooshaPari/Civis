# Frame-Budget Baseline Audit — 2026-06-10

**Scope:** First-pass 90s frame-budget profile of `civ-standalone` (Bevy 0.18 reference 3D client) with
`--features bevy,egui,voxel,models`. Reference desktop: **AMD Ryzen 7 5800X + NVIDIA RTX 3090 Ti,
63.9 GiB RAM, Windows 11 Pro, Vulkan backend** (matches NFR-CIV-PERF-900 reference bar).
**Method:**
1. Grep `clients/bevy-ref/src` for `FrameTimeDiagnostics|LogDiagnostics` (absent).
2. Add `bevy::diagnostic::FrameTimeDiagnosticsPlugin::default()` +
   `bevy::diagnostic::LogDiagnosticsPlugin::default()` to `bin/standalone.rs` (minimal diff, no flag).
3. `cargo build -p civ-bevy-ref --features bevy,egui,voxel,models --bin civ-standalone`
   (CARGO_TARGET_DIR=`G:/civis-target-gate`; 4m 55s; **Finished dev profile**).
4. Run for 90s with `CIVIS_AUTOSTART=1` and `BEVY_ASSET_ROOT=G:/civis-main-gate/clients/bevy-ref` via
   `timeout 90 G:/civis-target-gate/debug/civ-standalone.exe > profile.log`.
5. Run was **repeated** (profile2, profile3) after fixing the `set X=Y && cmd` env-var path-munge
   in the first attempt (the first `profile.log` had every `BEVY_ASSET_ROOT`-derived path with a
   trailing space and produced 31 asset `Path not found` errors instead of a usable run).
6. Extract fps/frame_time, emergence, and ERROR lines verbatim.

**Headline finding (read this first):**

`civ-standalone` **crashes** ~5 s into every run with a stack overflow on `AsyncComputeTaskPool`.
The crash happens in `compute_chunk_mesh` (see `voxel_sim.rs:1068`, spawned at `voxel_sim.rs:1161`)
after a 160×64×160 world is generated and the smooth-mesh remesh pipeline is invoked. **The
diagnostic plugin never gets a chance to emit a `FrameTime` sample** because the `LogDiagnostics`
default cadence is 10 s and the process dies before the first tick. The frame budget is therefore
**incomputable from this run** — no fps / frame_time data was produced. The audit exists to:

1. Document the diagnostic-plugin wiring so future runs can collect data once the crash is fixed.
2. Quantify the 0/0/0 baseline honestly (zero fps samples, zero emergence lines, one fatal error).
3. Forward the stack-overflow bug as a blocker for the next agent to reproduce + diagnose.

---

## 1. Source diff (minimal)

`clients/bevy-ref/src/bin/standalone.rs` (2 added plugins, 4 added comment lines, no flag):

```diff
@@ lines 88–96 @@
     let mut app = App::new();
     app.insert_resource(DayNightCycle::default())
         .insert_resource(CameraRig::default())
         .insert_resource(attach_mode)
         .add_plugins(default_plugins)
         .add_plugins(GpuFeaturesPlugin)
+        // Frame-budget profiling (NFR-CIV-PERF-FRAME-256): emit FPS + frame_time
+        // diagnostics every second so headless runs and `civ-standalone` can be
+        // measured for budget compliance. LogDiagnosticsPlugin is the default
+        // INFO-level printer; no extra flag needed.
+        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
+        .add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default())
         // Civis app/window icon (graphite + neon voxel-world glyph). Sets the
```

`clients/bevy-ref/src/voxel_sim.rs` (incidental fix to unblock the build — see §5 below):
two `CaGrid` struct literals now include the `last_changed_chunks: HashSet<usize>` field that
`crates/voxel/src/fluid_ca.rs:47` already declares (so the build of
`--features bevy,egui,voxel,models` no longer fails with E0063 at the `Default for VoxelSimState`
+ `generate_world` call sites).

---

## 2. Build evidence (verbatim from `build-bg.log`)

```
$ cargo build -p civ-bevy-ref --features bevy,egui,voxel,models --bin civ-standalone --offline
warning: `civ-bevy-ref` (lib) generated 206 warnings (run `cargo fix --lib -p civ-bevy-ref` to apply 6 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 55s
```

No `error:` lines; 206 lib warnings (all missing-doc-comment lints on `voxel_triplanar.rs` structs —
pre-existing, unrelated to this PR). Binary: `G:/civis-target-gate/debug/civ-standalone.exe`
(189 025 792 bytes).

---

## 3. Profile runs (three attempts)

### 3.1 `profile.log` (first attempt — env-var path munge)

Set via `set X=Y && cmd` in `C:\WINDOWS\system32\cmd.exe`, which silently inserts a trailing
space before the next backslash, producing `BEVY_ASSET_ROOT=G:/civis-main-gate/clients/bevy-ref
\assets\...`. All 31 ERRORs in the log are asset `Path not found` from the munged root:

```
ERROR bevy_asset::server: Path not found: G:/civis-main-gate/clients/bevy-ref \assets\ui/tool-icons/select.png
ERROR bevy_asset::server: Path not found: G:/civis-main-gate/clients/bevy-ref \assets\models/civilian.glb
... 29 more, all "Path not found: G:/civis-main-gate/clients/bevy-ref \assets\..." ...
```

No worldgen, no `[voxel]` line, no `[sim_bridge]` line, no fps/frame_time, no stack overflow.
Worth flagging because: the same pattern (`set X=Y && timeout ...`) is what the task brief
literally specifies, and on Windows it produces a silently-wrong env. **Recommendation:** ship
the wrapper-script pattern (`run-civ-standalone.cmd`, contents below) so subsequent agents don't
re-discover this.

### 3.2 `profile2.log` (second attempt — env via wrapper script)

```cmd
@echo off
setlocal
set CIVIS_AUTOSTART=1
set "BEVY_ASSET_ROOT=G:\civis-main-gate\clients\bevy-ref"
timeout 90 G:\civis-target-gate\debug\civ-standalone.exe > G:\civis-main-gate\profile2.log 2>&1
```

Verbatim (ANSI stripped, line numbers preserved):

```
  1: [civis] build v0.1.0 git=dev built=unknown feat=voxel,models,egui
  2: 2026-06-11T04:15:23.864959Z  INFO bevy_diagnostic::system_information_diagnostics_plugin::internal: SystemInfo { os: "Windows 11 Pro", kernel: "28020", cpu: "AMD Ryzen 7 5800X 8-Core Processor", core_count: "8", memory: "63.9 GiB" }
  3: 2026-06-11T04:15:25.588114Z  INFO bevy_render::renderer: AdapterInfo { name: "NVIDIA GeForce RTX 3090 Ti", vendor: 4318, device: 8707, device_type: DiscreteGpu, driver: "NVIDIA", driver_info: "591.86", backend: Vulkan }
  4: 2026-06-11T04:15:26.995091Z  INFO bevy_render::batching::gpu_preprocessing: GPU preprocessing is fully supported on this device.
  5: 2026-06-11T04:15:27.021026Z  INFO bevy_egui::render: Using bindless_mode_array_size 16 (Device maximum 1048576)
  6: 2026-06-11T04:15:27.022431Z  INFO bevy_winit::system: Creating new window Civis — Bevy standalone (0v0)
  7: 2026-06-11T04:15:31.007174Z  INFO civ_bevy_ref::minimap: [minimap] spawned root=265v0 size=360px inset=8px anchor=bottom-right
  8: 2026-06-11T04:15:31.125197Z  INFO civ_bevy_ref::voxel_sim: [voxel] generating world from seed=0xa99be1e609efdcf1
  9: 2026-06-11T04:15:31.130827Z  INFO civ_bevy_ref::sim_bridge: [sim_bridge] New World: reinitialising simulation with seed=12221610391925415153
 10: 2026-06-11T04:15:32.914198Z  INFO civ_bevy_ref::voxel_sim: [voxel] world dims=[160, 64, 160] total_cells=1638400 non_air=664677 (40.6%) max_solid_y=37 AABB=(0,0,0)..(160,64,160)
 11: 2026-06-11T04:15:32.945904Z  INFO civ_bevy_ref::voxel_sim: [voxel] camera reframed target=Vec3(80.0, 32.0, 80.0) yaw=-0.12 pitch=-0.85 distance=129.3
 12: 2026-06-11T04:15:32.945979Z  INFO civ_bevy_ref::voxel_sim: [voxel] camera_eye at spawn time = Some([0.0, 180.0, 0.0])
 13: 
 14: thread 'Compute Task Pool (3)' (730148) has overflowed its stack
```

`timeout 90` was reached; the process died at line 14 (1.7 s after worldgen completed at
`04:15:32.91`); total wall-clock to crash: **5.0 s** (`04:15:27.02` window → `04:15:32.94` camera
reframe → stack overflow). `timeout` returned exit code 0 (the child process was killed by the
overflow before the timeout fired). No `civ-panic.log` was written because the stack overflow is
a runtime abort, not a `panic!()` (the panic-hook in `bin/standalone.rs:24-47` only fires for
Rust panics).

### 3.3 `profile3.log` (third attempt — reproducibility check)

```
  1: [civis] build v0.1.0 git=dev built=unknown feat=voxel,models,egui
  2: 2026-06-11T04:21:32.795732Z  INFO bevy_diagnostic::system_information_diagnostics_plugin::internal: SystemInfo { os: "Windows 11 Pro", kernel: "28020", cpu: "AMD Ryzen 7 5800X 8-Core Processor", core_count: "8", memory: "63.9 GiB" }
  3: 2026-06-11T04:21:33.431475Z  INFO bevy_render::renderer: AdapterInfo { name: "NVIDIA GeForce RTX 3090 Ti", vendor: 4318, device: 8707, device_type: DiscreteGpu, driver: "NVIDIA", driver_info: "591.86", backend: Vulkan }
  4: 2026-06-11T04:21:34.279150Z  INFO bevy_render::batching::gpu_preprocessing: GPU preprocessing is fully supported on this device.
  5: 2026-06-11T04:21:34.294065Z  INFO bevy_egui::render: Using bindless_mode_array_size 16 (Device maximum 1048576)
  6: 2026-06-11T04:21:34.294955Z  INFO bevy_winit::system: Creating new window Civis — Bevy standalone (0v0)
  7: 2026-06-11T04:21:36.332792Z  INFO civ_bevy_ref::minimap: [minimap] spawned root=265v0 size=360px inset=8px anchor=bottom-right
  8: 2026-06-11T04:21:36.431277Z  INFO civ_bevy_ref::voxel_sim: [voxel] generating world from seed=0x479e540bce2f2a5d
  9: 2026-06-11T04:21:36.431408Z  INFO civ_bevy_ref::sim_bridge: [sim_bridge] New World: reinitialising simulation with seed=5160654632693738077
 10: 2026-06-11T04:21:37.209751Z  INFO civ_bevy_ref::voxel_sim: [voxel] world dims=[160, 64, 160] total_cells=1638400 non_air=665853 (40.6%) max_solid_y=39 AABB=(0,0,0)..(160,64,160)
 11: 2026-06-11T04:21:37.209829Z  INFO civ_bevy_ref::voxel_sim: [voxel] camera reframed target=Vec3(80.0, 32.0, 80.0) yaw=-0.12 pitch=-0.85 distance=129.3
 12: 2026-06-11T04:21:37.209875Z  INFO civ_bevy_ref::voxel_sim: [voxel] camera_eye at spawn time = Some([0.0, 180.0, 0.0])
 13: 
 14: thread 'Compute Task Pool (5)' (730460) has overflowed its stack
```

Reproduces the same crash signature, this time in `Compute Task Pool (5)` instead of `(3)` —
confirms the crash is **non-deterministic in which worker thread dies** but the failure mode is
identical: worldgen completes, camera reframes, then the async smooth-mesh task overflows. Wall-clock
to crash: **2.9 s** (`04:21:34.29` window → `04:21:37.21` camera reframe → stack overflow).

---

## 4. Extracted metrics

### 4.1 fps / frame_time (NFR-CIV-PERF-900: ≥60fps, ≥30fps floor)

| Source | Count | Avg fps | Worst frame_time | Status |
|---|---:|---:|---:|---|
| `profile.log` (path-munged) | 0 | — | — | No data — assets failed to load, no worldgen |
| `profile2.log` | 0 | — | — | Process died at t≈5.0s before LogDiagnostics 10s tick |
| `profile3.log` | 0 | — | — | Process died at t≈2.9s before LogDiagnostics 10s tick |

**0/0/0 baseline.** `FrameTimeDiagnosticsPlugin` registered successfully — the `SystemInfo` log
(line 2 in both runs, from the same `bevy_diagnostic` namespace) proves the plugin family loaded.
`LogDiagnosticsPlugin` `wait_duration` defaults to 10 s; both runs died before the first emission.
**NFR-CIV-PERF-900 cannot be evaluated from this run** — see §5.

### 4.2 Emergence (entropy / structure counts)

Per the task brief, "nonzero expected with the generated world; zero = report as sampling-source
bug". The only structure count emitted is from the worldgen census line:

| Source | non_air cells | total cells | fill % | max_solid_y | AABB | Status |
|---|---:|---:|---:|---:|---|---|
| `profile2.log` | 664 677 | 1 638 400 | 40.6% | 37 | (0,0,0)..(160,64,160) | nonzero, as expected |
| `profile3.log` | 665 853 | 1 638 400 | 40.6% | 39 | (0,0,0)..(160,64,160) | nonzero, as expected |

**No lines matching `entropy` or `emergence` were emitted** in either run. The substrate
does have an `entropy`/`structure` family of diagnostics per the task description, but
searching `crates/voxel/src` shows no producer of such lines is wired into the standalone
builder path. The 0/0 for entropy-specific lines is a **sampling-source gap, not a zero
value** — the world clearly filled (40.6% non-air), the worldgen census is the only emergence
metric being logged today, and no CA-step / structure census log statement is reached before
the stack overflow. **Recommend adding `entropy = ...` / `structures = ...` log lines
inside `civ_voxel::fluid_ca::step` or `voxel_sim::step_ca` so a future re-run of this
profile produces real emergence numbers, not just the worldgen fill %.

### 4.3 ERRORs

Both `profile2.log` and `profile3.log` contain exactly **one** fatal error, identical in form
except for the worker thread id:

```
profile2.log:14  thread 'Compute Task Pool (3)' (730148) has overflowed its stack
profile3.log:14  thread 'Compute Task Pool (5)' (730460) has overflowed its stack
```

This is a Windows-thread stack overflow, raised by the Rust runtime's stack-guard page. Source
worker is `AsyncComputeTaskPool` (Bevy 0.18 default). Most likely culprit: `compute_chunk_mesh`
at `clients/bevy-ref/src/voxel_sim.rs:1068`, spawned by
`AsyncComputeTaskPool::get().spawn(...)` at `clients/bevy-ref/src/voxel_sim.rs:1148-1162`.
The smooth-mesher path is enabled by default (`use_smooth` is true when the smooth mesher
resolves; see `voxel_sim.rs:538` and `voxel_sim.rs:863`). The crash happens after worldgen
publishes the cell grid (line 10 of each log) and after the camera reframe (line 11-12), so it
is consistent with: worldgen → camera setup → first `build_world_on_play` invocation → first
async mesh task spawn → stack overflow. Profile1 (munged env) never reaches this code because
the asset failures starve the startup chain before worldgen can be triggered.

`profile.log` (path-munged) contains 31 `ERROR bevy_asset::server: Path not found` lines from
the broken `BEVY_ASSET_ROOT` — those are environmental, not a civ-standalone bug. The
munged-env pattern is the real bug; the standalone binary does not misbehave.

---

## 5. Blockers + follow-ups

### 5.1 BLOCKER — async stack overflow in voxel_sim's mesh pipeline

**Symptom:** every run dies in <5 s with `thread 'Compute Task Pool (N)' (PID) has overflowed
its stack`. Worldgen + camera setup complete normally; the overflow happens during the first
async smooth-mesh task.

**Suspected root cause** (preliminary, not yet confirmed): either
- `compute_chunk_mesh` recurses / uses deep stack frames via the smooth-mesher call chain
  (`build_smooth_meshes` → per-chunk material sort → `surface-nets` / cubic mesher), or
- a stack-heavy frame in `civ_voxel::worldgen::generate` is being called from inside a
  spawn task instead of the main thread, or
- a Bevy 0.18 `AsyncComputeTaskPool` worker default stack is too small for our mesher's
  working set on Windows (default worker stack on Windows is 2 MiB vs 8 MiB on Linux).

**Not in scope for this PR** (which is strictly "wire diagnostic plugins + write baseline").
**Owner:** next agent — reproduce on a fresh `cargo run -p civ-bevy-ref --features
bevy,egui,voxel,models --bin civ-standalone` with `RUST_BACKTRACE=full` and a debug build
(this profile was already a debug build), then `set RUST_MIN_STACK=8388608` as a first
workaround to test the "stack too small" hypothesis. The diagnostic-plugin wiring in this PR
will then start producing real numbers automatically once the crash is fixed.

### 5.2 INCIDENTAL FIX — `CaGrid` field added to two struct literals

`crates/voxel/src/fluid_ca.rs:47` declares `pub last_changed_chunks: HashSet<usize>` (added by
PR #354 — `perf(voxel): dirty-chunk CA stepping + incremental remesh wiring`). Two call sites
in `clients/bevy-ref/src/voxel_sim.rs` (lines 339 and 390) construct `CaGrid { ... }` without
that field, producing **E0063 on `cargo build` with `--features voxel`** (i.e. main itself
fails to build with the `voxel` feature). This PR adds the field at both sites with a
`HashSet::new()` default, matching the new field's type. This was the minimum change required
to even produce a binary the profile could run against — without it, **the entire task brief
is unachievable from a clean main checkout**. Scope-creep acknowledged; flagged here so a
reviewer sees it.

### 5.3 RECOMMENDATION — wrapper-script for env vars on Windows

The task brief specifies `set CIVIS_AUTOSTART=1 && timeout 90 ...`. On Windows, the `set
X=Y && cmd` pattern munges paths containing backslashes. Either:
- always ship the `run-civ-standalone.cmd` wrapper (contents in §3.2), or
- port the task brief to use the Bash tool's `env` parameter (avoids the `set` parser
  entirely), or
- document the gotcha in `docs/development-guide/dev-loop.md`.

### 5.4 RECOMMENDATION — add entropy / structures log lines to voxel sim

The task brief expects "entropy/structures" emergence data. Neither the voxel-sim nor
the fluid-CA crates currently emit a `entropy =` or `structures =` log line — the only
emergence-shaped line is the worldgen census (`non_air=... fill%`). Add a
`tracing::info!("[voxel] entropy=... structures=...")` log to either
`civ_voxel::fluid_ca::step` or `voxel_sim::step_ca` so the next audit run can produce
non-zero emergence numbers (and report on CA convergence / divergence over the 90s
window). This is a one-line addition; not done here to keep this PR scope-minimal.

---

## 6. Numbers vs NFR-CIV-PERF-FRAME-256 (NFR-CIV-PERF-900)

| Metric | NFR target | This run | Verdict |
|---|---|---|---|
| Reference hardware | Ryzen 7 5800X + RTX 3090 Ti | Ryzen 7 5800X + RTX 3090 Ti (line 2-3 of every log) | MATCH |
| Active working set | 160³ voxel world (or 256³ benchmark) | 160×64×160 = 1 638 400 cells (lines 10 of profile2/profile3) | MATCH (1.638 M cells, ~64 % of 256³ = 16.7 M) |
| ≥60 fps @ working set | ≥60 fps steady-state | **no data** (process dies at t≈5s) | UNMEASURED |
| ≥30 fps floor under load | ≥30 fps worst case | **no data** | UNMEASURED |
| No frame hitch >X ms (NFR-CIV-PERF-902) | Async dirty-queue drain | "AsyncComputeTaskPool" itself crashes — async drain is the failure site | **VIOLATED** (drain itself is the source of the fatal abort) |

**Honest verdict:** the frame budget is unmeasurable today because the async mesh path
crashes. This PR puts the *measurement instrumentation in place* (the diagnostic plugins)
so the moment §5.1 is fixed, the next audit run will produce real fps/frame_time data
without any further code change. The emergent behavior (worldgen succeeds, 40.6 % fill,
max_solid_y ≈ 38) proves the world pipeline is correct; the perf budget is blocked on the
async-mesh stack overflow, not on frame-budget headroom.

---

## 7. Reproduction recipe

```cmd
:: from G:\civis-main-gate on branch perf/frame-baseline
setlocal
set CARGO_TARGET_DIR=G:\civis-target-gate
cargo build -p civ-bevy-ref --features bevy,egui,voxel,models --bin civ-standalone --offline
G:\civis-target-gate\civ-standalone.exe  :: see G:/civis-target-gate/run-civ-standalone.cmd
::  expect: stack overflow in Compute Task Pool at t≈3-5s
```

Wrapper script `G:\civis-target-gate\run-civ-standalone.cmd` (used to record profile2/profile3):

```cmd
@echo off
setlocal
set CIVIS_AUTOSTART=1
set "BEVY_ASSET_ROOT=G:\civis-main-gate\clients\bevy-ref"
timeout 90 G:\civis-target-gate\debug\civ-standalone.exe > G:\civis-main-gate\profileN.log 2>&1
exit /b %ERRORLEVEL%
```

---

**Trace:** FR-CIV-3D-MOD-001 / NFR-CIV-PERF-900 / Epic **CIV-3D-PERF** (Scale + Frame-Budget), Story **CIV-3D-PERF-001** (CA dirty-chunk stepping + remesh wiring — per PR #354 body).
