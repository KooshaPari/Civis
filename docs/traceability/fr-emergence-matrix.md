# Emergence Systems Traceability Matrix

**Status:** Active — promotes untraced emergence/dormant-phase `FR-CIV-*` families from [`COVERAGE_AUDIT.md`](COVERAGE_AUDIT.md) (946 untraced IDs; ~171 emergence priority subset here).
**Charter:** [`emergent-systems-tracelinks.md`](emergent-systems-tracelinks.md) + `FR-CIV-0100` §3 emergence.
**Format:** FR-ID | Requirement (1-line) | Crate/File path | Test pattern | **Acceptance Contract** | Status

Status values: `traced` (prior matrix row) | `code-only` (implemented, no matrix until now) | `stub` (spec/design only) | `dormant` (code exists, not in `Simulation::tick`).

The **Acceptance Contract** column is the machine-checkable oracle hook — concrete pass/fail predicates for batchable agent iteration.

**Continuation:** Remaining untraced families (`FR-CIV-SPECIES`, `FR-SESSION`, `FR-CIV-VEHICLE`, …) — see COVERAGE_AUDIT §4 — are **TODO** for a follow-up matrix sweep.

---

## Emergence charter (FR-CIV-0100)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-0100 | Emergence charter — life/society/economy/belief/diplomacy emerge from state with bidirectional coupling | `crates/engine/src/emergence.rs` | `phase_*` regression suite in `engine.rs` | Scalar phases removed 2026-06-25: policy fns + `phase_economy`/`phase_disasters`/`phase_emergence` still satisfy coupling tests in emergent-systems-tracelinks | code-only |
| FR-CIV-0100-int | Emergence charter E2E — emergent-systems-tracelinks rows 1–11 spec+code+test | `docs/traceability/emergent-systems-tracelinks.md` | emergent-systems ledger tests | All 11 system rows marked `COVERED` with named test(s) non-empty | code-only |
| FR-CIV-0100-int2 | Couplings ledger rows 1–30 wired in tick graph | `crates/engine/src/engine.rs` | `unrest_delta_*`, `cohesion_delta_*`, `phase_economy_*` | Each coupling row in emergent-systems-tracelinks has ≥1 passing unit test | code-only |
| FR-CIV-0100-int3 | Dormant phases tagged until `phase_*` wiring lands | this matrix | matrix status column | Every dormant-family row in this file has `Status=dormant` or explicit `code-only` tick path | traced |

---

## Language (FR-CIV-LANG-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-LANG-001 | Language distance drives diplomatic tension (with PSYCHE-912) | `crates/engine/src/engine.rs` | `engine/tests/n13_coverage.rs` | Same seed → identical `faction_language_centroids`; distance > threshold ⇒ tension delta > 0 | code-only |
| FR-CIV-LANG-002 | Phoneme drift deterministic under fixed seed | `crates/agents/src/culture.rs` | `culture_phase_drifts_cluster_profiles` | Two runs same seed: phoneme vectors bitwise-equal at tick N | dormant |
| FR-CIV-LANG-003 | Contact zones show dialect mixing | `crates/agents/src/culture.rs` | TODO: `language_contact_mixing` | Agents at faction border: centroid distance < interior pairs after K contact ticks | dormant |
| FR-CIV-LANG-004 | No authored language lookup table | `crates/agents/src/culture.rs` | grep audit (no static lang table) | Zero hardcoded language-name enums in `agents/` | dormant |
| FR-CIV-LANG-005 | Language vector keyed by cluster id | `crates/agents/src/cluster.rs` | `cluster::assignment_order_independent` | Cluster merge/split preserves deterministic language centroid order | dormant |
| FR-CIV-LANG-006 | Language drift rate bounded per tick | `crates/agents/src/culture.rs` | TODO: `language_drift_rate_cap` | Per-tick L2 drift ≤ configured `max_drift_permille` | dormant |
| FR-CIV-LANG-007 | Mutual unintelligibility threshold configurable | `crates/engine/src/engine.rs` | `n13_coverage.rs` | Below threshold: tension delta = 0; above: delta > 0 | code-only |
| FR-CIV-LANG-008 | Language state serializes in replay snapshot | `crates/engine/src/replay.rs` | TODO: `replay_language_roundtrip` | Save/load replay: language centroids hash-equal | dormant |
| FR-CIV-LANG-009 | Language emergence feed non-empty after warmup | `crates/engine/src/emergence_metrics.rs` | `emergence_metrics::*` | After 250 ticks with ≥2 factions: `emergence_feed.language_regions.len() ≥ 1` | dormant |
| FR-CIV-LANG-010 | Creole formation at high contact intensity | `crates/agents/src/culture.rs` | TODO: `creole_formation` | Contact intensity > creole threshold ⇒ blended centroid between parent dialects | dormant |
| FR-CIV-PSYCHE-912 | Language SHALL drift over contact networks (spec alias) | `docs/specs/requirements/FR-CIV-PSYCHE.md` | `n13_coverage.rs` | Distinct language regions detectable; contact zones show mixing (spec acceptance) | code-only |

