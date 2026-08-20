# civ-traffic

> Emergent and user-authored infrastructure layer with desire paths, congestion modeling, and lane-level flow.

## Overview

The `civ-traffic` crate implements a dual-authored road network where both agent behavior and player decisions shape infrastructure. Agents accumulate traffic weight along paths they travel, promoting desire paths through a natural progression: None -> Trail -> Road -> Highway. Players can additionally freehand-place roads to preempt or override emergent routes.

The graph supports lane-level modeling with directional flow, congestion cost calculation, and a service grid that evaluates infrastructure quality across the map. Flow priority determines right-of-way at intersections, enabling realistic traffic behavior.

All infrastructure carries provenance metadata tracking whether a segment was agent-emergent or player-authored, supporting both simulation fidelity and modding transparency.

## Features

- Dual-authored graph: agent-emergent and player-placed infrastructure
- Desire path promotion (None -> Trail -> Road -> Highway)
- Lane-level graph with directional flow modeling
- Congestion cost calculation and pathfinding integration
- Service grid for infrastructure quality evaluation
- Flow priority and right-of-way at intersections
- Provenance tracking for every infrastructure segment

## Usage

```rust
use civ_traffic::*;
```

## Architecture

- **TrafficGraph** — Core graph data structure holding all road segments and intersections
- **RoadKind** — Enum representing infrastructure tier (Trail, Road, Highway, etc.)
- **InfraProvenance** — Metadata tracking whether a segment was emergent or player-authored
- **LaneGraph** — Sub-graph for lane-level directional flow modeling
- **ServiceGrid** — Spatial grid evaluating infrastructure coverage and quality
- **PathCongestion** — Cost model for traffic density on each segment

The `TrafficGraph` is queried by pathfinding and updated each tick as agents traverse it, while player modifications are validated against grammar rules before commit.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
