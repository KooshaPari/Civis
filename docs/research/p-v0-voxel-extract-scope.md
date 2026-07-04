# P-V0 — phenotype-voxel extraction scope

**Status:** Scoping only. No extraction started.
**Branch base:** `main` @ `9e441865` (fix(ci): correct pr-governance-gate actions/github-script SHA #338)
**Related plan:** `C:\Users\koosh\.claude\plans\weve-spent-a-lot-toasty-reddy.md` (Phase V0)
**Author memory:** `[[project: phenotype-voxel kernel]]` — kernel already exists at `C:/Users/koosh/Dev/phenotype-voxel`

---

## TL;DR — the "extraction" is already half-done

`civ-voxel` is **not a fresh standalone voxel engine**. It is a Civis-side adapter
that re-exports the `phenotype-voxel` kernel verbatim and wraps it with
Civis-specific glue modules (boundary, fluid CA, materials, reactions, streaming,
worldgen, LOD). The actual voxel substrate (SVO + dense 16³ leaves, dirty queue,
fixed-point coords, `Mesher` trait) lives in the kernel crate, **already**.

The "extraction" task therefore reduces to:

1. **Swap `phenotype-voxel = { git = "...", rev = "0bbd1b7" }` → `phenotype-voxel = { path = "../../phenotype-voxel" }`**
   in `crates/voxel/Cargo.toml` (and any sibling crate that consumes `phenotype-voxel` directly).
2. **Audit which of the 7 glue modules (boundary, fluid_ca, lod, material, reactions, stream, worldgen)
   truly belong in Civis vs. the shared kernel.** A shim layer stays in `civ-voxel` for
   anything that is genuinely Civis-specific (e.g. WGSL material phases, ECS-bevy coupling).
3. **Re-examine the test surface in `lib.rs::stub_tests`** (8 tests, FR-CIV-VOXEL-000..010)
   to confirm they still pass with the path-dep swap.

The `civ-voxel` crate name and `path = "../voxel"` consumer wiring **stays**
unchanged — downstream crates (engine, agents, server, tactics, watch, build, etc.)
do not need to be touched in this phase.

---

## 1. WSM3D location (Task 1)

| Field | Value |
|-------|-------|
| Path | `C:/Users/koosh/Dev/WorldSphereMod` |
| Purpose | Unity 2022.3 BRP + D3D11 mod that ports WorldBox to true 3D voxels (companion consumer of `phenotype-voxel` ideas, NOT a Cargo consumer) |
| Status | Mod project, **does not** use `phenotype-voxel` directly (Unity + C#) |
| Implication | The "WSM3D bridge" worktree is for designing a kernel-side shim that BOTH Bevy Civis and Unity WSM3D can target, not a `Cargo.toml` cross-import. |

If the WSM3D kernel adapter is needed, it must be a **C# wrapper around the same
algorithm contracts** (chunk coord, dirty queue order, fixed-point scale) — this
is a separate worktree, not part of P-V0.

## 2. Existing `phenotype-voxel` local repo

| Field | Value |
|-------|-------|
| Path | `C:/Users/koosh/Dev/phenotype-voxel` |
| Top-level | `benches/  Cargo.lock  Cargo.toml  docs/  examples/  LICENSE-APACHE  LICENSE-MIT  README.md  sonar-project.properties  specs/` |
| Implication | The "shared kernel" already lives locally. The "extraction" is just changing the `Cargo.toml` source pointer. |

## 3. `civ-voxel` public surface inventory (Task 2)

### 3.1 Total counts

| Category | Count | Notes |
|----------|------:|-------|
| Source files | 9 | boundary, fluid_ca, lib, lod, material, reactions, stream, worldgen (+ lib) |
| Public `mod` declarations | 6 | boundary, fluid_ca, lod, material, reactions, stream, worldgen |
| Re-export blocks (kernel + stream + worldgen) | 4 | in `lib.rs` |
| Public enums | 4 | `BoundaryFace`, `BoundaryMode`, `Phase`, plus kernel `MaterialId` re-export |
| Public structs | ~14 | `ChunkRenderPlan`, `BoundaryConfig`, `Bounds3`, `MaterialDef`, `MaterialRegistry`, `CaGrid`, `StreamConfig`, `StreamStats`, `ChunkStore`, `StreamingWorld`, `ReactionResult`, `ReactionRule`, `GenWorld`, `HeightFieldGen` |
| Public traits | 1 | `WorldGen` (Send + Sync) |
| Public functions | ~16 | `contains_world_coord`, `seed_boundary_walls`, `enforce_boundary_walls`, `step`, `step_with_config`, `step_n`, `step_n_with_config`, `step_world`, `step_world_with_config`, `settle_world`, `select_mesh_detail_level`, `plan_chunk_render`, `reaction_for`, `sea_level`, `surface_height`, `generate` |
| Public consts | ~44 | `CHUNK_EDGE`, `CHUNK_EDGE_I32`, `SCHEMA_VERSION`, plus 41 `MaterialId` constants (`AIR`..`MOLD`) |
| Public arrays | 1 | `STANDARD_MATERIALS: [MaterialDef; 41]` |
| Kernel re-exports (from `phenotype_voxel`) | 25 | `Chunk`, `ChunkCoord`, `ChunkId`, `ChunkView`, `CubicMesher`, `CubicVoxel`, `DirtyChunkEvent`, `LodLevel`, `LodPolicy`, `MaterialId`, `MaterialPalette`, `MeshBuffer`, `MeshError`, `MeshResult`, `MeshVertex`, `Mesher`, `OctreeNode`, `VoxelMaterial`, `VoxelOctree`, `VoxelScaleMultiplier`, `VoxelWorld`, `WorldCoord`, `WriteSeq`, `FIXED_SCALE`, `select_lod`, `to_chunk_coord` |
| **Total `pub` items in civ-voxel** | **~80** | excludes the 25 kernel re-exports |

### 3.2 What moves to the shared kernel (path-dep swap already covers this)

**Nothing needs to "move" from civ-voxel to phenotype-voxel as a code change.**
The kernel already provides: `Chunk`, `ChunkCoord`, `ChunkId`, `ChunkView`,
`CubicMesher`, `CubicVoxel`, `DirtyChunkEvent`, `LodLevel`, `LodPolicy`, `MaterialId`,
`MaterialPalette`, `MeshBuffer`, `MeshError`, `MeshResult`, `MeshVertex`, `Mesher`,
`OctreeNode`, `VoxelMaterial`, `VoxelOctree`, `VoxelScaleMultiplier`, `VoxelWorld`,
`WorldCoord`, `WriteSeq`, `FIXED_SCALE`, `select_lod`, `to_chunk_coord`.

The change is **only** in `crates/voxel/Cargo.toml`:

```diff
-phenotype-voxel = { git = "https://github.com/KooshaPari/phenotype-voxel.git", rev = "0bbd1b7c64e64e7723c42a7417680c23860dfb5a" }
+phenotype-voxel = { path = "../../phenotype-voxel" }
```

### 3.3 What stays in `civ-voxel` as Civis glue

| Module | Why it stays | Notes |
|--------|--------------|-------|
| `boundary.rs` | Civis-specific simulation boundary walls (axis-aligned bbox clamping, world-coord containment); not kernel concern | `BoundaryFace`, `BoundaryMode`, `BoundaryConfig`, `Bounds3`, `contains_world_coord`, `seed_boundary_walls`, `enforce_boundary_walls` |
| `fluid_ca.rs` | Cellular-automata fluid step is a Civis sim mechanic, not voxel storage | `CaGrid`, 8 `step*` fns |
| `lod.rs` | Civis-side render-planning (chunk render plan), distinct from kernel `LodLevel`/`LodPolicy` | `ChunkRenderPlan`, `select_mesh_detail_level`, `plan_chunk_render` |
| `material.rs` | Civis material registry with 41 hand-authored `MaterialDef`s and 41 `MaterialId` consts | `Phase`, `MaterialDef`, `MaterialRegistry`, `STANDARD_MATERIALS`, all 41 consts |
| `reactions.rs` | Civis material-reaction rule table (fire+water → steam, acid+metal → …) | `ReactionResult`, `ReactionRule`, `REACTIONS`, `reaction_for` |
| `stream.rs` | Civis streaming-world orchestrator + `WorldGen` trait | `WorldGen` trait, `StreamConfig`, `StreamStats`, `ChunkStore`, `StreamingWorld`, `CHUNK_EDGE*` |
| `worldgen.rs` | Civis heightfield + biome-style worldgen | `GenWorld`, `sea_level`, `surface_height`, `generate`, `HeightFieldGen` |
| `lib.rs` | The adapter root, kernel re-exports, `SCHEMA_VERSION` constant | nothing moves |
| `lib.rs::stub_tests` | 8 tests, FR-CIV-VOXEL-000..010 | must keep passing through path-dep |

### 3.4 Cross-project reuse opportunities (CLAUDE.md mandate)

| Candidate | From | To | Reason |
|-----------|------|-----|--------|
| `Phase` enum (Solid/Liquid/Gas/Plasma) | `crates/voxel/src/material.rs` | phenotype-voxel `MaterialId` metadata | Pluggable phase is engine-agnostic; other Phenotype-org voxel clients (WSM3D, future PhenoSandbox) would benefit |
| `MaterialDef` shape (name, color, density, phase) | `crates/voxel/src/material.rs` | phenotype-voxel `MaterialPalette` | The 41-material hand-authored table is the most reusable Civis artifact; consider lifting the **shape** (not the contents) into the kernel |
| `ReactionRule` table driver | `crates/voxel/src/reactions.rs` | NEW: `phenotype-sim` (separate crate) | Reactions are game-mechanic, NOT voxel; but a generic (left, right) → (result) table driver is reusable. Don't move the 41 specific rules. |
| `HeightFieldGen` (parametric noise heightmap) | `crates/voxel/src/worldgen.rs` | phenotype-voxel examples/ | Already a kernel example pattern; consider promoting it from a Civis module to a kernel example to document the API |
| `STANDARD_MATERIALS` const array | `crates/voxel/src/material.rs` | stays in Civis | The 41 specific materials ARE Civis content, not kernel concern |

**Migration order** (forward-only, per PHENOTYPE_LONGTERM_STABILITY_PROTOCOL):
1. P-V0: path-dep swap (this PR, no behavior change)
2. P-V1: extract `Phase` + `MaterialDef` shape into kernel `MaterialPalette` extension
3. P-V2: extract `ReactionRule` driver into a new `phenotype-sim` crate
4. P-V3 (optional): promote `HeightFieldGen` to a kernel example

**Each step must be its own PR with stacked ordering** (per
PHENOTYPE_GIT_DELIVERY_PROTOCOL).

## 4. Dependency graph (Task 2.c)

`civ-voxel` is consumed by **11 sibling crates** in `C:/Users/koosh/Dev/civis-game`:

| Consumer | Path | Notes |
|----------|------|-------|
| `crates/agents` | `path = "../voxel"` | agent brain coupling |
| `crates/build` | `path = "../voxel"` | build/pack integration |
| `crates/civ-traffic` | `path = "../voxel"` | traffic sim on voxel grid |
| `crates/civlab-sdk` | `path = "../voxel"` | SDK surface for the CivLab game client |
| `crates/watch` | `path = "../voxel"` | file-watcher tool |
| `crates/protocol-3d` | `path = "../voxel"` | 3D wire protocol |
| `crates/engine` | `path = "../voxel"` | core game engine (with a `Phenotype-org sibling repo` comment) |
| `crates/server` | `path = "../voxel"` | multiplayer server |
| `crates/tactics` | `path = "../voxel"` | combat tactics |
| `clients/godot-ref/rust` | `path = "../../../crates/voxel"` | Godot client (secondary) |
| `clients/bevy-ref` | `path = "../../crates/voxel"` | Bevy client (primary) |

**Total: 11 consumers, all use `path = "../voxel"` (or equivalent).** Path-dep swap
on `phenotype-voxel` (not `civ-voxel`) does not require touching any of these.

## 5. Risks (Task 2.e)

| Risk | Severity | Mitigation |
|------|---------:|------------|
| Local `phenotype-voxel` is **behind** the pinned `0bbd1b7` rev (newer features added in the git dep since the pin) | Medium | `cargo check` after path swap; if errors, cherry-pick the missing commit from upstream into local first |
| Local `phenotype-voxel` is **ahead** of the pinned `0bbd1b7` rev (uncommitted local changes) | Low | `git status` in `C:/Users/koosh/Dev/phenotype-voxel` before swap; commit or stash local work first |
| CI cannot resolve the path-dep (relative path won't resolve in CI's checkout root) | High | Use a Cargo `[patch.crates-io]` or `[patch."ssh://..."]` section in workspace root, OR vendor `phenotype-voxel` as a git submodule. **Not** `path = ...` for CI. |
| Bevy client at `clients/bevy-ref` lives 2 dirs deep — `path = "../../crates/voxel"` already correct, so the new `path = "../../phenotype-voxel"` from `crates/voxel/Cargo.toml` resolves cleanly | Low | confirmed by manual path resolution |
| Test `dirty_events_sort_deterministically_through_reexport` in `lib.rs` exercises `phenotype_voxel::DirtyChunkEvent` — must pass with local kernel | Low | run `cargo test -p civ-voxel` after swap |
| WSM3D (Unity) cannot consume Rust kernel directly | High (out of scope for P-V0) | WSM3D shim work is `p-v0-wsm3d-bridge` worktree, design-only this phase |

## 6. Effort estimate (Task 2.f, agent-terms per CLAUDE.md)

| Subtask | Tool calls | Wall clock | Owner |
|---------|-----------:|-----------:|-------|
| Confirm `phenotype-voxel` local path, `git log -1`, clean tree | 2 | <30s | this PR (done) |
| Edit `crates/voxel/Cargo.toml`: git → path | 1 | <5s | `p-v0-shared-crate` worktree |
| `cargo check -p civ-voxel` (compile verify) | 1 | 30-90s | `p-v0-shared-crate` worktree |
| `cargo test -p civ-voxel` (8 stub tests pass) | 1 | 30-60s | `p-v0-shared-crate` worktree |
| `cargo check --workspace` (all 11 consumers build) | 1 | 1-3 min | `p-v0-shared-crate` worktree |
| Commit + open PR | 2 | <30s | `p-v0-shared-crate` worktree |
| **Total P-V0 PR** | **~8** | **3-6 min** | single agent, sequential |

The P-V0 PR is genuinely **3-6 minutes** of agent work. It is **not** a
"major refactor" — the heavy design work was already done when the original
`phenotype-voxel` extraction landed. We are just flipping a `Cargo.toml` line.

## 7. Worktree scaffolds (Task 4)

| Worktree | Branch | Purpose | Status |
|----------|--------|---------|--------|
| `.worktrees/p-v0-shared-crate` | `p-v0-shared-crate` | the actual extraction PR (path-dep swap + tests) | empty scaffold |
| `.worktrees/p-v1-civis-integration` | `p-v1-civis-integration` | re-wire `civ-voxel` modules after path swap lands (extract `Phase`+`MaterialDef` shape) | empty scaffold |
| `.worktrees/p-v0-wsm3d-bridge` | `p-v0-wsm3d-bridge` | C# adapter design for WSM3D, mirror of kernel contracts | empty scaffold |

All three branch from `main` @ `9e441865`.

## 8. What this PR does NOT do (deferred to P-V1+)

- Does **not** move any code out of `crates/voxel/src/`.
- Does **not** rename `civ-voxel` or change its public API.
- Does **not** touch the 11 consumer crates' `Cargo.toml` (path-dep on `phenotype-voxel` is one level deeper).
- Does **not** introduce a `[patch]` section (deferred until CI breakage is observed).
- Does **not** open a Civis↔WSM3D C# bridge (that's a separate design task).
