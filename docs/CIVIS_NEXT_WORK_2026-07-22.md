# Civis next worklist

Discovered from the active branch on 2026-07-22. This is a planning artifact; no implementation or merge is implied.

## P0 — close security-policy ambiguity

`Cargo.lock` contains `quick-xml 0.39.4` through `wayland-scanner 0.31.10`, but `cargo deny check advisories` reports both `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` as “advisory not detected.” Remove or re-justify those unmatched exceptions. The same gate currently reports yanked `num-bigint 0.4.7` via `bollard -> testcontainers` and yanked `spin 0.9.8` via `hecs`; resolve those lockfile entries next. Acceptance: `cargo deny check advisories` passes without unmatched-exception warnings or yanked-crate warnings.

## P0 — make CI security gates real — verified complete

The current tree has real checks: `.github/workflows/security-guard-hook-audit.yml` verifies hook wiring and runs `cargo deny check advisories`; `.ci/actions/coverage-check/action.yml` uses tarpaulin or llvm-cov and exits 1 when neither tool exists. Remaining work is only to add fixture-based failure tests if stronger gate evidence is required.

## P1 — remove compile-preserving engine stubs — history slice complete

`crates/engine/src/engine.rs` still contains cleanup-surgeon TODOs, forward-declared placeholder types/functions, and no-op/stub implementations for several systems. The `history.rs` slice is now restored from commit `37f31e998` with bounded transitions, chronicle accessors, and serde coverage: `cargo test -p civ-engine history --lib` passed 3 tests. Remaining engine stub slices still need separate owners/tests.

## P1 — complete voxel-bridge substrate

`crates/voxel-bridge/src/` contains TODO stub modules for world generation, streaming/window eviction, LOD, scale budget, PBR materials, fluids, reactions, boundaries, and HUD. Acceptance: define the minimum public contract for each module, implement one vertical path from worldgen through streaming/render metadata, and add invariant tests for eviction, LOD selection, and material completeness.

Package activation slice completed: added `civ-voxel-bridge` to the workspace,
provided its `civ-voxel` type seam, implemented deduplicated dirty-chunk
queueing, and added the first ring-distance/prefetch contracts. Validation:
`cargo test -p civ-voxel-bridge` passed (1 test). Remaining modules are still
explicit stubs and need separate vertical slices.

### Engine CA prerequisite (audited 2026-07-22)

`CaGrid` and deterministic `AbiogenesisSuitability` already exist in
`crates/voxel`, but `Simulation` has no resident CA-grid, abiogenesis-site
storage, or world-to-grid sampling contract. The two `phase_voxel_ca` tests
remain intentionally ignored until that schema is specified; no historical
implementation exists in reachable Git refs. Next prerequisite: define
resident-window ownership and coordinate mapping, then wire
`last_tick_abiogenesis_sites()`.

## P1 — replace AI provider stubs

`crates/ai/src/providers/` documents `LocalSlmProvider`, `EmbedProvider`, and `OllamaDevProvider` as stubs; `ollama_dev.rs`, `local_slm.rs`, `embed.rs`, and Firepass/Kimi paths still return pending/unavailable behavior. Acceptance: capability discovery reports unavailable providers accurately, and at least one provider has a real integration test behind its feature flag.

## P2 — finish economy/build semantics

`crates/economy` still labels market, institution, ledger, and allocation paths as stubs; `crates/build` exposes a `0.1.0-stub` schema and has a TODO for per-good stock. Acceptance: conservation and clearing behavior are tested with multi-good, multi-institution scenarios and schema versions no longer claim production semantics prematurely.

## P2 — replace placeholder coverage tests

Several engine coverage tests are intentionally compile-only placeholders (`uncovered_coverage.rs`, `n5_n6_n8_coverage.rs`, `invariants_proptest.rs`). Acceptance: each placeholder is replaced by an executable invariant/property test or explicitly removed from the coverage denominator with a traceability explanation.

## Recommended order

```text
security graph/policy
        |
        v
CI fail-closed gates
        |
        +--> engine stubs / hash-chain / history
        +--> voxel-bridge vertical path
        +--> AI provider integration
        +--> economy/build semantics
```
