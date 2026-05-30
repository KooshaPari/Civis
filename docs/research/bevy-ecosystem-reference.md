# Bevy Ecosystem Reference for Civis (wrap-over-handroll)

## Overview
Civis is Bevy 0.18, desktop-primary (DX12/DXR/DLSS). Per the org's wrap-over-handroll charter, this surveys Bevy-ecosystem crates/plugins/open games Civis should **reuse** rather than build. Bevy 0.18 itself ships procedural atmosphere with occlusion + PBR shading and a `ScatteringMedium` asset for skies/fog/alien atmospheres ([Bevy 0.18](https://bevy.org/news/bevy-0-18/)) — relevant to Civis planet/climate render. Curated plugin/game index: [Bevy Assets](https://bevy.org/assets/); weekly pulse: [This Week in Bevy](https://thisweekinbevy.com/).

## Crate survey (crate · maturity · Civis use · gaps)

| Need | Crate | Maturity | Civis use | Gaps/notes |
|---|---|---|---|---|
| Selection/picking | `bevy_picking` (in-engine since 0.15) | Stable, first-party | FR-CIV-INSPECT-900 click-any-entity; FR-CIV-GODTOOL brush targeting | Voxel picking needs custom backend over `phenotype-voxel` |
| Large-world / floating origin | `big_space` | Mature, widely used | NFR-CIV-SCALE-900 20mi extent without f32 precision loss | Must integrate with fixed-point `WorldCoord` |
| GPU particles | `bevy_hanabi` | Mature | Disasters, weather, combat FX (FR-CIV-GODTOOL-912) | GPU-only; fine for desktop target |
| Dev/info UI | `bevy_egui` + `egui_plot` | Mature | Info-view legends, inspector panels, stats dashboards (FR-CIV-INFOVIEW, NOTIFY-910) | egui for tooling; final HUD likely bevy_ui |
| Inspector/debug | `bevy-inspector-egui` | Mature | Dev-time entity inspection; prototype for FR-CIV-INSPECT | Dev tool, not shipping UI |
| RTS/strategic camera | `bevy_panorbit_camera` / community RTS camera plugins | Moderate | Strategic↔tactical pan/zoom/tilt (matrix §7) | May need custom for seamless god↔tactical zoom (EAW-style) |
| Adaptive audio | `bevy_kira_audio` (Kira) | Mature | Adaptive score + spatial SFX (CIV-0800, RND-007) | Already chosen in RND-007 |
| ECS at scale | `bevy_ecs` vs `hecs` | Both mature | Civis uses `hecs` in `civ-agents`/`civ-engine` for 100k+ agents (RND-001) | Bridge hecs↔Bevy render world |
| Massive instancing | Bevy instancing / custom indirect-draw | 0.18 GPU-driven improving | NFR-CIV-PERF-901 100k+ agents/voxels | May need custom indirect pipeline; watch Bevy GPU-driven roadmap |
| Voxel meshing | community voxel crates (e.g. block-mesh-rs lineage) | Moderate | Re-mesh dirty chunks; but `phenotype-voxel` owns the Mesher trait | Per-engine mesher already speced; reuse algorithms not whole crates |
| Terrain/atmosphere | Bevy 0.18 built-in atmosphere + community terrain | New (0.18) | Planet/climate visuals; GI status in `bevy-gi-status.md` | First-party atmosphere reduces handroll |
| Save/replay | `serde` + `bevy` scene + custom deterministic snapshot | — | CIV-1000 deterministic save/replay | Determinism needs custom; no turnkey replay crate |

## Open Bevy games worth studying
- A **free/modern open-source RTS** built on Bevy (ecosystem-listed) — for selection, command, camera, unit-management patterns.
- A **Dwarf-Fortress/RimWorld-like colony sim in Rust/Bevy** — for ECS job systems, needs, agent scheduling at scale.
- Browse [Bevy Assets → Games](https://bevy.org/assets/) for current colony/strategy examples.

## What to ADOPT
- First-party `bevy_picking`, Bevy 0.18 atmosphere, and `bevy_kira_audio` over custom. `[UI/QoL]`.
- `big_space` for the 20mi floating-origin world. NFR → NFR-CIV-SCALE-900.
- `egui_plot` for stats dashboards. `[UI/QoL]` → FR-CIV-NOTIFY-910.
- Keep `hecs` for sim ECS (RND-001); bridge to Bevy render world rather than porting.

## What to AVOID
- Don't adopt whole voxel-engine crates that fight `phenotype-voxel`'s SVO+leaf model; reuse meshing *algorithms* only.
- Don't rely on a generic replay crate for determinism — author it against the fixed-point contract (NFR-CIV-DET).
- Avoid f32 world coords at 20mi scale — `big_space` is mandatory, not optional.

## Sources
- Bevy 0.18 release notes — https://bevy.org/news/bevy-0-18/
- Bevy Assets (plugins + games index) — https://bevy.org/assets/
- Bevy GitHub — https://github.com/bevyengine/bevy
- This Week in Bevy — https://thisweekinbevy.com/
- Bevy wiki/notes (nikiv) — https://wiki.nikiv.dev/games/gamedev/game-engines/bevy
