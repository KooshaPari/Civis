# Plan: Core Simulation Engine (civ-001)

## Phased WBS

### Phase 1: Harden existing tick loop (P0.1–P0.6)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| P0.3 | Replay harness in `determinism_proptest.rs` with full `ReplayLog` coverage | P0.2 | Partial |
| P0.4 | .civreplay save/load restores tick and voxel chunk count | P0.3 | Partial |
| P0.6 | Audit remaining crates for `ChaCha8Rng` seeding | P0.5 | Partial |

### Phase 2: ECS and entity model (E1.2)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| E1.2a | Stabilize entity ID serialization contract | P0.4 | Planned |
| E1.2b | O(n) component query benchmark in CI | E1.2a | Planned |

### Phase 3: Policy and stochastic phases (E1.3, E1.5)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| E1.3 | Implement policy evaluation phase (pure function, scenario-overridable) | E1.2b | Planned |
| E1.5 | Implement stochastic event phase (ChaCha20Rng, logged draws) | E1.3 | Planned |

### Phase 4: Multi-client command queue (E1.7)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| E1.7 | Priority queue; admin > player > research; cutoff deferral | E1.3 | Planned |

### Phase 5: Performance and hardening (E1.10)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| E1.10 | Tick budget CI gate; profiling breakdown on > 16 ms | E1.7 | Planned |

## DAG Dependencies

```
P0.3 → P0.4 → P0.6
P0.4 → E1.2a → E1.2b → E1.3 → E1.5
E1.3 → E1.7 → E1.10
```
