# FR-CIV-INFOVIEW — Info-View Overlay Suite

**Owner:** UI Lead. **Source gap:** Feature Matrix §7 (info-views = **BLIND**, the #1 "feels blind" gap).
**Reference bar:** Cities: Skylines 2 ships ~33 info-view overlays ([CS2 Info views wiki](https://cs2.paradoxwikis.com/Info_views)). Civis currently has only a minimal HUD.
**Emergence note:** Overlays are **[UI/QoL]** — they *visualize* emergent/measured state; they MUST read existing simulation measurements, never introduce hardcoded categories. If an overlay needs data that doesn't exist yet, the data must emerge from Layer-0 laws first (cross-ref the owning domain FR).
**Traceability:** PRD → Feature Matrix §7 → these FR → AC → test (`crates/watch` + bevy-ref client) → client overlay.

## Requirements

| ID | Requirement | Acceptance Criteria | Emergent? |
|---|---|---|---|
| FR-CIV-INFOVIEW-900 | The client SHALL provide a toggleable info-view layer system with a panel of selectable overlays, mutually exclusive active overlay + map shading. | Toggling an overlay re-shades the world map within 1 frame; only one shading overlay active at a time; off-state restores normal render. | UI |
| FR-CIV-INFOVIEW-901 | Overlay registry SHALL be data-driven (each overlay = id, label, data-source binding, color ramp), so new overlays add without code forks. | New overlay added via registry entry only; renders without recompiling render core. | UI |
| FR-CIV-INFOVIEW-910 | Environmental overlays: air/ground/water/noise pollution, land value, temperature, water/wind flow. | Each reads `civ-planet`/`civ-voxel` measured fields; gradient legend shown; matches sampled cell values. | UI (reads [LAW] data) |
| FR-CIV-INFOVIEW-911 | Resource overlays: natural resources (ore/wood/fertile/energy), production/supply flow. | Reads `civ-economy` + `civ-laws`; deposits + utilization shown. | UI (reads [EMERGENT]) |
| FR-CIV-INFOVIEW-912 | Population & well-being overlays: population density, age, happiness, wealth, health, education. | Reads `civ-agents` Needs + emergent demographics; per-cell aggregation. | UI (reads [EMERGENT]) |
| FR-CIV-INFOVIEW-913 | Society overlays: ideology/culture clusters, language/dialect regions, kinship/contact density, polity-membership cluster overlap. | Reads emergent culture/social graph; clusters NOT from `faction:u32`; overlap visualized as continuous field. | UI (reads [EMERGENT]) |
| FR-CIV-INFOVIEW-914 | Infrastructure overlays: roads/traffic, building level, service coverage, transport lines. | Reads emergent architecture/road graph; coverage falloff shown. | UI (reads [EMERGENT]) |
| FR-CIV-INFOVIEW-920 | Each overlay SHALL show a legend (scale + units) and update live as the sim ticks. | Legend present; values update ≥4 Hz at Hot LOD without stalling render. | UI |

**Validation:** snapshot test per overlay group in `crates/watch` SSE + a client visual smoke (screenshot per overlay, vision-verified). Overlays with no backing data are flagged INCOMPLETE and linked to the owning domain FR rather than faked.
