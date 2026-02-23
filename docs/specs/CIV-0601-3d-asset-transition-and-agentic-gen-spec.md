# CIV-0601: 3D Asset Transition and Agentic Generation Pipeline

**Version:** 1.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Engine, Art Pipeline, and Infrastructure Teams
**References:**
- CIV-0001 — Core Simulation Loop (Deterministic Tick Architecture)
- CIV-0200 — Client Protocol (JSON-RPC WebSocket)
- CIV-0300 — RTS UI/UX Specification
- CIV-0500 — Performance Optimization Spec
- CIV-0600 — 2D SVG Art Pipeline (ArtSpec IR)

---

## Table of Contents

1. [Transition Roadmap and Philosophy](#1-transition-roadmap-and-philosophy)
   - 1.1 [Why 2D First](#11-why-2d-first)
   - 1.2 [Non-Blocking Architecture Contract](#12-non-blocking-architecture-contract)
   - 1.3 [ArtSpec IR as the Renderer-Agnostic Boundary](#13-artspec-ir-as-the-renderer-agnostic-boundary)
   - 1.4 [Milestone Gates](#14-milestone-gates)
   - 1.5 [Decision Log](#15-decision-log)
2. [3D Client Architecture](#2-3d-client-architecture)
   - 2.1 [Technology Stack](#21-technology-stack)
   - 2.2 [Scene Graph Design](#22-scene-graph-design)
   - 2.3 [Camera System](#23-camera-system)
   - 2.4 [LOD Architecture](#24-lod-architecture)
   - 2.5 [Protocol Compatibility (CIV-0200)](#25-protocol-compatibility-civ-0200)
3. [glTF Asset Pipeline](#3-gltf-asset-pipeline)
   - 3.1 [Format Decisions](#31-format-decisions)
   - 3.2 [Optimization Chain](#32-optimization-chain)
   - 3.3 [LOD Chain Specification](#33-lod-chain-specification)
   - 3.4 [PBR Material System](#34-pbr-material-system)
   - 3.5 [Animation Catalog](#35-animation-catalog)
4. [Agentic 3D Asset Generation](#4-agentic-3d-asset-generation)
   - 4.1 [Tool Evaluation Matrix](#41-tool-evaluation-matrix)
   - 4.2 [Decision: Meshy.ai + InstantMesh](#42-decision-meshyai--instantmesh)
   - 4.3 [Prompt Engineering for Game Assets](#43-prompt-engineering-for-game-assets)
   - 4.4 [Post-Processing Pipeline](#44-post-processing-pipeline)
   - 4.5 [Reproducibility via Seed Capture](#45-reproducibility-via-seed-capture)
   - 4.6 [Quality Gate Automation](#46-quality-gate-automation)
5. [ArtSpec IR Extension for 3D](#5-artspec-ir-extension-for-3d)
   - 5.1 [IR Schema (3D Variant)](#51-ir-schema-3d-variant)
   - 5.2 [Backward Compatibility with 2D IR](#52-backward-compatibility-with-2d-ir)
   - 5.3 [IR Validation Rules](#53-ir-validation-rules)
6. [Procedural Terrain Generation](#6-procedural-terrain-generation)
   - 6.1 [Heightmap Generation](#61-heightmap-generation)
   - 6.2 [Deterministic Seeding](#62-deterministic-seeding)
   - 6.3 [Biome System](#63-biome-system)
   - 6.4 [Three.js Terrain Implementation](#64-threejs-terrain-implementation)
   - 6.5 [Terrain Streaming](#65-terrain-streaming)
   - 6.6 [Hex Grid to Heightmap Mapping](#66-hex-grid-to-heightmap-mapping)
7. [Instanced Rendering and Performance Architecture](#7-instanced-rendering-and-performance-architecture)
   - 7.1 [InstancedMesh for Buildings](#71-instancedmesh-for-buildings)
   - 7.2 [Citizen Sprite Strategy](#72-citizen-sprite-strategy)
   - 7.3 [Particle Systems](#73-particle-systems)
   - 7.4 [Draw Call Budget](#74-draw-call-budget)
   - 7.5 [FPS Targets by Hardware Tier](#75-fps-targets-by-hardware-tier)
8. [Nation Visual Identity System](#8-nation-visual-identity-system)
   - 8.1 [Nation Identity Schema](#81-nation-identity-schema)
   - 8.2 [Runtime Material Override](#82-runtime-material-override)
   - 8.3 [Flag System](#83-flag-system)
   - 8.4 [Border Rendering](#84-border-rendering)
9. [Transition Implementation Plan](#9-transition-implementation-plan)
   - 9.1 [Phase 0 — 2D Only (Current)](#91-phase-0--2d-only-current)
   - 9.2 [Phase 1 — Three.js Scaffold](#92-phase-1--threejs-scaffold)
   - 9.3 [Phase 2 — First GLTF Buildings](#93-phase-2--first-gltf-buildings)
   - 9.4 [Phase 3 — Agentic Generation Integration](#94-phase-3--agentic-generation-integration)
   - 9.5 [Phase 4 — Full Terrain and FX](#95-phase-4--full-terrain-and-fx)
   - 9.6 [Phase 5 — Native Client (Optional)](#96-phase-5--native-client-optional)
10. [Agentic Generation Orchestration](#10-agentic-generation-orchestration)
    - 10.1 [Python Orchestrator Architecture](#101-python-orchestrator-architecture)
    - 10.2 [Async Batch Generation](#102-async-batch-generation)
    - 10.3 [Cost Model](#103-cost-model)
    - 10.4 [Visual Inspector Tool](#104-visual-inspector-tool)
    - 10.5 [Manifest Lifecycle](#105-manifest-lifecycle)
11. [Blender Pipeline — Artist Path](#11-blender-pipeline--artist-path)
    - 11.1 [Blender Workflow](#111-blender-workflow)
    - 11.2 [Export and Optimization Chain](#112-export-and-optimization-chain)
    - 11.3 [Auto-LOD via Blender Script](#113-auto-lod-via-blender-script)
    - 11.4 [CI Gate Specification](#114-ci-gate-specification)
12. [Functional Requirements](#12-functional-requirements)
13. [Integration with Simulation Core](#13-integration-with-simulation-core)
    - 13.1 [Protocol Attachment](#131-protocol-attachment)
    - 13.2 [Event-Driven Scene Updates](#132-event-driven-scene-updates)
    - 13.3 [View Layer Purity Contract](#133-view-layer-purity-contract)
14. [Performance Benchmarks and Budgets](#14-performance-benchmarks-and-budgets)
    - 14.1 [Scene Initialization](#141-scene-initialization)
    - 14.2 [Memory Budgets](#142-memory-budgets)
    - 14.3 [Texture Atlas Specification](#143-texture-atlas-specification)
    - 14.4 [Benchmark Measurement Protocol](#144-benchmark-measurement-protocol)
15. [Appendices](#15-appendices)
    - A. [glTF Validator Rules Reference](#a-gltf-validator-rules-reference)
    - B. [Meshy.ai API Reference Stub](#b-meshyai-api-reference-stub)
    - C. [Asset Manifest Schema (3D)](#c-asset-manifest-schema-3d)
    - D. [Biome Texture Palette](#d-biome-texture-palette)

---

## 1. Transition Roadmap and Philosophy

### 1.1 Why 2D First

CivLab ships with a 2D SVG-first pipeline as defined in CIV-0600. This is an explicit product decision, not a constraint of ignorance. The reasoning is:

**Iteration velocity.** SVG assets are editable in code, version-controlled as text, and can be regenerated from ArtSpec IR in seconds. 3D assets require generation pipelines, optimization passes, and visual QA. In the MVP phase, the ability to rapidly change building designs, rework visual language, and fix art bugs without pipeline delays is worth more than visual fidelity.

**Separation of complexity.** 3D rendering introduces a large surface area of new complexity: shader authoring, LOD chain management, texture memory budgets, VRAM profiling, physics mesh generation, animation state machines. Introducing this complexity simultaneously with game system development (economy, diplomacy, military) would create compounding risk.

**Research-grade legibility.** CivLab's secondary audience is researchers and policy analysts. 2D iconographic representations (clearly readable symbols, color coding, data overlays) serve this audience better than immersive 3D. 3D is a secondary presentation mode, not the primary research interface.

**Cost of 3D assets.** A complete building catalog (12 building types × 6 eras = 72 models) via professional artists costs between $10k and $30k. Agentic generation tools reduce this to under $10, but require pipeline infrastructure that would block MVP delivery if built first.

The 2D pipeline is not a prototype. It is a production-quality system that remains the primary visual mode for the web research client. 3D is a separate, additive capability.

### 1.2 Non-Blocking Architecture Contract

The central contract governing this spec is:

> **The headless simulation core is renderer-agnostic. 3D is just another client renderer.**

This is enforced structurally by CIV-0001 and CIV-0200:
- The simulation core (Rust) emits state snapshots and events over a JSON-RPC WebSocket interface.
- All rendering logic lives in the client layer.
- Clients are interchangeable and can run simultaneously against the same core.

As a result, the 2D client and the 3D client are independently deployable. Work on one does not block the other. The 3D client is developed alongside the 2D client without requiring any modification to:
- The simulation core (CIV-0001)
- The client protocol (CIV-0200)
- The economy, diplomacy, or AI behavior systems

The only shared artifact between the two clients is the ArtSpec IR (CIV-0600), which is intentionally renderer-agnostic.

**Architecture diagram:**

```
┌───────────────────────────────────────────────────────────┐
│                   Headless Simulation Core (Rust)          │
│              CIV-0001  —  Deterministic Tick Engine        │
└───────────────────────┬───────────────────────────────────┘
                        │  JSON-RPC WebSocket (CIV-0200)
           ┌────────────┴────────────┐
           │                         │
┌──────────▼──────────┐   ┌──────────▼──────────┐
│  2D Web Client       │   │  3D Web Client       │
│  (Pixi.js / SVG)     │   │  (Three.js r168)     │
│  CIV-0300 / CIV-0600 │   │  CIV-0601            │
└─────────────────────┘   └─────────────────────┘
                                      │
                           ┌──────────▼──────────┐
                           │  3D Native Client    │
                           │  (Bevy 0.15 — opt.)  │
                           │  CIV-0601 §2.1       │
                           └─────────────────────┘
```

### 1.3 ArtSpec IR as the Renderer-Agnostic Boundary

The ArtSpec Intermediate Representation (IR) is the canonical description of a visual asset, independent of render target. Defined in CIV-0600, extended here for 3D.

**What the IR captures:**
- Asset identity (asset_id, asset_type, building_class, era)
- Nation identity (primary_color, secondary_color, architectural_style)
- Render mode (`"2d"` or `"3d"`)
- Generation parameters (method, seed, reference asset)
- LOD budget constraints

**What the IR does NOT capture:**
- Render-specific shader code
- Three.js-specific scene graph state
- Pixi.js sprite sheet layout
- GPU memory allocation strategy

The IR is consumed by two downstream pipelines:
1. **2D pipeline (CIV-0600):** Generates SVG assets from IR using deterministic procedural rules.
2. **3D pipeline (this spec):** Generates .glb GLTF assets from IR using either agentic generation tools or Blender artist workflow.

Both pipelines read the same IR. An asset manager may convert a 2D IR to a 3D IR by changing `render_mode` and adding 3D-specific fields. The 2D reference fields remain intact for cross-referencing.

### 1.4 Milestone Gates

| Gate | Name | Entry Criteria | Exit Criteria |
|------|------|---------------|---------------|
| G0 | 2D MVP | — | 2D SVG pipeline complete; all 12 building types × 6 eras rendered in 2D; simulation core stable |
| G1 | 3D Prototype | G0 complete | Three.js scene renders placeholder box buildings on hex grid; terrain heightmap visible; no LOD |
| G2 | 3D Alpha | G1 complete | 12 GLTF building models (LOD0 + LOD1 only); nation color injection working; 30 FPS on M2 |
| G3 | 3D Beta | G2 complete | Full 72-model catalog (12 × 6 eras); LOD chain complete; terrain biomes; animated buildings |
| G4 | 3D Production | G3 complete; QA sign-off | 60 FPS on M2; 30 FPS on GTX 1060; all FRs passing; full CI pipeline green |
| G5 | Native Client | G4 + business case | Bevy 0.15 client feature-parity with Three.js client |

Gate reviews are automated where possible (CI gates) and documented per the CIV-0001 review process.

### 1.5 Decision Log

| ID | Decision | Rationale | Alternatives Considered |
|----|----------|-----------|------------------------|
| ADR-3D-001 | Three.js r168 for web 3D | Most mature WebGL/WebGPU library; glTF support built-in; large ecosystem | Babylon.js (heavier bundle), PlayCanvas (commercial), raw WebGPU (too early) |
| ADR-3D-002 | glTF 2.0 / .glb binary | Industry standard; Three.js native support; Blender native export; extensible | FBX (proprietary), OBJ (no materials), USD (too heavy for web) |
| ADR-3D-003 | Meshy.ai for hero assets | Quality/speed tradeoff optimal for game assets; API supports seed reproducibility | Shap-E (lower quality), Wonder3D (slower), Tripo3D (no seed control) |
| ADR-3D-004 | InstantMesh for bulk buildings | Open-source; local execution; no per-call cost; batch-friendly | CSM.ai (cost), Point-E (lower quality for buildings) |
| ADR-3D-005 | glTF Transform for optimization | Handles draco, quantization, pruning; CLI + programmatic API | Meshopt (compression only), manual Blender optimization |
| ADR-3D-006 | Bevy as optional native target | Rust ecosystem consistency; deterministic; ECS aligns with sim core | Unity (C#, incompatible ecosystem), Unreal (too heavy) |

---

## 2. 3D Client Architecture

### 2.1 Technology Stack

**Web 3D Client (Primary)**

| Component | Technology | Version | Notes |
|-----------|-----------|---------|-------|
| Renderer | Three.js | r168 | WebGL2 primary; WebGPU via `THREE.WebGPURenderer` when available |
| Physics mesh | Rapier.js | 0.12 | Collision detection only; no full physics simulation |
| Post-processing | Three.js PostProcessing | r168 | SSAO, bloom, edge detection for borders |
| Asset loading | Three.js GLTFLoader | r168 | Supports Draco, KTX2 |
| State bridge | Zustand | 5.x | Same store architecture as 2D client (CIV-0300) |
| Build toolchain | Vite | 6.x | ESM-native; same pipeline as 2D client |
| Protocol | JSON-RPC over WebSocket | CIV-0200 | Identical to 2D client |

**Native 3D Client (Phase 5 — Optional)**

| Component | Technology | Version | Notes |
|-----------|-----------|---------|-------|
| Renderer | Bevy | 0.15 | PBR rendering; WGPU backend |
| Asset pipeline | bevy_gltf | 0.15 | Native GLTF loading |
| Protocol | Same JSON-RPC over WS | CIV-0200 | Shared protocol; no core changes needed |
| Build | Cargo | latest stable | Workspace member under `clients/bevy_3d/` |

**Why not replace Three.js with Bevy in the browser?**
Bevy compiles to WASM but bundle size (~10 MB) and startup time (~2-3s cold) are unacceptable for a research web tool. Three.js delivers the same visual quality at 200 KB bundle size (tree-shaken). Bevy targets native desktop only.

### 2.2 Scene Graph Design

The 3D scene is structured as follows. Every node in the scene corresponds to a simulation entity. The mapping is maintained by the `SceneEntityRegistry`, a client-side lookup table from `entity_id` (simulation) to `THREE.Object3D` (scene node).

```
THREE.Scene
├── TerrainGroup                       (world.terrain)
│   ├── HexCellMesh[0..N]              (PlaneGeometry per cell, ShaderMaterial)
│   └── WaterMesh                      (animated, ShaderMaterial)
├── BuildingGroup                      (world.buildings)
│   ├── InstancedMesh[granary]         (all granary instances, any nation)
│   ├── InstancedMesh[farm]
│   ├── InstancedMesh[barracks]
│   └── ... (one InstancedMesh per building type per era)
├── CitizenGroup                       (world.citizens)
│   └── InstancedMesh[citizen_sprite]  (billboard quads)
├── UnitGroup                          (world.units)
│   ├── LOD[soldier_z1]
│   └── ...
├── FXGroup                            (world.fx)
│   ├── THREE.Points[smoke]
│   ├── THREE.Points[fire]
│   └── THREE.Points[weather]
├── UIGroup                            (world.ui — screen-space)
│   ├── BorderEdgeLines               (LineSegments)
│   └── SelectionRing                 (RingGeometry)
└── LightingGroup
    ├── AmbientLight                   (0xffffff, intensity: 0.4)
    ├── DirectionalLight               (sun, castShadow: true)
    └── HemisphereLight               (sky/ground ambient)
```

**SceneEntityRegistry contract:**

```typescript
interface SceneEntityRegistry {
  register(entityId: string, object: THREE.Object3D): void;
  lookup(entityId: string): THREE.Object3D | null;
  remove(entityId: string): void;
  getInstanceIndex(entityId: string): { mesh: THREE.InstancedMesh; index: number } | null;
}
```

This registry is the exclusive mechanism for mapping simulation events to scene mutations. No component outside `SceneEntityRegistry` directly references scene nodes.

### 2.3 Camera System

**Primary mode:** Isometric perspective.

The camera is positioned at a fixed elevation angle (45 degrees) with an azimuth that the player can rotate. This gives the appearance of isometric projection while retaining perspective depth cues. Orthographic projection is available as a toggle for accessibility and for performance fallback.

```typescript
interface CameraConfig {
  mode: "perspective" | "orthographic";
  elevation_deg: 45;          // Fixed: not user-adjustable
  fov_deg: 40;                // Tight FOV for pseudo-isometric look
  near: 0.1;
  far: 2000;
  zoom_levels: {
    zoom1: { distance: 800; hex_cells_visible: "full map" };
    zoom2: { distance: 200; hex_cells_visible: "12×12" };
    zoom3: { distance: 60;  hex_cells_visible: "3×3" };
  };
}
```

**Camera controls:**
- Pan: middle-mouse drag or WASD (same keybinds as 2D client, CIV-0300 §8)
- Rotate: right-mouse drag (azimuth only; elevation fixed at 45°)
- Zoom: scroll wheel; smooth lerp transition between zoom levels
- Focus: double-click entity centers camera with smooth animation

**Orthographic fallback conditions:**
- GPU capability flag indicates WebGL2 not available
- User explicitly toggles in settings
- FPS < 20 for > 2 seconds (auto-switch with user notification)

### 2.4 LOD Architecture

Three.js `THREE.LOD` is used for per-building LOD management. Each building slot in the scene graph is a `THREE.LOD` object containing all LOD variants.

```typescript
// LOD distance thresholds (camera-to-object world units)
const LOD_DISTANCES = {
  LOD0: 0,    // 0–80 units: high-poly GLTF (up to 5k triangles)
  LOD1: 80,   // 80–200 units: medium GLTF (up to 1k triangles)
  LOD2: 200,  // 200–500 units: low GLTF (up to 200 triangles)
  LOD3: 500,  // 500+ units: billboard quad (2 triangles)
};

function createBuildingLOD(models: BuildingModelSet): THREE.LOD {
  const lod = new THREE.LOD();
  lod.addLevel(models.lod0, LOD_DISTANCES.LOD0);
  lod.addLevel(models.lod1, LOD_DISTANCES.LOD1);
  lod.addLevel(models.lod2, LOD_DISTANCES.LOD2);
  lod.addLevel(models.lod3_billboard, LOD_DISTANCES.LOD3);
  return lod;
}
```

At Zoom 1 (strategic view), nearly all buildings render at LOD3 (billboard). At Zoom 3 (citizen view), buildings in the 3×3 visible area render at LOD0. This is the primary mechanism for achieving the < 200 draw call budget.

LOD transitions use hysteresis (5-unit dead zone) to prevent visible popping as the camera hovers at a boundary distance.

### 2.5 Protocol Compatibility (CIV-0200)

The 3D client is a first-class CIV-0200 client. It subscribes to the same event stream and sends the same command messages as the 2D client. No changes to CIV-0200 are required for 3D.

**Connection lifecycle:**

```typescript
// Identical to 2D client WebSocket setup
const ws = new WebSocket(`ws://${host}/sim`);
ws.onopen = () => {
  ws.send(JSON.stringify({
    jsonrpc: "2.0",
    method: "client.register",
    params: { client_id: uuid(), client_type: "3d_web", zoom: 2 },
    id: 1
  }));
};
```

**Events consumed by 3D client (superset of 2D client events):**

| Event | 3D Client Action |
|-------|-----------------|
| `world.snapshot.v1` | Full scene rebuild from snapshot |
| `buildings.constructed.v1` | Spawn GLTF model at hex position |
| `buildings.demolished.v1` | Remove GLTF model; particle debris FX |
| `citizen.migrated.v1` | Move citizen sprite between hex cells |
| `citizen.spawned.v1` | Add sprite to CitizenGroup |
| `citizen.died.v1` | Remove sprite; death particle FX |
| `war.declared.v1` | Activate conflict border shader |
| `war.ended.v1` | Deactivate conflict border shader |
| `climate.event.v1` | Trigger weather particle system |
| `research.completed.v1` | Building upgrade animation trigger |
| `tick.advanced.v1` | Update time-of-day lighting |

Events not consumed by the 3D client (e.g., UI-only events for the HUD) are ignored. The 3D client does not implement or duplicate HUD logic — that is handled by the React/DOM overlay layer (same architecture as CIV-0300).

---

## 3. glTF Asset Pipeline

### 3.1 Format Decisions

**Primary format:** glTF 2.0 binary (.glb)

glTF 2.0 is the "JPEG of 3D" — an open, vendor-neutral, runtime-efficient format with broad tooling support. The binary variant (.glb) packs geometry, materials, and textures into a single file, eliminating asset bundle fragmentation.

**Rationale for glTF over alternatives:**

| Criterion | glTF 2.0 | FBX | USD | OBJ |
|-----------|----------|-----|-----|-----|
| Open standard | Yes (Khronos) | No (Autodesk) | Yes (Pixar/Apple) | Yes |
| Three.js native | Yes (GLTFLoader) | No (requires convert) | Partial (usdz only) | Yes (OBJLoader) |
| Blender native export | Yes | Yes | Yes | Yes |
| Binary packing | .glb | Yes | Yes | No |
| PBR material standard | Yes (metallic-roughness) | Partial | Yes | No |
| Animation support | Yes | Yes | Yes | No |
| Web bundle size | Optimal | Poor | Poor | Medium |
| Draco compression | Extension (KHR_draco_mesh_compression) | No | No | No |

**Extensions used:**

| Extension | Purpose | Required |
|-----------|---------|----------|
| `KHR_draco_mesh_compression` | Geometry compression (60-80% reduction) | Yes for LOD0/LOD1 |
| `KHR_materials_unlit` | Billboard LOD3 assets (no lighting cost) | Yes for LOD3 |
| `KHR_texture_basisu` | KTX2/BasisU texture compression | Optional (progressive) |
| `KHR_mesh_quantization` | Vertex attribute quantization | Yes |
| `EXT_mesh_gpu_instancing` | Instanced mesh data baking | Optional |

### 3.2 Optimization Chain

Every .glb asset passes through a deterministic optimization chain before entering the asset manifest. The chain is implemented as a sequence of glTF Transform operations:

```
raw_output.glb
    │
    ▼ [prune] — Remove unused nodes, materials, textures
    │
    ▼ [dedup] — Deduplicate identical accessors
    │
    ▼ [quantize] — Quantize vertex attributes (POSITION: 14-bit, NORMAL: 8-bit, TEXCOORD: 12-bit)
    │
    ▼ [draco] — Apply Draco mesh compression (quantization bits: position=14, normal=8)
    │
    ▼ [flatten] — Flatten scene hierarchy (single mesh node per LOD level)
    │
    ▼ [palette] — Remove duplicate materials
    │
    ▼ optimized_lod0.glb
```

**CLI invocation (glTF Transform):**

```bash
gltf-transform optimize \
  --draco \
  --quantize \
  --flatten \
  --dedup \
  --prune \
  raw_output.glb \
  optimized_lod0.glb
```

Optimization is run per LOD level independently. LOD3 (billboard) skips Draco compression (geometry is trivial: 2 triangles) but applies texture compression.

### 3.3 LOD Chain Specification

Each building asset has four LOD levels. The triangle budgets are hard limits enforced by the CI gate.

| LOD Level | Nickname | Triangle Budget | Use Case | Generation Method |
|-----------|----------|----------------|----------|------------------|
| LOD0 | High | 5,000 max | Camera distance 0–80 units; Zoom 3 focus | Meshy.ai or Blender (full detail) |
| LOD1 | Medium | 1,000 max | Camera distance 80–200 units | Auto-LOD from LOD0 (50% decimation target) |
| LOD2 | Low | 200 max | Camera distance 200–500 units | Auto-LOD from LOD1 (80% decimation target) |
| LOD3 | Billboard | 2 (quad) | Camera distance 500+ units | Generated: imposter quad with baked texture |

Billboard (LOD3) generation: render LOD0 from 8 cardinal angles + top-down, bake to a sprite sheet (512×512 per angle, 8-angle × 1-top = 9 tiles on 2048×2048 atlas). At runtime, the billboard selects the correct tile based on camera angle.

**LOD naming convention in asset manifest:**

```
assets/3d/
├── granary/
│   ├── classical/
│   │   ├── granary_classical_lod0.glb
│   │   ├── granary_classical_lod1.glb
│   │   ├── granary_classical_lod2.glb
│   │   └── granary_classical_lod3_billboard.glb
│   ├── medieval/
│   └── ...
├── farm/
└── ...
```

### 3.4 PBR Material System

All non-LOD3 assets use glTF PBR metallic-roughness materials. The material system is designed so that nation color is injected at runtime without requiring texture rebaking.

**Base material properties (per building class):**

| Building Class | Roughness | Metalness | Base Color (Neutral) | Notes |
|---------------|-----------|-----------|---------------------|-------|
| Granary | 0.85 | 0.0 | #C8A87A (stone/thatch) | Nation color replaces wood trim |
| Barracks | 0.70 | 0.2 | #8B8B8B (stone) | Nation color on banner/shield |
| Farm | 0.90 | 0.0 | #6B4C2A (soil/wood) | Nation color on fencing |
| Library | 0.65 | 0.05 | #D4C5A9 (marble) | Nation color on dome |
| Market | 0.75 | 0.1 | #B8860B (timber frame) | Nation color on awning |
| Temple | 0.60 | 0.15 | #E8E0D0 (carved stone) | Nation color on spire |
| Forge | 0.80 | 0.6 | #5A5A5A (iron/stone) | Nation color on insignia plate |
| Harbor | 0.85 | 0.3 | #4A6FA5 (wood/metal) | Nation color on flag |
| Tower | 0.70 | 0.4 | #7A7A7A (stone/iron) | Nation color on banner |
| Palace | 0.55 | 0.2 | #F5F5DC (ornate stone) | Nation color on roof tiles |
| Academy | 0.65 | 0.1 | #D8CCBA (limestone) | Nation color on insignia |
| Cathedral | 0.60 | 0.05 | #E5E0D5 (white stone) | Nation color on stained glass |

**Nation color injection mechanism:**

Each model is authored with a dedicated material slot named `nation_color_material`. At runtime, the Three.js loader identifies this material by name and replaces the `color` property with the nation's primary color:

```typescript
function applyNationColor(model: THREE.Group, nationColor: string): void {
  model.traverse((node) => {
    if (node instanceof THREE.Mesh) {
      const materials = Array.isArray(node.material) ? node.material : [node.material];
      materials.forEach((mat) => {
        if (mat.name === "nation_color_material" && mat instanceof THREE.MeshStandardMaterial) {
          mat.color.set(nationColor);
          mat.needsUpdate = true;
        }
      });
    }
  });
}
```

This avoids texture rebaking entirely. The material is shared across all instances of the same building type for the same nation, reducing GPU state changes.

### 3.5 Animation Catalog

All building animations are embedded in the .glb file as glTF animation clips. Three.js `THREE.AnimationMixer` manages playback.

| Animation Name | Building Types | Duration | Loop | Trigger |
|---------------|---------------|---------|------|---------|
| `idle_smoke` | Forge, Granary (with stored food) | 3.0s | Loop | Always when food > 0 |
| `idle_flag_wave` | Barracks, Palace, Harbor, Tower | 2.5s | Loop | Always |
| `idle_ambient` | All | 1.0–4.0s | Loop | Always (subtle sway) |
| `construction_progress` | All | 5.0s | Once, driven by progress | On `buildings.construction_progress.v1` |
| `construction_complete` | All | 1.5s | Once | On `buildings.constructed.v1` |
| `combat_attack` | Barracks, Tower, Harbor | 0.8s | Once per attack | On `combat.attack.v1` |
| `damage_flash` | All | 0.3s | Once | On `buildings.damaged.v1` |
| `demolish_collapse` | All | 2.0s | Once | On `buildings.demolished.v1` |

Animation state machine (simplified):

```
[IDLE] ──────────────────────────── always looping `idle_*` clips
   │
   ├── On construction_progress.v1 → [CONSTRUCTING] (drive clip by progress param)
   │         └── On constructed.v1 → [IDLE]
   │
   ├── On combat.attack.v1 → [ATTACKING] (play once)
   │         └── Animation end → [IDLE]
   │
   └── On demolished.v1 → [COLLAPSING] (play once; remove node at end)
```

---

## 4. Agentic 3D Asset Generation

### 4.1 Tool Evaluation Matrix

The following tools were evaluated for automated 3D building generation. Evaluation criteria: output quality for low-poly game assets, API availability, seed/reproducibility support, cost per asset, batch throughput, and glTF export capability.

| Tool | Quality (1-10) | API | Seed Support | Cost/Asset | Batch Throughput | glTF Export | Decision |
|------|---------------|-----|-------------|-----------|-----------------|------------|---------|
| **Meshy.ai** | 8 | REST API (v2) | Yes (seed param) | ~$0.05 | 10 parallel | Yes (native) | **Hero assets** |
| **InstantMesh** | 6 | Local inference | Hash-based | $0 (GPU cost) | CPU-limited | Yes (via convert) | **Bulk buildings** |
| Shap-E (OpenAI) | 4 | Python lib | Yes | ~$0.02 | High | Partial (mesh only) | Rejected: quality |
| Wonder3D | 7 | Local inference | Partial | $0 (GPU cost) | Slow (8 min/asset) | Partial | Rejected: speed |
| Tripo3D | 7 | REST API | No | ~$0.08 | 5 parallel | Yes | Rejected: no seed |
| CSM.ai | 7 | REST API | Partial | ~$0.12 | Limited | Yes | Rejected: cost |
| Point-E (OpenAI) | 3 | Python lib | Yes | ~$0.01 | High | No (point cloud) | Rejected: quality |

### 4.2 Decision: Meshy.ai + InstantMesh

**Meshy.ai** is used for "hero" assets — the canonical representative of each building type for the earliest era (Classical). These are the highest-visibility assets that appear most often in promotional materials, screenshots, and gameplay. Quality is the primary criterion.

**InstantMesh** is used for bulk generation — all remaining era variants of each building. InstantMesh takes a reference image (the Meshy.ai LOD0 render, or the 2D SVG reference) and generates a 3D mesh that is stylistically consistent. This creates era variation while maintaining architectural family coherence.

**Hybrid workflow:**

```
For each building_type in [granary, farm, barracks, ...]:
    # Step 1: Hero asset via Meshy.ai
    hero = meshy_api.generate(
        prompt=build_prompt(building_type, era="classical"),
        seed=CANONICAL_SEEDS[building_type],
        style="game-low-poly"
    )

    # Step 2: Era variants via InstantMesh
    for era in [medieval, renaissance, industrial, modern, futuristic]:
        variant_prompt = f"{building_type}_{era}_style_reference.png"
        variant = instantmesh.generate(
            reference_image=render_front_view(hero),
            era_image=ERA_REFERENCE_IMAGES[era]
        )
        variants.append(variant)
```

This hybrid produces 72 models total (12 types × 6 eras) at approximately $3.60 total API cost (12 Meshy hero assets × $0.05 + 60 InstantMesh variants at $0 marginal cost).

### 4.3 Prompt Engineering for Game Assets

Prompt quality is the largest determinant of Meshy.ai output quality. The following template is the canonical prompt structure, locked by this spec.

**Canonical prompt template:**

```
low-poly {building_type} building, {era} architectural style,
{nation_aesthetic} regional design, game asset, clean topology,
PBR textures, glTF format, isometric view optimized,
{polygon_budget} triangles maximum, professional game art quality,
modular, no floating geometry, manifold mesh
```

**Template variable definitions:**

| Variable | Type | Example Values |
|----------|------|---------------|
| `building_type` | Enum | `granary`, `barracks`, `library`, `harbor`, `temple`, `market`, `forge`, `tower`, `palace`, `academy`, `farm`, `cathedral` |
| `era` | Enum | `ancient`, `classical`, `medieval`, `renaissance`, `industrial`, `modern`, `futuristic` |
| `nation_aesthetic` | String | `mediterranean`, `nordic`, `east asian`, `sub-saharan`, `mesoamerican`, `central asian`, `middle eastern` |
| `polygon_budget` | Int (rendered) | `5000` for LOD0 |

**Negative prompt (Meshy.ai negative_prompt field):**

```
floating geometry, non-manifold mesh, ngons, texture seams,
overlapping UVs, high-poly photorealism, organic shapes,
humans or animals in building mesh, excessive detail on back face
```

**Era-specific style modifiers:**

| Era | Style Modifier Appended |
|-----|------------------------|
| Ancient | `"stone and mud brick construction, minimal ornamentation, pre-wheel era"` |
| Classical | `"columns, pediments, fired brick, mortar joints"` |
| Medieval | `"timber framing, thatched or tiled roofs, defensive crenellations"` |
| Renaissance | `"symmetrical facades, arched windows, terracotta roof tiles"` |
| Industrial | `"iron framing, brick chimneys, corrugated metal roofing"` |
| Modern | `"concrete and glass, clean lines, modular prefab aesthetic"` |
| Futuristic | `"floating elements, energy conduits, bioluminescent materials"` |

### 4.4 Post-Processing Pipeline

After generation, each raw .glb file passes through a mandatory post-processing pipeline before it is added to the asset manifest.

```
raw_generated.glb
    │
    ▼ [manifold check] — Ensure mesh is watertight (no holes, no self-intersections)
    │   Tool: ManifoldPlus (C++) or Blender Python API
    │   Failure action: re-generate with temperature=0.9 (varied seed)
    │
    ▼ [UV unwrap validation] — Detect UV island overlap (> 2% overlap = fail)
    │   Tool: xatlas (CPU UV atlas packing)
    │   Failure action: re-unwrap with xatlas automatic mode
    │
    ▼ [polygon count enforcement] — Assert triangle count within LOD budget
    │   Failure action: Decimate via openmesh until within budget
    │
    ▼ [LOD generation] — Auto-generate LOD1, LOD2 from LOD0
    │   Tool: openmesh (local) or simplygon (cloud, quality tier)
    │   LOD1 = 80% decimation of LOD0 (target: 1000 tri from 5000)
    │   LOD2 = 80% decimation of LOD1 (target: 200 tri from 1000)
    │
    ▼ [billboard generation] — Render 9-angle imposter sprite sheet
    │   Tool: headless Three.js renderer (Node.js + puppeteer)
    │   Output: lod3_billboard.glb (quad with baked imposter texture)
    │
    ▼ [nation_color_material tagging] — Ensure slot named "nation_color_material" exists
    │   Tool: gltf-transform custom transform script
    │   Failure action: Assign to highest-luminance material automatically
    │
    ▼ [glTF optimization] — Run full glTF Transform optimization chain (§3.2)
    │
    ▼ [glTF Validator] — Run official Khronos glTF Validator
    │   Threshold: zero errors; warnings <= 5
    │
    ▼ PASS → Add to asset_manifest_3d.json
      FAIL → Log to generation_failures.jsonl; alert orchestrator
```

### 4.5 Reproducibility via Seed Capture

Every generated asset must be reproducible from its manifest entry alone. This is a hard requirement for the research-grade integrity of CivLab.

**Meshy.ai seed capture:**

```python
response = meshy_client.generate_3d(
    prompt=prompt,
    negative_prompt=negative_prompt,
    seed=CANONICAL_SEEDS[building_type],  # Pre-assigned per building type
    art_style="game-low-poly",
    topology="quad",
    target_polycount=5000,
)

manifest_entry = {
    "asset_id": f"building_{building_type}_{era}_lod0",
    "generation_method": "meshy_api",
    "generation_seed": response.seed,  # Store the actual seed used (may differ if overridden)
    "meshy_task_id": response.task_id,  # Meshy internal ID for re-fetching
    "meshy_api_version": "v2",
    "prompt": prompt,
    "negative_prompt": negative_prompt,
    "generated_at": response.created_at,
    "post_processing_hash": sha256_of_pipeline_config(),
}
```

**InstantMesh seed capture:**

InstantMesh uses a hash-based seed derived from the input image and era parameters:

```python
reference_hash = hashlib.sha256(reference_image_bytes).hexdigest()[:16]
era_hash = hashlib.sha256(era.encode()).hexdigest()[:8]
instantmesh_seed = int(reference_hash + era_hash, 16) % (2**32)
```

This ensures that if the reference image is identical and the era is identical, the InstantMesh output is deterministic (within model version constraints).

**Canonical seed table (pre-assigned, locked at G0):**

| Building Type | Canonical Meshy Seed |
|--------------|---------------------|
| granary | 101742 |
| farm | 209834 |
| barracks | 317492 |
| library | 428173 |
| market | 512890 |
| temple | 634821 |
| forge | 749302 |
| harbor | 851029 |
| tower | 923847 |
| palace | 1047382 |
| academy | 1182039 |
| cathedral | 1291847 |

These seeds are stored in `tools/asset_gen/canonical_seeds.json` and are treated as immutable after G0 gate. Changing a seed constitutes a breaking change to the art pipeline and requires a new asset_id and deprecation of the old one.

### 4.6 Quality Gate Automation

The quality gate runs automatically for every generated asset. It is implemented as a Python script (`tools/asset_gen/quality_gate.py`) invoked by the orchestrator after post-processing.

**Gate checks (ordered by severity):**

| Check | Severity | Tool | Pass Threshold |
|-------|----------|------|----------------|
| Manifold mesh | CRITICAL | ManifoldPlus | 100% watertight |
| UV overlap | HIGH | xatlas | < 2% island overlap |
| Polygon count | HIGH | gltf-transform stats | Within LOD budget ± 10% |
| Material slot naming | HIGH | gltf-transform custom | `nation_color_material` slot present |
| glTF Validator | HIGH | Khronos glTF Validator | 0 errors, ≤ 5 warnings |
| Texture resolution | MEDIUM | PIL (Pillow) | Power-of-2; ≤ 2048×2048 |
| File size | LOW | os.path.getsize | ≤ 2 MB per LOD level |
| Animation clip names | LOW | gltf-transform stats | Names match animation catalog (§3.5) |

Assets failing CRITICAL or HIGH checks are logged to `generation_failures.jsonl` and excluded from the manifest. The orchestrator retries failed assets once with a varied seed before escalating to manual review.

---

## 5. ArtSpec IR Extension for 3D

### 5.1 IR Schema (3D Variant)

The ArtSpec IR defined in CIV-0600 is extended with a `"3d"` render_mode and additional fields specific to 3D generation. The extended schema is backward-compatible: 2D IR documents remain valid and unmodified.

```json
{
  "$schema": "https://civlab.dev/schemas/artspec-ir/v2.json",
  "asset_id": "building_granary_z2_3d",
  "asset_type": "building",
  "render_mode": "3d",
  "building_class": "granary",
  "era": "classical",
  "nation_color_primary": "#8B4513",
  "nation_color_secondary": "#D2691E",
  "architectural_style": "mediterranean",

  "lod_budget": {
    "lod0": 5000,
    "lod1": 1000,
    "lod2": 200,
    "lod3": "billboard"
  },

  "generation_method": "meshy_api",
  "generation_seed": 101742,
  "meshy_task_id": "task_abc123def456",
  "meshy_api_version": "v2",

  "reference_2d": "building_granary_z2",
  "reference_image_path": "assets/2d/reference/granary_classical_front.png",

  "animations": ["idle_smoke", "construction_progress", "construction_complete", "demolish_collapse"],

  "material_overrides": {
    "nation_color_material": "primary",
    "secondary_accent_material": "secondary"
  },

  "post_processing": {
    "uv_atlas": "xatlas_auto",
    "lod_generation": "openmesh_decimate",
    "optimization": "gltf_transform_full",
    "billboard_angles": 9
  },

  "quality_gate_result": {
    "passed": true,
    "checked_at": "2026-02-21T14:30:00Z",
    "gltf_validator_errors": 0,
    "gltf_validator_warnings": 2,
    "triangle_count_lod0": 4823,
    "triangle_count_lod1": 967,
    "triangle_count_lod2": 189,
    "uv_overlap_pct": 0.4,
    "file_size_bytes_lod0": 1247892
  }
}
```

**Schema version:** `v2`. The `render_mode` field is the discriminator. IR readers that do not support `render_mode: "3d"` must skip (not fail) on encountering it, per the backward-compatibility contract.

### 5.2 Backward Compatibility with 2D IR

The following rules govern coexistence of 2D and 3D IR documents:

1. A 2D IR document (CIV-0600 schema v1) is valid and unchanged. No fields are removed or renamed.
2. A 3D IR document (this spec, schema v2) MUST contain `"render_mode": "3d"`. If `render_mode` is absent, it defaults to `"2d"` for backward compatibility.
3. The `reference_2d` field in a 3D IR document MUST resolve to a valid 2D asset_id in the 2D asset manifest. This creates an explicit link between 2D and 3D representations of the same logical asset.
4. IR consumers (e.g., the asset loading system) dispatch on `render_mode` and invoke the appropriate pipeline.

### 5.3 IR Validation Rules

The IR is validated at generation time and at CI. The following rules are enforced:

| Rule ID | Description | Severity |
|---------|-------------|----------|
| IR-3D-001 | `asset_id` must end in `_3d` for render_mode=3d | ERROR |
| IR-3D-002 | `era` must be one of the canonical era enum values | ERROR |
| IR-3D-003 | `building_class` must be one of the 12 canonical building types | ERROR |
| IR-3D-004 | `lod_budget.lod0` must be ≤ 5000 | ERROR |
| IR-3D-005 | `lod_budget.lod1` must be ≤ 1000 | ERROR |
| IR-3D-006 | `lod_budget.lod2` must be ≤ 200 | ERROR |
| IR-3D-007 | `generation_seed` must be a positive integer | ERROR |
| IR-3D-008 | `reference_2d` must resolve in 2D manifest | WARNING |
| IR-3D-009 | `material_overrides` must include `nation_color_material` key | ERROR |
| IR-3D-010 | `animations` must be a subset of the animation catalog (§3.5) | WARNING |
| IR-3D-011 | `nation_color_primary` must be a valid 6-digit hex color | ERROR |
| IR-3D-012 | `quality_gate_result.passed` must be true for asset to be loadable | ERROR |

---

## 6. Procedural Terrain Generation

### 6.1 Heightmap Generation

Terrain height data is generated procedurally using multi-octave simplex noise. The implementation lives in the Rust simulation core (`crates/terrain/src/heightmap.rs`) and is serialized to the client as a compact binary heightmap embedded in the `world.snapshot.v1` message.

**Noise parameters:**

```rust
pub struct HeightmapConfig {
    pub width: u32,           // Heightmap resolution (pixels per hex cell: 4)
    pub height: u32,
    pub octaves: u8,          // 6 octaves for realistic terrain
    pub persistence: f32,     // 0.5 — amplitude halves each octave
    pub lacunarity: f32,      // 2.0 — frequency doubles each octave
    pub scale: f32,           // 200.0 — controls terrain "zoom"
    pub seed: u64,            // Derived from scenario seed (§6.2)
}

impl HeightmapConfig {
    pub fn default_for_map_size(map_width: u32, map_height: u32) -> Self {
        Self {
            width: map_width * 4,
            height: map_height * 4,
            octaves: 6,
            persistence: 0.5,
            lacunarity: 2.0,
            scale: 200.0,
            seed: 0, // Set by caller
        }
    }
}
```

The `noise` crate (Rust) provides the simplex noise implementation. Simplex noise is preferred over Perlin noise for its lower computational cost and better isotropy at high octave counts.

**Heightmap serialization format:**

The heightmap is serialized as a 16-bit grayscale PNG (values 0–65535 representing elevation from sea level to maximum mountain height). Dimensions are `(map_width × 4) × (map_height × 4)`. For a 64×64 hex map, the heightmap is 256×256 pixels.

The PNG is embedded in `world.snapshot.v1` as a base64-encoded data URI:

```json
{
  "world_snapshot": {
    "terrain": {
      "heightmap": "data:image/png;base64,iVBORw0KGgo...",
      "heightmap_scale": { "min_elevation": -200, "max_elevation": 3000 },
      "biome_map": "data:image/png;base64,..."
    }
  }
}
```

### 6.2 Deterministic Seeding

Terrain generation is fully deterministic given the scenario seed. The terrain seed is derived from the scenario seed using BLAKE3 hash to prevent correlation between terrain and other seeded subsystems.

```rust
use blake3;

pub fn derive_terrain_seed(scenario_seed: u64) -> u64 {
    let input = format!("{scenario_seed}terrain");
    let hash = blake3::hash(input.as_bytes());
    let bytes = hash.as_bytes();
    // Take first 8 bytes as u64 (little-endian)
    u64::from_le_bytes(bytes[0..8].try_into().unwrap())
}
```

This ensures that:
- Two simulations with the same scenario seed produce identical terrain.
- Changing the scenario seed changes terrain unpredictably (no obvious correlation).
- Changing only the terrain generation algorithm does not affect other seeded subsystems.

The `"terrain"` string is a domain separator. Other derived seeds use different separators: `"economy"`, `"weather"`, `"npc_names"`, etc.

### 6.3 Biome System

Six biome types are defined. Each cell in the hex grid is assigned a biome based on its elevation and latitude (row position in the map grid, used as a proxy for climate zone).

| Biome | Elevation Range | Latitude Range | Visual Description |
|-------|----------------|---------------|--------------------|
| Ocean | -200 to 0 | Any | Deep blue water; animated wave shader |
| Plains | 0 to 400 | 20%–80% of map height | Green grass; mild variation |
| Forest | 200 to 600 | 25%–75% of map height | Dense green; tree sprites |
| Desert | 0 to 300 | 0%–25% and 75%–100% | Sandy yellow; heat shimmer FX |
| Tundra | 0 to 400 | 0%–15% and 85%–100% | Snow-covered; ice texture |
| Mountain | 600 to 3000 | Any | Rocky grey; snow cap above 1500 |

Biome blending: adjacent cells of different biomes blend via a shader transition zone. The biome map is stored as a separate 8-bit-per-channel RGBA image where each channel encodes biome weights for up to 4 biomes (for smooth blending). The shader samples this biome weight map and lerps between biome textures.

**Biome assignment algorithm:**

```rust
pub fn assign_biome(elevation: f32, latitude_normalized: f32) -> Biome {
    if elevation < 0.0 {
        return Biome::Ocean;
    }
    if elevation > 600.0 {
        return Biome::Mountain;
    }
    let polar = latitude_normalized < 0.15 || latitude_normalized > 0.85;
    let tropical = latitude_normalized < 0.25 || latitude_normalized > 0.75;

    match (polar, tropical, elevation as u32) {
        (true, _, _) => Biome::Tundra,
        (false, true, 0..=300) => Biome::Desert,
        (false, false, 200..=600) => Biome::Forest,
        _ => Biome::Plains,
    }
}
```

### 6.4 Three.js Terrain Implementation

The Three.js terrain is composed of individual `THREE.PlaneGeometry` instances, one per hex cell, with a custom `ShaderMaterial` for biome blending.

```typescript
function createHexCellTerrain(cell: HexCell, heightmap: HeightmapData): THREE.Mesh {
  const geometry = new THREE.PlaneGeometry(
    HEX_CELL_SIZE,
    HEX_CELL_SIZE,
    8,  // Width segments (subdivision for displacement)
    8   // Height segments
  );

  // Apply heightmap displacement to vertices
  const positions = geometry.attributes.position.array as Float32Array;
  for (let i = 0; i < positions.length; i += 3) {
    const u = (positions[i] / HEX_CELL_SIZE) + 0.5;
    const v = (positions[i + 1] / HEX_CELL_SIZE) + 0.5;
    positions[i + 2] = sampleHeightmap(heightmap, cell, u, v);
  }
  geometry.attributes.position.needsUpdate = true;
  geometry.computeVertexNormals();

  const material = new THREE.ShaderMaterial({
    uniforms: {
      biomeWeights: { value: getBiomeWeightTexture(cell) },
      plainsTex: { value: BIOME_TEXTURES.plains },
      forestTex: { value: BIOME_TEXTURES.forest },
      desertTex: { value: BIOME_TEXTURES.desert },
      tundraTex: { value: BIOME_TEXTURES.tundra },
      mountainTex: { value: BIOME_TEXTURES.mountain },
      oceanTex: { value: BIOME_TEXTURES.ocean },
    },
    vertexShader: TERRAIN_VERTEX_SHADER,
    fragmentShader: TERRAIN_FRAGMENT_SHADER,
  });

  return new THREE.Mesh(geometry, material);
}
```

### 6.5 Terrain Streaming

At Zoom 1, the entire map terrain is loaded (typically 64×64 = 4096 cells). At Zoom 2 and below, terrain is loaded on demand as the camera moves. Cells outside the visible frustum plus a 2-cell buffer margin are unloaded (geometry and textures freed from GPU memory).

**Streaming state machine:**

```
UNLOADED ──────[enter buffer zone]──────► LOADING
    ▲                                        │
    │                             [load complete]
    │                                        ▼
[exit buffer zone] ◄──────────────────── LOADED
    │
[GC cycle]
    ▼
FREED (geometry disposed; VRAM reclaimed)
```

The streaming manager updates every 200ms (not every frame). It computes the set of cells within the camera frustum plus buffer, diffs against the currently-loaded set, and dispatches load/unload operations.

### 6.6 Hex Grid to Heightmap Mapping

Each hex cell maps to a 4×4 pixel region in the heightmap (at the default scale). The center pixel of each hex region defines the canonical elevation for that cell.

```
Hex cell (q, r) → heightmap pixel (q*4 + 2, r*4 + 2) [center pixel]
```

Edge interpolation: the height at the shared edge between two adjacent hex cells is the average of the two center elevations. This ensures smooth terrain transitions without abrupt cliffs at cell boundaries, except for Mountain–Ocean transitions (which intentionally have sharp drop-offs representing coastal cliffs).

```typescript
function getEdgeElevation(cellA: HexCell, cellB: HexCell): number {
  // Cliff exception: Mountain adjacent to Ocean
  if (
    (cellA.biome === Biome.Mountain && cellB.biome === Biome.Ocean) ||
    (cellA.biome === Biome.Ocean && cellB.biome === Biome.Mountain)
  ) {
    return Math.min(cellA.elevation, cellB.elevation); // Cliff: use lower
  }
  return (cellA.elevation + cellB.elevation) / 2; // Smooth blend
}
```

---

## 7. Instanced Rendering and Performance Architecture

### 7.1 InstancedMesh for Buildings

All buildings of the same type and LOD level are rendered using `THREE.InstancedMesh`. This reduces draw calls from O(n_buildings) to O(n_building_types × n_lod_levels), which is a fixed constant (12 types × 4 LOD levels = 48 draw calls maximum for buildings).

```typescript
class BuildingInstanceManager {
  private meshes: Map<string, THREE.InstancedMesh> = new Map();
  private instanceData: Map<string, InstanceEntry[]> = new Map();

  initialize(buildingTypes: BuildingType[]): void {
    for (const type of buildingTypes) {
      for (const lod of [0, 1, 2]) {
        const key = `${type}_lod${lod}`;
        const geometry = BUILDING_GEOMETRIES.get(key)!;
        const material = BUILDING_MATERIALS.get(type)!;
        // Pre-allocate for max expected instances (adjust as needed)
        const mesh = new THREE.InstancedMesh(geometry, material, MAX_BUILDINGS_PER_TYPE);
        mesh.count = 0; // Start with zero visible instances
        this.meshes.set(key, mesh);
      }
    }
  }

  addBuilding(entityId: string, type: string, position: THREE.Vector3, nationColor: string): void {
    const dummy = new THREE.Object3D();
    dummy.position.copy(position);
    dummy.updateMatrix();

    const key = `${type}_lod0`; // Will be managed by LOD system
    const mesh = this.meshes.get(key)!;
    const index = mesh.count++;
    mesh.setMatrixAt(index, dummy.matrix);
    mesh.setColorAt(index, new THREE.Color(nationColor));
    mesh.instanceMatrix.needsUpdate = true;
    mesh.instanceColor!.needsUpdate = true;

    this.instanceData.set(entityId, [{ meshKey: key, index }]);
  }
}
```

**Per-instance color override:** Three.js `InstancedMesh` supports per-instance color via `instanceColor` (a `THREE.InstancedBufferAttribute`). This eliminates the need for separate material instances per nation, allowing all nations' buildings of the same type to share a single draw call.

### 7.2 Citizen Sprite Strategy

Citizens are rendered as billboard sprites (screen-facing quads) at all zoom levels except Zoom 3 close focus. At Zoom 3, citizens within 3 hex cells of the camera center render as simplified GLTF character models (single LOD, ~300 triangles).

```typescript
const CITIZEN_SPRITE_CONFIG = {
  sprite_size_world_units: 2.5,
  atlas_texture: "assets/textures/citizen_atlas.png",
  atlas_cols: 8,
  atlas_rows: 4,
  // 32 sprite variants: 8 occupation types × 4 animation frames
  occupation_row_map: {
    farmer: 0,
    soldier: 1,
    merchant: 2,
    scholar: 3,
    priest: 4,
    craftsman: 5,
    noble: 6,
    laborer: 7,
  },
  gltf_switch_distance: 60, // Switch to GLTF below this camera distance
};
```

Citizens at Zoom 1 are represented as aggregate density overlays (heatmap shader), not individual sprites. This avoids tens of thousands of sprite instances at the strategic zoom level.

### 7.3 Particle Systems

Weather and FX are implemented using `THREE.Points` with a custom `ShaderMaterial`. Each particle system is a separate `THREE.Points` instance.

| System | Max Particles | Lifetime | Emitter | Trigger |
|--------|--------------|---------|---------|---------|
| Building smoke | 200/building | 3.0s | Building chimney position | `building.smoke_producing` flag |
| Combat fire | 500/battle | 1.5s | Battle hex center | `combat.active.v1` |
| Rain | 2000 (global) | 1.0s | Grid above camera | `climate.event.v1` with type=rain |
| Snow | 1500 (global) | 4.0s | Grid above camera | `climate.event.v1` with type=snow |
| Dust storm | 800 (regional) | 8.0s | Desert biome cells | `climate.event.v1` with type=dust |
| Explosion | 300 (one-shot) | 0.5s | Destroyed building | `buildings.demolished.v1` |

Particle systems are GPU-accelerated: position updates are computed in the vertex shader using time uniform, avoiding CPU-side particle position updates.

### 7.4 Draw Call Budget

The draw call budget at Zoom 2 (12×12 cell view, the primary gameplay view) is strictly enforced:

| Category | Max Draw Calls | Strategy |
|----------|---------------|---------|
| Terrain cells | 1 | Merge all terrain into single geometry buffer |
| Buildings (all types) | 48 | InstancedMesh (12 types × 4 LOD levels) |
| Citizens | 2 | InstancedMesh (sprite quad) + optional GLTF close |
| Particle systems | 5 | One Points object per active weather type |
| Border lines | 1 | Single LineSegments object (all borders merged) |
| Water | 1 | Single animated ShaderMaterial quad |
| UI overlay | N/A | DOM layer (not WebGL draw calls) |
| **Total** | **< 60 draw calls** | Well under 200-call budget |

The 200-call budget provides headroom for:
- Per-nation flag meshes (up to 8 nations × 1 draw call)
- Combat animation overlays
- Selection ring geometry
- Debug visualization (dev builds only)

### 7.5 FPS Targets by Hardware Tier

| Hardware Tier | Target FPS | Min FPS | Zoom Level | Fallback Strategy |
|--------------|-----------|---------|-----------|------------------|
| M2 MacBook (Apple Silicon) | 60 | 45 | All | None needed |
| NVIDIA GTX 1060 (6 GB) | 45 | 30 | Zoom 1–2 | Drop shadow quality |
| NVIDIA GTX 1060 | 30 | 20 | Zoom 3 | Reduce particle count |
| Intel UHD 630 (integrated) | 30 | 15 | Zoom 1 only | Disable particles; LOD2 forced |
| Mobile (WebGL) | 20 | 10 | Zoom 1 only | Disable shadows; LOD3 forced |

FPS measurement: Rolling 60-frame average. Adaptive quality kicks in if 60-frame average drops below threshold for 3 consecutive windows. Quality reduction is one step at a time; quality is re-raised when average sustains above threshold+10 for 10 windows.

---

## 8. Nation Visual Identity System

### 8.1 Nation Identity Schema

Every nation in CivLab has a visual identity record. This record is part of the simulation state (serialized in `world.snapshot.v1`) and is consumed by both the 2D and 3D clients.

```typescript
interface NationVisualIdentity {
  nation_id: string;                    // "roman_empire", "han_dynasty", etc.
  primary_color: string;               // "#8B0000" — dominant color
  secondary_color: string;             // "#FFD700" — accent color
  architectural_style: ArchStyle;      // "mediterranean" | "nordic" | "east_asian" | ...
  icon_symbol: string;                 // SVG path string (used in 2D and as flag icon)
  flag_texture_url: string;            // URL to nation flag SVG (converted to texture at runtime)
  era_aesthetic_modifiers: Record<Era, string>;  // Per-era style override for prompt generation
}

type ArchStyle =
  | "mediterranean"
  | "nordic"
  | "east_asian"
  | "sub_saharan"
  | "mesoamerican"
  | "central_asian"
  | "middle_eastern"
  | "south_asian"
  | "andean";
```

**Built-in nation identities (at launch):**

| Nation | Primary | Secondary | Style | Architectural Notes |
|--------|---------|-----------|-------|---------------------|
| Roman Empire | #8B0000 (crimson) | #FFD700 (gold) | mediterranean | Columns, arches, terracotta |
| Han Dynasty | #CC0000 (red) | #FFD700 (gold) | east_asian | Upturned eaves, jade accents |
| Maurya Empire | #FF6600 (saffron) | #006400 (green) | south_asian | Carved stone, stupas |
| Norse Kingdoms | #003366 (navy) | #C0C0C0 (silver) | nordic | Timber, longhouse proportions |
| Mali Empire | #8B4513 (earth) | #FFD700 (gold) | sub_saharan | Mud brick, tapered towers |
| Aztec Empire | #006400 (jade) | #8B0000 (red) | mesoamerican | Step pyramids, featherwork motifs |
| Mongol Khanate | #808080 (grey) | #8B4513 (brown) | central_asian | Tent-derived shapes, felt motifs |
| Byzantine Empire | #4B0082 (purple) | #FFD700 (gold) | mediterranean | Domed, mosaic-like |

Nations created via scenario editor may define custom visual identities. The schema validation ensures all required fields are present and colors are valid hex codes.

### 8.2 Runtime Material Override

The runtime color injection system works as follows. When a building GLTF model is loaded, the `NationColorInjector` traverses the scene graph and updates the `nation_color_material` slot:

```typescript
class NationColorInjector {
  private cache: Map<string, THREE.MeshStandardMaterial> = new Map();

  applyToModel(model: THREE.Group, identity: NationVisualIdentity): void {
    const cacheKey = `${model.userData.building_type}_${identity.primary_color}`;

    model.traverse((node) => {
      if (!(node instanceof THREE.Mesh)) return;

      const mats = Array.isArray(node.material) ? node.material : [node.material];
      mats.forEach((mat, idx) => {
        if (mat.name === "nation_color_material") {
          // Use cached material if available (shared across same-nation instances)
          if (!this.cache.has(cacheKey)) {
            const clone = (mat as THREE.MeshStandardMaterial).clone();
            clone.color.set(identity.primary_color);
            this.cache.set(cacheKey, clone);
          }
          if (Array.isArray(node.material)) {
            (node.material as THREE.Material[])[idx] = this.cache.get(cacheKey)!;
          } else {
            node.material = this.cache.get(cacheKey)!;
          }
        }
        if (mat.name === "secondary_accent_material") {
          // Similar pattern for secondary color
          const secKey = `${cacheKey}_sec`;
          if (!this.cache.has(secKey)) {
            const clone = (mat as THREE.MeshStandardMaterial).clone();
            clone.color.set(identity.secondary_color);
            this.cache.set(secKey, clone);
          }
          if (Array.isArray(node.material)) {
            (node.material as THREE.Material[])[idx] = this.cache.get(secKey)!;
          } else {
            node.material = this.cache.get(secKey)!;
          }
        }
      });
    });
  }

  clearCache(): void {
    this.cache.forEach((mat) => mat.dispose());
    this.cache.clear();
  }
}
```

Material cloning is performed once per (building_type, nation_color) combination. Subsequent instances of the same combination share the cached material, avoiding O(n_buildings) material objects. The cache is cleared when the simulation resets.

### 8.3 Flag System

Nation flags are rendered as animated GLTF meshes attached to flagpole buildings (Barracks, Palace, Harbor, Tower). The flag mesh is a subdivided quad (20×10 segments) with a vertex shader that simulates cloth wave motion.

**Flag vertex shader (simplified):**

```glsl
// Vertex shader for flag wave animation
uniform float time;
uniform float waveStrength;  // 0.3 for gentle breeze, 1.0 for storm
uniform sampler2D flagTexture;

varying vec2 vUv;

void main() {
  vUv = uv;
  vec3 pos = position;

  // Wave amplitude increases toward free edge (right side of flag)
  float edgeFactor = uv.x; // 0 at flagpole, 1 at free edge

  // Primary wave
  float wave = sin(pos.x * 3.0 + time * 2.5) * 0.15 * edgeFactor;
  // Secondary harmonic
  float wave2 = sin(pos.x * 7.0 + time * 3.8 + 0.5) * 0.05 * edgeFactor;

  pos.z += (wave + wave2) * waveStrength;
  pos.y += sin(pos.x * 2.0 + time * 2.0) * 0.05 * edgeFactor;

  gl_Position = projectionMatrix * modelViewMatrix * vec4(pos, 1.0);
}
```

The `flagTexture` is derived from `nation.flag_texture_url`. The URL is an SVG, converted to a 128×64 `THREE.CanvasTexture` at runtime using a `<canvas>` element. This avoids pre-baking nation flag images while still achieving correct rendering.

**Flag wind speed driven by climate events:**

| Climate Event | `waveStrength` | Notes |
|--------------|---------------|-------|
| Clear / default | 0.3 | Gentle constant breeze |
| Wind event | 0.7 | Elevated wave amplitude |
| Storm event | 1.0 | Maximum flag agitation |
| Drought / calm | 0.1 | Near-static flag |

### 8.4 Border Rendering

Nation territory borders are rendered using a post-processing edge detection pass applied to a separate render target (the "nation ID buffer"). This avoids per-cell border geometry management.

**Nation ID buffer:** A `THREE.WebGLRenderTarget` rendered at half resolution (512×288 minimum). Each pixel contains the encoded nation ID as a color value. The post-processing pass reads this buffer and draws border lines at nation ID discontinuities.

**Border fragment shader (key excerpt):**

```glsl
uniform sampler2D nationIdBuffer;
uniform vec2 texelSize;
uniform vec3 peaceBorderColor;     // Default: white, 30% opacity
uniform vec3 conflictBorderColor;  // War state: red, 70% opacity

void main() {
  vec4 center = texture2D(nationIdBuffer, vUv);
  vec4 right  = texture2D(nationIdBuffer, vUv + vec2(texelSize.x, 0.0));
  vec4 down   = texture2D(nationIdBuffer, vUv + vec2(0.0, texelSize.y));

  float edgeX = abs(center.r - right.r) > 0.01 ? 1.0 : 0.0;
  float edgeY = abs(center.r - down.r)  > 0.01 ? 1.0 : 0.0;
  float edge  = max(edgeX, edgeY);

  // Conflict state: borders adjacent to at-war nations glow red
  bool isConflictEdge = isWarActive(center.r, right.r) || isWarActive(center.r, down.r);
  vec3 borderColor = isConflictEdge ? conflictBorderColor : peaceBorderColor;

  gl_FragColor = vec4(borderColor, edge * (isConflictEdge ? 0.85 : 0.40));
}
```

This approach renders borders as a single full-screen quad pass, with zero additional draw calls per nation or per hex cell. The conflict state is passed as a uniform buffer object updated when `war.declared.v1` or `war.ended.v1` events are received.

---

## 9. Transition Implementation Plan

### 9.1 Phase 0 — 2D Only (Current)

**Status:** Active development. Deliverables defined in CIV-0600.

**3D work in Phase 0:**
- Define this spec (CIV-0601) — complete
- Define ArtSpec IR v2 schema — complete (§5.1)
- Pre-assign canonical generation seeds (§4.5) — complete
- Create `clients/web_3d/` scaffold directory with placeholder README
- Add `render_mode` field to CIV-0600 IR schema (non-breaking extension)

**What does NOT happen in Phase 0:**
- No Three.js code written
- No GLTF assets generated
- No 3D-specific CI gates added (avoid blocking 2D pipeline)

**Exit criteria for Phase 0:** 2D MVP Gate (G0) — see §1.4.

### 9.2 Phase 1 — Three.js Scaffold

**Target:** G1 gate (3D Prototype).
**Duration estimate:** 2–3 agent-weeks (parallel to 2D polish work).

**Deliverables:**

1. `clients/web_3d/` — Vite + Three.js r168 project scaffold
2. CIV-0200 WebSocket client (adapted from 2D client; same protocol)
3. Hex grid rendered as flat `PlaneGeometry` tiles (no heightmap yet)
4. Heightmap generation in terrain crate (Rust); serialization in `world.snapshot.v1`
5. Terrain heightmap applied to hex grid (displacement in Three.js)
6. Placeholder box buildings (`THREE.BoxGeometry`) per cell — nation-colored, no LOD
7. Camera system (isometric, pan/zoom/rotate) — §2.3
8. `SceneEntityRegistry` — §2.2

**What is NOT in Phase 1:**
- GLTF assets (placeholder boxes only)
- LOD system
- Particle systems
- Nation flag meshes
- Border rendering

**Phase 1 CI gates:**
- `npm run build` succeeds (Vite build)
- Three.js renderer initializes without WebGL errors
- `world.snapshot.v1` received and terrain rendered (automated E2E with headless Chromium)

### 9.3 Phase 2 — First GLTF Buildings

**Target:** G2 gate (3D Alpha).
**Duration estimate:** 3–4 agent-weeks.

**Deliverables:**

1. GLTF asset pipeline tooling:
   - `tools/asset_gen/quality_gate.py` — automated quality checks
   - `tools/asset_gen/optimize.sh` — glTF Transform invocation script
   - `tools/asset_gen/billboard_gen.ts` — headless billboard renderer
2. First 12 GLTF building models (one per building type, Classical era only):
   - Generated via Meshy.ai with canonical seeds (§4.5)
   - LOD0 and LOD1 only (LOD2 and LOD3 deferred to Phase 3)
   - All passing quality gate
3. `BuildingInstanceManager` — §7.1
4. `NationColorInjector` — §8.2
5. `THREE.LOD` integration for LOD0/LOD1 switching
6. Asset manifest v2 (`asset_manifest_3d.json`) with 12 Classical entries

**Performance target:** 30 FPS sustained at Zoom 2 on M2 MacBook with all 12 building types visible.

**Phase 2 CI gates:**
- All 12 GLB files pass glTF Validator (0 errors)
- All GLB files within polygon budget (LOD0 ≤ 5000, LOD1 ≤ 1000)
- Nation color injection verified (automated pixel-color test in headless renderer)
- 30 FPS minimum confirmed via automated benchmark (headless Chromium with performance.now timing)

### 9.4 Phase 3 — Agentic Generation Integration

**Target:** G3 gate (3D Beta).
**Duration estimate:** 4–6 agent-weeks (asset generation dominates).

**Deliverables:**

1. Python orchestrator (`tools/asset_gen/orchestrator.py`) — §10.1
2. `missing_assets.json` generator (diffs manifest against required catalog)
3. Full 72-model catalog generated and validated:
   - 12 building types × 6 eras
   - Meshy.ai for Classical era hero assets (already done in Phase 2)
   - InstantMesh for remaining 60 era variants
4. LOD2 and LOD3 (billboard) for all 72 assets
5. Terrain biome textures and biome blend shader — §6.3, §6.4
6. Particle systems (smoke, fire, weather) — §7.3
7. Animated buildings (idle, construction, combat clips) — §3.5
8. Nation flag meshes — §8.3
9. Border rendering post-processing pass — §8.4

**Phase 3 CI gates:**
- All 72 GLB files pass quality gate
- Manifest completeness check: all building_type × era × LOD combinations present
- Animation clip name validation against catalog (§3.5)
- Biome terrain renders without shader errors in all 6 biome types
- Performance: 45 FPS minimum at Zoom 2 on M2 MacBook

### 9.5 Phase 4 — Full Terrain and FX

**Target:** G4 gate (3D Production).
**Duration estimate:** 3–4 agent-weeks.

**Deliverables:**

1. Terrain streaming system (§6.5) — dynamic cell load/unload
2. Water mesh with animated wave shader
3. Weather particle systems (rain, snow, dust storm) — driven by climate events
4. Full lighting model: directional sun, hemisphere ambient, time-of-day cycle
5. Post-processing: SSAO, bloom (subtle), chromatic aberration (off by default)
6. Mobile WebGL optimization pass:
   - Detect WebGL capabilities on init
   - Downgrade LOD bias on mobile
   - Disable SSAO on low-end devices
7. Performance profiling sweep across all hardware tiers (§7.5)
8. Full FR validation run (§12) — all 15 FRs passing

**Phase 4 CI gates:**
- All FRs from §12 pass automated checks
- FPS benchmarks confirmed on all hardware tiers (manual QA for GTX 1060; automated for M2)
- Memory budget validation: VRAM ≤ 512 MB, RAM ≤ 256 MB for Zoom 2 view
- Scene init time ≤ 2s for Zoom 2 view (§14.1)

### 9.6 Phase 5 — Native Client (Optional)

**Gate:** G5 (business-case dependent).
**Prerequisite:** G4 complete; explicit product decision to invest in native client.

**Scope:**
- Bevy 0.15 client (`clients/bevy_3d/`) sharing same CIV-0200 WebSocket protocol
- Same glTF asset manifest consumed by bevy_gltf loader
- Same ArtSpec IR v2 consumed for runtime color injection
- Bevy-native LOD system (distance-based mesh swapping)
- Native desktop packaging: macOS app bundle, Windows installer

**Bevy-specific notes:**
- `bevy_gltf` automatically loads animations; `AnimationPlayer` component manages playback
- Color override requires a custom `NationColorPlugin` that modifies `StandardMaterial` at load time
- The CIV-0200 WebSocket connection uses `bevy_web_asset` or a custom `bevy_tokio` integration
- ECS component structure mirrors simulation entity model for conceptual alignment

**Exclusions:** Bevy WASM target is excluded. Three.js serves the browser client. Bevy serves native desktop only.

---

## 10. Agentic Generation Orchestration

### 10.1 Python Orchestrator Architecture

The orchestrator is a single Python script (`tools/asset_gen/orchestrator.py`) that coordinates the full agentic generation pipeline. It is designed to run as a batch job (overnight generation) or incrementally (generate only missing assets).

```python
# Simplified orchestrator structure (not production code — pseudocode for spec)

class AssetOrchestrator:
    def __init__(self, config: OrchestratorConfig):
        self.meshy = MeshyClient(api_key=config.meshy_api_key)
        self.instantmesh = InstantMeshRunner(model_path=config.instantmesh_model)
        self.quality_gate = QualityGate(config=config.quality_gate)
        self.manifest = AssetManifest3D(path=config.manifest_path)
        self.failures = FailureLog(path=config.failure_log_path)

    async def run(self, required: list[AssetSpec]) -> OrchestratorResult:
        missing = self._compute_missing(required)
        hero_assets = [s for s in missing if s.era == Era.CLASSICAL]
        variant_assets = [s for s in missing if s.era != Era.CLASSICAL]

        # Hero assets: Meshy.ai (quality priority)
        hero_results = await asyncio.gather(*[
            self._generate_hero(spec) for spec in hero_assets
        ])

        # Variant assets: InstantMesh (cost priority)
        # InstantMesh runs sequentially due to GPU memory constraints
        variant_results = []
        for spec in variant_assets:
            reference = self._get_hero_reference(spec.building_type)
            result = await self._generate_variant(spec, reference)
            variant_results.append(result)

        # Post-processing: parallel for all generated assets
        all_raw = [r for r in hero_results + variant_results if r.success]
        post_results = await asyncio.gather(*[
            self._post_process(r) for r in all_raw
        ])

        # Quality gate: fail-fast per asset, do not block others
        for result in post_results:
            gate_result = self.quality_gate.check(result.output_path)
            if gate_result.passed:
                self.manifest.add(result.to_manifest_entry(gate_result))
            else:
                self.failures.log(result, gate_result)

        self.manifest.save()
        return OrchestratorResult(
            generated=len(all_raw),
            passed=self.manifest.new_count,
            failed=self.failures.new_count,
        )

    def _compute_missing(self, required: list[AssetSpec]) -> list[AssetSpec]:
        existing_ids = {e.asset_id for e in self.manifest.entries}
        return [s for s in required if s.asset_id not in existing_ids]
```

**Configuration (`tools/asset_gen/config.yaml`):**

```yaml
orchestrator:
  meshy_api_key: "${MESHY_API_KEY}"        # From environment
  instantmesh_model: "models/instantmesh_v1.pt"
  manifest_path: "assets/3d/asset_manifest_3d.json"
  failure_log_path: "logs/generation_failures.jsonl"
  output_dir: "assets/3d"
  parallelism_meshy: 10                    # Max concurrent Meshy API calls
  retry_failed_assets: 1                   # Retry count before escalating to manual
  quality_gate:
    max_triangle_lod0: 5000
    max_triangle_lod1: 1000
    max_triangle_lod2: 200
    max_uv_overlap_pct: 2.0
    max_gltf_errors: 0
    max_gltf_warnings: 5
    max_file_size_mb: 2.0
```

### 10.2 Async Batch Generation

Meshy.ai supports up to 10 concurrent generation requests on the standard API tier. The orchestrator uses `asyncio.gather` with a semaphore to cap concurrency.

```python
MESHY_SEMAPHORE = asyncio.Semaphore(10)

async def _generate_hero(self, spec: AssetSpec) -> GenerationResult:
    async with MESHY_SEMAPHORE:
        prompt = build_prompt(spec.building_type, spec.era, spec.nation_aesthetic)
        try:
            task = await self.meshy.create_text_to_3d(
                prompt=prompt,
                negative_prompt=NEGATIVE_PROMPT,
                seed=CANONICAL_SEEDS[spec.building_type],
                art_style="game-low-poly",
                topology="quad",
                target_polycount=spec.lod_budget.lod0,
            )
            # Poll until complete (Meshy uses async task model)
            result = await self.meshy.wait_for_task(task.task_id, timeout=300)
            glb_path = await self.meshy.download_glb(result, self.config.output_dir)
            return GenerationResult(success=True, spec=spec, raw_path=glb_path, seed=result.seed)
        except MeshyAPIError as e:
            # Fail loudly — no silent fallback
            raise GenerationError(f"Meshy generation failed for {spec.asset_id}: {e}") from e
```

**Note on failure behavior:** Per the global guidelines, there are no silent fallbacks. If Meshy generation fails after the configured retry count, the orchestrator raises an exception and logs the failure. The asset is excluded from the manifest and must be regenerated or manually authored.

### 10.3 Cost Model

Full catalog generation cost estimate (at time of spec writing, 2026-02-21):

| Component | Count | Unit Cost | Total |
|-----------|-------|-----------|-------|
| Meshy.ai hero assets (Classical era) | 12 | $0.05 | $0.60 |
| InstantMesh variants (5 eras × 12 types) | 60 | $0.00 (local GPU) | $0.00 |
| GPU compute for InstantMesh (A100 cloud, est.) | 60 × 2 min | $0.04/min | $4.80 |
| glTF Transform optimization (CPU, local) | 72 assets | $0.00 | $0.00 |
| Billboard render (headless Chromium, local) | 72 assets | $0.00 | $0.00 |
| **Total** | | | **~$5.40** |

If InstantMesh runs on local hardware (M2 MacBook with MPS backend or a development GPU machine), the cloud compute cost drops to $0. The Meshy.ai API cost of $0.60 for 12 hero assets is a fixed floor.

**Regeneration cost:** If canonical seeds are changed (a breaking change requiring new asset_ids), the full catalog must be regenerated. Cost is identical to initial generation.

### 10.4 Visual Inspector Tool

The visual inspector is a minimal Three.js single-page application for reviewing generated assets before adding them to the manifest.

```
URL: http://localhost:5174/inspector?asset_id=building_granary_classical_3d&lod=0

Controls:
  - Orbit camera (drag)
  - Toggle LOD level (keys 0–3)
  - Toggle nation color (cycle through all 8 nations)
  - Toggle wireframe
  - Show triangle count overlay
  - Approve / Reject buttons (write to review_queue.json)
```

The inspector is part of the dev toolchain only (`tools/inspector/`). It is not shipped in the production client bundle.

**Inspector review workflow:**

```
orchestrator completes → review_queue.json updated with pending assets
                                    ↓
         Developer opens inspector → reviews each asset visually
                                    ↓
          Approve: manifest entry confirmed; asset moved to assets/3d/approved/
          Reject: asset added to manual_rework.json; orchestrator logs failure
```

For automated (no-human) generation runs (e.g., overnight CI job), the visual inspection step is skipped and assets are auto-approved if they pass all quality gate checks. A weekly human review of auto-approved assets is recommended.

### 10.5 Manifest Lifecycle

The `asset_manifest_3d.json` file is the authoritative record of all 3D assets. It is checked into version control.

**Manifest update rules:**
1. The orchestrator ONLY adds entries; it never removes or modifies existing entries.
2. To replace an asset, create a new asset_id (with `_v2` suffix or era-variant suffix) and deprecate the old one by adding `"deprecated": true` to the old entry.
3. Manifest entries are immutable after creation. If the generation pipeline changes, new assets are generated with new asset_ids.
4. The manifest is validated in CI on every pull request: schema validation, required fields, unique asset_ids, all referenced file paths exist.

**Manifest file structure:**

```json
{
  "$schema": "https://civlab.dev/schemas/asset-manifest-3d/v1.json",
  "version": "1",
  "generated_at": "2026-02-21T00:00:00Z",
  "total_assets": 72,
  "assets": [
    {
      "asset_id": "building_granary_classical_3d",
      "building_class": "granary",
      "era": "classical",
      "lod_files": {
        "lod0": "assets/3d/granary/classical/granary_classical_lod0.glb",
        "lod1": "assets/3d/granary/classical/granary_classical_lod1.glb",
        "lod2": "assets/3d/granary/classical/granary_classical_lod2.glb",
        "lod3": "assets/3d/granary/classical/granary_classical_lod3_billboard.glb"
      },
      "generation_method": "meshy_api",
      "generation_seed": 101742,
      "meshy_task_id": "task_abc123",
      "quality_gate_result": { "passed": true, "checked_at": "2026-02-21T14:30:00Z" },
      "deprecated": false
    }
  ]
}
```

---

## 11. Blender Pipeline — Artist Path

### 11.1 Blender Workflow

The Blender pipeline is the artist-driven complement to the agentic generation pipeline. Artists use Blender 4.x to create high-quality base meshes for hero assets, which are then processed through the same optimization chain as agentic outputs.

**When to use Blender over agentic generation:**
- The agentic output for a specific building type fails quality gate repeatedly
- A building requires specific silhouette characteristics that prompt engineering cannot reliably produce
- A hero asset is used prominently in promotional materials and requires pixel-perfect quality
- Custom animation rigs (beyond the standard animation catalog) are required

**Blender version:** 4.x (specifically whichever version is current at the time of Phase 2; pin in `tools/blender/README.md`).

**Blender project conventions:**

| Convention | Specification |
|-----------|--------------|
| Unit system | Metric; 1 Blender unit = 1 meter |
| Origin placement | Center of base footprint, at ground level (Z=0) |
| Forward axis | -Y (matches glTF convention) |
| Up axis | +Z |
| Scale | Apply all transforms before export (Ctrl+A → All Transforms) |
| Naming | Object names match material slot names: `granary_body`, `nation_color_material`, etc. |
| Modifiers | Apply all modifiers except Armature before export |
| Linked libraries | No external Blender file links; everything packed |

### 11.2 Export and Optimization Chain

Blender exports glTF 2.0 binary (.glb) using the built-in glTF exporter (shipped with Blender 4.x).

**Blender glTF export settings (canonical):**

```
Format: glTF Binary (.glb)
Include: Selected Objects only
Transform:
  Y Up: True
Geometry:
  Apply Modifiers: True
  UV Maps: All UV Maps
  Vertex Colors: None (use material instead)
  Normals: True
  Tangents: False (computed by Three.js)
  Loose Edges: False
  Loose Points: False
Material:
  Export: Export
  Images: Automatic
  Image Format: JPEG (for albedo/roughness); PNG (for normal maps)
  JPEG Quality: 85
Animation:
  Export: All Actions
  Limit to Playback Range: True
  NLA Strips: False (use individual actions)
  Force Sampling: False
  Optimize Animation Size: True
Compression: None (glTF Transform handles this post-export)
```

After Blender export, the same optimization chain from §3.2 runs on the output .glb file. The output is then validated with the same quality gate as agentic outputs (§4.6).

### 11.3 Auto-LOD via Blender Script

Blender's Decimate modifier is used to generate LOD1 and LOD2 from the artist-authored LOD0 mesh. A Python script automates this:

```python
# tools/blender/auto_lod.py
# Run via: blender --background --python tools/blender/auto_lod.py -- input.blend output_dir/

import bpy
import sys
import os

LOD_CONFIGS = [
    {"suffix": "lod0", "ratio": 1.0},    # Original mesh
    {"suffix": "lod1", "ratio": 0.20},   # 20% of original triangles
    {"suffix": "lod2", "ratio": 0.04},   # 4% of original triangles (~200 tri from 5000)
]

def generate_lods(base_object_name: str, output_dir: str) -> None:
    base_obj = bpy.data.objects[base_object_name]

    for config in LOD_CONFIGS:
        # Duplicate base mesh for this LOD level
        bpy.ops.object.select_all(action='DESELECT')
        base_obj.select_set(True)
        bpy.context.view_layer.objects.active = base_obj
        bpy.ops.object.duplicate()
        lod_obj = bpy.context.active_object
        lod_obj.name = f"{base_object_name}_{config['suffix']}"

        if config["ratio"] < 1.0:
            # Add and apply Decimate modifier
            decimate = lod_obj.modifiers.new(name="Decimate", type='DECIMATE')
            decimate.ratio = config["ratio"]
            decimate.use_collapse_triangulate = True
            bpy.ops.object.modifier_apply(modifier="Decimate")

        # Export this LOD level as separate GLB
        output_path = os.path.join(output_dir, f"{base_object_name}_{config['suffix']}.glb")
        bpy.ops.object.select_all(action='DESELECT')
        lod_obj.select_set(True)
        bpy.ops.export_scene.gltf(
            filepath=output_path,
            use_selection=True,
            export_format='GLB',
            export_yup=True,
            export_apply=True,
        )

        # Remove the duplicate after export
        bpy.data.objects.remove(lod_obj, do_unlink=True)

if __name__ == "__main__":
    args = sys.argv[sys.argv.index("--") + 1:]
    base_object_name = args[0]
    output_dir = args[1]
    generate_lods(base_object_name, output_dir)
```

**Expected LOD triangle counts from Decimate with base at 5000 triangles:**

| LOD | Ratio | Expected Triangles | Within Budget |
|-----|-------|--------------------|--------------|
| LOD0 | 1.0 | 5,000 | Yes (exactly at limit) |
| LOD1 | 0.20 | ~1,000 | Yes (at limit) |
| LOD2 | 0.04 | ~200 | Yes (at limit) |

Blender's Decimate modifier introduces some variation around these targets. The quality gate polygon count check (§4.6) verifies the output and flags any that exceed the budget.

### 11.4 CI Gate Specification

Every PR that adds or modifies a .glb file in `assets/3d/` triggers the following CI pipeline (`ci/validate-3d-assets.sh`):

```bash
#!/usr/bin/env bash
set -euo pipefail

# Discover all GLB files modified in this PR
CHANGED_GLBS=$(git diff --name-only origin/main...HEAD -- '*.glb')

for GLB in $CHANGED_GLBS; do
  echo "Validating: $GLB"

  # 1. glTF Validator (Khronos official)
  npx gltf-validator "$GLB" --stdout --format json | \
    python3 ci/check_gltf_validator_output.py --max-errors 0 --max-warnings 5

  # 2. Polygon count check
  python3 tools/asset_gen/quality_gate.py \
    --check polygon_count \
    --file "$GLB" \
    --config tools/asset_gen/config.yaml

  # 3. UV overlap check
  python3 tools/asset_gen/quality_gate.py \
    --check uv_overlap \
    --file "$GLB" \
    --config tools/asset_gen/config.yaml

  # 4. Material slot naming check
  python3 tools/asset_gen/quality_gate.py \
    --check material_slots \
    --file "$GLB" \
    --config tools/asset_gen/config.yaml

  # 5. File size check
  python3 tools/asset_gen/quality_gate.py \
    --check file_size \
    --file "$GLB" \
    --config tools/asset_gen/config.yaml

  echo "PASS: $GLB"
done

echo "All 3D asset validations passed."
```

CI failure on any gate check blocks the PR from merging. No exceptions and no bypass flags.

---

## 12. Functional Requirements

### FR-CIV-3D-001 — glTF Format Compliance

**SHALL:** All 3D building assets SHALL be delivered as glTF 2.0 binary (.glb) files that pass Khronos glTF Validator with zero errors and five or fewer warnings.

**Verification:** CI gate runs `gltf-validator` on every modified .glb file. Automated; blocks merge on failure.

---

### FR-CIV-3D-002 — LOD Budget Enforcement

**SHALL:** Each building asset SHALL have four LOD levels. Triangle counts SHALL not exceed: LOD0 ≤ 5,000; LOD1 ≤ 1,000; LOD2 ≤ 200; LOD3 = 2 (billboard quad).

**Verification:** CI gate polygon count check. Automated; blocks merge on failure.

---

### FR-CIV-3D-003 — Frame Rate — Primary Hardware

**SHALL:** The 3D web client SHALL maintain a minimum of 45 frames per second at 1080p resolution when rendering a Zoom 2 view (12×12 hex cell grid) on M2 MacBook hardware.

**Verification:** Automated benchmark via headless Chromium with performance.now frame timing. Run on CI hardware with Apple Silicon runner. Pass threshold: 95th percentile frame time ≤ 22.2ms over a 300-frame sample.

---

### FR-CIV-3D-004 — Frame Rate — Secondary Hardware

**SHALL:** The 3D web client SHALL maintain a minimum of 30 frames per second at 1080p when rendering Zoom 2 on NVIDIA GTX 1060 (6 GB VRAM) hardware.

**Verification:** Manual QA benchmark on GTX 1060 test machine. Documented in Phase 4 test report.

---

### FR-CIV-3D-005 — Nation Color Injection

**SHALL:** Building GLTF models SHALL support runtime nation color injection without requiring texture rebaking. The primary nation color SHALL be applied to the `nation_color_material` slot within one animation frame of the scene receiving a `world.snapshot.v1` event with updated nation data.

**Verification:** Automated pixel-color test: render scene with known nation color; sample pixel at known building location; assert color matches nation primary within ±5 RGB units. Run in CI with headless Chromium.

---

### FR-CIV-3D-006 — Terrain Determinism

**SHALL:** Given identical scenario seeds, terrain heightmap generation SHALL produce bit-identical output across multiple runs, on any supported platform (x86_64, arm64).

**Verification:** Property-based test (`crates/terrain/tests/determinism_test.rs`): generate heightmap with N random seeds, serialize as PNG, hash with BLAKE3; assert hashes match across 2 runs. Run in CI on both x86_64 and arm64 runners.

---

### FR-CIV-3D-007 — Agentic Generation Reproducibility

**SHALL:** Given an identical canonical seed, Meshy.ai API version, and prompt, the agentic generation pipeline SHALL produce the same .glb output. Reproducibility is verified by storing the Meshy task_id and seed in the manifest and confirming that re-fetching the task produces the same file hash.

**Verification:** Manual verification once per new Meshy.ai API version. Documented in asset manifest changelog.

---

### FR-CIV-3D-008 — Draw Call Budget

**SHALL:** The Three.js renderer SHALL issue fewer than 200 WebGL draw calls per frame when rendering a Zoom 2 view with all building types visible and at least one active weather particle system.

**Verification:** Automated benchmark using Three.js renderer info (`renderer.info.render.calls`). Assert < 200 in integration test with a synthetic scene containing one instance of each building type and active rain particles.

---

### FR-CIV-3D-009 — Scene Initialization Time

**SHALL:** The 3D client SHALL complete initial scene construction (terrain rendered, buildings spawned, camera positioned) within 2 seconds of receiving the first `world.snapshot.v1` message, measured on M2 MacBook hardware.

**Verification:** Automated E2E test: connect 3D client to headless simulation; measure time from first `world.snapshot.v1` to `renderer.domElement.renderTime.firstFrame`. Assert ≤ 2000ms.

---

### FR-CIV-3D-010 — VRAM Budget

**SHALL:** Total GPU VRAM consumption for the 3D client in Zoom 2 view SHALL not exceed 512 MB.

**Verification:** Manual profiling with Chrome DevTools GPU memory panel on M2 MacBook. Documented in Phase 4 test report. Automated approximation via Three.js texture/geometry memory tracking in integration tests.

---

### FR-CIV-3D-011 — Biome Coverage

**SHALL:** The terrain generation system SHALL produce terrain featuring all six biome types (ocean, plains, forest, desert, tundra, mountain) when using the default scenario seed set.

**Verification:** Unit test in `crates/terrain/tests/biome_coverage_test.rs`: generate terrain with 20 different seeds; assert all 6 biomes appear in at least 15 of the 20 generated maps.

---

### FR-CIV-3D-012 — Protocol Agnosticism

**SHALL:** The 3D client SHALL connect to the simulation core using the identical JSON-RPC WebSocket protocol defined in CIV-0200, with zero modifications to the core or protocol.

**Verification:** Integration test: run both 2D and 3D clients against the same headless core instance simultaneously; verify both receive identical `world.snapshot.v1` payloads (compare JSON hashes).

---

### FR-CIV-3D-013 — Animation Catalog Completeness

**SHALL:** At G3 gate, all 72 building models (12 types × 6 eras) SHALL include the mandatory animation clips: `idle_ambient`, `construction_progress`, `construction_complete`, and `demolish_collapse`. Optional clips (per the animation catalog in §3.5) are not required for all building types.

**Verification:** CI gate animation clip name check: parse each .glb with gltf-transform; assert mandatory clip names present in animation list.

---

### FR-CIV-3D-014 — View Layer Purity

**SHALL:** The 3D client SHALL contain zero simulation logic. All state transitions SHALL be driven exclusively by events received from the simulation core over the CIV-0200 protocol. No simulation calculations SHALL occur in the client codebase.

**Verification:** Static analysis: grep 3D client source for known simulation logic patterns (tick computation, resource arithmetic, AI decision logic). Assert zero matches. Enforced as a code review requirement with documentation in PR template.

---

### FR-CIV-3D-015 — Texture Atlas Completeness

**SHALL:** All terrain biome textures SHALL be packed into a single 4096×4096 texture atlas. The atlas SHALL be loadable in a single GPU texture upload operation.

**Verification:** CI check: verify atlas file dimensions (exactly 4096×4096), power-of-two, and that texture count in terrain shader uniforms equals 1 (atlas) rather than N (individual textures).

---

## 13. Integration with Simulation Core

### 13.1 Protocol Attachment

The 3D client attaches to the simulation core via the same JSON-RPC WebSocket endpoint used by the 2D client (CIV-0200). No special configuration is required on the simulation core side. The core does not distinguish between 2D and 3D clients.

**Client registration payload (3D variant):**

```json
{
  "jsonrpc": "2.0",
  "method": "client.register",
  "params": {
    "client_id": "3d-web-abc123",
    "client_type": "3d_web",
    "protocol_version": "2.0",
    "capabilities": ["snapshot", "events", "commands"],
    "zoom_preference": 2
  },
  "id": 1
}
```

The `client_type: "3d_web"` field is informational only; the core treats all clients identically. The `zoom_preference` field hints to the core which zoom-level event granularity to send (e.g., Zoom 1 clients receive aggregated nation-level events, not per-citizen events).

### 13.2 Event-Driven Scene Updates

The 3D client maintains a `SceneEventHandler` that maps simulation events to Three.js scene mutations. All mutations are batched and applied at the start of the next animation frame (via `requestAnimationFrame`), not immediately on WebSocket message receipt, to avoid mid-frame scene graph mutations.

```typescript
class SceneEventHandler {
  private pendingMutations: SceneMutation[] = [];

  onEvent(event: SimulationEvent): void {
    switch (event.type) {
      case "buildings.constructed.v1":
        this.pendingMutations.push({
          type: "spawn_building",
          entityId: event.data.building_id,
          buildingClass: event.data.building_class,
          era: event.data.era,
          position: hexToWorld(event.data.hex_q, event.data.hex_r),
          nationId: event.data.nation_id,
        });
        break;

      case "buildings.demolished.v1":
        this.pendingMutations.push({
          type: "remove_building",
          entityId: event.data.building_id,
          playAnimation: "demolish_collapse",
        });
        break;

      case "citizen.migrated.v1":
        this.pendingMutations.push({
          type: "move_citizen",
          entityId: event.data.citizen_id,
          fromPosition: hexToWorld(event.data.from_q, event.data.from_r),
          toPosition: hexToWorld(event.data.to_q, event.data.to_r),
          durationMs: 800,
        });
        break;

      case "war.declared.v1":
        this.pendingMutations.push({
          type: "update_border_shader",
          nationA: event.data.aggressor_id,
          nationB: event.data.defender_id,
          conflictState: true,
        });
        break;

      case "climate.event.v1":
        this.pendingMutations.push({
          type: "activate_weather",
          weatherType: event.data.climate_event_type,
          intensity: event.data.intensity,
          affectedCells: event.data.affected_hex_cells,
        });
        break;
    }
  }

  flushMutations(scene: THREE.Scene, registry: SceneEntityRegistry): void {
    for (const mutation of this.pendingMutations) {
      applyMutation(mutation, scene, registry);
    }
    this.pendingMutations = [];
  }
}
```

`flushMutations` is called at the top of each `requestAnimationFrame` callback, before rendering. This ensures scene state is always consistent with the last complete set of simulation events received.

### 13.3 View Layer Purity Contract

The 3D client is a pure view layer. This contract is enforced architecturally:

**FORBIDDEN in 3D client source code:**
- Tick computation or timestep logic
- Resource arithmetic (food, gold, production calculations)
- Pathfinding algorithms
- AI decision logic
- Diplomacy state computation
- Any formula from CIV-0100, CIV-0105, CIV-0106, or CIV-0107

**PERMITTED in 3D client source code:**
- Visual interpolation (smooth citizen movement between event states)
- Camera animation (smooth zoom transitions)
- LOD distance computation (which LOD level to render)
- Particle system simulation (cosmetic FX, not simulation state)
- Asset loading and caching
- Protocol message parsing

If a visual decision requires knowledge of simulation state beyond what is in the received events (e.g., "should this building be on fire?"), the answer is: add a field to the relevant event rather than computing it in the client. The simulation core is the single source of truth.

---

## 14. Performance Benchmarks and Budgets

### 14.1 Scene Initialization

**Definition:** Scene initialization time = elapsed time from receipt of first `world.snapshot.v1` message to the first rendered frame in which all visible buildings (Zoom 2 view) and terrain are present.

**Targets:**

| View | Scene | Target | Maximum |
|------|-------|--------|---------|
| Zoom 2 (12×12 cells) | ~50 buildings | 1.5s | 2.0s |
| Zoom 1 (full map, 64×64) | ~500 buildings (LOD3 only) | 3.0s | 5.0s |
| Zoom 3 (3×3 cells) | ~8 buildings (LOD0) | 0.5s | 1.0s |

**Optimization strategies for initialization:**
1. Asset preloading: critical assets (the 12 Classical era models) are preloaded before the simulation starts.
2. Progressive loading: LOD3 billboards load first (instant, no GLTF needed); LOD0 loads in background.
3. Terrain first: render terrain immediately from heightmap data (no GLTF dependency); buildings stream in after.
4. Instanced building spawning: all buildings of the same type are spawned in a single batch operation.

### 14.2 Memory Budgets

**VRAM budget (GPU memory):**

| Category | Budget | Notes |
|----------|--------|-------|
| Terrain texture atlas (4096×4096 RGBA) | 64 MB | Uncompressed; 16 MB with KTX2 compression |
| Building LOD0 GLTF geometry (72 assets) | 80 MB | ~1.1 MB average per model geometry |
| Building LOD1/LOD2 geometry | 30 MB | Smaller poly count |
| Billboard atlas textures | 20 MB | 72 assets × 9 angles × 512×512 |
| Citizen sprite atlas | 4 MB | 8 variants × 4 frames × 256×256 |
| Shadow maps | 32 MB | 2 cascades × 2048×2048 16-bit |
| Framebuffer / render targets | 32 MB | Nation ID buffer + post-processing |
| Particle system buffers | 8 MB | Vertex buffers for all particle systems |
| Misc (shaders, uniforms, etc.) | 16 MB | Compiled shader programs |
| **Total** | **286 MB** | 44% headroom within 512 MB budget |

**RAM budget (CPU/system memory):**

| Category | Budget |
|----------|--------|
| Asset loading cache (pre-loaded GLBs) | 128 MB |
| Heightmap data (64×64 map, 16-bit) | 8 MB |
| Scene graph overhead | 32 MB |
| WebSocket receive buffer | 8 MB |
| JavaScript heap (Three.js + app) | 80 MB |
| **Total** | **256 MB** | Exactly at budget |

### 14.3 Texture Atlas Specification

**Terrain biome atlas:** 4096×4096 RGBA8. Contains 6 biome base textures tiled within the atlas:

```
┌──────────┬──────────┬──────────┐
│  Plains  │  Forest  │  Desert  │  (1365×1024 per tile, top row)
├──────────┼──────────┼──────────┤
│  Tundra  │ Mountain │  Ocean   │  (1365×1024 per tile, bottom row)
└──────────┴──────────┴──────────┘
```

Each biome tile is 1365×1024 pixels (approximately 1/6 of atlas area, with minor padding). Tiles are seamless (wrap at all edges). Biome blending is performed in the shader using the biome weight texture (§6.3) to sample and lerp between tiles.

**Building billboard atlas:** Per building type, a 2048×2048 atlas contains 9 render captures (8 cardinal azimuths + top-down view):

```
┌──────┬──────┬──────┐
│  N   │  NE  │  E   │  (9 tiles on ~683×683 each — rounded to 512×512 with padding)
├──────┼──────┼──────┤
│  SE  │  S   │  SW  │
├──────┼──────┼──────┤
│  W   │  NW  │ TOP  │
└──────┴──────┴──────┘
```

**Citizen sprite atlas:** 2048×512 RGBA8. Contains 8 occupation types × 4 animation frames = 32 sprites. Each sprite is 256×256 pixels (pixel art upscaled).

### 14.4 Benchmark Measurement Protocol

To ensure consistent and reproducible performance measurements:

**Hardware specification (reference tier — M2 MacBook):**
```
CPU: Apple M2 (8-core CPU, 10-core GPU)
RAM: 16 GB unified memory
Display: 1080p (forced via display scaling or headless render target)
Browser: Chrome 133 stable (headless mode for CI; headed mode for QA)
WebGL: WebGL2 (ANGLE Metal backend on macOS)
```

**Measurement procedure:**
1. Load the 3D client in headless Chrome with `--enable-gpu` flag.
2. Connect to a headless simulation running the standard benchmark scenario (64×64 map, 8 nations, all building types present, rain weather active).
3. Navigate to Zoom 2 view (12×12 visible cells).
4. Record `performance.now()` timestamps at `requestAnimationFrame` entry.
5. Discard first 60 frames (warm-up).
6. Record 300 frames; compute: mean frame time, 95th percentile, 99th percentile, min FPS, max FPS.
7. Pass criteria: 95th percentile frame time ≤ 22.2ms (≥ 45 FPS).

**Automated benchmark runner:** `tools/benchmarks/run_3d_benchmark.ts` (Playwright-based). Outputs JSON report to `logs/benchmark_results/`.

**CI integration:** Benchmark runs on every merge to `main`. Results are compared against the baseline stored in `tools/benchmarks/baseline.json`. A regression of > 10% in 95th-percentile frame time triggers a CI warning (not a block; performance regression requires human investigation).

---

## 15. Appendices

### A. glTF Validator Rules Reference

The following glTF Validator message codes are treated as errors (causing CI failure) in addition to the default error severity messages:

| Code | Message | Reason for Elevation |
|------|---------|---------------------|
| `MESH_PRIMITIVE_NO_POSITION` | Mesh primitive missing POSITION attribute | Unrenderable mesh |
| `ACCESSOR_NON_UNIT` | Normal accessor not normalized | Incorrect lighting |
| `UV_WRAPPING` | UVs outside [0,1] with non-repeat wrap mode | Texture bleeding |
| `MATERIAL_DOUBLE_SIDED_CULL` | Double-sided material with back-face culled | Visual artifact |

The following Validator warning codes are suppressed (counted but not blocking, up to the 5-warning threshold):

| Code | Justification for Suppression |
|------|-------------------------------|
| `MESH_PRIMITIVE_TANGENT_SPACE_INVALID` | Three.js computes tangents at runtime |
| `UNUSED_OBJECT` | glTF Transform prune handles this in optimization step |

### B. Meshy.ai API Reference Stub

At time of spec writing, the Meshy.ai API v2 (Text to 3D) is invoked as follows. This stub is for orchestrator implementation reference; consult official Meshy docs for current endpoint details.

**Create task:**
```http
POST https://api.meshy.ai/v2/text-to-3d
Authorization: Bearer {MESHY_API_KEY}
Content-Type: application/json

{
  "mode": "preview",                    // "preview" for fast draft; "refine" for quality
  "prompt": "...",
  "negative_prompt": "...",
  "art_style": "game-low-poly",
  "should_remesh": true,
  "topology": "quad",
  "target_polycount": 5000,
  "seed": 101742
}

Response 202:
{
  "result": "task_abc123def456",        // task_id
  "status": "PENDING"
}
```

**Poll task status:**
```http
GET https://api.meshy.ai/v2/text-to-3d/{task_id}
Authorization: Bearer {MESHY_API_KEY}

Response 200 (in-progress):
{ "status": "IN_PROGRESS", "progress": 45 }

Response 200 (complete):
{
  "status": "SUCCEEDED",
  "model_urls": { "glb": "https://cdn.meshy.ai/.../{task_id}.glb" },
  "seed": 101742,
  "created_at": 1740103800,
  "finished_at": 1740103920
}
```

**Download .glb:**
```python
import httpx

async def download_glb(task: MeshyTask, output_dir: str) -> str:
    url = task.model_urls["glb"]
    filename = f"{task.task_id}.glb"
    output_path = os.path.join(output_dir, filename)
    async with httpx.AsyncClient() as client:
        response = await client.get(url)
        response.raise_for_status()
        with open(output_path, "wb") as f:
            f.write(response.content)
    return output_path
```

**API limits (standard tier, 2026-02-21):**
- 10 concurrent tasks
- 200 tasks per day
- Task timeout: 10 minutes (typically completes in 2-4 minutes for game-low-poly)
- Cost: ~$0.05 per task (exact pricing: consult Meshy.ai pricing page)

### C. Asset Manifest Schema (3D)

Full JSON Schema for `asset_manifest_3d.json`:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CivLab 3D Asset Manifest",
  "type": "object",
  "required": ["$schema", "version", "generated_at", "assets"],
  "properties": {
    "$schema": { "type": "string" },
    "version": { "type": "string", "enum": ["1"] },
    "generated_at": { "type": "string", "format": "date-time" },
    "total_assets": { "type": "integer", "minimum": 0 },
    "assets": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["asset_id", "building_class", "era", "lod_files",
                     "generation_method", "generation_seed", "quality_gate_result"],
        "properties": {
          "asset_id": {
            "type": "string",
            "pattern": "^building_[a-z_]+_(ancient|classical|medieval|renaissance|industrial|modern|futuristic)_3d(_v[0-9]+)?$"
          },
          "building_class": {
            "type": "string",
            "enum": ["granary","farm","barracks","library","market","temple",
                     "forge","harbor","tower","palace","academy","cathedral"]
          },
          "era": {
            "type": "string",
            "enum": ["ancient","classical","medieval","renaissance","industrial","modern","futuristic"]
          },
          "lod_files": {
            "type": "object",
            "required": ["lod0", "lod1", "lod2", "lod3"],
            "properties": {
              "lod0": { "type": "string" },
              "lod1": { "type": "string" },
              "lod2": { "type": "string" },
              "lod3": { "type": "string" }
            }
          },
          "generation_method": {
            "type": "string",
            "enum": ["meshy_api", "instantmesh", "blender_artist"]
          },
          "generation_seed": { "type": "integer", "minimum": 1 },
          "meshy_task_id": { "type": "string" },
          "meshy_api_version": { "type": "string" },
          "quality_gate_result": {
            "type": "object",
            "required": ["passed", "checked_at"],
            "properties": {
              "passed": { "type": "boolean" },
              "checked_at": { "type": "string", "format": "date-time" },
              "gltf_validator_errors": { "type": "integer" },
              "gltf_validator_warnings": { "type": "integer" },
              "triangle_count_lod0": { "type": "integer" },
              "triangle_count_lod1": { "type": "integer" },
              "triangle_count_lod2": { "type": "integer" },
              "uv_overlap_pct": { "type": "number" },
              "file_size_bytes_lod0": { "type": "integer" }
            }
          },
          "deprecated": { "type": "boolean", "default": false }
        }
      }
    }
  }
}
```

### D. Biome Texture Palette

Reference color palettes used when sourcing or commissioning biome textures. All textures must be tileable (seamless). Recommended texture resolution: 1024×1024 source; downsampled to 512×512 in atlas.

| Biome | Primary Color | Secondary Color | Texture Characteristics |
|-------|--------------|----------------|------------------------|
| Plains | #7CBA5A (grass green) | #C8B878 (dry grass) | Grass blade pattern; seasonal variation via shader |
| Forest | #2D6A3F (deep green) | #5C4A2A (bark brown) | Leaf litter ground; moss accents |
| Desert | #D4AA70 (sand) | #C87941 (iron-rich sand) | Fine grain pattern; dune ripple normals |
| Tundra | #E8E8F0 (snow white) | #7A9A9A (ice grey) | Snow compression pattern; frost crackle |
| Mountain | #8A8A8A (granite grey) | #F0F0F0 (snow cap) | Rock face fracture lines; erosion streaks |
| Ocean | #1A4A7A (deep blue) | #2E7DB5 (shallow blue) | Animated via shader (not atlas); placeholder color only |

Ocean is not a static texture — it uses an animated `ShaderMaterial` (wave normal animation) and does not require a slot in the texture atlas. The atlas allocates its 6th tile slot for a placeholder flat-color quad that is never sampled in production; this slot is reserved for future use (e.g., lava biome in futuristic era maps).

---

*End of CIV-0601: 3D Asset Transition and Agentic Generation Pipeline*

*Document version 1.0 — 2026-02-21 — CIV Engine, Art Pipeline, and Infrastructure Teams*
