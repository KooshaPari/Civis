# Plan: Diplomacy, Laws, and Government (civ-007)

## Phased WBS

### Phase 1: Government type and law loading (FR-CIV-GOV-001, FR-CIV-GOV-002)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| D1.1 | Define `GovernmentType` enum with legitimacy/tax/rebellion modifiers | civ-003 P2.4 | Planned |
| D1.2 | Validate `civ-laws` RON stubs at scenario init; error on invalid schema | D1.1 | Planned |

### Phase 2: Diplomatic FSM (FR-CIV-DIPLO-001)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| D2.1 | `DiplomaticState` enum (8 states); `DiplomaticRelation` keyed by `(ActorId, ActorId)` | D1.1 | Planned |
| D2.2 | Transition logic: threshold-driven, deterministic, config-driven | D2.1 | Planned |
| D2.3 | All transitions logged to event stream | D2.2 | Planned |

### Phase 3: Influence capital (FR-CIV-DIPLO-002)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| D3.1 | `influence_capital: Fixed` per nation; accumulation from trade surplus | D2.3 | Planned |
| D3.2 | Influence spending on alliance formation and sanction lifting | D3.1 | Planned |

### Phase 4: Shadow networks (FR-CIV-DIPLO-003)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| D4.1 | Shadow flow struct: `(source, destination, flow_type, quantity)` | D3.2 | Planned |
| D4.2 | Leakage conservation enforcement (non-negative type assertion) | D4.1 | Planned |
| D4.3 | Enforcement intensity → legitimacy modifier with overreach detection | D4.2 | Planned |

### Phase 5: Threshold metrics (CIV-0104/0105)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| D5.1 | Compute L₀, E*, C₀ each tick; expose as monitoring metrics | D4.3 | Planned |

## DAG Dependencies

```
(civ-003 P2.4) → D1.1 → D1.2; D1.1 → D2.1 → D2.2 → D2.3 → D3.1 → D3.2 → D4.1 → D4.2 → D4.3 → D5.1
(civ-006 W1.2) → D2.2 [ActiveConflict unit availability]
```
