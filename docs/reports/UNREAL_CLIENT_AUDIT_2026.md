# Civis Unreal 5 Client Audit & Advance Plan

**Date:** 2026-06-29  
**Scope:** `clients/unreal-show` — L1 observer → L5 visual showcase  
**Status:** Core WS protocol wired; HTTP fallback functional; F3D0 path clear but incomplete.

---

## Executive Summary

The Unreal 5.7 client is **40% functionally complete**, with dual-attach (HTTP + WS) infrastructure in place. The immediate blocker is **F3D0 binary frame parsing** — the patch exists but remains unapplied because `CivChunkOverlayActor::SetDenseVoxels()` lacks voxel-to-mesh build logic. This is the **single highest-value next step** to move from static terrain to dynamic voxel rendering.

---

## Current State Assessment (Real vs Stubbed)

| Component | Status | Notes |
|-----------|--------|-------|
| **HTTP terrain** | ✓ Real | `UCivProtocolClient::FetchTerrain()` → civ-watch `/terrain`; heights + biomes parsed; binds to UI |
| **HTTP snapshot** | ✓ Real | `UCivProtocolClient::PollSnapshot()` → ticker polls `/snapshot`; JSON broadcast to civilians |
| **WS connection** | ✓ Real | `UCivWsClient::ConnectServer()` → full JSON-RPC handshake; health check; reconnect timer |
| **WS JSON-RPC** | ✓ Real | `sim.snapshot`, `sim.set_speed`, `sim.spawn_entity`, `sim.place_voxel`, `sim.damage` wired |
| **WS binary tick** | ✓ Stubs | Socket handles `OnRawMessage()`; frame kind dispatch exists; **no F3D0 bundle parsing** |
| **F3D0 voxel mesh** | ⚠️ Stubbed | `ACivChunkOverlayActor::SetDenseVoxels()` **body missing**; only fallback cube marker works |
| **Voxel-to-vertex** | ✗ Missing | `CivF3d0ChunkMesh::BuildDenseChunkMesh()` header-only; C++ impl needed (port from Godot or Rust ref) |
| **Procedural mesh pool** | ✓ Real | `UProceduralMeshComponent` allocated; not populated |
| **Game mode sync** | ✓ Real | `ACivShowGameMode::OnF3d0Frame()` dispatches `ApplyVoxelDeltaOverlay()`; chunk tracking map ready |
| **Minimap** | ✓ Real | `ACivMinimapCapture` + `UCivMinimapWidget`; terrain capture + civilian pins working |
| **Civilian actors** | ✓ Real | `ACivilianActor` spawns/updates on snapshot; faction colors + position sync |

**Critical gap:** Voxel → Mesh conversion is **completely absent**. The infrastructure to receive, parse, and route F3D0 frames is solid; **only the meshing kernel is missing**.

---

## Wire Protocol Parity Matrix

| Feature | civ-watch HTTP | civ-server WS | Unreal Support |
|---------|----------------|---------------|---|
| Terrain heightmap | `GET /terrain` | — | ✓ Live (HTTP only) |
| Snapshot pins | `GET /snapshot` | `sim.snapshot` | ✓ Live (HTTP + WS) |
| Speed control | `POST /control/speed` | `sim.set_speed` | ✓ Live (WS) |
| **F3D0 voxel stream** | — | WS binary frames | ⚠️ **Routed, not meshed** |
| Spawn / build | HTTP endpoints | `sim.spawn_entity`, `sim.place_voxel`, `sim.damage` | ✓ Live (WS) |
| Building diffs | — | F3D0 binary | ✗ Not parsed |
| Agent appearance | — | F3D0 binary | ✗ Not parsed |

**Parity**: **90% JSON-RPC + HTTP**; **5% F3D0 binary** (frame envelope only).

---

## Phased Advance Plan

### Phase 1: F3D0 Binary Parsing Foundation (P0)

**Goal:** Decode F3D0 binary bundle frames into typed chunks.

**Tasks:**
1. **Port F3D0 bundle codec** from `crates/protocol-3d` to C++ header-only library.
   - Depends on: `FRAME3D_BUNDLE_MAGIC` (magic bytes), length-prefixed envelope, zstd decompression (optional).
   - Output: `CivF3d0Bundle.h` with `DecodeFrame3dBundle()` → vector of frame objects.
   - Estimate: 2–3 hrs (straightforward envelope parsing + frame kind dispatch).