---

## Psyche (FR-CIV-PSYCHE-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-PSYCHE-900 | Agent carries emergent psyche: drives, temperament, mood, memory | `crates/agents/src/psyche.rs` | `tick_invokes_emergence_phase` | Identical DNA+history → identical `PsycheState` hash | code-only |
| FR-CIV-PSYCHE-901 | Mood from need satisfaction, memory, env, social events | `crates/agents/src/psyche.rs` | `psyche_phase_mutates_mood_over_ticks` | Mood recomputed each Hot tick; monotonic response to need delta in fixture | code-only |
| FR-CIV-PSYCHE-910 | Kinship + contact social graph emerges from co-location/reproduction | `crates/agents/src/social.rs` | `social::*` mod tests | Edge weight > 0 after co-location fixture; decays when separated | dormant |
| FR-CIV-PSYCHE-911 | Beliefs/norms drift producing ideology clusters | `crates/agents/src/culture.rs` | `culture_phase_drifts_cluster_profiles` | Fixed seed: cluster count and centroids stable across replay | code-only |
| FR-CIV-PSYCHE-920 | Significant events recorded in queryable chronicle | `crates/legends/` | `legends_phase_ingests_death_events` | Death event → chronicle entry queryable by agent id | code-only |
| FR-CIV-PSYCHE-921 | Psyche/social/history LOD-tiered (Hot/Cold) | `crates/agents/` | `agents::lod_gestalt_no_divergence` | Cold→Hot promotion: psyche hash matches always-Hot control | dormant |

---

## Polity / Faction (FR-CIV-POLITY-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-POLITY-001 | Faction formation from cluster density threshold | `crates/agents/src/cluster.rs` | `form_faction_*` in `agents/src/lib.rs` | Population ≥ threshold ⇒ new `FactionId` emitted deterministically | code-only |
| FR-CIV-POLITY-002 | Join existing faction via social ties | `crates/agents/src/lib.rs` | `join_faction_*` | Agent with tie weight > W joins neighbor's faction same tick | code-only |
| FR-CIV-POLITY-003 | Faction roster order-independent | `crates/agents/src/cluster.rs` | `cluster::assignment_order_independent` | Permuting agent iteration order: same faction assignments | code-only |
| FR-CIV-POLITY-004 | Polity treasury aggregate from members | `crates/economy/src/institution.rs` | TODO: `polity_treasury_sum` | Sum member wealth == faction treasury ± rounding | dormant |
| FR-CIV-POLITY-005 | Faction split on ideology divergence | `crates/agents/src/cluster.rs` | TODO: `faction_split_ideology` | Centroid distance > split threshold ⇒ 2 factions from 1 | dormant |
| FR-CIV-POLITY-006 | Polity naming from language drift (no hardcoded) | `crates/legends/src/rumor.rs` | `legends NameRef` tests | Generated polity `NameRef` stable under fixed seed | dormant |
| FR-CIV-POLITY-007 | Multi-faction coexistence in single region | `crates/agents/src/cluster.rs` | TODO: `multi_faction_region` | ≥2 distinct `FactionId` in same spatial bin after 500 ticks | dormant |
| FR-CIV-POLITY-008 | Faction dissolution when population zero | `crates/agents/src/cluster.rs` | TODO: `faction_dissolve_empty` | Last member death ⇒ faction removed from registry | dormant |

---

## Religion / Belief (FR-CIV-REL-* / FR-CIV-RELIGION-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-REL-001 | Belief scalar accrues from population | `crates/engine/src/engine.rs` | `phase_belief_accrues_from_population` | +1 belief per pop/2000 per tick (fixture population) | dormant |
| FR-CIV-REL-002 | Unrest feeds belief under hardship | `crates/engine/src/engine.rs` | `phase_unrest_feeds_belief_under_hardship` | Unrest > 0 ⇒ belief delta > 0 same tick | dormant |
| FR-CIV-REL-003 | Temple level boosts belief accrual | `crates/engine/src/engine.rs` | `phase_institutions_grows_temple_with_belief` | `temple_level` adds +level belief per tick | dormant |
| FR-CIV-REL-004 | Divine intervention spend-or-fail | `crates/engine/src/disasters.rs` | `invoke_divine_disaster_requires_faith` | Insufficient belief ⇒ `Err`; sufficient ⇒ belief debited | code-only |
| FR-CIV-RELIGION-002 | Religion→diplomacy emergence coupling | `crates/engine/src/engine.rs` | TODO: `religion_diplomacy_coupling` | High belief + cohesion biases diplomacy toward peace (threshold tests) | code-only |

---

