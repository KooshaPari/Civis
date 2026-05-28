# Changelog

All notable changes to **civ** are documented here.

## [Unreleased]

### Added
- Hash chain implementation: BLAKE3 tick-hash emission (`FR-CORE-005`) and append-only
  chain construction (`FR-CORE-006`) in `crates/engine/src/hash_chain.rs`
- `civ-tactics` crate scaffolded for P-W1 tactical warfare (voxel-destructible per-soldier
  combat, doctrine evolution genetic-algo, Phase 4 war system integration seams)
- `phenotype-voxel` kernel shipped as a new shared Phenotype-org repo (P-V0): SVO + dense
  leaf chunks, deterministic dirty queue, `Mesher` trait, Bevy reference adapter
- Voxel foundation (P-V1): adaptive substrate wired into engine tick; `civ-protocol-3d`
  carrying voxel deltas; all three reference clients render empty terrain at 60 FPS
- Quality manifest at `.ci/quality-manifest.json` tracking per-crate gate thresholds
- Bevy client atmosphere, minimap, live-scene, and ground-material modules
- Dev-parity CI workflow (`.github/workflows/dev-parity.yml`)
- Developer-launch script (`scripts/dev-launch.mjs`) and process-compose helpers

### Changed
- STATUS.md: removed stale billing-blocked note; updated to local-first-CI + quality-manifest
- docs/WORKLOG.md: marked P-V1, P-W1 complete; added completed phases table
- docs/traceability/TRACEABILITY_MATRIX.md: FR-CORE-005 and FR-CORE-006 promoted to `implemented`
- docs/roadmap/plan-3d-phases.md: added completion status column

---

*Last updated: 2026-05-28*
