# 2026-05-22 — Civis 3D Extension Kickoff

**Branch:** `feat/civis-3d-foundation`
**PR:** [#296](https://github.com/KooshaPari/Civis/pull/296)
**Plan:** `~/.claude/plans/weve-spent-a-lot-toasty-reddy.md` (owner-approved)

## Scope

Civis is extended (not pivoted) into a 3D high-fidelity WorldBox-class civilization
sandbox. Fuses WorldBox + Dinosaur sim spawn UX, Cities Skylines 2 + Manor Lords
autonomous city growth, Star Wars EAW:FOC + Call to Arms warfare layering, Galimulator
+ Dwarf Fortress depth. Single planet + moon. Eras prehistoric → near-future scifi.

## Decided

- **Renderers:** Bevy (CI + determinism + agent-driven) + Godot 4 (UX iteration) +
  Unreal 5 (visual showcase), all attaching via existing WebSocket protocol. See
  ADR-007.
- **Voxel substrate:** adaptive/hybrid (SVO + dense 16³ leaf chunks). Extracted as a
  new Phenotype-org shared crate `phenotype-voxel`. Bevy adapter ships in that crate;
  Godot + Unreal adapters in their respective clients. See ADR-005.
- **WSM3D coordination:** existing WSM3D voxel work informs the shared crate. WSM3D
  consumes the Rust crate via `ffi-core`/`cbindgen` after extraction.
- **LLM event sourcing:** every LLM output is hash-keyed and cached; Canonical/Hybrid/
  Free modes preserve `.civreplay` bit-identity. See ADR-006.
- **Genetics:** algorithmic only, no LLM in the genetic loop. See ADR-008.
- **Visual bar:** COD / Rust / The Finals tier — PBR + per-engine GI.

## Landed this iteration

- 11 new crate stubs scaffolded under `crates/` (voxel, build, genetics, species,
  agents, diffusion, laws, research, tactics, planet, protocol-3d). Each has a
  `SCHEMA_VERSION` and an FR-CIV-`<NS>`-000 placeholder test.
- Workspace `Cargo.toml` updated.
- `cargo build / test / clippy -D warnings / fmt --check` all green.
- One pre-existing clippy lint fixed in `civ-engine` tests
  (`field-reassign-with-default`, exposed by the Rust 1.95 toolchain bump).
- Local `rustup stable` bumped 1.83 → 1.95 (proptest/blake3 transitive `edition2024`
  requirement).
- Docs addenda (this worklog + PRD-3d-extension + ADRs 005–008 + plan-3d-phases +
  fr-3d-additions).
- Memory entries saved: `project_civis_3d_pivot`, `feedback_codex_harness`.

## Open

- **Modding day-1 vs post-MVP** — RON tech cards loadable from `mods/` from P-L1
  onward? Owner to decide.
- **Unreal client EULA** — ship under Epic's standard EULA, or constrain to a
  shell-only renderer (no game logic) to dodge royalty surface?
- **LLM tick budget** — resolved as async-multi-tick per build.md.
- **WorldBox-style "anything spawnable" boundary** — kingdoms/factions post-hoc
  spawnable (assumed yes) or scenario-init only?

## Next iteration

1. Bootstrap `KooshaPari/phenotype-voxel` repo per
   `~/.claude/plans/civis-3d-scratch/phenotype-voxel-design.md` (P-V0). ✅
2. Wire `crates/voxel` as a path-dep on the new repo (once available locally). ✅
3. Begin FR-CIV-VOXEL-001 / 002 / 003 implementation (P-V1). ✅ (storage,
   write+drain, replay-determinism)

## Iteration 2 — 2026-05-22 continued

Landed on the same kickoff day; PR #296 now has 8 commits:

1. **phenotype-voxel kernel** — bootstrapped at
   <https://github.com/KooshaPari/phenotype-voxel> (private). 9 modules
   (chunk, coord, cubic_mesher, delta, lod, material, mesh, octree, world).
   **21 passing tests.** Working `VoxelWorld<T>` with deterministic
   write+drain and an engine-neutral reference `CubicMesher` with face culling.
2. **Civis civ-voxel** — re-exports kernel types + `VoxelWorld` + `CubicMesher`;
   adds FR-CIV-VOXEL-005 replay-determinism smoke and FR-CIV-VOXEL-010
   `VoxelWorld → CubicMesher` end-to-end smoke.
3. **Civis civ-protocol-3d** — first real types (`VoxelDeltaFrame`,
   `BuildingDiffFrame`, `AgentAppearanceFrame`, `Frame3d` union). Versioned
   `SCHEMA_VERSION = 0`. JSON round-trip verified as a determinism floor.
4. **Civis `clients/`** — three reference renderers scaffolded:
   - `clients/bevy-ref` is a real workspace member; headless binary builds a
     4³ voxel cube and meshes it with `CubicMesher` (`cargo run -p civ-bevy-ref`
     → `dirty events: 64 / mesh: 384 vertices, 576 indices`).
   - `clients/godot-ref` ships a README scaffold describing the Godot 4 +
     GDExt/Rust layout (project files land in the godot-ref PR).
   - `clients/unreal-show` ships a README scaffold describing the UE5 + cbindgen
     layout (project files land in the unreal-show PR, gated on the EULA
     decision).
5. **Justfile** — added `just civis-3d-verify`, `civis-3d-bevy-smoke`,
   `civis-3d-voxel-kernel` targets so the standard gate is one command.

**Workspace state:** 30 passing tests; `cargo build / test / clippy
--all-targets -D warnings / fmt --check` all green via `just civis-3d-verify`.

## Resolved this iteration

- **Cargo edition2024 transitive deps** — local toolchain bumped 1.83 → 1.95
  via `rustup update stable` (after removing the deprecated `wasm32-wasi`
  target). Resolves proptest/blake3 chains.
- **Pre-existing clippy lint in `civ-engine` tests** —
  `field-reassign-with-default` exposed by the toolchain bump, fixed.
- **`WriteSeq::next` name collision with `Iterator::next`** — renamed to
  `advance`.
- **`MaterialId` derivable default** — derived `Default` (id 0 = "air"
  convention).

## Still open

- LLM tick budget (resolved as async-multi-tick per build.md).
- Modding day-1 vs post-MVP — owner.
- Unreal client EULA path — owner.
- phenotype-voxel public/private visibility — owner.

## Cross-references

- Plan: `~/.claude/plans/weve-spent-a-lot-toasty-reddy.md`
- Build/progression design: `~/.claude/plans/civis-3d-scratch/build.md`
- Voxel kernel design: `~/.claude/plans/civis-3d-scratch/phenotype-voxel-design.md`
- PR: [#296](https://github.com/KooshaPari/Civis/pull/296)

## Iteration 3 — 2026-05-22 watch harness

- Added `civ-watch` as the local live-view harness. It ticks a background
  `Simulation` at ~10 Hz, publishes SSE snapshots at `GET /events`, serves
  the latest snapshot at `GET /snapshot`, and exposes the dashboard static
  build at `GET /`.
- Added `web/dashboard/` as a Bun/Vite/React/TanStack Router app. The UI
  subscribes to `/events`, falls back to `/snapshot` for the first poll, shows
  live/disconnected state, and renders a top-down canvas placeholder for voxel
  chunk dots.
- Added `just` targets for the combined watch/dev flow and the dashboard build
  path, plus `.gitignore` entries for `web/dashboard/node_modules/` and
  `web/dashboard/dist/`.
