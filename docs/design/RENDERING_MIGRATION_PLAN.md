# RENDERING_MIGRATION_PLAN

> **Status:** Proposed (design / pre-implementation). Not a ratified ADR.
> **Branch:** `research/rendering-migration-plan`
> **Worktree:** `.worktrees/wt-rmp`
> **Scope:** `crates/voxel` + `crates/voxel-bridge` only. No `cargo` here — this
> is the phased work-breakdown that downstream PRs will execute and verify.

---

## 0. Why this document is a design doc, not ADR-019

The `docs/adr/README.md` index reserves **ADR-019 as a vacant gap** and forbids
filling it "without first re-allocating the gap or renumbering ADR-020." The
branch base has no `ADR-019-rendering-substrate-selection.md`. We therefore
deliberately record the substrate decision in a **design doc** (this file)
plus the existing `docs/adr/ADR-007-three-renderers.md` and
`docs/adr/ADR-voxel-streaming-scale.md`. The eventual ADR-019 closure is its
own housekeeping task (see §7.2).

## 1. Substrate decision (inferred from branch signals)

The Civis 3D rendering substrate is already implicitly chosen by the existing
code path. The substrate direction is unambiguous from three converging
signals:

| Signal (file:line) | Reads as |
|---|---|
| `crates/voxel/src/lib.rs` re-exports `phenotype_voxel` and pins the `phenotype-gfx` host (lib.rs doc comment) | The kernel is `phenotype-gfx` (Phenotype's `phenotype_voxel` module is the kernel world type). |
| `crates/voxel-bridge/Cargo.toml` declares `phenotype-voxel` as a `path = "../../../Phenotype/crates/phenotype-voxel"` dep | The bridge talks to the same kernel via the local Phenotype checkout. |
| `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md` "Mesh instancing" callout | When the kernel publishes a dense `voxels: [u16; 16³]` field, the renderer MUST materialise a single 16³ instanced mesh (not per-voxel `Mesh3d` entities). |
| `AGENTS.md` ("F3D0 — Bevy full `Frame3d`, Godot/Unreal **16³ mesh** when dense `voxels`") | The 16³ instanced mesh is the **Bevy branch's** dense path; the 10 Hz `VoxelDelta`/`BuildingDiff`/`AgentAppearance` `Frame3d` push is the sparse path. |
| `docs/adr/ADR-007-three-renderers.md` (Bevy primary + Godot + Unreal) | Three clients must remain in lockstep. |
| `docs/guides/voxel-emergent-vision-and-migration.md` (P-VM-3 "Bevy voxel chunk renderer") | The Bevy chunk renderer is the migration target, not a redesign. |

**Chosen substrate: hybrid kernel-driven streaming over a `phenotype-gfx`
`VoxelWorld`, with two render-paths selected per chunk by density:**

- **Sparse path (default):** consume `VoxelDelta` / `BuildingDiff` /
  `AgentAppearance` frames at 10 Hz over the WebSocket (`Frame3d` protocol)
  and write the dirty set into the kernel `VoxelWorld` via the bridge.
  This is the current production path; do not regress it.
- **Dense path (opt-in per chunk):** when a chunk's `voxels` field is dense
  (≥ 1024 non-air voxels in the 16³ block), the renderer materialises a
  single 16³ instanced mesh per chunk using `InstancedMesh` / `Mesh3d` +
  `StandardMaterial` with one `InstanceData` per solid cell. This is the
  "mesh instancing" recommendation from CIV-0601 and is the migration target
  for dense builds (settlements, megascans, Fortress interiors).

The hybrid is **not** an interim step: both paths coexist permanently.
Sparse handles edits and low-density terrain (where the dominant cost is
mutations, not triangles); dense handles megascans, buildings, and
Godot/Unreal dense snapshots where per-cell `Mesh3d` would blow the entity
budget.

## 2. Scope, non-goals, and invariants

### 2.1 In scope
- `crates/voxel` (engine-agnostic kernel helpers: `lod`, `stream`, `worldgen`,
  `fluid_ca`, `material`, `material_pbr`, `hud`, `lib`).
- `crates/voxel-bridge` (the Bevy ECS chunk lifecycle glue: `drain_and_schedule_remesh`).

### 2.2 Out of scope (deferred or owned elsewhere)
- **Godot** render migration — owned by `clients/godot-ref`. Touched here
  only via the shared `Frame3d` protocol contract.
- **Unreal CivShow** — owned by `clients/unreal-show`. The dense 16³ mesh
  path here lands the same `Frame3d` payload; Unreal has its own UBT
  importer.
- **Quixel / Megascans** mesh import — engineering slots only, artists
  import via Bridge (AGENTS.md "Product-only"). Not part of this migration.
- **Full CIV-0700** modding (capability enforcement, mod store, hot reload)
  — explicitly out of scope; v3 partial-good covers manifest + `.civmod`.

### 2.3 Hard invariants
1. **Determinism:** same seed + tick + input must yield bit-identical chunk
   snapshots. The bridge must never reorder `DirtyChunkEvent`s or split a
   single `ChunkDirty::Both` into two consumer-side passes.
2. **God-tool editability:** all 50 verbs in `GODTOOLS_IMPL_PLAN.md` must
   route through `push_voxel_write` → bridge → `ChunkDirty` → render. The
   dense path is read-side; edits stay sparse.
3. **Physics coupling:** `PHYSICS_INTEGRATION_PLAN.md` reads
   `phenotype_voxel::VoxelWorld` as the source of truth. The renderer's
   dense mesh is a *view* — physics never reads it.
4. **Replay-bus parity:** `mod.loaded.v1` and `Frame3d` events on the
   replay bus are unchanged in shape.
5. **No bevy dep in `crates/voxel`:** `crates/voxel/src/hud.rs` already
   asserts "NO `bevy` dependency" — the dense-path mesh materialisation
   must live in `voxel-bridge` (which is allowed to depend on `bevy_ecs`)
   or in a new `voxel-render` crate, not in `voxel`.

## 3. Current rendering architecture (baseline)

```
         ┌────────────────────┐                ┌──────────────────────┐
server → │ Frame3d (10 Hz WS) │ ──VoxelDelta──▶│ civ-server dispatcher│
         └────────────────────┘                └──────────┬───────────┘
                                                          │ push_voxel_write
                                                          ▼
                                            ┌────────────────────────┐
                                            │ crates/voxel::VoxelWorld│
                                            │ (phenotype_voxel re-exp)│
                                            └──────────┬─────────────┘
                                                       │ DirtyChunkEvent
                                                       ▼
                                  ┌────────────────────────────────────┐
                                  │ crates/voxel-bridge                │
                                  │   drain_and_schedule_remesh        │
                                  │   (despawn stale entity,           │
                                  │    spawn replacement entity)       │
                                  └──────────────┬─────────────────────┘
                                                 ▼
                                  ┌────────────────────────────────────┐
                                  │ Bevy ECS: Mesh3d + StandardMaterial │
                                  │   per chunk (one entity per chunk)  │
                                  └────────────────────────────────────┘
```

- The bridge holds `chunk_entities: HashMap<IVec3, Entity>` and despawns +
  respawns one entity per dirty chunk per tick (one-shot remesh). For
  dense chunks this is wasteful — the entity is regenerated from scratch
  on every material change.
- `crates/voxel/src/lod.rs` already exports `plan_chunk_render(...)` and
  `ChunkDirty::{StorageChanged, MeshLodChanged, Both}` — these are the
  hand-off points the new dense path will consume.
- `crates/voxel/src/fluid_ca.rs` exports `last_changed_chunks` —
  explicitly documented as "the consumer-visible remesh list … powers the
  Bevy despawn+respawn loop and the kernel-side `DirtyChunkEvent`
  writeback." This is the second render hand-off (CA surfaces).

## 4. Target architecture (hybrid sparse + dense)

```
         ┌────────────────────┐                ┌──────────────────────┐
server → │ Frame3d (10 Hz WS) │ ──VoxelDelta──▶│ civ-server dispatcher│
         └────────────────────┘                └──────────┬───────────┘
                                                          │ push_voxel_write
                                                          ▼
                                            ┌────────────────────────┐
                                            │ crates/voxel::VoxelWorld│
                                            │ (phenotype_voxel re-exp)│
                                            └──────────┬─────────────┘
                                                       │ DirtyChunkEvent
                                                       ▼
                                  ┌────────────────────────────────────┐
                                  │ crates/voxel-bridge                │
                                  │   classify_dirty_chunk(key)        │
                                  │     → Sparse (default)             │
                                  │     → Dense  (if voxels.density    │
                                  │         ≥ DENSE_THRESHOLD)         │
                                  └──────────────┬─────────────────────┘
                                                 │
                          ┌──────────────────────┴──────────────────────┐
                          ▼                                             ▼
        ┌───────────────────────────────┐         ┌───────────────────────────────┐
        │ SPARSE: drain_and_schedule_   │         │ DENSE: dense_chunk_instanced  │
        │   remesh (unchanged)          │         │   (one entity per chunk with  │
        │   despawn + respawn per       │         │    InstancedMesh / 16³        │
        │   chunk                       │         │    instance buffer)           │
        └───────────────────────────────┘         └───────────────────────────────┘
                          │                                             │
                          └──────────────────────┬──────────────────────┘
                                                 ▼
                                  ┌────────────────────────────────────┐
                                  │ Bevy ECS:                          │
                                  │   Sparse → Mesh3d + StandardMat.   │
                                  │   Dense  → Mesh3d + InstanceBuffer │
                                  │                + StandardMat.      │
                                  └────────────────────────────────────┘
```

- **Classification** is a pure function over the chunk's snapshot. It is
  recomputed on every `ChunkDirty::StorageChanged`. The threshold and the
  classification logic are owned by `crates/voxel-bridge` (new module)
  with a thin trait that `crates/voxel` exposes (no `bevy` dep).
- **Path transitions** are allowed per chunk per tick. A chunk that
  crosses the threshold in either direction is `ChunkDirty::Both` —
  the sparse and dense handlers both flush.
- **Fluid CA surfaces** stay sparse — they are inherently high-mutation,
  low-density (water / gas cells), and instancing them is wasted work.

## 5. Edit sites (the work-breakdown)

The two crates touch only a handful of well-isolated seams. Below is the
authoritative site map. The column "Public surface?" is the constraint that
keeps `crates/voxel` engine-agnostic.

| # | File | Lines (current) | What changes | Public surface change? |
|---|------|-----------------|--------------|------------------------|
| E1 | `crates/voxel/src/lib.rs` | re-export block | Add `pub use phenotype_voxel::ChunkDensity;` (or define a thin newtype `voxel::ChunkVoxelCount(pub u32)`) so `voxel-bridge` can read density without importing `bevy`. | Yes — additive re-export only. No `bevy` dep. |
| E2 | `crates/voxel/src/lod.rs` | `plan_chunk_render` (≈L120) | Extend the return to include `pub density: ChunkVoxelCount` so the render planner emits density alongside LOD. New helper `pub fn classify_density(count: ChunkVoxelCount) -> ChunkRenderClass { Sparse \| Dense }`. | Yes — additive. |
| E3 | `crates/voxel/src/stream.rs` | streaming world tick | When streaming a new chunk in, populate the kernel with the dense `voxels` field so the bridge can decide sparse vs dense at first remesh. No API change; just contract. | No — internal. |
| E4 | `crates/voxel/src/fluid_ca.rs` | `last_changed_chunks` (L63) | Document (in rustdoc) that fluid surfaces stay **sparse** regardless of density — CA churn is per-cell and per-tick, so instancing wastes CPU. | No — rustdoc only. |
| E5 | `crates/voxel/src/material.rs` + `material_pbr.rs` | palette tables | Add a `pub fn palette_emit_mode(id: MaterialId) -> EmitMode { Solid \| Transparent \| Emissive }` so the dense path can pre-sort the instance buffer by emission for forward+ rendering. | Yes — additive. |
| E6 | `crates/voxel/src/worldgen.rs` | `HeightFieldGen` (L541+) | Add a test that asserts a generated chunk reports `ChunkVoxelCount > 0` (so the bridge has something to classify). | No — test only. |
| E7 | `crates/voxel-bridge/src/lib.rs` | `drain_and_schedule_remesh` (L79) | Split into `sparse::remesh(bridge, commands, chunk_entities)` and `dense::remesh(bridge, commands, dense_entities)`. Top-level `drain_and_schedule_remesh` dispatches based on `classify_density` from E2. | No — internal split, public fn signature preserved. |
| E8 | `crates/voxel-bridge/src/lib.rs` | `chunk_key_from_chunk_id` (L115) | Add a sibling `chunk_density_from_chunk_id(bridge, chunk_id) -> ChunkVoxelCount` reading from the kernel world. | Yes — additive. |
| E9 | `crates/voxel-bridge/src/lib.rs` | `CivisVoxelBridge` (L24) | Add `dense_entities: HashMap<IVec3, Entity>` and `dense_instance_buffers: HashMap<IVec3, Arc<Vec<InstanceData>>>`. Keep `chunk_entities` field for the sparse path — backwards compatible. | Yes — additive fields on a struct that is constructed in tests, not exposed as part of any public API. |
| E10 | `crates/voxel-bridge/src/lib.rs` | tests (L140+) | Add tests: `dense_chunk_uses_instance_buffer`, `chunk_transitions_sparse_to_dense_dense_to_sparse_are_idempotent`, `ca_surfaces_stay_sparse`. | No — tests only. |
| E11 | `crates/voxel-bridge/Cargo.toml` | `[dependencies]` | Add `bevy_pbr = { workspace = true }` (for `StandardMaterial`) only if not already pulled in. The new dense module reuses existing types — no new top-level deps. | Yes — additive. |

All other `crates/voxel*` files are out of scope for this migration.

## 6. Phased work-breakdown

Each phase ends with a verification gate (the AGENTS.md "verify before you
claim done" table). No phase advances until its gate is green.

### Phase 0 — Decision housekeeping (no code)

- **Goal:** stop the ADR-019 ambiguity from leaking into downstream PRs.
- **Work:**
  - Open a small follow-up ADR-PR that either (a) re-allocates the gap
    and renumbers ADR-020, or (b) declares the substrate decision in
    `docs/adr/ADR-007-three-renderers.md` as a §"Substrate" addendum.
  - Link this design doc from both `docs/adr/README.md` and
    `docs/design/README.md` (if it exists; create if not).
- **Gate:** design doc reachable from `docs/adr/README.md` "see also" list.
- **Effort:** < 1 day.

### Phase 1 — Density classification in the engine-agnostic layer

- **Goal:** every dirty chunk can be classified Sparse vs Dense from a
  pure function, with no `bevy` import in `crates/voxel`.
- **Work (E1, E2, E6):**
  - `voxel::ChunkVoxelCount` newtype + `classify_density` helper.
  - Extend `plan_chunk_render` to emit density.
  - Add `worldgen` test that asserts density > 0 for any non-trivial seed.
- **Gate:** `cargo test -p voxel` green; `cargo clippy -p voxel -- -D warnings`
  green; **no** `bevy` line in `cargo tree -p voxel`.
- **Effort:** 1–2 days.

### Phase 2 — Sparse path preserved (regression safety net)

- **Goal:** prove the existing sparse path is byte-identical to today
  after the dispatcher split in E7.
- **Work (E7 — sparse half, E10 sparse regression test):**
  - Move the current `drain_and_schedule_remesh` body into
    `voxel_bridge::sparse::remesh(...)`. Functionally identical.
  - Re-export at the crate root as
    `pub use sparse::remesh as drain_and_schedule_remesh;` so all
    existing call sites (in `civ-server`, `civ-watch`, the Bevy client)
    continue to compile.
  - Add a "before/after" snapshot test: replay a 200-tick fixture and
    assert the chunk-entity churn order matches the pre-migration
    golden.
- **Gate:** `cargo test -p voxel-bridge` green; the golden test is the
  regression net. **No entity churn delta** for any sparse-only fixture.
- **Effort:** 2–3 days.

### Phase 3 — Dense path: instanced 16³ mesh

- **Goal:** a single `Mesh3d` + `InstancedMesh` per dense chunk, with
  one `InstanceData` per solid cell, sorted by `EmitMode`.
- **Work (E5, E7 — dense half, E8, E9, E10 dense tests, E11):**
  - `voxel_bridge::dense::remesh(bridge, commands, dense_entities)`.
  - Reads `ChunkVoxelCount` (from Phase 1) and the chunk snapshot;
  builds a flat `Vec<InstanceData>` of `vec3` position + `vec4` tint
  per solid cell, sorted by `palette_emit_mode` for forward+.
  - One persistent entity per dense chunk, mesh asset cached. Reuses
  the entity across `StorageChanged` ticks — only the instance
  buffer is rebuilt. This is the dominant win.
  - Threshold constant `DENSE_DENSITY_THRESHOLD: u32 = 1024` lives in
    `voxel_bridge::dense` and is documented as "tunable, not part of
    the public API."
  - Tests: dense chunk uses 1 entity regardless of mutation count;
    sparse↔dense transitions are idempotent (a chunk that flickers
    across the threshold does not multiply entities).
- **Gate:** `cargo test -p voxel-bridge` green (incl. new dense tests);
  entity-count micro-benchmark: a 200-tick dense fixture must show
  ≤ 1 entity per chunk (currently 1 per chunk per dirty tick, i.e.
  dozens → 1).
- **Effort:** 5–7 days.

### Phase 4 — God-tool editability preservation

- **Goal:** every one of the 50 verbs in `GODTOOLS_IMPL_PLAN.md`
  remains routed through the dispatcher, sparse path, and
  `push_voxel_write`. The dense path is a viewer; it never short-
  circuits an edit.
- **Work:**
  - Add a contract test: walk the 50 god-tools verb enum, instantiate
    one of each, and assert it produces a `DirtyChunkEvent` whose
    `ChunkDirty` is `StorageChanged` (never `MeshLodChanged` alone) —
    i.e. the dispatcher always takes the sparse path for edits.
  - `voxel_bridge` test: a single god-tool verb against a dense
    chunk produces exactly one sparse-path event and one dense-path
    buffer rebuild, in that order, never skipping the sparse step.
- **Gate:** god-tool contract test green; manual smoke through the
  web L2 panel (using the existing `npm test` + `npm run build`
  gates from AGENTS.md).
- **Effort:** 2–3 days.

### Phase 5 — Physics coupling preservation

- **Goal:** physics continues to read `phenotype_voxel::VoxelWorld`
  via `crates/voxel`. The dense mesh is a view, not a source.
- **Work:**
  - Document (in `crates/voxel-bridge/src/dense.rs` rustdoc) that the
    dense instance buffer is **not** a collision source.
  - Cross-check `PHYSICS_INTEGRATION_PLAN.md` (already on file): any
    "physics reads dense mesh" plan is invalid — the kernel
    `VoxelWorld` is the single source. Add a `compile_fail`-style
    doctest on the dense module asserting the bridge does not export
    a `DenseChunkCollider` type.
  - Confirm `crates/voxel-bridge` does not add a `bevy_rapier` or
    `rapier3d` dep.
- **Gate:** `cargo tree -p voxel-bridge | grep -i rapier` returns
  nothing; design doc + `PHYSICS_INTEGRATION_PLAN.md` cross-link
  updated.
- **Effort:** 1 day (mostly docs + a single compile_fail doctest).

### Phase 6 — Cross-client parity (Godot + Unreal)

- **Goal:** Godot's `Zylann/godot_voxel` path and Unreal's 16³ mesh
  import path both consume the same `Frame3d` payload that the Bevy
  dense path consumes. **No protocol change.**
- **Work:**
  - Confirm (in `docs/development-guide/fr-godot-attach.md` and
    `docs/development-guide/fr-unreal-agent-playbook.md`) that the
    `voxels: [u16; 16³]` dense field already drives the 16³ mesh
    path on both clients. If not, file follow-up issues — do **not**
    change the protocol in this migration.
  - Add a doc-only cross-link from
    `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md`
    "Mesh instancing" section to this design doc.
- **Gate:** doc cross-links green; no protocol diff in
  `docs/specs/CIV-0101-two-zoom-lod-v1.md` (L2 spec frozen).
- **Effort:** 0.5–1 day.

### Phase 7 — Rollout & golden-state cleanup

- **Goal:** ship the hybrid behind a feature flag, then flip the
  default. No big-bang.
- **Work:**
  - `voxel-bridge` reads `cfg(flag = "voxel_dense_renderer")` (or env
    var `CIVIS_VOXEL_DENSE=1`) to choose between sparse-only and
    hybrid. Default = hybrid (both paths compiled in, classification
    decides per chunk).
  - Update `scripts/agent-smoke.ps1` to set the flag; verify the
    smoke gate (the AGENTS.md "Verify before you claim done" table)
    is still green with the flag enabled.
  - Capture a new golden-state file (`crates/voxel-bridge/tests/
    golden/dense_hybrid_200t.json`) for future regression.
- **Gate:** full `just civis-3d-verify` + `.\scripts\agent-smoke.ps1`
  green; new golden in tree.
- **Effort:** 2–3 days.

### Phase dependency DAG

```
Phase 0 (housekeeping) ─────┐
                            ├──▶ Phase 1 (classify) ──▶ Phase 2 (sparse regression) ──┐
                                                                                      ├──▶ Phase 3 (dense impl)
                                                                                      ├──▶ Phase 4 (god-tools)
                                                                                      ├──▶ Phase 5 (physics)
                                                                                      └──▶ Phase 6 (cross-client docs)
                                                                                                      │
                                                                                                      ▼
                                                                                              Phase 7 (rollout)
```

Phases 4, 5, and 6 are independent and can run in parallel after Phase 3.
Total effort estimate: **13–19 days**, not including review and the eventual
ADR-019 closure (see §7.2).

## 7. Risks and mitigations

### 7.1 Render↔physics drift
**Risk:** someone "optimises" by having physics read the dense instance
buffer directly. **Mitigation:** Phase 5 compile_fail doctest + the
"kernel `VoxelWorld` is the single source" invariant in §2.3.

### 7.2 ADR-019 drift
**Risk:** a future PR writes `ADR-019-rendering-substrate-selection.md`
without first clearing the gap. **Mitigation:** Phase 0 closeout;
explicit "see also" link from `docs/adr/README.md`.

### 7.3 Bevy version skew
**Risk:** `bevy_pbr` / `bevy_ecs` bump changes `InstancedMesh` API. **Mitigation:**
pin the Bevy version in `Cargo.toml` workspace; if a bump is required,
re-run the entity-count micro-benchmark from Phase 3.

### 7.4 Dense-path memory bloat
**Risk:** the instance buffer for a single 16³ chunk is up to
16 × 16 × 16 × `(vec3 + vec4)` = 4096 × 32 B = 128 KiB. With 100 dense
chunks in view, that's 12.5 MiB. **Mitigation:** cap with
`DENSE_DENSITY_THRESHOLD` + an LRU eviction in `dense_entities` (sized
by visible chunk count, not total world chunks).

### 7.5 Sparse/dense flicker
**Risk:** a chunk that hovers around the threshold causes path-switch
churn. **Mitigation:** Phase 3 test
`chunk_transitions_sparse_to_dense_dense_to_sparse_are_idempotent`
asserts the entity survives a round-trip; the threshold has built-in
hysteresis (`DENSE_ENTER` ≥ 1024, `DENSE_EXIT` ≤ 768).

## 8. Open questions (parking lot)

| # | Question | Owner | Decision deadline |
|---|----------|-------|-------------------|
| Q1 | Is the threshold a u32 absolute count, or a percentage? | `civ-server` maintainer | Before Phase 3 start. |
| Q2 | Should `MaterialId::Transparent` ever go dense (glass walls)? | `voxel-bridge` maintainer | Before Phase 3 start. Default = no. |
| Q3 | Does the dense path need to publish to the replay bus? | Replay bus owner | Before Phase 7. Default = no (sparse is the source of truth on the bus). |
| Q4 | Where does the `bevy_pbr` dep live — `voxel-bridge` or a new `voxel-render` crate? | `voxel-bridge` maintainer | Before Phase 3 start. Default = `voxel-bridge` (one fewer crate to wire). |
| Q5 | Does Quixel / Megascans (AGENTS.md "Product-only") change the threshold? | L5 visual pass owner | Out of scope — Phase 6 only cross-links, does not decide. |

## 9. Verification matrix (one row per phase, one column per gate)

| Phase | Code change | Unit tests | Clippy / fmt | `cargo test -p voxel` | `cargo test -p voxel-bridge` | Agent smoke | Web dashboard | Cross-client docs |
|-------|-------------|------------|--------------|------------------------|--------------------------------|-------------|----------------|---------------------|
| 0 | — | — | — | — | — | — | — | link added |
| 1 | E1, E2, E6 | ✓ | ✓ | ✓ | — | — | — | — |
| 2 | E7 sparse | ✓ | ✓ | — | ✓ (golden) | — | — | — |
| 3 | E5, E7 dense, E8, E9, E10, E11 | ✓ | ✓ | — | ✓ | — | — | — |
| 4 | god-tool contract test | ✓ | ✓ | — | ✓ | ✓ (panel) | ✓ (`npm test` + `npm run build`) | — |
| 5 | rustdoc + compile_fail doctest | ✓ | ✓ | — | ✓ | — | — | cross-link to PHYSICS_INTEGRATION_PLAN |
| 6 | doc cross-links | — | — | — | — | — | — | ✓ |
| 7 | feature flag + golden | ✓ | ✓ | — | ✓ | ✓ (`agent-smoke.ps1`) | — | — |

End of plan.
