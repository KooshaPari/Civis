---
spec_id: civ-011
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-05-29
---

# Specification: Bevy Primary Client (3D, DX12/DLSS)

**Slug**: civ-011-bevy-primary-client | **Epic**: E7 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

Bevy 0.18 is the primary and only mandatory game client. The desktop experience uses DX12 Ultimate + DXR (raytracing) + DLSS on the RTX 3090 Ti target hardware, with Metal fallback on M1 MacBook. The client must render the 3D voxel world at 60 FPS, display all HUD panels (tool palette, tech tree, diplomacy, event feed, menus), connect to the simulation server via binary tick frames, and support click-to-build interaction. Web client is deprecated; Bevy is the definitive UI surface.

## Target Users

- Bevy client developers in `clients/bevy-ref`
- Game designers validating HUD design from CIV-0300
- Players interacting with the 3D world

## Functional Requirements

- [ ] **FR-CLIENT-001**: Bevy client renders 60 FPS on reference hardware (RTX 3090 Ti, DX12); 1,000+ visible entities rendered with instancing; click-to-build sends `build_command` via JSON-RPC; reconnects within 5 s after server restart
- [ ] **FR-CIV-HUD-001**: Tool palette — dockable panel listing buildable structure types by tier; keyboard shortcut per tier; selected tool highlighted; disabled when out of resources
- [ ] **FR-CIV-HUD-002**: Tech tree panel — directed acyclic graph of research nodes; unlocked nodes highlighted; cost and prerequisites displayed; clicking a node queues research if affordable
- [ ] **FR-CIV-HUD-003**: Diplomacy panel — per-nation diplomatic state indicator (FSM state badge + influence capital bar); treaty slot list; "Propose Alliance" / "Impose Sanctions" action buttons; requires `DiplomaticState` data from server
- [ ] **FR-CIV-HUD-004**: Event feed — scrollable feed of simulation events (disasters, wars, elections, market crashes); events color-coded by severity; clicking event focuses camera on affected region
- [ ] **FR-CIV-HUD-005**: Menu system — main menu (New Game / Load / Settings / Quit); pause menu; settings panel (graphics quality, DLSS mode, resolution, key bindings)

## Non-Functional Requirements

- Bevy version: 0.18 with `default-features = true` (ensures tonemapping LUTs, avoids black PBR window)
- Target GPU: RTX 3090 Ti (DX12 Ultimate + DXR + DLSS); fallback: M1 Metal
- Client crate: `clients/bevy-ref` in workspace
- Voxel substrate consumed from `phenotype-voxel` sibling crate
- HUD panels use Bevy UI (not egui) for production quality; egui permitted only for debug overlays
- 3D asset pipeline: `CIV-0601-3d-asset-transition-and-agentic-gen-spec.md`

## Constraints and Dependencies

- Depends on: FR-PROTO-003 (client handshake + bootstrap) for server connection
- Depends on: FR-PROTO-004 (binary frames) for tick delta rendering
- Depends on: `phenotype-voxel` SVO + dense leaf chunk system for world rendering
- Depends on: FR-CIV-DIPLO-001 (diplomatic FSM states) for diplomacy panel data
- Depends on: FR-CIV-BUILD-001 (building tiers) for tool palette item generation
- Web client deprecated; TypeScript/React client artifacts archived

## Acceptance Criteria

- [ ] Client renders 60 FPS on RTX 3090 Ti with 1,000 entities
- [ ] Click-to-build sends `build_command` and reflects result within one tick
- [ ] Client reconnects within 5 s after server restart
- [ ] Tool palette correctly disables items when resources insufficient
- [ ] Tech tree DAG renders with correct unlock state from server data
- [ ] Diplomacy panel shows correct FSM state badge for each known nation
- [ ] Event feed populates from tick event stream; camera focus works on click
- [ ] Menu system: New Game/Load/Settings/Quit all functional
- [ ] DLSS quality mode selectable from settings; renders without black screen

## Implementation Notes

- Client crate path: `clients/bevy-ref` (already in workspace)
- `bevy_tonemapping_luts` must be included (see memory: Bevy default-features fix)
- Instancing: ensure `enableInstancing = true` not silently killed by BRG flags
- HUD spec reference: `docs/specs/CIV-0300-rts-ui-ux-spec.md`
- 3D spec reference: `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md`

## Status

| Story | Status |
|-------|--------|
| E6.1 Bevy plugin integration | Partial (Bevy client stub in workspace; ECS sync not wired) |
| Tool palette HUD | Planned |
| Tech tree HUD | Planned |
| Diplomacy panel HUD | Planned |
| Event feed HUD | Planned |
| Menu system | Planned |
| DX12 / DLSS integration | Planned |
| 60 FPS instanced rendering | Planned |
