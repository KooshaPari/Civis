# Plan: Bevy Primary Client (civ-011)

## Phased WBS

### Phase 1: Server connection and tick delta rendering (FR-CLIENT-001)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| B1.1 | WebSocket connection + handshake in Bevy `App` startup system | civ-010 PR1.2 | Planned |
| B1.2 | Binary frame deserialization → Bevy ECS entity sync | B1.1 | Planned |
| B1.3 | Reconnect logic: backoff, restore within 5 s | B1.2 | Planned |

### Phase 2: 3D voxel world rendering (FR-CLIENT-001 perf)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| B2.1 | Integrate `phenotype-voxel` mesher into Bevy render pipeline | B1.2 | Planned |
| B2.2 | GPU instancing for 1,000+ entities; verify 60 FPS on DX12 | B2.1 | Planned |
| B2.3 | DLSS quality mode integration; settings-driven | B2.2 | Planned |

### Phase 3: Click-to-build interaction (FR-CLIENT-001)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| B3.1 | Ray-cast mouse pick to world tile | B2.2 | Planned |
| B3.2 | Send `build_command` via JSON-RPC; reflect result in ECS within one tick | B3.1 | Planned |

### Phase 4: HUD panels (FR-CIV-HUD-001–005)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| B4.1 | Tool palette panel: building tiers, keyboard shortcuts, resource gate | B3.2 | Planned |
| B4.2 | Tech tree panel: DAG render, unlock state, research queue action | B4.1 | Planned |
| B4.3 | Diplomacy panel: FSM state badge, influence capital, action buttons | B4.2 | Planned |
| B4.4 | Event feed: scrollable, color-coded by severity, camera focus on click | B4.3 | Planned |
| B4.5 | Menu system: main menu, pause menu, settings panel | B4.4 | Planned |

## DAG Dependencies

```
(civ-010 PR1.2) → B1.1 → B1.2 → B1.3
B1.2 → B2.1 → B2.2 → B2.3
B2.2 → B3.1 → B3.2 → B4.1 → B4.2 → B4.3 → B4.4 → B4.5
(civ-004 B1.2) → B4.1 [building tiers for tool palette]
(civ-007 D2.3) → B4.3 [diplomatic FSM state for diplomacy panel]
```
