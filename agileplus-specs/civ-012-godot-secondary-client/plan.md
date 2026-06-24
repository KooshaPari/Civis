# Plan: Godot Secondary Client (civ-012)

## Phased WBS (v2 target)

### Phase 1: WebSocket connection and handshake (FR-CIV-CLIENT-GODOT-001)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| GO1.1 | GDScript WebSocket client connecting to `civ-server`; handshake < 2 s | civ-010 PR1.2 | Planned |
| GO1.2 | Binary frame deserialization in GDScript; delta applied to scene tree | GO1.1 | Planned |

### Phase 2: 3D scene rendering (FR-CIV-CLIENT-GODOT-002)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| GO2.1 | Nation-scale entity rendering with instanced meshes | GO1.2 | Planned |
| GO2.2 | Camera pan + zoom; 60 FPS on RTX 3090 Ti and M1 Metal | GO2.1 | Planned |

### Phase 3: Minimal HUD
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| GO3.1 | Resource bar from server state | GO2.2 | Planned |
| GO3.2 | Event feed from tick event stream | GO3.1 | Planned |

### Phase 4: Protocol conformance test
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| GO4.1 | Test: Bevy + Godot clients see identical snapshot simultaneously | GO3.2 | Planned |

## DAG Dependencies

```
(civ-010 PR1.2) → GO1.1 → GO1.2 → GO2.1 → GO2.2 → GO3.1 → GO3.2 → GO4.1
(civ-011 B1.2) → GO4.1 [Bevy baseline for conformance]
```
