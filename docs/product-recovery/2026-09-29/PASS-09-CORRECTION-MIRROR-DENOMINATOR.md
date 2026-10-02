# Pass 9 correction — mirror denominator

Date 2026-09-29. Concurrent Civis source: `54d5758970249c8d1f24688ea45920b530e77299`.

A prior recovery note repeated the code comment that `save_state_mirror_to` covers 22 fields and left two unnamed fields unresolved. **Full-function extraction falsified that note.**

The function body mirrors exactly 20 fields:
faction_relations, grief_accumulator, stance_engine, deep_diplomacy, language_state, faction_languages, institutions, institution_levels_emitted, build_sites, econ_focus, cluster_cultures, faction_ideologies, faction_aggression, unrest_settlement_gini, riot_accumulator, migrant_accumulator, scenario_taxation, era_progression, emergence_sample, significance.

The comment claiming 22 is stale. There are no two hidden fields to invent.

A second asymmetry is now source-confirmed:
- `save_state_mirror_to(&self, target)`, used by CivSaveBundle serialization, copies all 20.
- `save_state_mirror(&mut self)`, called at end of every tick immediately before `replay_log.record_tick`, copies only 17 and omits `era_progression`, `emergence_sample`, and `significance`.
- `load_dir` restores all 20 from WorldState to live Simulation.

This is not automatically a product defect. The next falsification is to trace mutation timing/authority for those three fields and compare post-tick replay-facing state with immediate save-facing state. If they are independently synchronized, the asymmetry may be harmless; if Simulation can hold newer values than WorldState at tick recording, replay audit state can lag direct-save state.
