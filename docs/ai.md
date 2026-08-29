---
title: AI
description: Citizen needs, ideology spectrum, faction goal planner, and behavior trees in Civis.
---

# AI

## Overview

Civis runs two layers of AI in the same tick phase:

- **Citizen AI** — per-citizen needs, ideology, and immediate action selection.
- **Faction AI** — per-faction goal planner that picks diplomacy, expansion, trade, war, and research goals based on aggregate state.

Both are deterministic: given the same input state and RNG, every AI decision is reproducible. The AI subsystem lives in `crates/ai` and runs in the AI tick phase, immediately after Economy.

## Citizen AI

Each citizen has:

- **Needs**: hunger, happiness, safety. Each is a `Fixed` in `[0.0, 1.0]`.
- **Ideology**: a `Fixed` in `[-1.0, +1.0]` (libertarian to authoritarian).
- **Skills**: combat, farming, research, trade, each `Fixed` in `[0.0, 1.0]`.
- **Job**: `JobType` enum (`Farmer`, `Warrior`, `Scholar`, `Trader`, `Priest`, `Admin`, `Unemployed`).

### Decision Priority

The citizen decision tree runs every AI tick in this fixed order:

1. **Survival** — if hunger exceeds `0.8`, the citizen seeks food.
2. **Safety** — if a hostile military unit is adjacent, flee or engage based on skills.
3. **Job satisfaction** — continue current job, or switch if welfare is chronically low.
4. **Ideological** — support faction goals aligned with personal ideology.
5. **Social** — interact with adjacent citizens of compatible ideology.

Each branch is deterministic; the first branch whose predicate holds wins.

### Need Updates

Needs are updated by the simulation each tick based on consumption and surrounding state:

| Need | Increase | Decrease |
|------|----------|----------|
| Hunger | Always (+0.005/tick) | Fed by Farm production assigned to citizen |
| Happiness | Temple presence, ideology alignment | Tyranny index, war, unemployment |
| Safety | Garrison units nearby | Adjacent hostile units, recent attacks |

## Faction AI

Each faction has:

- **Goals** — current top-K goals selected by the goal planner.
- **Resources** — treasury, population, military strength.
- **Memory** — historical decisions and outcomes, used to bias future planning.
- **Ideology** — aggregate citizen ideology, used as a soft constraint.

### Goal Types

```rust
enum Goal {
    Expand { target: Position },
    Trade { partner: FactionId },
    War { target: FactionId },
    Research { technology: TechId },
    Build { building: BuildingType },
    ProposeTreaty { counterparty: FactionId, terms: TreatyTerms },
}
```

### Goal Planner

The `GoalPlanner` runs a BFS/A* over the goal tree, evaluating each leaf by:

```
utility = expected_joules_gain / joule_cost
```

The top-K goals by utility are committed for the current tick. Goals that fail their preconditions are pruned; goals that succeed emit a `FactionActionEvent` consumed by the relevant subsystem (economy, diplomacy, etc.).

### Memory

Historical decisions are stored as `DecisionRecord { tick, goal, utility, outcome }`. The planner biases toward goals with positive historical outcomes for similar states and away from goals with negative outcomes. The memory window is the last `N` ticks, configurable per faction.

## Behavior Trees

Individual citizen decisions use a behavior tree scaffold defined in `crates/ai::behavior`:

```text
Selector
├── Sequence
│   ├── Condition: hunger > 0.8
│   └── Action: SeekFood
├── Sequence
│   ├── Condition: threat_nearby
│   └── Selector
│       ├── Action: Flee (if combat_skill < 0.3)
│       └── Action: Engage (if combat_skill >= 0.3)
└── Action: ContinueJob
```

Behavior trees are evaluated top-down, left-to-right. Each tick, every citizen evaluates its tree once.

## Modding Hooks

Mods can register custom behavior tree nodes, custom goal types, and custom need types via the `mod-host` ABI. See [Development](/development/) for the mod authoring workflow.

## See Also

- [Architecture](/architecture/) — AI crate location and tick phase ordering.
- [Simulation](/simulation/) — ECS components consumed by the AI phase.
- [Diplomacy](/diplomacy/) — goal planner diplomacy decisions.
- [Economy](/economy/) — citizen welfare and treasury as AI inputs.
- [API](/api/) — JSON-RPC methods for inspecting AI state.