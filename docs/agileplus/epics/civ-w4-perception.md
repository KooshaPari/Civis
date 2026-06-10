# CIV-W4 — Perception

**Status:** shipped  
**Wave:** perception  
**Primary intent:** turn the simulation into something the player can query, inspect, and reason about.

## FR Trace

- FR-CIV-INFOVIEW-900
- FR-CIV-INFOVIEW-901
- FR-CIV-INFOVIEW-910
- FR-CIV-INFOVIEW-911
- FR-CIV-INFOVIEW-912
- FR-CIV-INFOVIEW-913
- FR-CIV-INFOVIEW-914
- FR-CIV-INFOVIEW-920
- FR-CIV-INSPECT-900
- FR-CIV-INSPECT-901
- FR-CIV-INSPECT-902
- FR-CIV-INSPECT-903
- FR-CIV-INSPECT-910
- FR-CIV-INSPECT-920

## Stories

| Story | Title | FR coverage |
|---|---|---|
| W4.1 | Data-driven overlay registry | FR-CIV-INFOVIEW-900, FR-CIV-INFOVIEW-901 |
| W4.2 | Environmental overlays | FR-CIV-INFOVIEW-910 |
| W4.3 | Resource overlays | FR-CIV-INFOVIEW-911 |
| W4.4 | Population and well-being overlays | FR-CIV-INFOVIEW-912 |
| W4.5 | Society overlays | FR-CIV-INFOVIEW-913 |
| W4.6 | Infrastructure overlays | FR-CIV-INFOVIEW-914 |
| W4.7 | Live legends and update cadence | FR-CIV-INFOVIEW-920 |
| W4.8 | Click-to-inspect world entities | FR-CIV-INSPECT-900 |
| W4.9 | Agent, settlement, and material inspector fields | FR-CIV-INSPECT-901, FR-CIV-INSPECT-902, FR-CIV-INSPECT-903 |
| W4.10 | Hover tooltips and follow-cam history jump | FR-CIV-INSPECT-910, FR-CIV-INSPECT-920 |

## Story Breakdown

### W4.1 Data-driven overlay registry

- Make overlays data-driven so new views can be added without render-core forks.
- Keep one active shading overlay at a time.

### W4.2 Environmental overlays

- Surface pollution, land value, temperature, water, and wind flow.
- Bind each to measured world data.

### W4.3 Resource overlays

- Show natural resources and production/supply flow.
- Keep the readout tied to the sim, not to authored guesses.

### W4.4 Population and well-being overlays

- Present population density, age, happiness, wealth, health, and education.
- Aggregate per cell from measured agent state.

### W4.5 Society overlays

- Show ideology, culture, language, kinship, and polity overlap.
- Render continuous clusters instead of discrete faction IDs.

### W4.6 Infrastructure overlays

- Surface roads, traffic, building level, service coverage, and transport lines.
- Keep coverage falloff visible.

### W4.7 Live legends and update cadence

- Add legends with units and scales for every overlay.
- Update live while the sim ticks.

### W4.8 Click-to-inspect world entities

- Open the inspector for any selectable world element.
- Resolve the clicked entity under the cursor.

### W4.9 Agent, settlement, and material inspector fields

- Show the right fields for each inspector target type.
- Preserve blind fields as unmeasured rather than fabricating values.

### W4.10 Hover tooltips and follow-cam history jump

- Provide concise hover tooltips for interactive items.
- Let the user follow an agent and jump into lineage/history context.
