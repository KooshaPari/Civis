---
title: Diplomacy
description: Faction relations, treaties, alliance and war state transitions in Civis.
---

# Diplomacy

## Overview

The diplomacy subsystem governs inter-faction relations: treaties, alliances, trade agreements, non-aggression pacts, neutrality, rivalry, and war. Factions are first-class entities identified by a stable `u32` ID; their relations form a directed graph that the diplomacy phase advances every tick.

`crates/diplomacy` is the canonical location for relation state, treaty lifecycle, and war escalation logic. The diplomacy phase runs after AI and Governance so that policies set in the current tick have already been applied before relations are recalculated.

## Relation Model

Each pair of factions has a `FactionRelation` entry:

| Field | Type | Description |
|-------|------|-------------|
| `other` | `u32` | The other faction's ID. |
| `stance` | `FactionStance` | `Ally`, `Friendly`, `Neutral`, `Rival`, `Hostile`, `AtWar`. |
| `treaties` | `Vec<TreatyId>` | Active treaties binding the pair. |
| `trust` | `Fixed` | Trust score in `[-1.0, +1.0]`. |
| `last_action_tick` | `u64` | Most recent diplomatic action tick. |

Stance is derived from the highest-weight active treaty plus a base trust contribution. Adding or removing a treaty updates stance deterministically at the end of the tick.

## Treaty Lifecycle

Treaties have a discrete lifecycle:

| State | Meaning |
|-------|---------|
| `Proposed` | One faction has offered terms; awaiting counterparty acceptance. |
| `Active` | Both sides have accepted; effects apply. |
| `Suspended` | Temporarily inactive due to a treaty violation; can be restored. |
| `Expired` | Term length elapsed; no longer in force. |
| `Broken` | One side repudiated; trust penalty applied. |

`TreatyTerm` describes what a treaty does:

| Term | Effect |
|------|--------|
| `NonAggression(persistence_ticks)` | Prevents automatic war escalation. |
| `Trade(bonus_pct)` | Adds a multiplier to inter-faction trade income. |
| `MilitaryAccess` | Allows units to traverse the other faction's territory. |
| `Alliance(defense_pact: bool)` | Mutual defense obligation if `defense_pact` is set. |
| `Tribute(resource, amount_per_tick)` | Periodic resource transfer. |

Treaties are proposed via `treaty::propose(from, to, terms)`, accepted via `treaty::accept(treaty_id)`, and broken via `treaty::repudiate(treaty_id, reason)`. All transitions emit a `DiplomacyEvent` for the legends saga graph.

## War Escalation

War is reached either by explicit `declare_war(from, to)` or by automatic escalation. The automatic path is:

1. A faction's `aggression` index crosses the configurable threshold (`policy::war_threshold`).
2. The faction chooses a target from the top-N hostile rivals.
3. `declare_war` is invoked; the stance flips to `AtWar` and a war event is emitted.

Once at war, every tick applies:

- Border skirmishes between adjacent military units.
- Trade route disruption with a configurable economic penalty.
- Trust decay in any remaining non-aggression treaties with involved parties.

## Trust and Reputation

`trust` is a `Fixed` in `[-1.0, +1.0]` updated by:

- Treaty violations: `-0.25` per broken treaty.
- Fulfilled tribute: `+0.05` per tick of active tribute.
- Cooperative AI decisions: `+0.01` (capped per tick).
- Long peace: `+0.005` per 100 ticks of non-aggression (capped).

Trust decays by `-0.001` per tick toward `0.0` if no events occur. Trust directly influences AI goal selection in `crates/ai`.

## Diplomatic Events

Every state change emits a `DiplomacyEvent` consumed by `crates/legends`:

| Event | Payload |
|-------|---------|
| `TreatyProposed` | `proposer`, `counterparty`, `terms`. |
| `TreatyAccepted` | `treaty_id`, `tick`. |
| `TreatyBroken` | `treaty_id`, `by`, `reason`. |
| `WarDeclared` | `aggressor`, `target`, `cause`. |
| `PeaceSigned` | `faction_a`, `faction_b`, `treaty_id`. |
| `TrustChanged` | `faction_a`, `faction_b`, `delta`, `new_trust`. |

These events feed the legends saga graph and surface in the dashboard at `http://127.0.0.1:5173/`.

## See Also

- [Architecture](/architecture/) — diplomacy crate location and tick phase ordering.
- [Simulation](/simulation/) — ECS components that host relation state.
- [AI](/ai/) — how faction goals consume trust and stance.
- [Economy](/economy/) — trade treaties and tribute economics.
- [API](/api/) — JSON-RPC methods for querying faction relations.