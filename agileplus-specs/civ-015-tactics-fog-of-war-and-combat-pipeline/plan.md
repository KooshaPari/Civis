# Plan: Tactics, Fog-of-War & Combat Pipeline (civ-015)

## Phased WBS

### Phase 1: Pipeline invariants (E4.1–E4.4)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| W1.1 | Add a single integration test that runs LOS → formation → war bridge → fog → combat → replay for a fixed seed and asserts the combat log hash | — | Planned |
| W1.2 | Promote the per-stage tests to a `civ-tactics::pipeline` test module | W1.1 | Planned |

### Phase 2: Fog-of-war contract (E4.3, E4.7)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| W2.1 | Document the fog contract in `crates/tactics/fog_of_war.rs` rustdoc (visibility = f(unit, vision, LOS)) | — | Planned |
| W2.2 | Add a `fog_observer_filter` test in `crates/server` that asserts an observer with `fog_nation_id=N` never receives hidden state | W2.1 | Planned |
| W2.3 | Add a `dashboard_fog_overlay` test in `web/dashboard` that asserts the overlay only consumes `sim.snapshot` (no private channel) | W2.1 | Planned |

### Phase 3: PR #310 dashboard surface (E4.6)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| W3.1 | Document the tactics panel contract: state shape, event surface, jump-to-engagement action | W1.1 | Planned |
| W3.2 | Add a `tactics_panel_integration` test | W3.1 | Planned |

## DAG Dependencies

```
W1.1 → W1.2
W1.1 → W3.1 → W3.2
W2.1 → W2.2, W2.3
```