## Trade / Market (FR-CIV-MARKET-*)

Mapped to `crates/economy/src/market.rs` and `FR-ECON-003` (strategic matrix traces `FR-ECON-*`; `FR-CIV-MARKET-*` promoted here).

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-MARKET-001 | Market price discovery from supply/demand | `crates/economy/src/market.rs` | `apply_pressure_*` | Demand > supply ⇒ price increases next tick | code-only |
| FR-CIV-MARKET-002 | Faction wealth drives staple demand | `crates/engine/src/engine.rs` | `faction_wealth_drives_market_demand` | +treasury ⇒ staple demand component increases | code-only |
| FR-CIV-MARKET-003 | Carrying capacity caps supply side | `crates/engine/src/engine.rs` | `research_tier_and_capacity_grow_with_progress` | `TECH_IRRIGATION` ⇒ +200k carrying capacity | code-only |
| FR-CIV-MARKET-004 | Trade volume scales with surplus gap | `crates/engine/src/engine.rs` | `trade_volume_multiplier_scales_with_surplus_capped_at_2x` | Multiplier in [1.0, 2.0] from surplus differential | code-only |
| FR-CIV-MARKET-005 | Unrest reduces trade volume | `crates/engine/src/engine.rs` | `trade_volume_multiplier_*` | High unrest ⇒ trade factor in [0.5, 1.0] | code-only |
| FR-CIV-MARKET-006 | Cohesion increases trade volume | `crates/engine/src/engine.rs` | `trade_volume_multiplier_*` | High cohesion ⇒ trade factor in [1.0, 1.5] | code-only |
| FR-CIV-MARKET-007 | Order-book clearing deterministic | `crates/economy/src/market.rs` | `market.rs` mod tests | Same orders + seed ⇒ identical clearing prices | code-only |
| FR-CIV-MARKET-008 | Market state in replay hash chain | `crates/engine/src/hash_chain.rs` | TODO: `market_hash_chain` | Price delta changes replay digest | dormant |

---

## Architecture (FR-CIV-ARCH-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-ARCH-001 | Era-grammar transitions produce facade histogram | `crates/build/src/lib.rs` | `build::era_grammar_histogram` | Era shift: histogram bins match golden fixture | traced |
| FR-CIV-ARCH-002 | Freehand tools emit same mods as grammar | `crates/build/` | `build::freehand_matches_grammar` | Freehand stroke ⇒ same `BuildingGraph` delta as grammar op | traced |
| FR-CIV-ARCH-003 | Tile-sets keyed on style vector not enum | `crates/build/src/lib.rs` | inline `#[test]` in `lib.rs` | Style vector perturbation changes tile-set selection | code-only |
| FR-CIV-ARCH-004 | Demand + vector → deterministic template scores | `crates/build/src/lib.rs` | inline test `lib.rs` | Same demand signal ⇒ same ranked templates | code-only |
| FR-CIV-ARCH-005 | Tile-set selection stable under candidate reorder | `crates/build/src/lib.rs` | inline test `lib.rs` | Permuting candidate list: same winner | code-only |
| FR-CIV-ARCH-006 | BuildingGraph RON round-trip lossless | `crates/build/` | `build::graph_ron_roundtrip` | serialize→deserialize hash-equal | traced |
| FR-CIV-ARCH-007 | Canonical mode keys by culture/era | `crates/build/src/lib.rs` | inline test `lib.rs` | `(culture, era)` key maps to stable mode id | code-only |
| FR-CIV-ARCH-008 | Facade histogram tracks culture-vector divergence | `crates/build/src/lib.rs` | inline test `lib.rs` | Culture vector distance correlates with histogram L1 distance | code-only |
| FR-CIV-ARCH-NOSVG-001 | No runtime SVG parsing in asset bundle | `scripts/check_bundle_no_svg_runtime.sh` | CI script | Bundle scan: zero `.svg` in runtime load path | stub |

---

## Climate (FR-CIV-CLIMATE-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-CLIMATE-001 | Season calendar modifies fertility/health | `agileplus-specs/civ-005-climate-disasters-seasons/` | `build/tests/fr_matrix_batch12.rs:648` | Season index cycles deterministically; fertility modifier ≠ 0 in fixture | code-only |
| FR-CIV-CLIMATE-002 | Stochastic disaster events via ChaCha20Rng | `agileplus-specs/civ-005-*/` | `fr_matrix_batch12.rs:678` | Same seed ⇒ same disaster schedule | code-only |
| FR-CIV-CLIMATE-003 | Disaster effects deterministic given params | `crates/engine/src/disasters.rs` | `fr_matrix_batch12.rs:699` | Given disaster params: terrain/agent delta hash-stable | code-only |

---

