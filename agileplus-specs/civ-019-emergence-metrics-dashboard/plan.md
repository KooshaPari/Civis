# Plan: Emergence Metrics Dashboard (civ-019)

Surface wave-1 emergence outputs (psyche, social graph, ideology spread,
species, culture, insurgency) on a single dashboard panel.

## Phased WBS

### Phase 1: Metrics catalogue (E5.1)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| M1.1 | Add `crates/engine/src/emergence_metrics.rs` with: `ClusterEntropy`, `IdeologyHomophilyIndex`, `SentienceFraction`, `PsycheStability`, `DiplomacyTensionIndex` | — | Planned |
| M1.2 | Wire into `Simulation::tick` (read-only) at the end of the diffusion phase | M1.1 | Planned |
| M1.3 | Expose on `sim.snapshot.emergence` JSON-RPC payload | M1.2 | Planned |

### Phase 2: Dashboard panel (E5.2)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| M2.1 | `web/dashboard/emergence_panel.tsx` reads `sim.snapshot.emergence` only | M1.3 | Planned |
| M2.2 | Per-metric sparkline (last 120 ticks) and threshold-color chip | M2.1 | Planned |
| M2.3 | Tests: snapshot shape, no private channels, click-to-focus on tick | M2.2 | Planned |

### Phase 3: Bevy overlay (E5.3)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| M3.1 | `live_emergence_overlay` HUD in `clients/bevy-ref` | M1.3 | Planned |
| M3.2 | Toggle keybind (E) + glassmorphism chip group | M3.1 | Planned |

## DAG Dependencies

```
M1.1 → M1.2 → M1.3 → M2.1 → M2.2 → M2.3
                    └─→ M3.1 → M3.2
```
