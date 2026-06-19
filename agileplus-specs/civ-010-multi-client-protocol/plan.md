# Plan: Multi-Client Protocol (civ-010)

## Phased WBS

### Phase 1: Handshake and bootstrap (E3.3)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| PR1.1 | Client handshake: identity, role, bootstrap snapshot within 2 s | civ-001 E1.7 | Planned |
| PR1.2 | Role assignment during handshake; enforced on all subsequent commands | PR1.1 | Planned |

### Phase 2: Binary frames (E3.6)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| PR2.1 | Binary frame header: tick, frame type, uncompressed size, checksum | PR1.2 | Planned |
| PR2.2 | zstd compression; verify ratio >= 3:1 on test delta | PR2.1 | Planned |
| PR2.3 | Bandwidth CI gate: 10 clients at 60 FPS <= 10 Mbps | PR2.2 | Planned |

### Phase 3: Snapshot filtering (E3.5, E3.8)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| PR3.1 | Filter subscription command: entity type + region bbox | PR2.3 | Planned |
| PR3.2 | Server excludes filtered entities from delta frames | PR3.1 | Planned |
| PR3.3 | Filter updatable via subsequent subscription command | PR3.2 | Planned |

### Phase 4: Role authorization (E3.7, FR-CLIENT-003)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| PR4.1 | Role enforcement: research cannot send build/policy commands | PR1.2 | Planned |
| PR4.2 | Integration tests: all three role tiers | PR4.1 | Planned |

### Phase 5: Performance testing (E3.10)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| PR5.1 | Load test: 10 clients at 60 FPS; verify <= 10 Mbps | PR4.2 | Planned |

## DAG Dependencies

```
(civ-001 E1.7) → PR1.1 → PR1.2 → PR2.1 → PR2.2 → PR2.3 → PR3.1 → PR3.2 → PR3.3
PR1.2 → PR4.1 → PR4.2
PR4.2 → PR5.1
```
