# FR-CIV-ROAD — Emergent Roads, Desire-Paths, Vehicles & Architecture

**Owner:** Infrastructure Lead. **Source gap:** Feature Matrix §4 (roads/desire-paths, vehicles = **BLIND**; architecture INCOMPLETE; zoning BLIND).
**Reference bar:** Manor Lords organic growth + desire-lines + burgage plots (emergent placement); Cities: Skylines road tools (curves/snap/upgrade — gold for the *tooling*, not the rigid zoning model). CS2 traffic AI for vehicle flow.
**Emergence note:** **[EMERGENT]** for the simulation (roads form along desire-paths where agents repeatedly travel; structures built by settlements when need+resource allow; membership/land-use emerges) + **[UI/QoL]** for player affordances (district designation, manual road tools, blueprint). Charter: roads form along desire-paths; structures share data tags regardless of author.

## Requirements

| ID | Requirement | Acceptance Criteria | Tag |
|---|---|---|---|
| FR-CIV-ROAD-900 | Roads/trails SHALL emerge along desire-paths: accumulated agent traversal reinforces routes into trails→roads→highways. | Traversal counters reinforce edges; thresholds promote tier; deterministic under seed; matches Manor Lords desire-line behavior conceptually. | EMERGENT |
| FR-CIV-ROAD-901 | Structures SHALL be built by settlements/agents when needs + local resources + labor allow (self-organizing). | Build decisions read need+resource+labor; no scripted build orders; anarchic/decentralized regions possible. | EMERGENT |
| FR-CIV-ROAD-902 | All structures + roads SHALL carry shared data tags regardless of author (procedural vs player), via the building graph. | `civ-protocol-3d` building graph tags provenance but exposes uniform query API. | EMERGENT+UI |
| FR-CIV-ROAD-910 | Vehicles/transport agents SHALL emerge on the road network to move goods/people; flow visualized via traffic overlay (INFOVIEW-914). | Vehicles route on emergent roads; congestion measurable; no hardcoded routes. | EMERGENT |
| FR-CIV-ROAD-920 | Player SHALL have manual road tools (place/curve/snap/upgrade) that inject road edges the sim then treats identically to emergent ones. | Road tool with curves + snapping + tier upgrade; player roads share the same graph + tags. | UI/QoL |
| FR-CIV-ROAD-921 | Player SHALL be able to designate districts/zones as a *lens/hint* over emergent land-use, not a hardcoded zoning enum. | District = named region overlay; influences agent preference weights, does not force building types. | UI/QoL |

**Validation:** desire-path reinforcement test (repeated traversal → road tier promotion); shared-tag query test (author-agnostic); vehicle routing on emergent graph test.