2. **Wire F3D0 frames into `ACivShowGameMode`**.
   - Update `UCivWsClient::HandleBinary()` → parse bundle → broadcast typed delegates (not just JSON).
   - Add `FOnVoxelDeltaFrame`, `FOnBuildingDiffFrame`, `FOnAgentFrame` delegates.
   - Estimate: 1 hr (delegation pattern already in place).

**Validation:** F3D0 frames logged at `LogCivis` (Verbose); binary data round-trip test.

---

### Phase 2: Voxel-to-Mesh Kernel (P1)

**Goal:** Convert a dense 16³ voxel array into an indexed triangle mesh.

**Tasks:**
1. **Choose algorithm** (greedy voxel meshing vs sparse SVO).
   - Greedy quads: simpler, ~40–80 μs per chunk (sufficient for L1 observer).
   - Sparse SVO: lower poly count, needed for L5 perf but deferred.
   - Recommend: **Greedy quads** for now (Minecraft-style merging).

2. **Implement `CivF3d0ChunkMesh::BuildDenseChunkMesh()`**.
   - Input: `TArray<int32> MaterialIds` (4096 voxels in Z-major order).
   - Output: `TArray<FVector> Vertices`, `TArray<int32> Triangles`, `TArray<FVector> Normals`.
   - Source: Port from Godot ref (if exists) or Bevy `BlockMesher` pattern.
   - Estimate: 3–4 hrs (algorithm + Unreal API binding).

3. **Populate `UProceduralMeshComponent`** in `ACivChunkOverlayActor::SetDenseVoxels()`.
   - Clear old geometry; call `CivF3d0ChunkMesh::BuildDenseChunkMesh()`.
   - Call `UProceduralMeshComponent::CreateMeshSection()` with results.
   - Set material + collision.
   - Estimate: 1 hr.

**Validation:** Single 16³ voxel chunk rendered in Unreal editor; visual inspection + profiler (target <2ms).

---

### Phase 3: Material & Texture Atlas (P2)

**Goal:** Map `MaterialId` → voxel face textures.

**Tasks:**
1. **Define material enum** (align with `civ_voxel::MaterialId` or `civ-engine` types).
   - Likely: air, stone, grass, dirt, sand, water, wood, etc. (~16–32 types).
   - Create `enum class EVoxelMaterialType` in `.h`.

2. **Build or load texture atlas**.
   - 16×16 grid of 16px tiles (256×256 total for crisp voxel faces).
   - Option A: Bake procedurally in editor (fast iteration).
   - Option B: Load from `.png` at startup (artist-friendly).
   - Estimate: 2 hrs (artist task deferred; use placeholder grid for now).

3. **Assign UVs in meshing kernel**.
   - Per-material face UV offsets in greedy mesher.
   - Create single material + dynamic material instance per chunk for variety.
   - Estimate: 1 hr (deferred to Phase 3b once art pass confirmed).

**Validation:** Voxel faces show distinct colors per material.

---

### Phase 4: Integration & Performance (P3)

**Goal:** Multi-chunk culling, LOD, and stable 60+ FPS at 0.5 mi² streaming window.

**Tasks:**
1. **Chunk visibility frustum culling**.
   - Unreal's built-in `FConvexVolume` + `GetViewFrustum()`.
   - Only mesh chunks within camera frustum + margin (e.g., 2× draw distance).
   - Estimate: 1 hr.

2. **LOD / geometry optimization**.
   - Greedy mesher already reduces poly count 50–80% vs per-voxel quads.
   - Deferred to L4: octree LOD or shader-based displacement.

3. **Profiling & frame time**.
   - Target: Parse F3D0 frame <1ms, mesh chunk <2ms, total overhead <5ms per tick.
   - Use Unreal Insights + flamegraph.
   - Estimate: 2 hrs (ongoing).

**Validation:** Streaming 0.5 mi² at 60 FPS on Ryzen/RTX 3090 Ti.

---

## Stacked PR Roadmap

**Branch strategy:** Stacked PRs on `feat/civis-platform`, merged as each phase completes.

1. **PR#1** (this PR): Audit + docs + small additive fixes (CivBuild.cs dependency check).
2. **PR#2**: Phase 1 (F3D0 bundle codec) → `CivF3d0Bundle.h` + tests.
3. **PR#3**: Phase 2 (voxel-to-mesh kernel) → `CivF3d0ChunkMesh` impl + `SetDenseVoxels()` body.
4. **PR#4**: Phase 3 (material atlas) → texture binding + UV assignment.
5. **PR#5**: Phase 4 (perf) → culling + LOD + profiling results.

