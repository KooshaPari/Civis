# CIV-W3 — Infrastructure

**Status:** shipped  
**Wave:** infrastructure  
**Primary intent:** let roads, structures, and transport emerge from use while still allowing player guidance.

## FR Trace

- FR-CIV-ROAD-900
- FR-CIV-ROAD-901
- FR-CIV-ROAD-902
- FR-CIV-ROAD-910
- FR-CIV-ROAD-920
- FR-CIV-ROAD-921

## Stories

| Story | Title | FR coverage |
|---|---|---|
| W3.1 | Desire-path road emergence | FR-CIV-ROAD-900 |
| W3.2 | Self-organizing structure construction | FR-CIV-ROAD-901 |
| W3.3 | Shared tags across authored and emergent assets | FR-CIV-ROAD-902 |
| W3.4 | Vehicle flow on the road network | FR-CIV-ROAD-910 |
| W3.5 | Manual road tools for player guidance | FR-CIV-ROAD-920 |
| W3.6 | Districts as lens and hint overlays | FR-CIV-ROAD-921 |

## Story Breakdown

### W3.1 Desire-path road emergence

- Reinforce commonly traveled paths into trails, roads, and highways.
- Keep the route formation deterministic under a fixed seed.

### W3.2 Self-organizing structure construction

- Let settlements and agents build when needs, resources, and labor align.
- Avoid scripted build orders.

### W3.3 Shared tags across authored and emergent assets

- Put all roads and structures behind one query/tag model.
- Distinguish provenance without splitting the gameplay API.

### W3.4 Vehicle flow on the road network

- Let transport agents move people and goods across the emergent network.
- Visualize congestion and throughput as part of the system state.

### W3.5 Manual road tools for player guidance

- Provide place, curve, snap, and upgrade operations.
- Make player-built roads first-class members of the same graph.

### W3.6 Districts as lens and hint overlays

- Support districting as a soft influence, not a hard zoning enum.
- Use districts to bias preference rather than enforce outcomes.
