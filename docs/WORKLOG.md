# Worklog

Active work tracking for **civ** project.

---

## Current Sprint

| Item | Status |
|------|--------|
| Documentation sync (stale docs audit) | COMPLETE |
| Bevy client item-027 (live scene / atmosphere / minimap) | IN PROGRESS |

---

## Completed Phases

| Phase | Description | Completed |
|-------|-------------|-----------|
| P-V0 | phenotype-voxel kernel (new shared Phenotype-org repo: SVO + dense leaf storage, deterministic dirty queue, Mesher trait, Bevy adapter) | 2026-05 |
| P-V1 | Voxel foundation — adaptive substrate wired into engine tick; protocol carries voxel deltas; three reference clients render empty terrain at 60 FPS | 2026-05 |
| P-W1 | Tactical warfare stub — `civ-tactics` crate scaffolded; voxel-destructible per-soldier combat architecture in place; integrates Phase 4 war system seams | 2026-05 |
| FR-CORE-005/006 | Hash chain implementation — BLAKE3 tick-hash emission + append-only chain in `crates/engine/src/hash_chain.rs` | 2026-05 |

---

## Backlog

See `docs/roadmap/plan-3d-phases.md` for the full phase DAG and remaining work.
See `PLAN.md` for the foundation Phases 0–6 backlog.

---

*Last updated: 2026-05-28*