## Demographics / Life (FR-CIV-LIFE-* / FR-CIV-ACT-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-LIFE-000 | Schema version semver triple | `crates/needs/src/lib.rs` | `schema_version_is_semver` | `SCHEMA_VERSION` parses as semver | code-only |
| FR-CIV-LIFE-001 | Needs decay monotonically each tick | `crates/needs/src/lib.rs` | `needs_decay_toward_zero` | All needs non-increasing per tick unless satisfied | code-only |
| FR-CIV-LIFE-002 | Sickness from deprivation streak | `crates/needs/src/lib.rs` | `deprivation_triggers_sickness` | Unmet threshold N ticks ⇒ sickness flag set | code-only |
| FR-CIV-LIFE-003 | Death from sustained unmet needs | `crates/needs/src/lib.rs` | `unmet_needs_cause_death` | Critical integrity ⇒ `dead` terminal state | code-only |
| FR-CIV-LIFE-010 | Nearest POI of kind | `crates/agents/src/daily_path.rs` | `nearest_of_kind_returns_*` | Returns minimum Manhattan distance POI | code-only |
| FR-CIV-LIFE-011 | Pick target by highest-pressure need | `crates/agents/src/daily_path.rs` | `pick_target_chooses_*` | Lowest need value need kind selected | code-only |
| FR-CIV-LIFE-012 | Path step toward target without overshoot | `crates/agents/src/daily_path.rs` | `path_step_moves_*` | Distance to target strictly decreases or reaches 0 | code-only |
| FR-CIV-LIFE-013 | Deterministic path scoring | `crates/agents/src/daily_path.rs` | `scoring_is_deterministic_*` | Same inputs ⇒ same path choice | code-only |
| FR-CIV-LIFE-014 | Empty registry → no target | `crates/agents/src/daily_path.rs` | `empty_registry_yields_no_target` | `PoiRegistry::is_empty()` ⇒ `None` target | code-only |
| FR-CIV-LIFE-015 | Sated agents wander vs seek | `crates/agents/src/daily_path.rs` | `satisfied_needs_prefer_idle_wander_*` | All needs above threshold ⇒ wander mode | code-only |
| FR-CIV-LIFE-016 | Wander anchors local + deterministic | `crates/agents/src/daily_path.rs` | `wander_anchors_remain_local_*` | Wander endpoints within radius R of anchor | code-only |
| FR-CIV-LIFE-020 | World resource stocks for HUD | `crates/economy/src/stocks.rs` | `arbitrary_stock_updates_*` | Stock never negative after update | code-only |
| FR-CIV-LIFE-021 | Surplus/deficit signs vs stock | `crates/economy/src/stocks.rs` | `surplus_and_deficit_signs_*` | Net flow sign matches surplus/deficit label | code-only |
| FR-CIV-LIFE-022 | Comparative advantage = max net-surplus good | `crates/economy/src/stocks.rs` | `comparative_advantage_must_select_*` | Selected good has maximum net surplus | code-only |
| FR-CIV-LIFE-023 | Trade gain when advantages differ | `crates/economy/src/stocks.rs` | `trade_gain_is_positive_*` | Different advantages ⇒ gain > 0 | code-only |
| FR-CIV-LIFE-024 | Trade conserves total stock | `crates/economy/src/stocks.rs` | `valid_trades_conserve_*` | Pre/post trade sum invariant | code-only |
| FR-CIV-LIFE-025 | Reject trades with no mutual benefit | `crates/economy/src/stocks.rs` | `trade_proposals_are_rejected_*` | No mutual benefit ⇒ trade rejected | code-only |
| FR-CIV-LIFE-030 | Faction roster from life phase | `crates/engine/src/engine.rs` | `fr_matrix_batch1.rs` (getter) | `get_faction_roster()` non-empty after population fixture | code-only |
| FR-CIV-LIFE-035 | Cluster assignment order-independent | `crates/agents/src/cluster.rs` | `cluster::*` tests | Permuted agent order ⇒ same cluster ids | code-only |
| FR-CIV-ACT-001 | Citizen lifecycle (birth/init/age/death) | `crates/engine/src/engine.rs` | `phase_citizen_lifecycle` + build batch12 stub | Birth→age→death state machine reaches terminal death in fixture | code-only |
| FR-CIV-ACTOR-001 | Citizen lifecycle state machine | `agileplus-specs/civ-003-actor-citizen-lifecycle/` | `fr_matrix_batch12.rs:145` | States `{born, active, dead}` only; no orphan transitions | stub |
| FR-CIV-ACTOR-002 | Citizen needs / deprivation | `agileplus-specs/civ-003-*/` | `fr_matrix_batch12.rs:176` | Deprivation streak triggers sickness per FR-CIV-LIFE-002 contract | stub |

---

