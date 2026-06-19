# Plan: Culture Diffusion and Ideology Spread (civ-009)

## Phased WBS (v2 target)

### Phase 1: Culture entity (FR-CIV-CULT-001)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| CU1.1 | `Culture` struct + `culture_affinity` on Citizen | civ-003 P2.6 | Planned |
| CU1.2 | YAML schema for culture definitions | CU1.1 | Planned |

### Phase 2: Diffusion mechanics (FR-CIV-CULT-002)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| CU2.1 | Contact intensity from spatial adjacency graph | CU1.2 | Planned |
| CU2.2 | Per-tick spread: `spread_rate × contact_intensity × (1 - resistance)` | CU2.1 | Planned |
| CU2.3 | Diffusion events logged to event stream | CU2.2 | Planned |

### Phase 3: Ideology convergence (FR-CIV-CULT-003)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| CU3.1 | Ideology drift when `culture_affinity > CONVERGENCE_THRESHOLD` | CU2.3 | Planned |
| CU3.2 | Drift fed back into legitimacy model | CU3.1 | Planned |

## DAG Dependencies

```
(civ-003 P2.6) → CU1.1 → CU1.2 → CU2.1 → CU2.2 → CU2.3 → CU3.1 → CU3.2
(civ-007 D2.3) → CU2.1 [contact intensity from diplomatic state]
```
