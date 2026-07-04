# Civis Emergence Audit

**Date:** 2026-06-23
**Branch:** `research/emergence-audit`
**Scope:** Static read-only audit of every system the emergence charter
(`docs/guides/emergence-charter.md`) says must *emerge* — language, faction,
religion, trade, architecture, climate, economy, demographics, plus a few
adjacent ones the prior `EMERGENCE_COUPLING_AUDIT.txt` and the
`emergent-systems-tracelinks.md` matrix call out.
**Method:** Traced every emergence-related module in `crates/`, then
verified whether its public API is called from `Simulation::tick`
(`crates/engine/src/engine.rs:1172-1199`) or from any other production code
path. Server (`crates/server/src/ws_bridge.rs:1290`,
`crates/server/src/jsonrpc.rs:2827`) and watch (`crates/watch/src/sim_worker.rs:31`)
both drive the sim via that same 12-phase `tick()` — no other entry point
exists in production.

---

## 1. The actual `Simulation::tick` (PHASE_ORDER, 12 phases)

`crates/engine/src/engine.rs:55-68` declares `PHASE_ORDER` and `:1179-1191`
calls exactly these 12 phases from `tick()`:

| # | Phase          | Source file / line                  | What it actually does                                                   |
|---|----------------|-------------------------------------|-------------------------------------------------------------------------|
| 1 | production     | `engine.rs:1498`                    | Sum `Building`s → `state.resources` (Farm→food, Mine→metal, CityCenter→½energy) |
| 2 | citizen_lifecycle | `engine.rs:1525`                 | Starvation death, food need, RNG-birth at `tick%200`                    |
| 3 | military       | `engine.rs:1588`                    | Morale recovery, `tick_operational_movement`, mod-host hook             |
| 4 | policy         | `engine.rs:1748`                    | `policy.evaluate(&state)` → `last_control_signals` (no-op default)     |
| 5 | economy        | `engine.rs:1760`                    | `CapitalistAllocator.allocate`, `civ_economy::step`, **inline** `tick_trade_routes` |
| 6 | planet         | `engine.rs:1269`                    | Calls `civ_planet::compute_climate` + `compute_weather` (used downstream) |
| 7 | diplomacy      | `engine.rs:1695`                    | At `tick%500==0`: pick 2 factions by index, **60/40 coin flip** trade-vs-conflict, ±100/±50 treasury |
| 8 | tactics        | `engine.rs:1357`                    | `tick_operational_movement`; doctrine, war bridge, damage events        |
| 9 | voxel          | `engine.rs:1425`                    | Material CA step, abiogenesis sites, dirty chunk events                 |
|10 | compact        | `engine.rs:1430`                    | `sim.voxel` compact                                                     |
|11 | buildings      | `engine.rs:1437`                    | **Hardcoded constants** for `DemandSignals` (res 0.75, com 0.25, ind 0.25, civ 0.75), then `Allocator::allocate` |
|12 | diffusion      | `engine.rs:1470`                    | `civ_diffusion` step, propagates `Wardrobe.era` / `Tools.era`           |

**There is no emergence tail.** The prior `EMERGENCE_COUPLING_AUDIT.txt`
(`§1` of the prior doc) lists 24 phases and references a
`tick_with_emergence_source` driver. That function **does not exist** — it
is only mentioned in a doc comment at `engine.rs:4615` and in the body of
`fn tick_with_emergence_source_advances_tick_and_differs_on_ca_grid` (a
test at `engine.rs:4617`) which is itself *uncompilable*: every
`sim.phase_tech()` / `sim.phase_chronicle()` / etc. call the prior audit
implies exists in `Simulation` is dead — those function names appear
**zero** times as `fn phase_*` definitions in `crates/engine/src/engine.rs`
(verified by ripgrep across the whole repo).

The functions the prior audit lists (`phase_research`, `phase_tech`,
`phase_belief`, `phase_unrest`, `phase_cohesion`, `phase_social_mood`,
`phase_stratification`, `phase_institutions`, `phase_economic_focus`,
`phase_chronicle`, `phase_life`) **were never implemented as `Simulation`
methods**. They live only as prose in the tracelinks doc and as
phantom-target test calls (`engine.rs:4576, 4577, 4594, 4604, 4605`).

