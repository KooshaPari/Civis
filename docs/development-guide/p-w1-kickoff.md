# P-W1 tactical warfare — kickoff

**Phase:** P-W1 (`crates/tactics`)
**Depends on:** P-V1 (voxel), P-A1 (agents)
**Branch suggestion:** `feat/p-w1-tactics` off `main` after #296 merges

## Already wired

| Link | Location |
|------|----------|
| Voxel damage | `civ_tactics::apply_damage` used in `crates/engine/src/engine.rs` tick + `apply_damage_now` |
| Replay | `DamageEvent` in `crates/engine/src/replay.rs` |
| Authoring | `sim.damage` (server), `POST /control/damage` (watch), web/Godot damage tool |
| Doctrine GA | `evolve_doctrine` + tests `FR-CIV-TACTICS-010/011` |

## FR status (`docs/traceability/fr-3d-matrix.md`)

| FR ID | Status | Next step |
|-------|--------|-----------|
| FR-CIV-TACTICS-000 | implemented | — |
| FR-CIV-TACTICS-001 | implemented | Per-soldier damage events (not only sphere carve) |
| FR-CIV-TACTICS-010 | implemented | — |
| FR-CIV-TACTICS-002+ | planned | Line-of-sight, unit formations, Phase 4 war bridge |

## First PR slice (recommended)

1. **Test:** `engine::tick` with queued `DamageEvent` reduces voxel count — **done** (`pending_damage_drains_and_reduces_chunk_count` in `civ-engine`).
2. **Server:** `sim.snapshot` exposes `damage_events`, `damage_events_count`, `voxel_damage_removed_this_tick` — **done** (`feat/p-w1-tactics`).
3. **Web:** damage bursts + combat notifications — **done** (`scene3d.tsx`, `main.tsx`).
4. **Watch:** per-unit `unit_a` / `unit_b` on military `DamagePulse` — **done**.
5. **Doctrine GA:** `evolve_doctrine` every 64 ticks per faction in `phase_tactics` — **done**.
6. **Next:** FR-CIV-TACTICS-002+ (LOS, formations, war bridge).

## Run

```bash
cargo test -p civ-tactics
cargo test -p civ-engine pending_damage
just civis-3d-verify
```
