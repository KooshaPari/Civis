# Plan: Economy and Joule System (civ-002)

## Phased WBS

### Phase 1: Conservation and full JouleAllocator (E2.4, E2.5)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| P1.3 | Full `JouleAllocator` with actor-level splits | P1.2 | Planned |
| P1.4 | Conservation proptest: sum invariant over 1,000 ticks | P1.3 | Planned |

### Phase 2: Production event emission (E2.1)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| E2.1a | Emit production events to event log with entity ID + good + quantity | P1.4 | Planned |
| E2.1b | Halt production on missing inputs (no phantom goods) | E2.1a | Planned |

### Phase 3: Allocation, taxation, legitimacy (E2.5, E2.6, E2.7, E2.8)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| E2.5 | Full allocation: subsistence priority, deprivation counters, O(n log n) | E2.1b | Planned |
| E2.6 | Taxation: configurable per-institution rate, treasury credit | E2.5 | Planned |
| E2.7 | Budget system: spending, deficits, interest tracking | E2.6 | Planned |
| E2.8 | Legitimacy model: policy → satisfaction → rebellion risk | E2.7 | Planned |

### Phase 4: Multi-good markets (E2.3)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| E2.3a | Multi-good market clearing | E2.5 | Planned |
| E2.3b | TTL-based uncleared order expiry | E2.3a | Planned |

### Phase 5: Stress testing (E2.10)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| E2.10 | Market crash / supply shock stress tests | E2.3b | Planned |

## DAG Dependencies

```
P1.3 → P1.4 → E2.1a → E2.1b → E2.5 → E2.6 → E2.7 → E2.8
E2.5 → E2.3a → E2.3b → E2.10
```
