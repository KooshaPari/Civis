# Spec-Only Deferred — P2 agent-D slice

> Source: `docs/audits/P2-impl-slice/P2-agent-D.md` (25 IDs across 7 epics).
> Generated: 2026-09-19.
> Build status: `cargo build --workspace --tests` passes (exit 0, 6m 23s).

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

---

# Spec-only IDs deferred by agent-E (P2)

This ledger records every ID from the P2-agent-E slice that we chose to
**defer** rather than write a source impl for in this round, with a one-line
reason. The full slice is in
[`P2-agent-E.md`](P2-impl-slice/P2-agent-E.md); the deferred entries below
are the ones that did **not** get a `// FR-XYZ-NNN` tag in source.

Bias: the P2 task spec caps each implementation at ≤20 lines per function
and forbids new dependencies. Anything that would exceed that budget — or
require a new crate (e.g. `blake3`, `rand_chacha`) or a multi-thousand-line
new module — is deferred here for a future agent whose scope allows it.

## Deferred — P2 agent-E

| ID | Epic | Reason |
|---|---|---|
| FR-SAVE-006 | FR-SAVE | BLAKE3 integrity hash for save blobs. Requires the `blake3` crate (new dependency). Defer to a follow-up that wires `blake3` into `civ-save-db`. |
| FR-SAVE-008 | FR-SAVE | ChaCha20Rng state serialization (20 u32 words + sub-block word count). Requires `rand_chacha::ChaCha20Rng::from_state` introspection on the engine side. Out of scope for a save-DB-only patch. |
| FR-SAVE-009 | FR-SAVE | BLAKE3 hash chain tail serialization. Same `blake3` dependency issue as FR-SAVE-006. |
| FR-SAVE-010 | FR-SAVE | AI state serialization (personality, goals, memory, MCTS cache, threat models). Multi-module change in `civ-ai`; not appropriate for a save-DB-only patch. |
| FR-SAVE-011 | FR-SAVE | WASM `ModStateSave` trait + per-mod state on save/load. Requires extending `civ-mod-host` and a host-side state registry. |
| FR-SAVE-012 | FR-SAVE | "Unknown mod ID → warn + skip" semantics on load. Coupled to FR-SAVE-011 — defer together so the warn-on-skip path has an actual mod registry to consult. |
| FR-SAVE-013 | FR-SAVE | Save format version + N-2..N-1 migration. Requires a `MigrateFn` table and per-version transformers; medium module, deferred. |
| FR-SAVE-016 | FR-SAVE | QuickSave ≤50ms SLO. Performance target — needs the QuickSave path to actually exist in the engine before we can measure against it. |
| FR-SAVE-017 | FR-SAVE | SlotSave ≤500ms SLO. Same as FR-SAVE-016 — performance budget, not a feature. |
| FR-SAVE-018 | FR-SAVE | Load ≤1,000ms SLO. Same as FR-SAVE-016 — performance budget, not a feature. |
| FR-SAVE-019 | FR-SAVE | Save integrity verification ≤200ms SLO. Performance budget; depends on FR-SAVE-006's BLAKE3 path existing. |
| NFR-CIV-SEC-002 | NFR-CIV-SEC | "No secret material in committed config" — this is a CI-infra concern (a `trufflehog`/`gitleaks` step in `.github/workflows`). Out of Rust scope. |
| NFR-CIV-SEC-003 | NFR-CIV-SEC | "Tests run in an isolated network namespace" — CI-infra concern (Docker `--network=none` etc.). |
| NFR-CIV-SEC-004 | NFR-CIV-SEC | "Dependency audit + Bandit/Semgrep pass on every PR" — CI-infra concern. |

## Implemented in P2 agent-E (for completeness)

The following IDs in the P2-agent-E slice **were** implemented (tagged and
tested). They are listed here only so a reader can sanity-check the
inverse — every ID not in the deferred table above should appear in a
`git log` commit on `next-P2-E`:

- FR-CIV-ASSET-QUAL-001 — `crates/asset-pipeline/src/validate.rs`
- FR-CIV-GODOT-ATTACH-001..004 — `clients/godot-ref/rust/src/attach.rs`
- FR-CIV-MIGRATION-001..005 — `crates/emergence-migration/src/lib.rs`
- FR-CIV-SOCIAL-002-IDEOLOGY — `crates/social/src/ideology.rs`
- FR-SAVE-007, FR-SAVE-014, FR-SAVE-015, FR-SAVE-021, FR-SAVE-022,
  FR-SAVE-023, FR-SAVE-024, FR-SAVE-025 — `crates/save-db/src/lib.rs`