## Economy (FR-CIV-ECON-* / FR-ECON-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-ECON-001 | Chain substrate deterministic production | `crates/economy/src/chains.rs` | chains mod tests | Same recipe + inputs ⇒ identical outputs | code-only |
| FR-CIV-ECON-001-MARKET | Market price tracking (rename candidate) | `crates/economy/src/market.rs` | `market.rs` tests | `record_transaction` + `update_prices` deterministic | code-only |
| FR-CIV-ECON-002 | Chain substrate doesn't touch macro joule | `crates/economy/src/chains.rs` | chains test | Chain step: joule ledger unchanged | code-only |
| FR-CIV-ECON-002-JOULE | Joule allocator energy conservation | `crates/economy/src/allocator.rs` | allocator tests | Sum joules before == after allocation step | code-only |
| FR-CIV-ECON-004 | Policy-driven fiscal control | `crates/engine/src/policy.rs` | policy tests | Policy toggle changes tax rate in snapshot | code-only |
| FR-CIV-ECON-015 | Recipe I/O sorted; deterministic chain stepping | `crates/economy/src/chains.rs` | extensive chains tests | Recipe keys sorted; step order independent of map iteration | code-only |
| FR-ECON-001 | Buildings produce goods; halt on missing inputs | `crates/economy/` | `phase_production` tests | Missing input ⇒ zero output that tick | code-only |
| FR-ECON-002 | Joule conservation invariant | `crates/economy/src/allocator.rs` | allocator tests | Global joule sum invariant per tick | code-only |
| FR-ECON-003 | Market clearing / order-book price discovery | `crates/economy/src/market.rs` | `apply_pressure_*` | Clearing price equates supply/demand in fixture | code-only |
| FR-ECON-004 | Taxation → institution treasury | `crates/economy/src/institution.rs` | institution tax tests | Tax event increases treasury by expected amount | code-only |
| FR-ECON-005 | Subsistence-first allocation | `crates/economy/src/allocation.rs` | `allocation_behavior.rs` | Food need satisfied before luxury allocation | code-only |

---

## Emergence metrics (FR-CIV-EMERG-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-EMERG-001 | Five read-only emergence dashboard tiles | `crates/engine/src/emergence_metrics.rs` | `dashboard.rs` tests | Metrics block contains exactly 5 named fields | code-only |
| FR-CIV-EMERG-002 | Metrics deterministic for same seed | `crates/engine/src/emergence_metrics.rs` | `emergence_metrics` determinism test | Two runs: metric vectors equal at each sampled tick | code-only |
| FR-CIV-EMERG-003 | Expose on `sim.snapshot.emergence` + replay bus | `crates/server/src/jsonrpc.rs` | jsonrpc test ~2493 | JSON-RPC snapshot includes `emergence` object; replay bus event emitted | code-only |
| FR-CIV-EMERG-004 | Web `EmergencePanel` sparklines | `web/dashboard/` | TODO: web component test | Panel reads `sim.snapshot.emergence` only; 120-tick sparkline | stub |
| FR-CIV-EMERG-005 | Bevy `live_emergence_overlay` HUD | `clients/bevy-ref/` | TODO: bevy overlay test | Toggle E shows 5 chips matching dashboard thresholds | stub |

---

## Emergence mechanics (FR-CIV-EMERGENCE-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-EMERGENCE-001 | Abiogenesis / faction formation threshold | `crates/agents/src/lib.rs` | `form_faction_*` | CA/material threshold ⇒ proto-life/faction event emitted | code-only |
| FR-CIV-EMERGENCE-002 | Join existing faction via social ties | `crates/agents/src/lib.rs` | `join_faction_*` | Social tie weight > W ⇒ same faction id | code-only |
| FR-CIV-EMERGENCE-003 | Environment-vector fitness (voxel vision) | `docs/guides/voxel-emergent-vision-and-migration.md` | TODO: `env_vector_fitness` | Fitness scalar monotonic with environment match score | stub |
| FR-CIV-EMERGENCE-004 | Speciation registry driven by divergence | `crates/genetics/` | `genetics::speciation_trigger` | Divergence > threshold ⇒ new species record | stub |
| FR-CIV-EMERGENCE-010 | Spawn alignment from kinship proximity | `crates/agents/src/lib.rs` | spawn alignment tests | Kin proximity biases spawn faction alignment | code-only |
| FR-CIV-EMERGENCE-N10 | Kinship→cohesion upward causation | `crates/engine/src/engine.rs` | `engine.rs:5175+` tests | Higher kinship ⇒ cohesion delta ≥ baseline | code-only |
| FR-CIV-EMERGENCE-N11 | Psyche maturity→belief coupling | `crates/engine/src/engine.rs` | `engine.rs:5084+` tests | Maturity above threshold ⇒ belief accrual bonus > 0 | code-only |
| FR-CIV-EMERGENCE-N12 | Affinity→diplomacy threshold bias | `crates/engine/src/engine.rs` | `engine.rs:5261+` tests | High affinity lowers conflict threshold | code-only |
| FR-CIV-EMERGENCE-N13 | Language distance↔diplomatic tension | `crates/engine/tests/n13_coverage.rs` | full `n13_coverage.rs` | Language distance correlates with tension delta sign | code-only |

