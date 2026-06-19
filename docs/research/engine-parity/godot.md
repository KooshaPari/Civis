# Godot — Parity Path for Civis

**Verdict: the ONE truly-OSS major engine.** Godot is **MIT-licensed** — we can legally read, port, fork, and vendor its code. This makes it our best "donor" for clean-room ports of well-scoped subsystems, and the basis of a working Bevy↔Godot bridge.

---

## What to actually port / study (MIT → safe to reuse)

| Godot subsystem | Why it's worth studying/porting | Reuse path |
|---|---|---|
| **Navigation (Recast/Detour-based)** | Godot, Unreal, and Unity all use Recast for navmesh. The clean-room Rust port already exists | **Adopt `rerecast`/`bevy_rerecast`** (janhohenheim) — Rust port of Recast, "industry-standard navmesh generator used by Unreal, Unity, Godot". Don't re-port from Godot; use rerecast directly |
| **TileMap / TileSet** | Mature 2D tiling, autotiling, terrain bitmasking | Study the autotile/terrain-peering algorithm; reimplement against our voxel/overlay layer if needed (algorithm, not code-copy) |
| **GPUParticles2D/3D** | Solid GPU particle design | Prefer ecosystem `bevy_hanabi`; study Godot's process-material model for ideas |
| **Renderer (Forward+ / Mobile / RD abstraction)** | Modern clustered Forward+ on a clean RenderingDevice abstraction | Reference only — Bevy's renderer + bevy_solari is our path; Godot's RD is a design reference for clustered lighting, not a port target |
| **Skeletal anim / IK, CharacterBody controllers** | Battle-tested gameplay primitives | Study for behavior; reimplement in Bevy idioms |

**Rule:** because Godot is MIT, *direct line-by-line porting is legally fine* (preserve copyright/license notice). But prefer existing Rust crates (rerecast, hanabi) over hand-porting when they already wrap the same upstream (Recast).

---

## godot-bevy bridge status (2026)

- **`godot-bevy`** (bytemeadow) — bridges **Bevy ECS into Godot 4**, latest **0.11.0** (~early 2026), tracks **Bevy 0.16**. Actively maintained: transform syncing, plugin systems, utilities. Lets you run Bevy ECS game logic while using Godot's editor + renderer.
- Lineage: inspired by `bevy_godot` (Godot 3 era, rand0m-cloud / patrislav), now superseded for Godot 4 by godot-bevy.
- **Relevance to Civis:** Civis's renderer is **Bevy-native**, so the bridge is **not on the critical path**. It is useful only as an *optional secondary editor/showcase route* (Godot as a content/editor front-end), mirroring UE's showcase role but fully OSS. Note the bridge tracks **Bevy 0.16**, lagging our **0.18** target — adopting it would mean a version-compat gap to manage. Treat as "watch / optional", not "adopt now".

---

## Sources
- [godot-bevy (crates.io)](https://crates.io/crates/godot-bevy) · [godot-bevy GitHub](https://github.com/bytemeadow/godot-bevy) · [releases](https://github.com/bytemeadow/godot-bevy/releases)
- [rerecast — Rust port of Recast](https://github.com/janhohenheim/rerecast)
- [bevy_godot (Godot 3 predecessor)](https://github.com/rand0m-cloud/bevy_godot)
