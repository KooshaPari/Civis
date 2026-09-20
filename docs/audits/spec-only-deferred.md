# Spec-Only Deferred — P2 agent-D slice

> Source: `docs/audits/P2-impl-slice/P2-agent-D.md` (25 IDs across 7 epics).
> Generated: 2026-09-19.

The agent-D slice mixes FRs whose spec_refs point to UI/UX documents (Pixi.js /
Three.js / Babylon dashboards), Python pipeline scripts, and ops/build artifacts.
The decision per ID below records whether a minimal Rust source was written or
whether the ID was deferred with a concrete reason.

## Implemented (4 IDs)

| ID | Crate / File | Function |
|----|--------------|----------|
| `FR-CIV-LIFE-004` | `crates/needs/src/lib.rs` | `share_food_between(donor, receiver, donor_threshold, share_rate)` — Society → Lifecycle food-share per `docs/design/civ-003-emergent-lifecycle.md:101` |
| `FR-CIV-SOCIAL-001-INSTITUTIONS` | `crates/civ-institutions/src/policy.rs` | `InstitutionPolicy { members, policies, budget_bp, approval_rating_fp }` + `add_member` / `remove_member` / `update_policy` |
| `FR-CIV-ASSET-MANI-001` | `crates/asset-pipeline/src/manifest.rs` | `check_manifest_completeness(manifest_path, atlas_paths)` — manifest vs. atlas count parity |
| `FR-CIV-ASSET-MANI-002` | `crates/asset-pipeline/src/manifest.rs` | `check_manifest_schema(manifest_path)` — required-key presence |

## Deferred (21 IDs)

### Epic `FR-CIV-GEO` — 10 IDs (deferred)

The spec_refs for every FR-CIV-GEO-00x ID point to lines 2024-2032 of
`docs/specs/CIV-0300-rts-ui-ux-spec.md`, which describe Pixi.js / Three.js
client-side rendering (terrain tiles, biome overlays, district drill-down,
A* path preview overlays, choropleth economy overlay, social migration arrows,
disaster zone shading, LOD switching). The data model these features consume
already lives in `crates/planet/src/geology.rs` (`BiomeKind`, `GeologyMap`,
`classify_biome`, `ClimateDrift`) and is consumed by `crates/engine/src/`.
Adding Rust functions for "render the climate overlay" would not satisfy the
spec — these are rendering-side concerns.

- `FR-CIV-GEO-001` (Terrain Types & Properties) — UI rendering; data layer covered by `crates/planet/src/geology.rs`.
- `FR-CIV-GEO-002` (Map Generation & Biome Systems) — UI rendering + biome territory blocs in Pixi.js.
- `FR-CIV-GEO-003` (District & Region Subdivision) — district panel + breadcrumb UI.
- `FR-CIV-GEO-004` (Neighbor Queries & Pathfinding) — A* path preview overlay; engine-side pathfinding not yet wired.
- `FR-CIV-GEO-005` (Resource Distribution & Renewal) — resource bar UI + delta indicators.
- `FR-CIV-GEO-006` (Climate Events & Modulation) — climate overlay + alert feed.
- `FR-CIV-GEO-007` (District Connectivity & Trade Routes) — economy overlay arrows.
- `FR-CIV-GEO-008` (Population Density & Urban Growth) — population density UI + social overlay arrows.
- `FR-CIV-GEO-009` (Disaster Zones & Recovery) — disaster zone shading + recovery progress.
- `FR-CIV-GEO-010` (LOD Rendering Contract) — Zoom-1/Zoom-2 LOD data schema swap in Pixi.js.

### Epic `FR-CIV-WEB` — 4 IDs (deferred)

All four FRs are TypeScript code that already lives in `web/dashboard/src/`
and `web/src/`:

- `FR-CIV-WEB-000` — Dashboard build configuration; the dashboard lives in
  TypeScript (`web/dashboard/`). `vite build` already covers this; adding a
  Rust stub would not exercise the actual build.
- `FR-CIV-WEB-001` — WS URL resolution from `CIVIS_WS_URL` / `CIVIS_WS_ADDR`.
  Implemented in TypeScript at `web/dashboard/src/lib/civisSocket.ts`. A
  Rust mirror would be dead code.
- `FR-CIV-WEB-004` — Operator RPC controls (`sim.command`, `sim.set_speed`,
  `sim.set_policy`, `sim.reset`). Implemented in TypeScript at
  `web/dashboard/src/lib/civisServer.ts`.
- `FR-CIV-WEB-005` — Replay save/load round-trip. Implemented in TypeScript
  at `web/dashboard/src/lib/civisServer.ts` and tested via
  `web/tests/civRpc.test.mjs`.

### Epic `NFR-CIV-DEV-HYGIENE` — 1 ID (deferred)

- `NFR-CIV-DEV-HYGIENE-001` — `docs/ops/history-purge-plan.md` is a one-shot
  ops procedure (git-filter-repo purge of `target-check-*` artifact trees from
  history). It is not source code and there is no Rust implementation to write.

### Epic `NFR-CIV-SCALE` — 6 IDs (deferred)

These NFRs describe 20mi world-extent, LOD-tiered agent simulation, on-disk
chunk formats, and streaming-window determinism. The supporting crate
(`crates/voxel/`) already carries the streaming substrate (see the W5 stories
in `docs/agileplus/epics/civ-w5-scale.md`) and the FR-level scaling gap is
already covered by `NFR-CIV-SCALE-001..008`. Adding Rust stubs for the
NFR-level "900/902/910/920" cluster would duplicate the FR work.

- `NFR-CIV-SCALE-003` — 10-client connection bound. Pure load-test target;
  requires a 10-client harness not in scope.
- `NFR-CIV-SCALE-004` — Streaming/LOD determinism. Layered on FR-level
  voxel substrate (already shipped).
- `NFR-CIV-SCALE-900` — 20mi world extent. Substrate-driven; no standalone
  function to tag.
- `NFR-CIV-SCALE-902` — Compact on-disk chunk + LOD format. Substrate-driven.
- `NFR-CIV-SCALE-910` — LOD-tiered agent simulation. Substrate-driven.
- `NFR-CIV-SCALE-920` — Determinism across LOD/streaming. Substrate-driven.

## Summary

- IDs in slice: 25
- Implemented (source written + FR tag): 4
- Deferred: 21
- Deleted: 0
- Failed: 0
