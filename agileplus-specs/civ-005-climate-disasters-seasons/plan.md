# Plan: Climate, Disasters, and Seasons (civ-005)

## Phased WBS

### Phase 1: Season calendar (FR-CIV-CLIMATE-001)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| C1.1 | Define `Season` enum and season calendar computation | civ-001 E1.5 | Planned |
| C1.2 | Wire season → tile fertility multiplier in `civ-planet` | C1.1 | Planned |
| C1.3 | Wire season → Citizen health baseline in `civ-agents` | C1.1 | Planned |

### Phase 2: Stochastic disasters (FR-CIV-CLIMATE-002)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| C2.1 | Disaster event struct: `DisasterEvent { region, severity, duration }` | C1.2 | Planned |
| C2.2 | Stochastic disaster generation in `phase_stochastic` using `ChaCha20Rng` | C2.1 | Planned |
| C2.3 | Disaster events logged to event stream | C2.2 | Planned |

### Phase 3: Disaster effects (FR-CIV-CLIMATE-003)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| C3.1 | Fertility drop: `severity × base` in affected tiles | C2.3 | Planned |
| C3.2 | Building output halved in affected region during disaster | C3.1 | Planned |
| C3.3 | Citizen health decrement per affected tick | C3.2 | Planned |

## DAG Dependencies

```
(civ-001 E1.5) → C1.1 → C1.2; C1.1 → C1.3
C1.2 → C2.1 → C2.2 → C2.3 → C3.1 → C3.2 → C3.3
(civ-004 B2.2) → C3.2 [production chain coupling]
(civ-003 P2.2) → C3.3 [citizen health coupling]
```
