# Plan: Building Tiers and Production Chains (civ-004)

## Phased WBS

### Phase 1: Building tier enum and ECS (FR-CIV-BUILD-001)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| B1.1 | Define `BuildingTier` enum and `BuildingSpec` struct in `civ-economy` | civ-002 P1.4 | Planned |
| B1.2 | Register building entities in ECS with tier component | B1.1 | Planned |

### Phase 2: Production chain (FR-CIV-BUILD-002)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| B2.1 | Implement `inputs`/`outputs` chain on `BuildingSpec` | B1.2 | Planned |
| B2.2 | Halt logic: zero-input check before production each tick | B2.1 | Planned |
| B2.3 | Production event emission to event log | B2.2 | Planned |

### Phase 3: Scenario YAML schema (FR-CIV-BUILD-003)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| B3.1 | Extend scenario YAML schema with `buildings:` section | B1.1 | Planned |
| B3.2 | Schema validation in `scenario.rs` load path | B3.1 | Planned |

## DAG Dependencies

```
(civ-002 P1.4) → B1.1 → B1.2 → B2.1 → B2.2 → B2.3
B1.1 → B3.1 → B3.2
```