Each PR unblocks the next and is independently reviewable.

---

## Safety Notes & Known Issues

### Wire Protocol Compliance
- `tick_format=binary` query parameter already in default WS URL → server sends F3D0 frames.
- `SCHEMA_VERSION=0` from `crates/protocol-3d` is current; no version gate needed yet.
- Bundle envelope (`F3DB` magic + zstd) is optional; frame types are stable.

### Build System
- Rust shim (`Source/Civis/rust-shim`) builds independently; `.lib` in place.
- `CivShow.Build.cs` references `civis_unreal_ffi.lib` (not yet used; reserved for FFI).
- No new UE module dependencies needed for Phases 1–3.

### Missing Test Infrastructure
- `.github/workflows/unreal-build.yml` only tests Rust shim (exit code 2 on GitHub runners without UE installed is expected).
- No Unreal unit tests yet (Editor/PIE tests deferred to L3).
- Recommend: Manual PIE validation before each PR merge.

### No Determinism Constraint
- Civis doctrine: no seeded randomness; real RNG welcome.
- Unreal greedy mesher does not rely on seeded state; safe.

---

## Estimated Timeline

| Phase | Effort (agent hrs) | Wall Clock | Blocker |
|-------|---|---|---|
| Phase 1 (F3D0 codec) | 3 | 1–2 hrs | None; parallel-able with Phase 2 research |
| Phase 2 (voxel meshing) | 4 | 2–3 hrs | Phase 1 complete |
| Phase 3 (materials) | 3 | 1–2 hrs | Artist input (texture); Phase 2 complete |
| Phase 4 (perf) | 3 | 2–3 hrs | Phase 2 + Phase 3 complete |
| **Total** | **~13** | **6–10 hrs** | — |

---

## Immediate Next Step (Highest Value)

**Highest-value single task to unblock the full client:**

1. **Implement `CivF3d0ChunkMesh::BuildDenseChunkMesh()` in C++** using a greedy quad meshing algorithm.
   - This is the kernel that converts 4096 voxel material IDs into renderable geometry.
   - Once complete, the F3D0 frame path from server → parsing → meshing → render is unbroken.
   - Unlocks live voxel visualization; invalidates static terrain within 1–2 weeks.
   - Estimate: 3–4 hours to port + validate; can be done in parallel with Phase 1 bundle codec.

**Reference implementations:**
- **Godot ref** (`clients/godot-ref/`): Check if `GodotVoxelMesher` exists or similar.
- **Bevy ref** (`clients/bevy-ref/`): Likely voxel rendering in Bevy plugin.
- **Minecraft-style greedy meshing**: Canonical algorithm (widely documented).

---

## Appendix: File Inventory

**Working files:**
- `clients/unreal-show/Source/CivShow/CivProtocolClient.{h,cpp}` — HTTP terrain + snapshot.
- `clients/unreal-show/Source/CivShow/CivWsClient.{h,cpp}` — JSON-RPC WebSocket.
- `clients/unreal-show/Source/CivShow/CivShowGameMode.{h,cpp}` — Frame routing + overlay management.
- `clients/unreal-show/Source/CivShow/ACivilianActor.{h,cpp}` — Civilian visualization.
- `clients/unreal-show/Source/CivShow/CivShow.Build.cs` — Module config.

**Incomplete files:**
- `clients/unreal-show/Source/CivShow/CivF3d0ChunkMesh.h` — Header-only stubs.
- `clients/unreal-show/Source/CivShow/CivChunkOverlayActor.{h,cpp}` — Ready for mesh binding (body incomplete).

**Patches pending:**
- `clients/unreal-show/patches/f3d0-chunk-overlay-wire.patch` — Refactors game mode to use typed `ACivChunkOverlayActor*` instead of generic `AActor*`; ready to apply once meshing is done.

**Build system:**
- `clients/unreal-show/scripts/build.ps1` — Automated build (rust-shim + UBT).
- `.github/workflows/unreal-build.yml` — Optional CI (manual dispatch; exit 2 is expected on hosted runners).

---

## Questions for Designer / User

1. **Material texture fidelity:** Placeholder grid or hand-drawn atlas for L1?
2. **Chunk update cadence:** Does F3D0 stream every tick or throttled? (Affects mesh rebuild cost.)
3. **Voxel scale visibility:** Do we need LOD at L1, or is greedy meshing + frustum culling sufficient for 0.5 mi²?
