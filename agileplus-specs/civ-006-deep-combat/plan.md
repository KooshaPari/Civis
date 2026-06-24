# Plan: Deep Combat System (civ-006)

## Phased WBS

### Phase 1: Military unit entity (E4.1)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| W1.1 | Define `MilitaryUnit` struct with fixed-point fields | civ-001 E1.2b | Planned |
| W1.2 | Register military units in ECS | W1.1 | Planned |

### Phase 2: Combat resolution (E4.2)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| W2.1 | `resolve_combat(attacker, defender, terrain) -> CombatResult` — deterministic | W1.2 | Planned |
| W2.2 | Fatigue accumulation per engagement | W2.1 | Planned |
| W2.3 | Combat event logging to event stream | W2.2 | Planned |

### Phase 3: Casualty and territory (E4.3, E4.6)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| W3.1 | Dead unit removal from ECS same tick | W2.3 | Planned |
| W3.2 | Morale decrement by casualty ratio | W3.1 | Planned |
| W3.3 | Territory control transfer on army destruction | W3.2 | Planned |

### Phase 4: Battle replay CI test (E4.8)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| W4.1 | Property test: replay battle segment → identical casualties | W3.3 | Planned |

## DAG Dependencies

```
(civ-001 E1.2b) → W1.1 → W1.2 → W2.1 → W2.2 → W2.3 → W3.1 → W3.2 → W3.3 → W4.1
(civ-007 diplomacy FSM) → W2.1 [ActiveConflict gate]
```