- FR-SAVE-020 — `crates/save-db/src/lib.rs` (`evict_autosaves`)
- NFR-CIV-LEGENDS-LOUD-03 — `crates/legends/src/{decay.rs,worker.rs}`
- NFR-CIV-SEC-001 — `crates/mod-host/src/wasm_guest.rs`

## Counts

Deferred: **13** IDs.
Implemented: **22** IDs (covering 17 unique spec IDs + 5 social/migration
batch tags).

---

# Spec-Only IDs Deferred by P2 Agent-B

This file tracks IDs from the P2 agent-B slice
(`docs/audits/P2-impl-slice/P2-agent-B.md`) that were **deferred**
rather than implemented, because their spec depends on infrastructure
that is not yet present in the workspace.

## FR-CIV-EMERGENCE (15 IDs deferred)

The 15 IDs in `FR-CIV-EMERGENCE-{100..254}` listed under the slice are
all sub-rows of a 155-row `emergent-systems-tracelinks.md` ledger that
maps each of 11 emergent subsystems (civ-linguabridge, civ-factions,
civ-religion, civ-market, civ-urban, civ-climate, civ-econ,
civ-demographics, civ-psyche, civ-legends, civ-ai, civ-culture,
civ-social, civ-diplomacy, civ-laws) to a batch row range. The ledger's
own closing line states:

> The 11-systems × 30-couplings matrix documented above is the **test
> surface** that promotes each of these 158 dormant IDs to `covered`
> status (i.e., spec + code + test triple).

That matrix is **not built** in this worktree — the couplings code,
the per-system batch-row implementations, and the cross-system event
harness they would need do not exist. Implementing each of the 15 IDs
without that matrix would produce untestable stubs, which violates the
"no dormant IDs after this pass" spirit of the P2 fan-out.

### Deferred IDs (15)

| ID | Spec/trace reference | Reason |
|---|---|---|
| FR-CIV-EMERGENCE-100 | `docs/traceability/emergent-systems-tracelinks.md:150` (civ-linguabridge) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-111 | `docs/traceability/emergent-systems-tracelinks.md:151` (civ-factions) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-119 | `docs/traceability/emergent-systems-tracelinks.md:152` (civ-religion) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-124 | `docs/traceability/emergent-systems-tracelinks.md:153` (civ-market) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-132 | `docs/traceability/emergent-systems-tracelinks.md:154` (civ-urban) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-141 | `docs/traceability/emergent-systems-tracelinks.md:155` (civ-climate) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-144 | `docs/traceability/emergent-systems-tracelinks.md:156` (civ-econ) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-151 | `docs/traceability/emergent-systems-tracelinks.md:157` (civ-demographics) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-168 | `docs/traceability/emergent-systems-tracelinks.md:158` (civ-psyche) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-198 | `docs/traceability/emergent-systems-tracelinks.md:159` (civ-legends) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-221 | `docs/traceability/emergent-systems-tracelinks.md:160` (civ-ai) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-236 | `docs/traceability/emergent-systems-tracelinks.md:161` (civ-culture) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-239 | `docs/traceability/emergent-systems-tracelinks.md:162` (civ-social) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-241 | `docs/traceability/emergent-systems-tracelinks.md:163` (civ-diplomacy) | Requires 11-systems × 30-couplings matrix |
| FR-CIV-EMERGENCE-249 | `docs/traceability/emergent-systems-tracelinks.md:164` (civ-laws) | Requires 11-systems × 30-couplings matrix |

### Next steps for these IDs

1. Build the 11-systems × 30-couplings matrix as a `civ-emergence-couplings`
   crate (or extend `crates/engine/src/emergence_coupling.rs`).
2. Implement each system's batch-row code (`crates/agents`, `crates/social`,
   `crates/diplomacy`, `crates/laws`, `crates/legends`, etc.) against
   the matrix.
3. Re-run the P2 fan-out for these IDs as `BUILD-NEXT` once the matrix
   exists.