---

## Migration (FR-CIV-MIGRATION-*)

Emergent population migration: flows computed from cluster state (scarcity/disaster/war/overpopulation push vs. surplus/safety/capacity pull), reshaping settlements, mixing culture, and surging on disaster/war. Implemented in `crates/emergence-migration/`.

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-MIGRATION-001 | Push/pull engine — populations flow from high-stress to high-opportunity clusters | `crates/emergence-migration/src/lib.rs` | `population_flows_from_stress_to_opportunity` | Stressed cluster loses pop, opportunity cluster gains; total conserved | code-only |
| FR-CIV-MIGRATION-002 | Settlement reshaping — arrivals/departures resize cluster populations | `crates/emergence-migration/src/lib.rs` | `population_flows_*` / `*conserves_population` | `emigrants_from(src) == immigrants_to(dst)`; sum invariant per tick | code-only |
| FR-CIV-MIGRATION-003 | Cultural mixing counters language/religion divergence | `crates/emergence-migration/src/lib.rs` | `migration_mixes_culture_and_reduces_divergence` | Language + belief divergence between src/dst decreases after migration ticks | code-only |
| FR-CIV-MIGRATION-004 | Refugee surges on disaster/war; decay over time | `crates/emergence-migration/src/lib.rs` | `disaster_triggers_surge`, `war_triggers_surge`, `surge_decays_over_time` | Event ⇒ flow spike vs. baseline; surge monotonically decays toward 1.0 | code-only |
| FR-CIV-MIGRATION-005 | Deterministic per seed | `crates/emergence-migration/src/lib.rs` | `determinism_same_seed_same_outcome` | Same seed+state ⇒ identical flows and engine state | code-only |

---

## AI providers (FR-CIV-AI-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-AI-001 | AI provider port for research/services | `crates/ai/src/lib.rs` | partial provider tests | Provider trait callable; returns structured response | code-only |
| FR-CIV-AI-002 | Local SLM provider (mistral/GGUF) | `crates/ai/src/providers/local_slm.rs` | via dummy roundtrip | Local provider returns text for fixture prompt | dormant |
| FR-CIV-AI-003 | Ollama dev HTTP provider | `crates/ai/src/providers/ollama_dev.rs` | TODO: ollama integration | HTTP 200 + body parses when ollama running | dormant |
| FR-CIV-AI-004 | Cloud fallback (Firepass/Kimi); loud failure | `crates/ai/src/providers/firepass_kimi.rs` | preflight tests | Missing creds ⇒ explicit error, not silent fallback | dormant |
| FR-CIV-AI-005 | Embed provider (MiniLM 384-d) | `crates/ai/src/providers/embed.rs` | TODO: embed_dim | Output vector len == 384 | dormant |
| FR-CIV-AI-006 | Dummy deterministic provider; sync on promotions | `crates/ai/src/providers/dummy.rs` | `dummy_roundtrip.rs` | Same prompt+seed ⇒ identical completion bytes | code-only |
| FR-CIV-AI-007 | Provenance + generic cache | `crates/ai/src/cache.rs` | cache tests | Cache hit returns same provenance id | dormant |
| FR-CIV-AI-008 | Async worker pool never-await | `crates/ai/src/pool.rs` | `dummy_roundtrip.rs` | Hot path does not block on pool join | dormant |
| FR-CIV-AI-009 | Loud-failure preflight | `crates/ai/src/preflight.rs` | preflight tests | Missing required provider ⇒ named error | dormant |
| FR-CIV-AI-010 | `.env`-driven config | `crates/ai/src/config.rs` | config tests | Env var set ⇒ config field populated | dormant |

---

