# Plan: Actor and Citizen Lifecycle (civ-003)

## Phased WBS

### Phase 1: Citizen lifecycle (P2.1–P2.2)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| P2.1 | Failing test: `fr_citizen_lifecycle.rs` — Born → Employed → Retired → Dead | civ-001 P0.6 | Planned |
| P2.2 | Implementation: `civ-agents/src/citizen.rs` — `Citizen::tick()` state machine | P2.1 | Planned |

### Phase 2: Institution system (P2.3–P2.4)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| P2.3 | Failing test: `fr_institutions.rs` — Institution holds policies + citizens | P2.2 | Planned |
| P2.4 | Implementation: `civ-social/src/institution.rs` — `Institution { policies, members, budget, approval_rating }` | P2.3 | Planned |

### Phase 3: Ideology (P2.5–P2.6)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| P2.5 | Failing test: `fr_ideology.rs` — Citizen ideology in [-1, +1] | P2.4 | Planned |
| P2.6 | Implementation: `civ-social/src/ideology.rs` — `ideology_shift()` bounded | P2.5 | Planned |

## DAG Dependencies

```
(civ-001 P0.6) → P2.1 → P2.2 → P2.3 → P2.4 → P2.5 → P2.6
(civ-002 E2.5) → P2.2 [deprivation counter coupling]
```
