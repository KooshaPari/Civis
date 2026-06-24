# Native Ocean (bevy_water 0.18.1) — Implementation Plan

> **Item #106** · *Civis 3D / agentic line* · **Status:** Spec-only design.
> **Scope:** Replace the current flat translucent water plane with a real,
> shader-driven ocean surface (FFT waves + planar reflections + caustics) by
> activating the already-present `OceanPlugin` (wrapping `bevy_water 0.18.1`).
> **Out of scope:** Sim-side fluid model, gameplay-affecting wave dynamics,
> full CIV-0700 capability enforcement / mod store / hot-reload.

---

## 1. Current state — what we have today

### 1.1 The flat water plane

A single translucent plane is spawned in `setup_atmosphere` and animated by
`animate_water`:

| Concern | File · symbol |
|---|---|
| Plane entity spawn | `clients/bevy-ref/src/atmosphere.rs:84-101` (`WaterSurface` marker, `Mesh3d(Plane3d::default().mesh().size(256.0, 256.0))`, `StandardMaterial` with `base_color srgba(0.16, 0.34, 0.55, 0.78)`, `perceptual_roughness: 0.12`, `reflectance: 0.08`, `alpha_mode: AlphaMode::Blend`) |
| Plane Y position | `clients/bevy-ref/src/atmosphere.rs:100` (`Transform::from_xyz(0.0, WATER_LEVEL - 0.8, 0.0)`) |
| Per-frame bob animation | `clients/bevy-ref/src/atmosphere.rs:126-140` (`animate_water` — `transform.translation.y = WATER_LEVEL - 0.8 + (time.elapsed_secs() * 1.6).sin() * 0.06`) |
| Wiring into `Update` schedule | `clients/bevy-ref/src/bin/standalone.rs:72` (system tuple `(camera_input, update_camera, animate_water, update_lighting)`) |
| Sea-level constant | `clients/bevy-ref/src/terrain.rs:30` (`pub const WATER_LEVEL: f32 = 0.32 * HEIGHT_SCALE;` — i.e. `0.32 * 200.0 = 64.0` in metres; the task's "sim units" note that **a prior fix** had `seaLevel = 2.0` in voxel/sim grid units, see §1.2) |
| Terrain footprint | `clients/bevy-ref/src/terrain.rs:26` (`pub const WORLD_SIZE: f32 = 256.0;`) and `terrain.rs:28` (`pub const HEIGHT_SCALE: f32 = 200.0;`) |

**What it gives us:** a tinted, semi-transparent, *sinusoidally bobbing* plane
at the right altitude, sized 256×256 over the whole terrain footprint, with a
`reflectance` term that approximates a hint of sky on the surface. It does
**not** model wave shape, does **not** reflect the world, and does **not**
cast caustics.

### 1.2 Voxel-grid sea level (sim units)

The "prior fix `seaLevel=2.0`" referenced in the task is the **sim/voxel** sea
level, not the render sea level. The two constants serve different
coordinates:

- Render world: `WATER_LEVEL = 0.32 * HEIGHT_SCALE = 64.0` (metres),
  defined in `clients/bevy-ref/src/terrain.rs:30`.
- Voxel world: `worldgen::sea_level(world.dims)` returns the sim grid Y index
  for the topmost water layer; the BDD test
  `tests/requirements_bdd.rs:466-525` (`requirement_native_ocean_renders_with_sea_level_match`)
  asserts that no `WATER` cell sits **above** `sea_level` and that ≥95% of
  water-column surfaces are within 1 voxel of `sea_level`.

**Render-side, our source of truth is `WATER_LEVEL` (64.0).** The voxel sea
level is a separate concern owned by `worldgen` and only relevant for the
voxel-sandbox visual surface (`civ-voxel`'s `CubicMesher`), which the
`voxel`/`voxel_stream` features route through `OceanPlugin::water_plugin_only`
(owns shader infra) or `OceanPlugin::default` (owns shader + spawn). Neither
of those code paths is touched by this design — see §1.3.

### 1.3 What `ocean.rs` already provides

`clients/bevy-ref/src/ocean.rs` ships a complete wrapper, currently
**dormant outside the voxel path**:

| Symbol | Lines | Status |
|---|---|---|
| `OceanQuality` resource (`High`/`Low`) | `ocean.rs:60-68` | Ready |
| `OceanSurface` marker component | `ocean.rs:72-73` | Ready |
| `OceanPlugin { spawn_plane: bool }` | `ocean.rs:88-92` | Ready |
| `OceanPlugin::default()` (registers `WaterPlugin` + `WaterSettings` + spawn system) | `ocean.rs:94-98`, `ocean.rs:109-138` | Ready |
| `OceanPlugin::water_plugin_only()` (no spawn) | `ocean.rs:104-107` | Ready |
| `build_water_settings(water_quality)` (uses `WATER_LEVEL`, `HEIGHT_SCALE`, `WORLD_SIZE`) | `ocean.rs:150-170` | Ready |
| `spawn_ocean_plane` (creates `PlaneMeshBuilder` at `WATER_SIZE`, scales to `WORLD_SIZE`, applies `StandardWaterMaterial` + `WaterMaterial` extension) | `ocean.rs:178-229` | Ready |

The plugin currently only runs in the two `voxel*` feature branches at
`clients/bevy-ref/src/bin/standalone.rs:135-138` (`voxel`+`voxel_stream` →
`OceanPlugin::default()`; `voxel` only → `OceanPlugin::water_plugin_only()`).

The `Cargo.toml` dep is already present and pinned to the right Bevy
version:

- `clients/bevy-ref/Cargo.toml:38` — `bevy_water = { version = "0.18.1", optional = true }`
- `clients/bevy-ref/Cargo.toml:120` — `voxel = ["bevy", "dep:bevy_water"]` (currently the *only* feature that pulls `bevy_water`)

There is **no** `ocean` feature in `Cargo.toml:84-162` today. The task asks
us to add one (see §3.1).

### 1.4 The BDD contract we must keep green

`clients/bevy-ref/tests/requirements_bdd.rs:466-525` is the
*render-independent* contract. It iterates the voxel grid; it does not
touch the renderer. It will continue to pass after this change because we
do not modify `WATER_LEVEL`, `worldgen::sea_level`, or any voxel mesh path.

A new headless-visible signal must come from `CIVIS_DUMP` (see §6).

---

## 2. Goal & non-goals

### 2.1 Goal

When the new `ocean` feature is enabled, the user sees a real ocean surface
in the heightmap (non-voxel) build path:

- **FFT-driven waves** with hand-tuned amplitude/coords (already chosen in
  `ocean.rs:154` / `ocean.rs:166-167`).
- **Planar reflections** of the sky and visible terrain/actors.
- **Caustics** baked into the water shader.
- **Coherent with terrain:** surface plane exactly at `WATER_LEVEL`, no
  Z-fight at the coast, footprint covers the full `WORLD_SIZE` ×
  `WORLD_SIZE`.
- **Configurable quality** via the existing `OceanQuality` resource
  (`High` default, `Low` for CI/headless).

### 2.2 Non-goals

- **No sim-side fluid model.** Wave amplitude is purely visual. It does
  not feed back into erosion, floods, or any `civ-engine` system.
- **No replacement of the `voxel*` features.** `voxel`+`voxel_stream`
  continues to take `OceanPlugin::default()`; `voxel`-only continues to
  take `OceanPlugin::water_plugin_only()`. We do **not** change those
  branches.
- **No CIV-0700 capability surface.** The wrapper exposes a Rust API only;
  there is no JSON-RPC / mod-API hook for the ocean.
- **No HDR / Solari integration work.** We rely on whatever the project's
  default `bevy_pbr` lighting gives us. Refraction/SSR-quality follow-up
  is a separate doc.

---

## 3. Integration points

### 3.1 `Cargo.toml` — add the `ocean` feature

In `clients/bevy-ref/Cargo.toml`, in the `[features]` block (currently
`Cargo.toml:84-162`):

```toml
# Native ocean (bevy_water 0.18.1). Adds the heightmap-path wave surface
# to clients/bevy-ref/src/bin/standalone.rs. The `voxel*` features have
# their own wiring (see ocean.rs module docs) and do NOT depend on this
# flag.
ocean = ["bevy", "dep:bevy_water"]
```

Notes:

- We **do not** include `ocean` in any other feature, in particular **not**
  in `default`, **not** in `bevy`, **not** in `voxel`. The two build paths
  (heightmap vs. voxel) stay independent so that the existing voxel sandbox
  is unaffected.
- The task spec says "behind the existing `ocean` feature". Today, no such
  feature exists in `Cargo.toml:84-162`. We must add it; the wrapper code
  in `ocean.rs` is what makes the feature *meaningful* (the wrapper already
  has both wiring modes).
- `bevy_water 0.18.1` is already declared at `Cargo.toml:38`; we just need
  the new feature to pull it.

### 3.2 `bin/standalone.rs` — wire the plugin in the heightmap path

In `clients/bevy-ref/src/bin/standalone.rs`, near the existing voxel-path
branch (`standalone.rs:125-138`), add a new branch for the heightmap path:

```rust
// Heightmap (non-voxel) path: enable the real ocean surface when the
// `ocean` feature is on. Voxel paths continue to use their own branches
// below and are unchanged.
#[cfg(all(feature = "ocean", not(feature = "voxel")))]
app.add_plugins(OceanPlugin::default());
```

Place this **after** the existing voxel wiring (`standalone.rs:135-138`).
The `not(feature = "voxel")` guard is defensive: a user combining
`--features ocean,voxel` is on the voxel path and `voxel*` already owns
the surface, so we must not double-spawn.

In the same file:

- **Remove** the `WaterSurface` plane spawn from `setup_atmosphere` when
  `feature = "ocean"` is active (see §3.3). That means `setup_atmosphere`
  itself should branch on the feature, **not** the call site. See
  `atmosphere.rs:84-101` for the spawn we are removing.
- **Keep** `animate_water` removed under the same gate so the bob system
  does not try to animate a `WaterSurface` that no longer exists. The
  `Update` schedule tuple at `standalone.rs:72` is where it is wired:
  - Remove `animate_water` from the tuple when `feature = "ocean"`.
  - Remove `WaterSurface` from the `use` import at `standalone.rs:9` when
    `feature = "ocean"`.

### 3.3 `atmosphere.rs` — feature-gate the flat plane

In `clients/bevy-ref/src/atmosphere.rs`:

1. Wrap the `WaterSurface` spawn block (`atmosphere.rs:84-101`) with
   `#[cfg(not(feature = "ocean"))]`.
2. Wrap the `animate_water` function body (`atmosphere.rs:126-140`) with
   `#[cfg(not(feature = "ocean"))]`, and rename the function to
   `#[cfg(not(feature = "ocean"))] pub fn animate_water`. Mark
   `WaterSurface` (`atmosphere.rs:37-38`) the same way.

   This keeps `atmosphere.rs` compiling in both modes without leaving a
   half-typed identifier behind. When `feature = "ocean"` is off (the
   default), behaviour is byte-identical to today.

3. The `setup_atmosphere` signature is fine: the spawn is now under a
   `#[cfg]`, so the function compiles in both modes.

### 3.4 `lib.rs` — ensure `pub mod ocean;` is gated

`clients/bevy-ref/src/lib.rs:77` already declares `pub mod ocean;`. That
declaration must remain unconditional (or be made `#[cfg(any(feature =
"ocean", feature = "voxel"))]` — see §3.5 for the trade-off) because both
build paths use it.

### 3.5 `ocean.rs` — make it compilable without `voxel`

Today, `ocean.rs:50` `use crate::terrain::{HEIGHT_SCALE, WATER_LEVEL, WORLD_SIZE};`
and the rest of the module are unconditional. Good — that already works.
**No code change** to `ocean.rs` is strictly required for this design,
because the module compiles fine without `voxel` and is already the
authoritative `ocean` surface. We are only *wiring it in*.

### 3.6 Plugin add order

`OceanPlugin::default()` calls `app.add_plugins(WaterPlugin)` internally
(`ocean.rs:126`). It is safe to add at any point after `DefaultPlugins`
(per `standalone.rs:31-41`) — the WGSL shader compilation kicks in lazily
on first use. We add it after the voxel branches (see §3.2) so the
**non-voxel** path is the last one to take a water plugin add, mirroring
the existing pattern.

### 3.7 Where the sea level is set

Two locations, both must be coherent with `WATER_LEVEL` (64.0 m):

- `ocean.rs:153` — `WaterSettings { height: WATER_LEVEL, ... }` (the shader's
  base level for FFT/planar reflections). **Already correct.**
- `ocean.rs:224` — `Transform::from_xyz(WORLD_SIZE * 0.5, WATER_LEVEL,
  WORLD_SIZE * 0.5)` (the plane entity's Y). **Already correct.**

We do not introduce a second sea-level constant. The two Y values must
stay equal or the FFT base will drift from the plane centre and you get
phase artefacts at the coast. `build_water_settings` and `spawn_ocean_plane`
both already use `WATER_LEVEL` directly — no change needed.

---

## 4. Bevy 0.18 compatibility risk + fallback

### 4.1 Risk surface

`bevy_water 0.18.1` targets Bevy 0.18 (the same version pinned at
`Cargo.toml:37` and exercised by `ocean.rs` today). The known exposure
points are:

| Risk | Likelihood | Mitigation |
|---|---|---|
| Internal `bevy_water` API rename between 0.18.0 and 0.18.1 (e.g. `StandardWaterMaterial`, `WaterMaterial`, `WaterSettings` field names) | Low (we already compile `ocean.rs` against 0.18.1 in the voxel feature) | If it breaks, see §4.2 fallback. |
| WGSL shader incompatibility with current GPU backend (`wgpu 27` at `Cargo.toml:48`) | Low | The voxel-path build already pulls `bevy_water 0.18.1` and shaders compile. If the *heightmap* path's lighting differs (e.g. `SolariGiPlugin` from `feature = "gi"`), we test under both `default` and `gi` (see §6). |
| `PlaneMeshBuilder` / `Mesh3d` rename in 0.18.1 | Low | `ocean.rs:41-42` already uses these exact 0.18 symbols. If they shift, the same fallback applies. |
| Headless/CI does not have a GPU | Medium (CI typically runs `cargo check`/`cargo test`, not the windowed bin) | The plugin is a runtime plugin; `cargo check --features ocean` and unit tests in headless mode must pass. The `OceanQuality::Low` setting keeps VRAM low for headless dumps. |
| Bevy asset hot-reload (`dev`/`hot` features) clobbers the water shader in a long-running windowed session | Low | The water shader auto-recompiles; this is the desired behaviour. |

### 4.2 Fallback — the flat plane (today's behaviour)

If `bevy_water 0.18.1` ever fails to compile or the FFT shader misbehaves
on a target GPU, we have an in-tree fallback already:

1. **Compile-time fallback** — flip the default. Change
   `Cargo.toml:120` (`voxel = ["bevy", "dep:bevy_water"]`) and the new
   `ocean` feature to **not** pull `bevy_water`. The wrapper `ocean.rs`
   still compiles (it has no `#[cfg(feature = ...)]` blocks), but its
   `app.add_plugins(WaterPlugin)` line at `ocean.rs:126` would fail. To
   make a true zero-dep fallback, gate the `use bevy_water::...` imports
   (`ocean.rs:44-48`) and the `add_plugins(WaterPlugin)` call
   (`ocean.rs:126`) on a `#[cfg(feature = "ocean")]` so that with the
   feature off, `ocean.rs` reverts to a no-op struct.

   **This is a "future-only" hardening, not part of this PR.** We do not
   want to make `ocean.rs` silently no-op on the user's machine without a
   loud log line; the in-tree wrapper is *meant* to fail loud if its
   dep is missing. Tracking this as a follow-up: gate the
   `bevy_water::` imports on a new `ocean-shim` internal cfg.

2. **Runtime fallback — the existing flat plane** — flip
   `#[cfg(feature = "ocean")]` off, or just don't pass `--features ocean`.
   The flat plane in `atmosphere.rs:84-101` + `atmosphere.rs:126-140`
   returns, with the `WaterSurface` bob animation intact. This is the
   documented behaviour of "no `ocean` feature = today's build".

3. **Per-platform runtime override** — we **do not** add a runtime
   "fall back to flat" code path. The complexity of dynamically swapping
   between the shader and the flat plane in a live session is not worth
   the payoff: the user can simply rebuild without `--features ocean` to
   restore the old look.

---

## 5. Phased steps (file:fn names)

> Phases are **sequential**. Each phase ends with a green `cargo check` (or
> `cargo build --features ocean` for the feature-on path). The phases are
> ordered so that any one of them, if it fails, leaves the workspace in a
> buildable state (the `ocean` feature is **off by default** until the
> final phase).

### Phase A — Cargo feature plumbing (no behaviour change yet)

1. **Edit** `clients/bevy-ref/Cargo.toml`:
   - Add `ocean = ["bevy", "dep:bevy_water"]` to the `[features]` block
     (`Cargo.toml:84-162`), as a standalone feature.
2. **Verify** — `cargo check -p civ-bevy-ref` (no feature change yet) and
   `cargo check -p civ-bevy-ref --features ocean` and
   `cargo check -p civ-bevy-ref --features voxel` (existing voxel path
   must still build).

### Phase B — feature-gate the flat plane

1. **Edit** `clients/bevy-ref/src/atmosphere.rs`:
   - Add `#[cfg(not(feature = "ocean"))]` to the `WaterSurface`
     component decl (`atmosphere.rs:37-38`).
   - Add `#[cfg(not(feature = "ocean"))]` to the `WaterSurface` spawn
     block inside `setup_atmosphere` (`atmosphere.rs:84-101`).
   - Add `#[cfg(not(feature = "ocean"))]` to the `pub fn animate_water`
     signature and body (`atmosphere.rs:126-140`).
2. **Edit** `clients/bevy-ref/src/bin/standalone.rs`:
   - At `standalone.rs:9`, branch the `atmosphere::{...}` import on
     `#[cfg(not(feature = "ocean"))]` so `WaterSurface` / `animate_water`
     are not imported when `ocean` is on. The `setup_atmosphere` and
     `update_lighting` imports stay unconditional.
   - At `standalone.rs:60`, leave `setup_atmosphere` unconditional
     (the function still exists in both modes; the cfg-gated spawn block
     is inside it).
   - At `standalone.rs:72`, branch the system tuple so
     `animate_water` is omitted when `feature = "ocean"` is on:
     - Keep `(camera_input, update_camera, update_lighting)` always.
     - Add `animate_water` only under `#[cfg(not(feature = "ocean"))]`.
3. **Verify** — `cargo build -p civ-bevy-ref` (default features), then
   `cargo build -p civ-bevy-ref --features ocean` and
   `cargo build -p civ-bevy-ref --features voxel,voxel_stream`. All three
   must succeed.

### Phase C — add OceanPlugin to the heightmap path

1. **Edit** `clients/bevy-ref/src/bin/standalone.rs`:
   - Add the new branch from §3.2:
     ```rust
     #[cfg(all(feature = "ocean", not(feature = "voxel")))]
     app.add_plugins(OceanPlugin::default());
     ```
     placed after the existing voxel branches at `standalone.rs:135-138`.
   - At `standalone.rs:7`, the `use civ_bevy_ref::ocean::OceanPlugin;`
     is already behind `#[cfg(feature = "voxel")]` — extend the guard to
     `#[cfg(any(feature = "voxel", feature = "ocean"))]`.
2. **Verify** — `cargo build -p civ-bevy-ref --features ocean` succeeds,
   and a windowed run (`cargo run -p civ-bevy-ref --features ocean`) shows
   a wave-shaped surface with reflections and a slight caustic at the
   shore. (Manual visual gate — see §6.4.)

### Phase D — `OceanQuality` resource + headless dump (verification)

1. **Edit** `clients/bevy-ref/src/bin/standalone.rs` to insert
   `OceanQuality` *before* `add_plugins(OceanPlugin::default())`:
   ```rust
   #[cfg(all(feature = "ocean", not(feature = "voxel")))]
   app.insert_resource(civ_bevi_ref::ocean::OceanQuality::OceanQuality::default())
      .add_plugins(OceanPlugin::default());
   ```
   (This honours the contract documented at `ocean.rs:27-38` — quality
   must be present before the plugin builds.)
2. **Edit** `clients/bevy-ref/src/scene_dump.rs` to add an `ocean`
   section to the JSON output (see §6.2).
3. **Verify** — `cargo test -p civ-bevy-ref --features ocean` passes;
   BDD test `requirement_native_ocean_renders_with_sea_level_match`
   (`tests/requirements_bdd.rs:466-525`) still passes; CIVIS_DUMP run
   produces a JSON with the new `ocean` block.

### Phase E — (Optional, follow-up) ship the `ocean` feature on by default

**Not in this PR.** When the wrapper has been stable for ≥1 release
window, consider moving `ocean` into the `bevy` feature (or into
`default`) and removing the in-`atmosphere.rs` flat-plane code. Track in
a follow-up issue. Leaving the gate user-controlled for now keeps the
risk envelope tight.

---

## 6. Verification plan

### 6.1 Existing BDD contract (must still pass)

- `cargo test -p civ-bevy-ref --features ocean` runs the BDD suite, in
  particular `requirement_native_ocean_renders_with_sea_level_match`
  (`clients/bevy-ref/tests/requirements_bdd.rs:466-525`). This is the
  sim-side contract: ≥95% of water columns must have a surface within 1
  voxel of `sea_level`, and zero columns may have `WATER` strictly above
  `sea_level`. We do not modify the voxel/worldgen path, so this must
  remain green.

### 6.2 New `CIVIS_DUMP` probe — water mesh verts/spanY

`clients/bevy-ref/src/scene_dump.rs` is the headless introspection hook
(armed via the `CIVIS_DUMP` env var, default warmup 6 s, JSON output).
We add a new block that introspects the ocean surface directly.

**Arm the dump:**

```bash
CIVIS_DUMP=/tmp/ocean-probe.json \
  cargo run -p civ-bevy-ref --features ocean --bin civ-bevy-ref
```

**Proposed JSON shape (additions to the existing dump at
`scene_dump.rs:60-260`):**

```json
"ocean": {
  "present": true,
  "marker": "OceanSurface",
  "entity": <entity-id>,
  "transform": { "translation": [128.0, 64.0, 128.0], "scale": [2.0, 1.0, 2.0] },
  "water_settings": {
    "height": 64.0,
    "amplitude": 0.35,
    "clarity": 0.22,
    "water_quality": "High"
  },
  "mesh": {
    "vertex_count": <n>,
    "index_count": <m>,
    "min_y": <min vertex y>,
    "max_y": <max vertex y>,
    "span_y": <max_y - min_y>,
    "size_x": <mesh span x>,
    "size_z": <mesh span z>
  }
}
```

**Implementation:** in `scene_dump.rs`, add a new query against
`bevy_water::water::WaterTile` + `bevy::prelude::Mesh3d` + our
`OceanSurface` marker (already exported from `ocean.rs:72-73`). The
vertex Y range tells us whether the FFT is producing real wave
displacement (a flat plane has `span_y == 0`; the wrapper's
`amplitude = 0.35` plus per-tile noise should produce a `span_y` on the
order of `0.3`–`0.7` metres after a few seconds of warmup).

**Acceptance:**

- `present == true`.
- `transform.translation[1]` is within `0.001` of `WATER_LEVEL` (64.0).
- `water_settings.height` is exactly `WATER_LEVEL`.
- `span_y` is `> 0.1` (FFT is doing real work) and `< 2.0` (waves are
  sane; not blowing up to a kilometre high).
- `size_x` and `size_z` are `>= WORLD_SIZE` (256.0).
- `vertex_count` is `> 4` (we have a plane, not a degenerate quad).

### 6.3 Headless sanity

A second probe for **shader compilation** (because `span_y > 0` only
proves the *data* is changing, not that the *shader* is rendering it
correctly — the wave displacement could be mis-typed and still oscillate
at the CPU side). Run:

```bash
cargo test -p civ-bevy-ref --features ocean -- --nocapture ocean_smoke
```

and a windowed smoke run of:

```bash
CIVIS_DUMP_KEEP_ALIVE=1 \
  cargo run -p civ-bevy-ref --features ocean
```

The `CIVIS_DUMP_KEEP_ALIVE=1` path at `scene_dump.rs:266` keeps the
window open for visual confirmation. **Not** a CI gate; manual / agent
visual gate (see §6.4).

### 6.4 Manual / agent visual gates (advisory, not CI)

In a windowed run, an agent (or human) confirms:

- Coastline shows a smooth wave-line break, not a hard plane edge.
- Reflections of sun/moon direction are visible in deep water.
- Caustic pattern is visible on the underwater terrain at shallow
  angles.
- The water material does not Z-fight the terrain mesh at the coastline.
- A test with `feature = "gi"` (Solari GI on) and `feature = "ocean"`
  together still looks coherent.

### 6.5 `agent-smoke`

`./scripts/agent-smoke.ps1` from `AGENTS.md` runs the standard smoke
gates. We do not change any other JSON-RPC or snapshot shape, so the
existing smoke must continue to pass. The new `ocean` field in the
`CIVIS_DUMP` output is **additive** and uses an existing probe, so the
catalog drift check (`just civis-3d-catalog-check`) and the scenario
check (`just civis-3d-scenario-check`) are unaffected.

---

## 7. Touched files (this PR)

| File | Change |
|---|---|
| `clients/bevy-ref/Cargo.toml` | Add `ocean` feature; no other change. |
| `clients/bevy-ref/src/atmosphere.rs` | `#[cfg(not(feature = "ocean"))]` on the `WaterSurface` decl, its spawn in `setup_atmosphere`, and `animate_water`. |
| `clients/bevy-ref/src/bin/standalone.rs` | Branch the `WaterSurface`/`animate_water` import on `#[cfg(not(feature = "ocean"))]`; add the `animate_water` system under the same guard; add `#[cfg(all(feature = "ocean", not(feature = "voxel")))] app.add_plugins(OceanPlugin::default());`; extend the `use ocean::OceanPlugin` import guard. |
| `clients/bevy-ref/src/scene_dump.rs` | Add an `ocean` block to the JSON output. |

**No changes** to `clients/bevy-ref/src/ocean.rs`, `terrain.rs`,
`voxel_sim.rs`, `voxel_stream.rs`, or any other client. The wrapper is
already correct.

---

## 8. References

- **Task item:** #106 — *native ocean* (Civis 3D / agentic line).
- **Wrapper module:** `clients/bevy-ref/src/ocean.rs:1-230`.
- **Current flat plane:** `clients/bevy-ref/src/atmosphere.rs:37-38`,
  `atmosphere.rs:84-101`, `atmosphere.rs:126-140`.
- **Plugin wiring sites:** `clients/bevy-ref/src/bin/standalone.rs:7`,
  `standalone.rs:9`, `standalone.rs:60`, `standalone.rs:72`,
  `standalone.rs:125-138`.
- **Sea-level constants:** `clients/bevy-ref/src/terrain.rs:30`
  (`WATER_LEVEL`); voxel-grid `sea_level` is a
  `worldgen::sea_level(dims)`-derived value referenced in
  `tests/requirements_bdd.rs:476`.
- **Cargo dep + feature table:** `clients/bevy-ref/Cargo.toml:38`
  (`bevy_water 0.18.1`), `Cargo.toml:120` (`voxel` feature pulls
  `dep:bevy_water`), `Cargo.toml:84-162` (the `[features]` block to
  which we add `ocean`).
- **Verification hooks:** `clients/bevy-ref/src/scene_dump.rs:1-269`
  (CIVIS_DUMP); `clients/bevy-ref/tests/requirements_bdd.rs:466-525`
  (BDD contract).
- **AGENTS.md gates (unchanged):** `./scripts/agent-smoke.ps1`,
  `just civis-3d-verify`, `just civis-3d-catalog-check`,
  `just civis-3d-scenario-check`, `cd web && npm test` / `npm run build`.
- **bevy_water 0.18.1 API in use:** `bevy_water::WaterPlugin`,
  `bevy_water::water::{StandardWaterMaterial, WaterMaterial, WaterQuality,
  WaterSettings, WaterTile, WATER_SIZE}` — all accessed through the
  `ocean.rs` wrapper.