## Legends / Chronicle (FR-CIV-LEGENDS-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-LEGENDS-001 | Structured `HistoricalEvent` on bus; engine doesn't author outcomes | `crates/legends/src/model.rs` | `fr_legends_completion.rs` | Engine emits events; legends crate ingests only | code-only |
| FR-CIV-LEGENDS-002 | Chronicle from witnessed subsets only | `crates/legends/src/rumor.rs` | rumor tests | Chronicle entry cites witness agent ids ⊆ sim witnesses | code-only |
| FR-CIV-LEGENDS-003 | OCEAN gates rumor embellishment/swap | `crates/legends/src/rumor.rs` | `rumor.rs:713+` | High neuroticism ⇒ higher embellishment probability | code-only |
| FR-CIV-LEGENDS-004 | Deity sphere + bladeink salience on prose | `crates/legends/src/rumor.rs` | rumor tests | Deity-tagged events include sphere in prose template | code-only |
| FR-CIV-LEGENDS-005 | Saga-graph query API | `crates/legends/src/query.rs` | `saga_graph.rs` query tests | `saga_of(agent)` returns ordered event ids | code-only |
| FR-CIV-LEGENDS-006 | Loud gap + empty saga with reason | `crates/legends/src/graph.rs` | graph tests | Unknown agent ⇒ empty saga + `reason` field non-empty | code-only |
| FR-CIV-LEGENDS-007 | Cultural register separation in prose | `crates/legends/src/rumor.rs` | rumor tests | Distinct cultures ⇒ distinct register tokens | code-only |
| FR-CIV-LEGENDS-008 | `NameRef` from language drift | `crates/legends/src/rumor.rs` | rumor tests | No hardcoded faction strings in output | code-only |
| FR-CIV-LEGENDS-GRAPH-01 | Saga graph ingest inserts events | `crates/legends/src/lib.rs` | `saga_graph.rs:ingest_*` | Ingest N events ⇒ graph node count == N | code-only |
| FR-CIV-LEGENDS-INGEST-02 | Off-hot-path worker pipeline | `crates/legends/src/worker.rs` | `legends_phase_ingests_death_events` | Death on tick T appears in graph by T+k (k bounded) | code-only |
| FR-CIV-LEGENDS-RESOLVE-04 | Sim ID recycling / aggregate battles | `crates/legends/src/lib.rs` | `resolve_04_*` | Recycled id resolves to aggregate node | code-only |
| FR-CIV-LEGENDS-SIG-05 | Significance promotion + decay/prune | `crates/legends/src/lib.rs` | `sig_05_*` | Low significance pruned after decay window | code-only |
| FR-CIV-LEGENDS-CAUSAL-06 | Causal chains + acyclicity | `crates/legends/src/lib.rs` | `causal_06_*` | Graph remains DAG after ingest | code-only |
| FR-CIV-LEGENDS-QUERY-07 | Read-only query API + emergence feed | `crates/legends/src/query.rs` | `query_07_*` | Query does not mutate sim state | code-only |
| FR-CIV-LEGENDS-NARRATOR-13 | Epoch digest hash stable | `crates/legends/src/lib.rs` | `narrator_13_*` | Same epoch events ⇒ identical digest hash | code-only |
| FR-CIV-LEGENDS-CONFIG-04 | Legends config schema versioned | `crates/legends/src/config.rs` | TODO: config roundtrip | RON round-trip lossless | dormant |
| FR-CIV-LEGENDS-PERF-01 | Legends ingest P99 budget | `crates/legends/` | TODO: bench ingest | Ingest 1k events P99 < 50 ms | dormant |
| FR-CIV-LEGENDS-SCALE-02 | Graph node cap (NFR-SCALE-02) | `crates/legends/src/config.rs` | TODO: prune at cap | Node count ≤ `max_nodes` after prune | dormant |

---

## Culture diffusion (FR-CIV-CULT-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-CULT-001 | Culture entity with ideology centroid + affinity | `agileplus-specs/civ-009-culture-diffusion/` | TODO: culture entity test | Culture record has n-dim centroid; affinity ∈ [0,1] | code-only |
| FR-CIV-CULT-002 | Diffusion via adjacency/contact intensity | `crates/diffusion/` + `agents/culture.rs` | `diffusion::s_curve_adoption` | Contact intensity > 0 ⇒ centroid moves toward neighbor | code-only |
| FR-CIV-CULT-003 | Ideology convergence past threshold | `crates/agents/src/culture.rs` | `culture_phase_drifts_cluster_profiles` | Distance < ε for T ticks ⇒ merge cluster | code-only |

---

## Social institutions (FR-CIV-SOCIAL-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-SOCIAL-001 | Institution system with policies/members/budget | `agileplus-specs/civ-003-*/` | partial `economy/institution.rs` | Institution has members list + budget field | dormant |
| FR-CIV-SOCIAL-002 | Citizen ideology field + drift | `crates/agents/src/psyche.rs` | psyche tests | Ideology vector changes after social event fixture | code-only |
| FR-CIV-SOCIAL-001-INSTITUTIONS | Alias → institutions (civ-021) | `agileplus-specs/civ-021-*/` | TODO | Collapse alias to FR-CIV-SOCIAL-001 | stub |
| FR-CIV-SOCIAL-002-IDEOLOGY | Alias → ideology (civ-021) | `agileplus-specs/civ-021-*/` | TODO | Collapse alias to FR-CIV-SOCIAL-002 | stub |

---

