# Civis durable-state denominator — disposition rules and next closure pass

Date 2026-09-30. Companion to `STATE-MANIFEST-v0.json`.
Observed implementation revision: `54d5758970249c8d1f24688ea45920b530e77299`.
Authority: RECOVERY DRAFT.

The machine manifest inventories 125 Simulation fields and 48 WorldState fields. The denominator is not closed merely because every field has a row.

## Required terminal dispositions

Every field/domain must end in exactly one accepted semantic disposition:

- DURABLE_CANONICAL — must survive accepted save/restart continuity.
- DURABLE_EXTERNAL_BINDING — not stored as raw state, but save binds exact scenario/profile/artifact identity needed to reconstruct it.
- DERIVED_DETERMINISTIC — reconstructed from other accepted durable state by a specified/versioned function.
- EPHEMERAL_ACCEPTED — omission is intentional and proven not to affect accepted continuation semantics.
- AUDIT_ONLY — history/evidence, not authoritative live state.
- CACHE_REBUILDABLE — discarded and rebuilt without changing accepted world semantics.
- MIRROR — duplicate/projection; canonical owner identified.
- UNSUPPORTED/REMOVED — not part of accepted product contract.

UNKNOWN, SEMANTIC_DECISION, EPHEMERAL_CANDIDATE, REPLAY_DERIVED_CANDIDATE and DIRECT_COMPONENT_OR_DUPLICATE are **non-terminal**.

## Priority closure order

### P0 — future-state drivers / identity allocators

Resolve first because loss can silently fork future world behavior:
- rng;
- current_tick;
- next_civilian_id and other allocators/counters;
- pending durable operations/damage if they cross accepted save boundary;
- building_graph if it owns topology not derivable elsewhere;
- economy_state;
- settlement food/housing/crime;
- household/actor relationship and wealth/power state;
- kinship/trust;
- military/doctrine state;
- active queues/operations.

### P0 — reproduced losses

These already have executable evidence and should be moved from omission statuses to the accepted vNext semantic component disposition after integration acceptance:
- research_cache;
- economy_policy;
- control policy identity/configuration;
- market_state;
- tutorial_progress;
- religious_profiles;
- active_caravans;
- active mod-set/artifact identity + guest-memory ownership.

### P1 — emergence causal state

For Civis's thesis, classify whether these are causal durable state or recomputable observations:
- emergence;
- emergence_branching;
- culture/ideology/language/religion;
- institutions;
- stratification/cohesion/unrest;
- significance/legends;
- sentiment/sentience state;
- diplomacy events/relations.

An emergence label or metric being persisted is not enough if the causal substrate resets.

### P1 — environment/world substrate

Close:
- voxel completeness vs replay-derived claim;
- world/ECS durable subset;
- climate/weather/coastal/environment;
- worldgen config and planet/moon bindings.

### P2 — last_tick / last_* families

Do not mechanically persist them. For each:
- does it feed the next simulation tick?
- does it drive cooldown/hysteresis/edge detection?
- is it UI/audio-only?
- can it be rebuilt exactly from durable state?
- would resetting it create duplicate/missing events after restart?

Only then accept EPHEMERAL/CACHE.

## Oracle template per durable domain

1. construct non-default state;
2. record consequence-sensitive baseline;
3. save;
4. destroy process/Simulation;
5. load;
6. assert semantic state;
7. advance at least one relevant tick/action;
8. assert consequence continuity;
9. negative case: remove/corrupt domain and require fail/degrade/reconstruct according to contract.

Round-trip equality alone is insufficient for causal domains.

## Closure gate

The 125/48 denominator closes only when:
- every non-terminal row has an accepted disposition;
- every DURABLE_* row maps to component/binding, migration and oracle;
- every DERIVED/CACHE row names reconstruction source/function;
- every MIRROR names canonical owner;
- every EPHEMERAL row has a continuity rationale;
- duplicate ownership contradictions are resolved;
- fresh review can no longer identify a future-state driver that silently resets.

Until then, Civis persistence coverage has no defensible single percentage.
