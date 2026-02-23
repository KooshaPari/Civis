# CIV Functional Requirements

**Project:** CIV - Deterministic Civilization Simulation Engine  
**Version:** 1.0  
**Status:** Draft

---

## Core Simulation

| FR ID | Requirement | Priority | Status |
|-------|-------------|----------|--------|
| FR-CORE-001 | Deterministic tick-based simulation using fixed-point arithmetic | P0 | ✅ Done |
| FR-CORE-002 | Support 10,000+ simultaneous entities (citizens, buildings, units) | P0 | 🔄 In Progress |
| FR-CORE-003 | Entity Component System (ECS) architecture for extensibility | P0 | ✅ Done |
| FR-CORE-004 | Reproducible simulation from seed (deterministic RNG) | P0 | ✅ Done |
| FR-CORE-005 | Snapshot/restore world state for replay | P1 | ⬜ Pending |

## Economy System

| FR ID | Requirement | Priority | Status |
|-------|-------------|----------|--------|
| FR-ECON-001 | Joule-based energy economy (production, consumption, storage) | P0 | ✅ Done |
| FR-ECON-002 | Food/wood/metal resource management | P1 | ⬜ Pending |
| FR-ECON-003 | Citizen job assignment (Farmer, Warrior, Scholar, Trader, Priest, Admin) | P1 | ⬜ Pending |
| FR-ECON-004 | Building construction and maintenance costs | P1 | ⬜ Pending |
| FR-ECON-005 | Trade routes between settlements | P2 | ⬜ Pending |

## Metrics & Governance

| FR ID | Requirement | Priority | Status |
|-------|-------------|----------|--------|
| FR-METRICS-001 | Tyranny index calculation (consumption/budget ratio) | P0 | ✅ Done |
| FR-METRICS-002 | Legitimacy index (1 - tyranny) | P0 | ✅ Done |
| FR-METRICS-003 | Faction treasury tracking | P1 | ⬜ Pending |
| FR-METRICS-004 | Per-faction resource tracking | P1 | ⬜ Pending |
| FR-METRICS-005 | Historical metrics export (JSON) | P2 | ⬜ Pending |

## AI & Behavior

| FR ID | Requirement | Priority | Status |
|-------|-------------|----------|--------|
| FR-AI-001 | Citizen needs simulation (hunger, happiness, ideology) | P1 | ⬜ Pending |
| FR-AI-002 | Faction AI decision-making (expand, trade, war) | P1 | ⬜ Pending |
| FR-AI-003 | NPC behavior based on ideology spectrum | P1 | ⬜ Pending |
| FR-AI-004 | Event system (disasters, discoveries, wars) | P2 | ⬜ Pending |

## Multiplayer & Networking

| FR ID | Requirement | Priority | Status |
|-------|-------------|----------|--------|
| FR-NET-001 | Turn synchronization protocol | P2 | ⬜ Pending |
| FR-NET-002 | Real-time state sync via WebSocket | P2 | ⬜ Pending |
| FR-NET-003 | Conflict resolution for simultaneous actions | P2 | ⬜ Pending |

## Performance Targets

| Metric | Target | Priority |
|--------|--------|----------|
| Tick rate | 60 ticks/second (16ms per tick) | P0 |
| Entity limit | 50,000 entities | P1 |
| Memory per entity | < 1KB | P1 |
| Save/load time | < 1 second for full state | P2 |

---

## Notes

- All numerical calculations use fixed-point arithmetic (i64 with 10^6 scale)
- Simulation must be reproducible given same seed
- Network protocol uses NATS or similar message bus
