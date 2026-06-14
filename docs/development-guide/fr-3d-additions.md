# Functional Requirements — 3D Extension

**Status:** PROPOSED (additive to `FUNCTIONAL_REQUIREMENTS.md`)
**Date:** 2026-05-22

> Each FR below corresponds to a phase in `docs/roadmap/plan-3d-phases.md`.
> Stubs at `FR-CIV-<NS>-000` exist in every new crate's `lib.rs` and assert that
> the crate compiles with `SCHEMA_VERSION = 0`. Real FRs are filled in by the
> phase PRs that implement them.

---

## FR-CIV-VOXEL (P-V1, crate `civ-voxel`)

- **FR-CIV-VOXEL-000** — Crate compiles and exposes a `SCHEMA_VERSION` constant (stub).
- **FR-CIV-VOXEL-001** — Adaptive storage: writes within a 16³ leaf are O(1); writes
  outside instantiate octree branches deterministically.
- **FR-CIV-VOXEL-002** — Deterministic dirty queue: replay of N writes yields identical
  `DirtyChunkEvent` sequence ordered by `(chunk_id, write_seq)`.
- **FR-CIV-VOXEL-003** — Fixed-point world coords: no `f32` / `f64` cross the crate's
  public API.
- **FR-CIV-VOXEL-004** — `VoxelScaleMultiplier` invariant: LOD selector composes
  consistently across scale multipliers (WSM3D-lineage regression coverage).
- **FR-CIV-VOXEL-010** — `Mesher` trait: at least the Bevy implementation produces
  watertight meshes for a fixed test scene.

## FR-CIV-BUILD (P-V2, crate `civ-build`)

- **FR-CIV-BUILD-000** — Stub.
- **FR-CIV-BUILD-001** — `BuildingGraph` schema round-trips RON without loss.
- **FR-CIV-BUILD-010** — Autonomous demand-driven allocation: given a town with N
  demand signals, parcel scoring is deterministic and reproducible.
- **FR-CIV-BUILD-020** — Freehand authoring tools (paint / extrude / loft / mirror /
  radial / copy-rotate) emit the same `BuildingGraph` modifications as their
  procedural-grammar equivalents.
- **FR-CIV-BUILD-030** — Era-grammar transitions: a town advanced 100 ticks across an
  era boundary emits the expected facade-type histogram.

## FR-CIV-GENETICS (P-G1, crate `civ-genetics`)

- **FR-CIV-GENETICS-000** — Stub.
- **FR-CIV-GENETICS-001** — Mutation is deterministic under fixed seed.
- **FR-CIV-GENETICS-002** — Recombination of two parents yields an offspring DNA whose
  bytes are drawn deterministically from parental loci.
- **FR-CIV-GENETICS-010** — Speciation trigger: configurable threshold; firing is
  deterministic; emits a new species record without mutating the parent species.

## FR-CIV-SPECIES (P-G1, crate `civ-species`)

- **FR-CIV-SPECIES-000** — Stub.
- **FR-CIV-SPECIES-001** — Deterministic DNA → phenotype mapping: identical DNA →
  identical morphology + behavior weights.

## FR-CIV-AGENTS (P-A1, crate `civ-agents`)

- **FR-CIV-AGENTS-000** — Stub.
- **FR-CIV-AGENTS-001** — Per-civilian wardrobe + tools state ticks deterministically.
- **FR-CIV-AGENTS-010** — LOD tick: distant agents tick at lower frequency with no
  state divergence from full-fidelity ticks (gestalt mode).

## FR-CIV-DIFFUSION (P-A1, crate `civ-diffusion`)

- **FR-CIV-DIFFUSION-000** — Stub.
- **FR-CIV-DIFFUSION-001** — Bass/Rogers S-curve adoption: given fixed parameters,
  adoption fractions match the closed-form solution within tolerance.

## FR-CIV-LAWS (P-L1, crate `civ-laws`)

- **FR-CIV-LAWS-000** — Stub.
- **FR-CIV-LAWS-001** — Versioned RON schema loads + round-trips.
- **FR-CIV-LAWS-002** — Validator rejects fictional extensions that omit any of
  `{inputs, outputs, losses, dependencies}`.

## FR-CIV-RESEARCH (P-R1, crate `civ-research`)

- **FR-CIV-RESEARCH-000** — Stub.
- **FR-CIV-RESEARCH-001** — LLM cache hit short-circuits live calls; output is
  byte-identical to the cached value.
- **FR-CIV-RESEARCH-002** — Canonical-mode replay refuses to advance on the first
  `LlmEvent` encountered in the log.
- **FR-CIV-RESEARCH-003** — Hybrid-mode replay on cache miss refuses to advance.
- **FR-CIV-RESEARCH-004** — `LlmEvent::cache_key` is deterministic and byte-composed
  of `(prompt_hash, input_snapshot_hash, model_id, model_version)`.

## FR-CIV-TACTICS (P-W1, crate `civ-tactics`)

- **FR-CIV-TACTICS-000** — Stub.
- **FR-CIV-TACTICS-001** — Voxel-destructible damage application is deterministic.
- **FR-CIV-TACTICS-010** — Doctrine evolution: GA over compositions converges to a
  reproducible solution under fixed seed.
- **FR-CIV-TACTICS-020** — Voxel line-of-sight: solid materials block grid segments.
- **FR-CIV-TACTICS-021** — Unit formation offsets (line / wedge / square).
- **FR-CIV-TACTICS-022** — War bridge: military grid engagements queue voxel
  `DamageEvent`s on a fixed cadence when LOS is clear.

## FR-CIV-PLANET (P-P1, crate `civ-planet`)

- **FR-CIV-PLANET-000** — Stub.
- **FR-CIV-PLANET-001** — Day/night cycle is deterministic and tied to tick.
- **FR-CIV-PLANET-002** — Moon tides modulate coastal water level deterministically.

## FR-CIV-PROTO3D (crate `civ-protocol-3d`)

- **FR-CIV-PROTO3D-000** — Stub.
- **FR-CIV-PROTO3D-001** — Voxel delta frames serialize to binary; round-trip is lossless.
- **FR-CIV-PROTO3D-002** — Building diff frames carry tagged provenance (procedural vs
  freehand) without loss.
- **FR-CIV-PROTO3D-014** — All six `Frame3d` variants (`VoxelDelta`, `BuildingDiff`,
  `AgentAppearance`, `CivilianState`, `FactionState`, `EventFeed`) round-trip through
  the F3D0 binary envelope losslessly.

## FR-CIV-UX (P-U1, `clients/godot-ref`)

- **FR-CIV-UX-000** — Spawn API exposed via protocol; spawning N civilians from the UI
  emits N corresponding entity-create events.
- **FR-CIV-UX-001** — Era timelapse view advances ticks at configurable rates without
  state divergence vs real-time playback.

---

## Coverage policy

Each phase PR adds at least one passing FR test per listed FR ID. The existing
`fr-coverage.yml` workflow enforces traceability per Civis governance.
