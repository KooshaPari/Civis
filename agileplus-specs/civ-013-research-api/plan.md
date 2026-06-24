# Plan: Research API and Scenario System (civ-013)

## Phased WBS

### Phase 1: Scenario YAML hardening (FR-API-001)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| RA1.1 | Schema validation in `scenario.rs`; field-path error on violation | civ-001 P0.4 | Planned |
| RA1.2 | `data/scenarios/starting_settlement.yaml` CI validation gate | RA1.1 | Planned |

### Phase 2: .civreplay hardening (FR-REPLAY-001, FR-REPLAY-002)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| RA2.1 | SHA-256 checksum footer; verified on load | RA1.2 | Planned |
| RA2.2 | Replay CI gate: state hash at every tick; block merge on divergence | RA2.1 | Planned |

### Phase 3: Python scenario runner (FR-API-002, FR-API-003)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| RA3.1 | Python bridge: FFI or socket between `civlab` Python package and `civ-server` | RA2.2 | Planned |
| RA3.2 | `civlab.run_scenario(path, ticks=50)` < 5 s | RA3.1 | Planned |
| RA3.3 | Policy override dict merge + `ValueError` on invalid param names | RA3.2 | Planned |

### Phase 4: Data export (FR-API-004)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| RA4.1 | `civlab.export(run_id, format="csv")` per-tick metric table | RA3.3 | Planned |
| RA4.2 | JSON export: full event log with tick timestamps | RA4.1 | Planned |
| RA4.3 | Export < 30 s for 100,000-tick run (CI perf gate) | RA4.2 | Planned |

## DAG Dependencies

```
(civ-001 P0.4) → RA1.1 → RA1.2 → RA2.1 → RA2.2 → RA3.1 → RA3.2 → RA3.3 → RA4.1 → RA4.2 → RA4.3
```