## Diplomacy (FR-CIV-DIPLO-*)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-DIPLO-001 | 8-state diplomatic FSM | `crates/diplomacy/src/lib.rs` | diplomacy substrate tests | Valid transitions only; invalid ⇒ `Err` | code-only |
| FR-CIV-DIPLO-001-RELATIONS | Relations alias (civ-021) | `crates/diplomacy/src/lib.rs` | same as 001 | Collapse to FR-CIV-DIPLO-001 | code-only |
| FR-CIV-DIPLO-002 | Influence capital accrual/spend | `crates/diplomacy/src/lib.rs` | diplomacy tests | Spend > balance ⇒ fail loud | code-only |
| FR-CIV-DIPLO-003 | Shadow network covert flows | `crates/diplomacy/src/lib.rs` | diplomacy tests | Covert action deducts influence deterministically | code-only |
| FR-CIV-DIPLO-004 | War-goal substrate | `crates/diplomacy/src/lib.rs` | diplomacy tests | War goal enum covers fixture cases | code-only |
| FR-CIV-DIPLO-005 | Diplomatic offer/counter taxonomy | `crates/diplomacy/src/lib.rs` | diplomacy tests | Offer types round-trip serialize | code-only |
| FR-CIV-DIPLO-006 | War goal selection taxonomy | `crates/diplomacy/src/lib.rs` | diplomacy tests | Selected goal matches unrest/belief drivers | code-only |
| FR-CIV-DIPLO-007 | Treaty duration and expiry | `crates/diplomacy/src/lib.rs` | diplomacy tests | Treaty expires at tick T+duration | code-only |
| FR-CIV-DIPLO-008 | Accept/reject/counter integration | `crates/diplomacy/src/lib.rs` | 8 diplomacy tests | Counter offer resets FSM to negotiating | code-only |
| FR-CIV-DIPLOMACY | Emergent stance/treaties/reputation from history | `crates/diplomacy/src/emergent.rs` | `emergent::tests` | Stance derives from shared-enemy/trade/border/culture; treaty forms ≥ ally_threshold and breaks ≤ break_threshold → `EventKind::Treaty`/`Betrayal` to legends; betrayal lowers betrayer reputation globally; same inputs ⇒ identical state+events | code-only |

---

## Laws DB (FR-CIV-LAWS-* — untraced extensions)

| FR-ID | Requirement (1-line) | Crate/File path | Test pattern | Acceptance Contract | Status |
|-------|------------------------|-----------------|--------------|---------------------|--------|
| FR-CIV-LAWS-003 | Missing-dependency detection | `crates/laws/src/lib.rs` | `laws` validator test | Law missing dep ⇒ validator error names dep | code-only |
| FR-CIV-LAWS-004 | Duplicate-id detection | `crates/laws/src/lib.rs` | validator test | Duplicate law id ⇒ error | code-only |
| FR-CIV-LAWS-005 | Era filter returns unlocked laws only | `crates/laws/src/lib.rs` | era filter test | Era E: only laws with `unlock_era ≤ E` returned | code-only |
| FR-CIV-LAWS-007 | Mod overlay merge by id | `crates/laws/src/lib.rs` | merge test | Mod law same id overrides base fields | code-only |
| FR-CIV-LAWS-008 | Embedded default RON loads | `crates/laws/src/lib.rs` | embed test | Default `LawDb` non-empty | code-only |
| FR-CIV-LAWS-009 | Mod directory `laws.ron` merge | `crates/laws/src/lib.rs` | mod merge test | Mod dir load adds/overrides laws deterministically | code-only |

*Note: `FR-CIV-LAWS-000..002,006` traced in [`fr-3d-matrix.md`](fr-3d-matrix.md).*

---

## Continuation — TODO families (not in this pass)

The following untraced families from COVERAGE_AUDIT §4 remain for a follow-up matrix (`fr-platform-matrix.md` or family-specific files). Each needs the same six-column format with Acceptance Contract.

| Family prefix | Untraced count | Next action |
|---------------|---------------:|-------------|
| `FR-CIV-SPECIES` | 51 | Promote from `crates/species/` tests |
| `FR-SESSION` | 33 | Promote from `crates/server/` session IDs |
| `FR-CIV-PERF` (FR IDs) | 30 | Cross-link to [`fr-nfr-matrix.md`](fr-nfr-matrix.md) |
| `FR-CIV-VEHICLE` | 26 | Promote from vehicle crate |
| `FR-CIV-ASSET` | 23 | Promote from asset pipeline spec |
| `FR-CIV-CORE` | 23 | Merge with `TRACEABILITY_MATRIX.md` |
| `FR-CIV-RTS` | 23 | RTS client matrix |
| *775 others* | 775 | Batched by `tools/audit-fr-coverage/` |

---

## Row count

**146 data rows** (emergence + dormant phases). Covers ~10 emergence families + charter integration rows; 775+ remaining untraced IDs deferred per continuation table.

*Generated 2026-06-25 for P3-T2 traceability spine. Cross-ref: [`COVERAGE_AUDIT.md`](COVERAGE_AUDIT.md), [`emergent-systems-tracelinks.md`](emergent-systems-tracelinks.md).*
