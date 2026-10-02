# Civis P0 durable-state resolution pass 1

Date 2026-09-30.
Observed implementation: `590fad0643eb85cae89edd9e64ed6b991461de6e`.
Authority: RECOVERY SOURCE ANALYSIS. Not a completed persistence contract.

## Resolved more precisely

### current_tick — mirror with restore-gap candidate

Source establishes:
- every Simulation tick increments `state.tick` then assigns `current_tick = state.tick`;
- some phases, including belief/religion logic, read `self.current_tick` directly;
- source search did not find a load-side assignment restoring `current_tick = state.tick` after CivSaveBundle replaces/restores WorldState.

Therefore `current_tick` is not an independent durable owner, but it is not safely classifiable as a harmless cache yet. A load can potentially restore canonical `state.tick=N` while leaving the mirror at a constructor/replay-derived value until another full tick synchronizes it.

Required oracle: save at nonzero tick; load; before calling tick, compare state.tick/current_tick and execute/read a phase or snapshot that consumes current_tick. Terminal disposition is MIRROR only after restoration/rebuild is explicit and tested.

### next_civilian_id — durable canonical or explicitly rebuilt

Engine birth path:
- reads `self.next_civilian_id`;
- assigns it to a child;
- increments it monotonically.

Repository performance/design text explicitly says ID reuse is forbidden. No active save/restore mapping was found.

A Bevy helper independently computes a next civilian ID by scanning live civilians, but the engine birth path does not use that helper.

Accepted options:
1. persist the engine counter as DURABLE_CANONICAL; or
2. define a load-time rebuild from the authoritative live civilian ID set (e.g. max+1 with reserved-range semantics), prove no collision/reuse, then classify DERIVED_DETERMINISTIC.

Resetting to constructor default is not acceptable if live IDs may overlap.

### pending_damage — boundary-sensitive queue

Military phase appends combat damage into `pending_damage`; tactics phase drains it into voxel damage. Replay combat can reconstruct pending damage for replay semantics.

The persistence question is therefore save-boundary semantics:
- if supported saves occur only after the phase that drains the queue, omission may be safe;
- if save can observe an inter-phase world or pending operations cross a supported boundary, the queue is durable/pending realized work and must be persisted/reconstructed.

Terminal disposition requires an explicit supported save boundary plus fault/oracle. Do not persist it blindly and do not call it ephemeral blindly.

### economy_state — partially derived but stateful

`economy_state_from_world` reconstructs the energy budget and tick from WorldState. However EconomyState also advances its own ledger. `phase_economy` syncs energy from WorldState, drains/steps EconomyState, then writes budget back.

Thus:
- current budget/tick are DERIVED/MIRROR candidates;
- economy ledger/history needs a separate authority decision: durable audit/economic state, audit-only, or rebuildable.

One field row should not hide those two semantics.

### settlement_food_stocked — durable canonical candidate

This map is not merely a scenario seed:
- phase economy reads it to compute settlement trade flows;
- `apply_settlement_flow` mutates it across ticks;
- social mood also consumes it.

No active save component mapping was found. Loss changes future market and mood outcomes. This is a strong DURABLE_CANONICAL candidate unless a different canonical stock owner is recovered and this map is proven to be a mirror.

### settlement_housing_capacity / settlement_crime_pressure — durable or externally rebound decision

Both maps feed future social-mood/emergence calculations every tick. They are settable by scenario/load-facing APIs. No active save mapping was found.

Two acceptable models:
- world-owned mutable state -> persist;
- scenario/profile-owned configuration -> bind exact external configuration identity and reapply before first post-load phase.

Silent empty/default reconstruction is not accepted.

## Allocator ambiguity — do not collapse distinct systems

`Simulation.allocator` is the building allocator used with `building_graph`. Separately, `civ_economy::Allocator` is a stateful market/order allocator with bids/offers and monotonic order IDs.

These require separate state rows and authority decisions. A generic "allocator persisted?" requirement would be semantically wrong.

## Next P0 closure

Trace, in order:
1. building_graph + building allocator canonical/rebuild semantics;
2. ECS/world entity durable subset and civilian ID rebuild feasibility;
3. household/actor/kinship/trust causal state;
4. economy ledger/market allocator/order-book state;
5. doctrine/military queues;
6. RNG internal state under the no-global-determinism charter.

Each resolved row must name canonical owner, save/rebuild mechanism and consequence oracle.