The one phase that *would* be the MOAT emergence tail — `phase_emergence`
at `crates/engine/src/emergence.rs:159` — orchestrates `emergence_ensure_genomes
→ emergence_culture → emergence_social → emergence_psyche →
emergence_genetics_sentience → emergence_legends → emergence_civ_ai`. It is
**not called from `tick()`**. A repo-wide search for `.phase_emergence(`
yields only `emergence_metrics.rs:285` (a different function,
`phase_emergence_events_close`, which closes the per-tick branching-state
counters — also not called from `tick()`). `phase_emergence` itself is
exercised only by the `#[cfg(test)]` `mod tests` block at
`emergence.rs:731-1098`.

---

## 2. Per-system table

Legend:
- **Wired?** — `Y` if called from production code on the hot path (the 12
  `tick()` phases, watch sim worker, server bridge, mod-host), `partial` if
  read in test-only paths, `N` if no caller at all outside its own
  unit-test module.
- **Emergent?** — per charter ("Hardcode only physics/genomic law;
  everything else emerges with bidirectional coupling") this column
  answers "does the current implementation let this system arise from
  substrate state via real coupling, or is it a fixed rule / enum /
  scripted concept?"
- **Gap** — short note on what is missing or hardcoded.

| # | System            | Crate / module                                | Wired? | Emergent? | Gap |
|---|-------------------|-----------------------------------------------|--------|-----------|-----|
| 1 | Voxel CA + materials | `crates/voxel`                                 | Y      | **Y (Layer 0)** | Charter Layer-0 substrate. OK. |
| 2 | Physics / materials DB | `crates/laws`                              | Y (read by `phase_economy` for taxation, `phase_research`-style code never, `civ_economy::step`) | **Y (Layer 0)** | OK as substrate. |
| 3 | Climate (planet)  | `crates/planet` (`compute_climate`, `compute_weather`, `GeologyMap`) | Y (called from `phase_planet` engine.rs:1269) | **Y (Layer 0)** | OK. `civ_climate` (separate crate, global-warming box) is **not used anywhere** — dead. |
| 4 | Day/year/tides    | `crates/planet::is_daytime` etc.              | Y (watch `snapshot.rs` reads `sim.climate()`) | Y | OK. |
| 5 | Wildfire / quakes / disasters | `crates/engine/src/disasters.rs` (`phase_disasters`) | Y (in tests only — see §3 below) | Y (env-threshold-driven) | `phase_disasters` exists at `disasters.rs:70` but is **not called from `tick()`** either. It is run only from 4 unit tests (`disasters.rs:408, 479, 526, 564`). Wildfire/quake effects on `belief` and `population` therefore never reach production. |
| 6 | Genomics / DNA    | `crates/genetics` (`Dna`, `spawn_genome_with_divergence`, `SeedLibrary`, `cognition_score`) | partial — `civ_genetics::Dna` is imported in `engine.rs:28`, used by `emergence::emergence_ensure_genomes` (which is not wired) and by `phase_military` style bridge code paths. `evaluate_sentience` (`emergence.rs:472`) is the only place it matters for production, and that is in the dead tail. | Y (Layer 0 primitive) | The genome → aggression → faction behavior feedback exists in code but is in the dead tail. |
| 7 | Speciation / sentience threshold | `crates/genetics::sentience::evaluate_sentience` | N (called only from dead `phase_emergence` + 2 tests `emergence.rs:1074, 1088`) | Y | Dead. |
| 8 | **Demographics (age cohorts)** | `crates/engine/src/demographics.rs` (`tick_demographics`, `carrying_capacity_from_food`, `Demographics`, `AgeGroup`) | **N** — re-exported from `engine/src/lib.rs:5-7` but **zero non-test callers**. The actual production demographic model is the hardcoded `phase_citizen_lifecycle` (engine.rs:1525): `needs.food ± 0.008/0.03`, `birth_chance = 0.003 * (1 − overcrowding)`, no age cohorts, no disease, no logistic cohort flow. | **N (hardcoded in `phase_citizen_lifecycle`)** | The rich Leslie-style age-cohort engine is dead code. The hot path uses a flat scalar. **Charter gap: birth/death curves don't emerge from material/energy conditions; they are a fixed rate.** |
| 9 | Population hub    | `engine.rs:state.population`                  | Y (read by `phase_economy`, `phase_production`, `phase_diplomacy` indirect) | partial — `state.population` is a scalar mirror of the ECS count, kept in sync by `phase_citizen_lifecycle`. The macro scalar never re-aggregates from per-agent needs/displacement. | minor — duplicates ECS count, but OK for the 12-phase scope. |
|10 | **Cluster/settlement formation** | `crates/agents/src/cluster.rs` (`cluster_by_colocation`, `reconcile_membership`, `should_join`, `should_leave`, `ClusterId`, `ClusterMember`) | **N** — exported from `agents/src/lib.rs:32-34`, but every reference is in `cluster.rs`'s own `#[cfg(test)] mod tests` (lines 198, 221, 248, 260, 272, 308) and in `emergence::emergence_culture` (which is dead). No call from `tick()`. | **N** | Settlements don't co-locate & crystallize during a real run; the entire emergent settlement sub-system is unused. |
|11 | **Faction emergence (k-means, ideology)** | `crates/engine/src/faction_emergence.rs` (`cluster_into_factions`, `should_faction_split`, `should_faction_merge`, `AgentIdeology`, `FactionSeed`) | **N** — **not even declared in `engine/src/lib.rs`**. Pure orphan. Only its own `mod tests` (lines 279, 300, 313, 319, 326) call it. | **N (irrelevant; the path is dead)** | Charter explicitly says "faction is one possible shape; membership is emergent cluster overlap, NOT `faction: u32`". The simulation's `state.factions` is `BTreeMap<u32, Faction>`, populated by the JSON-RPC `faction.create` command (or scenario YAML). The actual emergent k-means / split / merge engine is never instantiated. |
|12 | `civ-diplomacy` (separate crate) | `crates/diplomacy` (`Relation`, `PolityId`, etc.) | **N** — zero non-test callers. Not imported by `engine.rs` at all. | **N** | The real diplomacy in `tick()` is a 60/40 RNG coin flip (engine.rs:1706) between two factions chosen by index modulo (engine.rs:1704-1705). **Charter violation: "polities emerge from co-location + kinship + culture + economic payoff + coercion" — instead, every 500 ticks a coin flip moves ±100/±50 treasuries.** |
|13 | `civ-agents` diplomacy matrix | `crates/agents/src/diplomacy.rs` (`DiplomacyMatrix`, `DiplomacySignal`, `RelationKind`, `RelationRecord`) | **N** — used by `engine/tests/diplomacy_behavior.rs` and `n13_coverage.rs` and `n5_n6_n8_coverage.rs` (all test paths), and re-exported via `agents/src/lib.rs:39-41`. Never called from `phase_diplomacy` or anywhere in production. | **N** | Same as #12 — the rich relation-matrix primitive is dead. |
|14 | **Language (phoneme drift, dialect split, word borrowing)** | `crates/engine/src/language.rs` (`tick_language`, `should_split`, `borrow_word`, `Phoneme`, `Morpheme`, `LanguageState`) | **N** — **not declared in `engine/src/lib.rs`**. Pure orphan. Only its own `mod tests` (lines 251, 259, 268, 285, 311, 316, 331) call it. | **N** | **No language exists in the sim.** Charter: "Ideology & culture & language drift + diffuse across populations … dialects/creoles emerge from contact." None of this happens. The two free functions `language_distance` and `cultural_distance` that *are* imported into `engine.rs:11` and used by `phase_diplomacy`-style peace-bonus logic (engine.rs:5530 test path) are never given any populations to operate on, so the bonus is always 0. |
|15 | **Religion (belief emergence, social spread, ritual)** | `crates/engine/src/religion.rs` (`emerge_belief`, `spread_religion`, `Belief`, `BeliefConcept`, `Religion`) | **N** — re-exported from `engine/src/lib.rs:4` but only its own `mod tests` (lines 105, 113, 124, 130, 149) and an ADR-018 doc call them. | **N** | The macro `state.belief` scalar is updated in `phase_belief` (which doesn't exist) — actually it's never written at all in the 12-phase loop. The only belief-related production code is the **hardcoded** `+50` on disaster in `disasters.rs:35-40` and the `+unrest/100` hardship-faith term mentioned by the prior audit (which would be in `phase_unrest` — also nonexistent as a function). |
|16 | **Culture (cluster culture drift, contact edges)** | `crates/agents/src/culture.rs` (`drift_populations`, `ContactEdge`, `CultureProfile`, `cultural_distance`, `language_distance`) | **N** — only called from `emergence::emergence_culture` (dead) and from the `cultural_distance`/`language_distance` import at `engine.rs:11` which the diplomacy code at engine.rs:5530 uses in a test. No production coupling. | **N** | Charter: "Belief systems, norms, and languages drift + diffuse across populations". Nothing in the hot path touches it. |
|17 | **Psyche (mood, temperament, beliefs)** | `crates/agents/src/psyche.rs` (`psyche_from_dna`, `update_beliefs`, `update_mood`, `nudge_temperament`, `belief_culture_exposure`) | **N** — only `emergence::emergence_psyche` (dead) and tests. | **N** | Same pattern. No production psyche updates. |
|18 | **Social graph (ties, interaction, decay)** | `crates/agents/src/social.rs` (`apply_social_event`, `decay_social_graph`, `SocialGraph`, `Tie`, `Interaction`) | **N** — only `emergence::emergence_social` (dead) and tests. | **N** | No live social graph in the hot path. |
|19 | **Daily path / utility-AI** | `crates/agents/src/daily_path.rs` (`choose_activity`, `pick_target`, `path_step`, `PoiRegistry`, `PoiKind`) | **N** — re-exported, not called from any phase. Watch calls `civ_agents::tick_movement` (separate fn) at `sim_worker.rs:42`, but that's a velocity-step, not a utility-AI. | **N** | Agents don't choose activities from needs. |
|20 | **Needs (food, shelter, safety, belonging)** | `crates/needs` (`Needs`, `LifecycleParams`, `Health`, `NeedKind`) | partial — `phase_citizen_lifecycle` reads/writes `Needs.food` directly (engine.rs:1543-1545) and `phase_military` reads agent misery (mentioned in prior audit). `civ_needs::Needs` is imported by `engine/src/disasters.rs:8` (dead) and `engine/src/save.rs:14` and the dead `phase_emergence`. | partial — scalar needs decay linearly, not from real resource pressure. | Charter gap: "needs" are 4 flat f32s mutated by hardcoded ±0.008/±0.03 deltas, not coupled to actual food production, shelter (no shelter model), safety (no combat loss coupling), belonging (no kin graph). |
|21 | **Life-cycle (births, deaths, aging)** | `civ_needs::LifecycleParams`, `demographics::tick_demographics` | **N** (the rich version). Production runs a scalar starvation model. | **N** | See #8. |
|22 | **Trade routes** | `crates/watch/src/snapshot.rs:771` `trade_routes(factions, tick)` and `:789` `apply_trade_routes`; engine has an **inline** `tick_trade_routes` at `engine.rs:1781` | Y (both run in watch, inline runs in `phase_economy`) | **N (hardcoded)** | The watch's `trade_routes()` is a pure function of `(factions, tick)` — `goods[((tick/180)+idx+to.id) % 6]`, `volume = 8 + ((tick/30)+ids)%16`. **There is no emergent market topology, no comparative advantage, no contact network, no route discovery, no per-resource arbitrage from real supply/demand curves.** The engine's `tick_trade_routes` runs only on routes that already exist in `state.trade_routes` (authored by the player or scenario). Charter: "Markets of varying types. Not one market model — gift, barter, commodity, mercantile, credit, planned — each emerging from local resource/trust/scarcity conditions." Violated. |
|23 | **Market pricing** | `crates/economy` (`MarketState`, `apply_pressure`, `step`) | Y (called from `phase_economy` engine.rs:1778) | partial — `MarketState::step` runs, but the **demand = pop + Σ treasuries, supply = carrying_capacity** heuristic in `phase_economy` is not driven by real production / consumption data. | Charter gap: market is a 1-D price model, not a multi-venue emergent market graph. |
|24 | **Economy (joule budget, allocator, taxation)** | `crates/economy` (`CapitalistAllocator`, `EconomyState`, `step`, `collect_taxes`, `Taxation`) | Y (`phase_economy`) | partial — emergent in the sense that the allocator distributes a real budget against real demand, but demand is policy-driven and taxation is hardcoded policy-rate. | OK for substrate. Could be richer. |
|25 | **Research / technology** | `crates/research` crate (`LawDb`, `tech_unlocks`, etc.) | **N** — `civ_research` is never imported anywhere in `crates/` outside its own tests. The 4 hardcoded `TECH_*` flags the prior audit mentions (`TECH_IRRIGATION`, `TECH_SANITATION`, `TECH_WRITING`, `TECH_GUNPOWDER`) live in **constants inside `engine.rs`**, not in the research crate. There's no `phase_research`, no `phase_tech`. `civ_laws::LawDb` is read by watch (`app.rs`, `snapshot.rs`) and by `civ_research`, but only as a static policy DB — there is no per-tick accrual toward tech unlocks. | **N (hardcoded tier gates)** | Charter: "technology emerges from research, not enum slots; unlocked capabilities should change material outcomes." Currently `tech_unlocks` doesn't even exist as a `Simulation` field — there are 4 hardcoded constant booleans gating `carrying_capacity` and a few test-only phase functions. |
|26 | **Era classification** | `crates/engine/src/era.rs` (`CivEra::evaluate`) | Y (watch `sim_worker.rs:32-36` reads `sim.state.tick/600` and writes `target_era`; the function is also called by tests) | partial — derived from `(population, research_cache().researched.len())`, which itself is hardcoded scenario data. | OK as on-demand evaluator, but `research_cache` is fed by `civ_research` (dead), so the "techs >= N" arms are unreachable in practice. |
|27 | **Belief / faith** | `engine::state.belief` (scalar) | **N (never written in 12-phase loop)** | **N (dead state)** | The prior audit lists `phase_belief` (population, temple_level, belief → belief with decay). That function does not exist. **In production, `state.belief` is read but never written.** Disasters `+50` exists in `disasters.rs` but `phase_disasters` is not called from `tick()`. |
|28 | **Unrest** | `engine::state.unrest` (scalar) | **N (never written in 12-phase loop)** | **N (dead state)** | Same. No `phase_unrest`. The rich multi-driver sum (food + energy + overcrowding + inequality + dispossession + agent_misery) is documented in the prior audit but the function that does the sum is not in the repo. |
|29 | **Cohesion** | `engine::state.cohesion` (scalar) | **N (never written)** | **N** | Same. |
|30 | **Stratification (`dispossessed_permille`)** | `engine::state.dispossessed_permille` (scalar) | **N (never written)** | **N** | Same. |
|31 | **Institutions (temple_level, garrison_level)** | `engine::state.temple_level`, `garrison_level` | **N (never written)** | **N** | Same. `phase_institutions` doesn't exist. |
|32 | **Economic focus (Agrarian / Industrial / Sacred / Mercantile)** | `engine::state.economic_focus` + `EconomicFocus` enum | **N (never written)** | **N (hardcoded enum)** | Charter says economy emerges, not labeled. The enum has 5 hardcoded values; the 2 that get applied as production bonuses are Agrarian/Industrial; Sacred/Mercantile are dead labels. |
|33 | **Chronicle (event log for HUD / inspector)** | `engine::state.chronicle` | **N (never written)** | **N (no producer)** | The only references are in `engine.rs:4573-4612` test functions that call `sim.phase_chronicle()` — but `phase_chronicle` is not defined as a method on `Simulation`. These tests would not compile against a clean tree. (The duplicate lockfile in this worktree makes `cargo check` hard to confirm; see §4.) |
|34 | **Divine powers (faith spend → disaster)** | `try_invoke_divine_disaster` | N (only called from tests) | N | Mechanic exists, but `state.belief` is never written, so the faith gate is always false. |
|35 | **Architecture / building demand** | `crates/build` (`Allocator`, `BuildingGraph`, `DemandSignals`, `Parcel`, `ParcelKind`, `BuildingProvenance`) | Y (called from `phase_buildings` engine.rs:1437) | **N (hardcoded signals)** | `phase_buildings` uses **constants** (residential=0.75, commercial=0.25, industrial=0.25, civic=0.75). The function `building_demand_signals` at `engine.rs:2481` correctly maps `(population, capacity, cohesion, research_tier, unrest, wood, metal)` to `DemandSignals` — **but it is never called from `phase_buildings`**. Charter: "Architecture & civ-driven engineering. Houses, roads/trails/highways, vehicles, tools, machines — built by agents/settlements when needs+resources allow". Architecture is allocated on hardcoded schedule, not from emergent demand. |
|36 | **Roads / desire paths** | `crates/build` (road grammar in `civ-build/src/lib.rs:60+`) + watch `snapshot.rs:roads()` | partial — watch synthesizes roads for the dashboard, but the engine never *creates* them. | **N (no producer in hot path)** | "Roads form along desire-paths" is charter language; no path-finding / desire-path code in `tick()`. |
|37 | **Tools / wardrobe era (technology adoption)** | `crates/diffusion` (`DiffusionParams`, `advance`) | Y (`phase_diffusion` engine.rs:1470; `propagate_era` from `civ-agents`) | Y (S-curve adoption from tech unlocks) | OK as substrate, but `tech_unlocks` source is hardcoded (see #25). |
|38 | **Tactics / doctrine / combat** | `crates/tactics` | Y (`phase_tactics` engine.rs:1357) | partial — doctrine evolution is a hill-climb on a fitness function; not emergent from real combat losses. | OK as substrate; richer real-coupling TBD. |
|39 | **Mods (`.civmod`, Ed25519 verify, wasmtime ticks)** | `crates/mod-host` | Y (loaded by engine; `phase_military` and `phase_economy` route through `mod_host.military_tick` / `mod_host.economy_tick`) | n/a (extension) | Charter gap per `AGENTS.md` "v3 partial–good"; not the focus of this audit. |
|40 | **Audio / SFX** | `crates/audio` | **N** — never imported by engine, watch, or server. | n/a | Not wired. |
|41 | **Climate (global warming box)** | `crates/climate` (`ClimateParams`, `ClimateState`) | **N** — only doc-comment example references it. | n/a (Layer 0 substrate) | Charter: "Climate/planet" should be hardcoded — but the *only* climate that actually runs is `civ_planet::compute_climate` (axial-tilt / insolation). The global-warming feedback box is dead. |
|42 | **Traffic / network flow** | `crates/civ-traffic` | **N** — never imported anywhere. | n/a | Dead. |
|43 | **Legends / saga graph / chronicle** | `crates/legends` (`LegendsWorker`, `SagaGraph`, `EventKind`, `RawSimEvent`) | **N (production)** — only `emergence::emergence_legends` (dead) and `legends/tests/*` reference it. | partial — saga graph is built per-tick in the dead tail; otherwise never. | Charter: "Narrative history is a *consequence* of state, never a generator of outcomes." OK as a consequence — but consequence is never computed in production. |
|44 | **Civ-AI naming** | `crates/ai` | **N** — never imported by engine. `emergence::emergence_civ_ai` uses a local `civ_ai_sync_generate` (deterministic hash) that doesn't call into the `civ-ai` crate. | n/a | The actual `civ-ai` provider pool, `AiCache`, `AiEvent` are not wired; the engine rolls its own dummy. |
|45 | **Faction creation (player/scenario)** | `engine::state.factions: BTreeMap<u32, Faction>` | Y (mutated by JSON-RPC `faction.create`, `scenario` YAML) | **N (authored, not emergent)** | Faction ids are `u32` keys authored from outside the sim. The `civ-agents` faction constants (`FORM_FACTION_COHESION`, `JOIN_FACTION_ACCEPTANCE`, `FRIEND_AFFINITY_THRESHOLD`, `KIN_THRESHOLD`) are defined at `agents/src/lib.rs:62-78` but never read by `phase_diplomacy` or any other phase. |
|46 | **Spawn palette / authoring** | `crates/engine/src/spawn.rs`, JSON-RPC `spawn.*` | Y (authoring path, not emergent) | n/a (authoring) | OK. |
|47 | **Faction treasury / resources / relations** | `engine::state.faction_treasury`, `faction_resources`, `faction_relations` | partial — `tick_trade_routes` (engine.rs:1781) moves resources; `phase_diplomacy` (engine.rs:1695) moves treasuries; `apply_trade_routes` (watch snapshot.rs:789) moves both. | **N (no organic relation matrix evolution)** | Relations are written by the 60/40 coin-flip diplomacy and not by the `civ_agents::diplomacy::DiplomacyMatrix`. |

---

## 3. Where the dead tail lives

The "missing" emergence systems are not just unwritten — they are *fully
implemented* in code, behind a function that is never invoked. The single
choke-point is `crates/engine/src/emergence.rs::phase_emergence`
(`emergence.rs:159`). Adding it to the call chain in `tick()` would
instantly light up:

- DNA → genome component per civilian
- `emergence_culture` → cluster culture drift, contact edges
- `emergence_social` → per-cluster social-graph events
- `emergence_psyche` → mood/belief/temperament per agent
- `emergence_genetics_sentience` → sentience threshold crossings,
  per-faction mean aggression, awakening coupling
- `emergence_legends` → saga graph ingest (births, deaths, sentience,
  diplomacy)
- `emergence_civ_ai` → naming decisions (currently local dummy)

The 7 sister phases in the prior audit's "emergence tail"
(`phase_disasters`, `phase_life`, `phase_research`, `phase_tech`,
`phase_belief`, `phase_unrest`, `phase_cohesion`, `phase_social_mood`,
`phase_stratification`, `phase_institutions`, `phase_economic_focus`,
`phase_chronicle`) are not even defined as methods on `Simulation`.
Implementing them, then inserting them after `phase_buildings` in
`PHASE_ORDER` and `tick()`, would activate the macro scalar layer
(`state.belief`, `state.unrest`, `state.cohesion`, `state.research_progress`,
`state.tech_unlocks`, `state.temple_level`, `state.garrison_level`,
`state.dispossessed_permille`, `state.economic_focus`, `state.chronicle`).

---

## 4. Test-only / dead-in-production summary

Functions defined in `crates/` whose only callers are in their own `#[cfg(test)]`
module (verified by ripgrep across the whole repo):

- `tick_demographics`, `carrying_capacity_from_food` — `crates/engine/src/demographics.rs`
- `emerge_belief`, `spread_religion` — `crates/engine/src/religion.rs`
- `tick_language`, `should_split`, `borrow_word` — `crates/engine/src/language.rs` (also not in lib.rs)
- `cluster_into_factions`, `should_faction_split`, `should_faction_merge` — `crates/engine/src/faction_emergence.rs` (also not in lib.rs)
- `cluster_by_colocation`, `reconcile_membership`, `should_join`, `should_leave` — `crates/agents/src/cluster.rs`
- `psyche_from_dna`, `update_beliefs`, `update_mood`, `nudge_temperament`, `belief_culture_exposure` — `crates/agents/src/psyche.rs` (only via dead `phase_emergence`)
- `apply_social_event`, `decay_social_graph` — `crates/agents/src/social.rs` (only via dead `phase_emergence`)
- `drift_populations` — `crates/agents/src/culture.rs` (only via dead `phase_emergence`)
- `evaluate_sentience` — `crates/genetics/src/sentience.rs` (only via dead `phase_emergence`)
- `LegendsWorker::ingest`, `SagaGraph::mark_died` — `crates/legends/src/graph.rs` (only via dead `phase_emergence`)
- `civ_ai_sync_generate` (in `emergence.rs:719`) — local dummy, never calls `civ-ai`
- `civ_research` crate's `LawDb` and tech APIs — only consumed by `watch` as a static policy DB
- `civ_diplomacy` crate — zero callers anywhere
- `civ_traffic` crate — zero callers anywhere
- `civ_climate` crate — only doc-comment examples
- `civ_audio` crate — never imported by engine/watch/server
- `language.rs` and `faction_emergence.rs` are not even declared in `engine/src/lib.rs` — they're compile-time orphans (compiled into the engine library, but the public API surface of the engine doesn't expose them)

Phantom-target test calls that won't compile against a clean source tree
(because the function they call is not defined):
- `engine.rs:4576, 4577, 4594, 4604, 4605` call `sim.phase_tech()` /
  `sim.phase_chronicle()`. These functions are not defined anywhere in
  `crates/engine/src/engine.rs` (verified by `fn phase_tech` / `fn
  phase_chronicle` ripgrep — zero hits in the engine source). They
  presumably belonged to a `phase_*` module that was deleted in a prior
  refactor and the tests were never updated.
- The doc text at `EMERGENCE_COUPLING_AUDIT.txt:9-11` and the
  `emergent-systems-tracelinks.md` matrix both refer to a
  `tick_with_emergence_source` driver that does not exist. Only a test
  (`engine.rs:4617`) and a doc comment (`emergence_metrics.rs:392`) mention
  it.

Side note on toolchain: this worktree's `Cargo.lock` is broken — it has a
duplicate `[[package]] name = "phenotype-voxel"` entry (lines 6400 and
6410). `cargo check -p civ-engine --lib` errors with "package
`phenotype-voxel` is specified twice in the lockfile". Removing the
duplicate (lines 6410-6418) lets `cargo` proceed. The audit above is
purely static and does not depend on `cargo check` succeeding.

---

## 5. Top 5 gaps to close (ranked by leverage × charter violation)

These are the 5 most-charter-violating, highest-leverage gaps. Each one
already has implementation code; the gap is wiring + ordering, not
authoring.

1. **Wire `phase_emergence` into `tick()`** (and unblock the 12 sister
   phases). One line change in `crates/engine/src/engine.rs::tick()` plus
   matching `PHASE_ORDER` entries. Instantly activates: per-civilian DNA,
   cluster culture drift, social graph, psyche (mood/belief/temperament),
   sentience threshold crossings, per-faction mean aggression, saga-graph
   ingest, civ-ai naming (currently a local dummy hash). Charter score:
   fixes the entire "MOAT emergence tail" at once.

2. **Replace the 60/40 RNG coin-flip `phase_diplomacy` with emergent
   relations.** The rich primitives already exist
   (`civ_agents::diplomacy::DiplomacyMatrix`,
   `civ_diplomacy::Relation`, `emergence::emergence_legends` ingest).
   Drive `DiplomacyMatrix` from `state.belief + state.cohesion +
   trade_volume + contact_edges + last_awakenings` rather than
   `rng.gen_bool(0.6)` (engine.rs:1706). The current code violates
   "polities emerge from co-location + kinship + culture + economic
   payoff + coercion" — every 500 ticks, two factions chosen by `tick %
   factions.len()` get ±100/±50 treasuries with zero reference to their
   state. Charter: states must emerge; the coin-flip is the largest
   hardcoded state mutation in the hot path.

3. **Replace the hardcoded `phase_buildings` demand constants with
   `building_demand_signals(state)`**. `engine.rs:1442-1447` uses
   `residential=0.75, commercial=0.25, industrial=0.25, civic=0.75`. The
   function `building_demand_signals(population, capacity, cohesion,
   research_tier, unrest, wood, metal)` at `engine.rs:2481` already maps
   live state to the same `DemandSignals` struct. Swap the call site. Fixes
   the "architecture built from needs+resources" charter requirement at
   the cost of one line.

4. **Replace `phase_citizen_lifecycle`'s hardcoded
   `±0.008/±0.03/0.003` model with the existing age-cohort `demographics`
   engine and the per-agent `Needs` from `civ-agents`.** `Demographics`,
   `AgeGroup`, `carrying_capacity_from_food`, and the disease/crowding
   terms are all in `crates/engine/src/demographics.rs:61-130` and
   `crates/needs/src/`. Wire them as the birth/death producer. Fixes the
   "demographics emerge from local material+energy conditions" charter
   requirement. Note: requires the dead `state.research_progress` /
   `state.tech_unlocks` path to also be wired (otherwise the carrying
   capacity stays at base).

5. **Replace `watch/src/snapshot.rs:771` `trade_routes(factions, tick)`'s
   hardcoded schedule (`goods[((tick/180)+idx+id) % 6]`,
   `volume = 8 + ((tick/30)+ids)%16`) with a real contact-network +
   comparative-advantage + scarcity-based route synthesizer.** Add a
   `phase_market` (or extend `phase_economy`) that consults per-cluster
   `civ_agents::culture::drift_populations` adjacency, current
   `state.resources` per faction, and per-faction `cluster_stocks` to
   synthesize `state.trade_routes`. Move the result to `state.trade_routes`
   so engine's `phase_economy.tick_trade_routes` (engine.rs:1781) consumes
   it; the watch's separate `apply_trade_routes` can then be deleted.
   Fixes the "markets emerge from local resource/trust/scarcity
   conditions; comparative advantage + surplus/deficit drive trade"
   charter requirement.

**Honorable mentions** (close behind):
- **6.** `phase_research` / `phase_tech` — replace the 4 hardcoded
  `TECH_*` booleans in `engine.rs` constants with a real
  `civ_research`-backed `state.tech_unlocks: u64` (or `Vec<LawId>`)
  producer. `phase_belief` / `phase_unrest` / `phase_cohesion` /
  `phase_stratification` / `phase_institutions` /
  `phase_economic_focus` — write the scalars they are supposed to mutate.
- **7.** `phase_life` — give `cluster_stocks` a sink (currently
  accumulates +∞ food per cluster member with no consumer;
  `emergence.rs` documents this as a known silo).
- **8.** Wire `civ_audio::SfxTrigger` into watch for HUD feedback (gap
  #40 in the table).
- **9.** `civ_traffic` and `civ_climate` (the global-warming box) are
  zero-callers — decide to either delete or wire (the climate box is the
  charter Layer-0 substrate that should be wired if kept).
- **10.** Fix the broken `Cargo.lock` (duplicate `phenotype-voxel`) and
  the phantom-target test calls in `engine.rs:4576-4605` so a clean
  `cargo check -p civ-engine --tests` actually runs.

---

## 6. One-line summary

**1-line:** Only the 12 phases in `PHASE_ORDER` run; every "emergence
tail" (language, religion, demographics, faction, psyche, social,
culture, sentience, legends, civ-ai, belief, unrest, cohesion,
stratification, institutions, economic_focus, chronicle, disasters,
research, tech) is dead — fully implemented behind `phase_emergence`
(`crates/engine/src/emergence.rs:159`) and phantom `phase_*` methods
that don't exist; diplomacy is a 60/40 RNG coin flip every 500 ticks,
architecture is allocated on hardcoded demand constants, trade routes
are a pure function of `(factions, tick)`, and the 5 highest-leverage
gaps are (1) wire `phase_emergence` into `tick()`, (2) replace the
diplomacy coin flip with emergent relation-matrix logic, (3) swap
`phase_buildings` constants for `building_demand_signals(state)`, (4)
swap `phase_citizen_lifecycle` for the `demographics::tick_demographics`
age-cohort engine, and (5) replace `watch::snapshot::trade_routes`'
hardcoded schedule with a contact-network + comparative-advantage
route synthesizer.
