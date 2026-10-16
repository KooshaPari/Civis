# Civis pass 9 — authoritative state manifest v0

Date 2026-09-29. Concurrent implementation `54d5758970249c8d1f24688ea45920b530e77299`. This is an incomplete denominator, not a claim that omitted rows are exhaustive.

## Mirrored Simulation -> WorldState fields observed in save_state_mirror_to

Source search recovers at least these named mirrors:
1. faction_relations
2. grief_accumulator
3. stance_engine
4. deep_diplomacy
5. language_state
6. faction_languages
7. institutions
8. institution_levels_emitted
9. build_sites
10. econ_focus
11. cluster_cultures
12. faction_ideologies
13. faction_aggression
14. unrest_settlement_gini
15. riot_accumulator
16. migrant_accumulator
17. scenario_taxation
18. era_progression
19. emergence_sample
20. significance

The code comment says the mirror covers 22 fields, so **two mirror fields remain unresolved in this pass** rather than being invented. This is exactly why field count is a denominator, not completion evidence.

## WorldState fields explicitly skipped by serde

Previously inspected source identifies six extended fields excluded from world_state.json:
- faction religions;
- faction language systems;
- civilian psyches;
- settlement building layouts;
- historical log;
- faction writing systems.

Each requires an explicit alternative durable component or an accepted ephemeral/derived classification. Presence on WorldState alone does not make it saved.

## Live Simulation-owned state outside the observed mirror/component set

High-priority owners already identified:
- economy_policy;
- active policy object + control signals;
- market_state;
- research_cache;
- loaded mod registry/artifact identities;
- mod guest memory (saved separately, identity dependency unresolved);
- RNG internal state;
- ECS world/components;
- voxel world;
- cluster stocks (separate component);
- environment planet/moon/climate/weather/coastal state (separate component);
- settlements/institutions/emitted levels (separate component plus overlap);
- pending damage/other queues;
- allocator;
- numerous last_tick buffers/histories.

Classification must be semantic: durable, derived/reconstructible, projection/cache, ephemeral, historical, mirror, or unknown.

## Existing repository audit corroboration

The current mod-host source itself contains an unbinding note for FR-CIV-MOD-020 stating that no save records a mod set, no manifest declares a supported save range, and no migration path exists. This is supporting internal audit evidence, not independent runtime reproduction, but it independently converges with the active bundle inspection.

## Candidate-bound runner

Recovery workflow:
- candidate: `432aa1059672c0caf4d0d3a9cfec1f5efddadb78`;
- run: `36622448639`;
- scope: `cargo test -p civ-engine recovery_oracle_`;
- runner: ubuntu-24.04;
- fail-closed;
- raw cargo log + candidate receipt artifact.

Current state at receipt time: queued. Queued is UNKNOWN, not green.

## Next state-manifest work

Resolve the two unnamed mirror fields from exact source; enumerate Simulation struct fields mechanically; for every non-ephemeral field map mutation sites -> save component -> restore site -> migration rule -> non-default fixture. The first executable probes already cover economy_policy, research_cache and orphan mod memory.
