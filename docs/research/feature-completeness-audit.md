# Feature Completeness Audit

Scope: `crates/engine/src/engine.rs`, `crates/agents/src/`, `crates/build/src/`, `crates/tactics/src/`, `crates/research/src/`, `crates/diffusion/src/`, `crates/planet/src/`, `web/dashboard/src/scene3d.tsx`, `clients/bevy-ref/src/bin/standalone.rs`.

Legend:
- `REAL` = working code that mutates game state or renders the feature.
- `STUB` = API exists, but behavior is partial, placeholder, or not wired into the main loop.
- `MISSING` = not present on that surface.

| System | Engine | Web | Bevy | Status | Notes |
|---|---|---|---|---|---|
| Engine tick loop | REAL | MISSING | MISSING | REAL | `Simulation::tick` advances production, citizen lifecycle, military, economy, diplomacy, tactics, voxel drain, compact, planet, buildings, diffusion, and replay logging every tick. |
| Civilians / agents | REAL | REAL | REAL | REAL | `civ-agents` has real spawn, movement, need scoring, LOD gating, and wardrobe/tools diffusion. Engine uses `spawn_child_near`, `spawn_civilian_at`, and cohort propagation each tick; web and Bevy both visualize civilians from snapshot/sim state. |
| Buildings / growth | REAL | REAL | MISSING | REAL | `civ-build` allocates parcels from demand signals, persists them in `BuildingGraph`, and assigns facades/provenance. Engine runs parcel allocation on cadence. Web renders buildings, roads, and building clusters. Bevy standalone does not spawn buildings. |
| Combat / tactics | REAL | REAL | MISSING | REAL | `civ-tactics` has real pathfinding, LOS, fog, formation, and `tick_war_bridge` combat resolution. Engine feeds military units into the bridge and applies HP loss + voxel damage. Web renders military units, conflict highlighting, combat pulses, and tactical overlays. Bevy standalone does not render combat units. |
| Research / tech progression | STUB | MISSING | MISSING | STUB | `civ-research` validates tech cards and caches responses, but the engine does not run a live research progression loop. `apply_replay_research` is a no-op beyond ticking state, so this is infrastructure rather than a gameplay system. |
| Diffusion / adoption | REAL | REAL | REAL | REAL | `civ-diffusion` implements Bass-style adoption math, and `civ-agents` applies it to wardrobe/tools era promotion across the cohort. Engine calls the cohort propagation phase each tick. Web and Bevy do not compute diffusion themselves, but they do reflect the resulting civilian state. |
| Planet / weather / climate | REAL | REAL | REAL | REAL | `civ-planet` computes deterministic climate from tick, planet, and moon configuration. Engine recomputes climate every tick. Web renders season/weather effects, lighting changes, snow/rain, and day/night state; Bevy standalone renders day/night atmosphere and lighting, but not the full weather UI. |
| Web 3D dashboard | MISSING | REAL | MISSING | REAL | `scene3d.tsx` is a rich renderer: terrain, water, civilians, buildings, roads, trade routes, factions, military units, danger/disaster rings, fog overlay, tooltips, and atmospheric/weather animation. |
| Bevy standalone client | MISSING | MISSING | REAL | REAL | `standalone.rs` renders terrain plus camera/atmosphere/decorations only. It does not currently spawn civilians, buildings, military units, or tactical overlays. |

## Bottom line

- The core simulation is not a stub: civilians, buildings, combat, diffusion, and planet/climate all have real state changes.
- Research is the main incomplete gameplay surface. The crate exists and is functional as a validator/cache, but tech progression is not part of the tick loop.
- The web dashboard is the most complete visual surface.
- The Bevy standalone client is intentionally narrow: terrain and atmosphere only.
