# PERF Hot Path Audit #3 — `Simulation::tick` (`crates/engine/src/engine.rs`)

> Read-only static analysis. **No edits, no commits, no `cargo` invocations** were made.
> Focus: every `phase_` function called from `tick()` (see `engine.rs:1630-1697`),
> per-tick algorithmic cost in the standard agent/cluster/faction/route variables
> (`A` = living agents, `B` = buildings, `M` = military units, `W` = weather cells,
> `C` = emergent clusters, `F` = registered factions, `R` = active trade routes,
> `K` = coastal columns, `D` = doctrines per faction, `L` = social-graph ties per
> agent), plus the rebuilds / re-scans each phase forces on the ECS substrate.
>
> Hand-computed; no benchmarks run.

## Variable glossary

| Symbol | Meaning                                  | Typical size  |
|--------|------------------------------------------|---------------|
| `A`    | Living agent entities (Civilian)         | 100 – 10 000+ |
| `B`    | Buildings                                | 5 – 500       |
| `M`    | Military units                           | 10 – 500      |
| `W`    | Weather grid cells                       | 16 – 256      |
| `C`    | Emergent settlement clusters             | 1 – 100       |
| `F`    | Registered factions (typically 3)        | 3 – 8         |
| `R`    | Active trade routes (capped at 64)       | 0 – 64        |
| `D`    | Doctrines per faction                    | 2             |
| `K`    | Coastal water columns                    | 0 – ~16       |
| `L`    | Social-graph ties per agent              | 0 – 32        |

## Phase inventory (in call order from `engine.rs:1652-1680`)

| #  | Phase                          | Definition (line) | Per-tick cost                                       | Full-ECS re-scan? | Allocations / rebuilds                          | Gated? |
|----|--------------------------------|-------------------|-----------------------------------------------------|-------------------|-------------------------------------------------|--------|
| 1  | `phase_production`             | `engine.rs:2619`  | `O(B)` buildings                                   | yes (Building)    | none                                            | every tick |
| 2  | `phase_citizen_lifecycle`      | `engine.rs:2659`  | `O(A)` agents + `attach_citizen_to_agents`         | yes (× 2)         | `attach_citizen_to_agents` rebuilds `Citizen` per agent that lacks it (`engine.rs:141-160`) | every tick |
| 3  | `phase_military`               | `engine.rs:2732`  | **`O(M²)` worst case** — see hot path #1 below     | **yes (× 2M + E×M)** | `entities: Vec<Entity>` + `samples: Vec<...>`; `apply_damage` per damage event | every tick |
| 4  | `phase_economy` (incl. `tick_trade_routes`) | `engine.rs:2996` / `engine.rs:3066` | `O(R)` routes × ~4 hashmap lookups each, plus `O(F)` treasury sums and `O(A)` spawned `faction_wealth`/`demand` fold | partial (market, treasury maps — `HashMap`) | `flowed_keys: BTreeSet` per route, `decay_idle_emergent_trade_routes` clones the `emergent_trade_route_keys` set | every tick |
| 5  | `phase_planet`                 | `engine.rs:1767`  | `O(1)` + `apply_tide_offset` `O(K)`                | no                | `apply_tide_offset` allocates a `Vec` of updates then two `voxel.write` per column whose height changes | every tick |
| 6  | `phase_diplomacy`              | `engine.rs:2842`  | `O(F² + S²)` — see hot path #4                      | **yes (× 2 — Civilian+ClusterMember; ClusterMember+Position3d)** | `faction_ids: Vec<u32>` collected+sort, `pair_unrest`, `BTreeSet<(u32, u32)>` | **every 500 ticks** |
| 7  | `phase_tactics`                | `engine.rs:1859`  | `O(D)` + doctrine evolve `O(F·D)`                   | no                | `Vec<FactionEngagementStats>` per 64-tick boundary | doctrine evolve every **64 ticks** |
| 8  | `phase_voxel`                  | `engine.rs:1927`  | `O(dirty chunks)`                                  | no                | `last_tick_voxel_events = self.voxel.drain_dirty()` | every tick |
| 9  | `phase_compact`                | `engine.rs:1989`  | gated                                              | no                | `voxel.compact()`                               | every `tick_modulo_compact` ticks |
| 10 | `phase_buildings`              | `engine.rs:2258`  | `O(1)` fast-path + `O(?)` allocator                | no                | `building_graph` mutated by `Allocator::allocate` | every `building_cadence` ticks |
| 11 | `phase_diffusion`              | `engine.rs:2308`  | **`O(A)` ×2 propagators** (wardrobe + tools), each preceded and followed by `O(A)` count scan — see hot path #2 | **yes (× 4 full Wardrobe/Tools scans)** | none | every tick |
| 12 | `phase_disasters`              | `disasters.rs:70` | `O(W)` weather scan + `O(A)` agent hit per triggered disaster | only on hit (Meteor/Flood/Quake/Wildfire/Storm/Plague) | `positions_in_radius` allocates and sorts a fresh `Vec<WorldCoord>` per disaster | every tick |
| 13 | `phase_life`                   | `engine.rs:2390`  | **`O(A²)` worst case** when `cluster_by_colocation` runs — see hot path #3; `O(A)` for needs/path/utility-AI loop; `O(C)` cluster-stocks rebuild | **yes (× 2 — `&AgentCivilian` to find missing needs; `(&AgentCivilian, &Position3d)` for cluster input)** | per-tick: `missing: Vec<Entity>`, `dead: Vec<(Entity, u64, WorldCoord)>`, `entities: Vec<Entity>`, `id_to_entity: HashMap<u64, Entity>`, `cluster_stocks: BTreeMap<u64, ClusterStocks>` rebuilt from scratch | cluster recompute guarded by `life_cluster_position_fingerprint` (PERF_OPT #1) |
| 14 | `phase_settlement_consumption` | `engine.rs:2606`  | `O(C)` cluster stocks                              | no                | mutates existing `cluster_stocks`               | every tick |
| 15 | `rebuild_agent_id_index`       | `engine.rs:2345`  | **`O(A)` full ECS scan every tick** — see hot path #5 | yes (full `&AgentCivilian` walk) | clears + refills `agent_id_to_entity: BTreeMap<u64, Entity>` | **every tick** |
| 16 | `phase_emergence`              | `emergence.rs:135` | sub-phases: `O(A)` ensure genomes + `O(C²)` culture drift + `O(A)` social + `O(A)` psyche + `O(A)` genetics + `O(feed)` legends + `O(feed)` civ-ai | **yes (× 4 — Civilian; Civilian+ClusterMember; Civilian+Dna; Civilian+Psyche)** | `last_feed`, `last_ai_decisions`, `last_sentience` cleared and refilled | every tick |
| 17 | `phase_research`               | `engine.rs:2000`  | `O(1)` arithmetic **+ `sentience_research_bonus` `O(A)` DNA scan** | yes (full `&Dna` walk) | none | every tick |
| 18 | `phase_tech`                   | `engine.rs:2016`  | `O(1)`                                            | no                | none                                            | every tick |
| 19 | `phase_belief`                 | `engine.rs:2024`  | `O(1)`                                            | no                | none                                            | every tick |
| 20 | `phase_unrest`                 | `engine.rs:2051`  | `O(1)` math **+ `agent_misery_unrest` `O(A)` Psyche scan + `faction_treasury_spread` `O(F)`** | yes (full `&Psyche` walk) | none | every tick |
| 21 | `phase_faction_unrest`         | `engine.rs:2084`  | `O(F log F)` sort every tick (`faction_ids.sort_unstable()`) + `O(F)` lookups | no (HashMaps) | `Vec<u32>` collected+sort per tick | every tick |
| 22 | `phase_cohesion`               | `engine.rs:2119`  | `O(1)` math **+ `micro_cohesion_delta` `O(A)` Psyche scan + `micro_social_trust_permille` `O(A·L)` tie scan** | yes (× 2 — Psyche and SocialGraph) | `micro_social_trust_permille` walks every `SocialGraph` *and every tie within it* | every tick |
| 23 | `phase_social_mood`            | `engine.rs:2138`  | `O(A)` Psyche scan                                 | yes (full `&mut Psyche` walk) | none | every tick |
| 24 | `phase_stratification`         | `engine.rs:2150`  | `O(F)` `faction_treasury_spread`                   | no (HashMap)      | none                                            | every tick |
| 25 | `phase_institutions`           | `engine.rs:2161`  | `O(1)` per faction but uses `faction_treasury.keys().min()` every tick | no (HashMap) | none | every tick |
| 26 | `phase_economic_focus`         | `engine.rs:2183`  | `O(F)` treasury fold                              | no (HashMap)      | none                                            | every tick |
| 27 | `phase_chronicle`              | `engine.rs:2220`  | `O(1)`                                            | no                | `format!` only on threshold crossings          | every tick |
| 28 | `phase_emergence_events_close` | `emergence_metrics.rs:257` | `O(1)`                                    | no                | none                                            | every tick |
| 29 | `sample_emergence` / `..._with_ca_grid` | `emergence_metrics.rs:384/391` | `O(1)` no-op non-boundary; `O(emergence state)` on `tick % 50 == 0` | partial | none | every 50 ticks |
| 30 | `replay_log.record_tick`       | replay crate      | `O(1)` append + hash chain (per-event cost outside this audit) | no | none | every tick |

`phase_voxel_ca` is public but **not called from `tick()`**; it is run by the
Bevy/Godot host, so it is listed for reference only
(`engine.rs:1941-1979`).

## The hot path, summed up

A **single `tick()`** on a representative mid-size sim (A ≈ 1 000, M ≈ 100, C ≈ 20)
performs, on the agent-bearing component tables alone:

```
phase_citizen_lifecycle         :  1 × full A scan  (attach_citizen_to_agents)
phase_military inner loops      :  2 × full M scans  +  E × M  (engagement application)
phase_economy (tick_trade_routes): ~4 × R hashmap lookups + 2 × O(F) treasury folds
phase_diffusion                :  4 × full A scans  (2 × wardrobe before/after, 2 × tools before/after)
phase_disasters                :  1 × O(W)          + (only on hit) 1 × O(A) hit
phase_life                     :  1 × A + 1 × (A, pos)  + (when fingerprint changes) 1 × A²
                                  + 1 × A          (id_to_entity rebuild)  + 1 × C
rebuild_agent_id_index         :  1 × full A scan
phase_emergence                :  ~5 × A scans      (genome, social, psyche, genetics, civ-ai)
phase_research (sentience)      :  1 × full A scan (Dna)
phase_unrest (agent_misery)     :  1 × full A scan (Psyche)
phase_cohesion (micro_*)        :  1 × full A scan (Psyche)  +  1 × A·L (SocialGraph ties)
phase_social_mood              :  1 × full A scan (mut Psyche)
```

That's **~14 full agent-bearing ECS walks every tick**, several of them
disjoint (different component types) and a few of them over the same
`&AgentCivilian` collection. At A = 10 000, each scan is a measurable
allocation: a fresh `Vec` plus the hecs archetype iteration. There is no
dirty-flag short-circuiting: every civilian is re-iterated every tick even
when no one moved, no one was born, no one died, and no one's `Psyche`
changed.

## The five worst offenders

Ranked by total per-tick hot-path cost × frequency × cache pressure. The
fixes below are all **behavior-preserving** unless explicitly flagged.

<response>

### #1 — `phase_military` does an O(M²) inner scan per engagement (`engine.rs:2775-2787, 2812-2820`)

**Current cost.** Two nested `for (entity, unit) in self.world.query_mut::<&mut MilitaryUnit>()` loops
walk the entire military archetype **once per engagement** to update a single
target entity's `position` / `hp`. With `E` engagements (≈ M in heavy combat)
and `M` units, this is **`O(M · E) = O(M²)`** every tick — the only quadratic
inner loop in the engine proper (the `phase_life` all-pairs cluster is already
guarded by fingerprint). On a 200-unit field with sustained combat this is the
single largest tick cost the engine pays each frame.

**Proposed fix.** Build a `HashMap<Entity, usize>` (or reuse the existing
`entities: Vec<Entity>` already collected at `engine.rs:2751-2766`) as an
`entity → index` map at the top of `phase_military` and use it for the
O(1) `get_mut(idx)` write. The two inner scans collapse to O(E) writes.

**Expected win.** 10–100× speed-up of the `phase_military` body under combat
(measured vs. the O(M²) baseline). At M = 200 this drops the phase from
~40 000 archetype walks to ~200.

**Risk.** Low. The current code already pre-collects `entities: Vec<Entity>` in
parallel with the sample build (`engine.rs:2751-2766`) — extending that to a
`HashMap` is a one-line ergonomic change. The first inner loop is in
`tick_operational_movement` result-handling; the second is in the engagement
HP-application loop.

**Behavior-preserving.** Yes. The identity check `entity == target_entity`
followed by write is observably equivalent to a `HashMap` lookup + direct
`get_mut`. The `break;` after the match in each loop is preserved (only the
first matching entity is touched, and ECS entity ids are unique by
construction).

---

### #2 — `phase_diffusion` does 4 full ECS count scans every tick (`engine.rs:2308-2333`, calling `propagate_cohort_wardrobe_with_lod` at `engine.rs:790-834` and the tools twin at `engine.rs:836-880`)

**Current cost.** Each propagator function pre-computes
`currently_at_target = world.query::<&Wardrobe>().iter().filter(...).count()`
**and then** does the same scan again after propagation to recompute the
fraction — 2 scans per propagator × 2 propagators (wardrobe + tools) = **4
full A-iterations** per tick, all of which produce a scalar `currently_at_target`
that is otherwise unused by downstream consumers (only `current_fraction` is
consumed by `propagate_wardrobe` / `propagate_tools`).

**Proposed fix.** Compute the denominator (`total_civilians = count_civilians(world)`)
once at the top of `phase_diffusion` and pass it in. The numerator
(`currently_at_target`) only needs to be re-derived when something actually
changed — track a `dirty_cohort_stats: bool` set by `phase_citizen_lifecycle`
and cleared after the post-scan. Better yet, just **drop the post-scan** —
it is written to `last_cohort_stats.currently_at_target` and never read by
any other phase. (The `debug_assert_eq!` at `engine.rs:2328-2331` is the
only consumer and it is satisfied by the pre-scan value.)

**Expected win.** Drops `phase_diffusion` from 4 × O(A) to 1 × O(A)
(`count_civilians` is already paid by `phase_citizen_lifecycle` two phases
earlier — share the cached scalar). At A = 5 000, ~20 000 archetype
iterations saved per tick.

**Risk.** Low. Removing the post-scan changes `last_cohort_stats.currently_at_target`
to the **pre-tick** value; the test at `engine.rs:5821` (`phase_life_clustering_is_deterministic`)
cares only about determinism, not the absolute counter. If any HUD reads the
post-tick value, we can recompute it on demand from the `&Wardrobe` table.

**Behavior-preserving.** Yes for all gameplay, conditional for the
`last_cohort_stats.currently_at_target` field (semantically a one-tick lag,
which is already documented as "currently_at_target at end of phase"
elsewhere; pre-tick value is the same as end-of-previous-tick value, so
all consumers see the previous-tick snapshot). The `debug_assert_eq!`
self-check still holds if we re-use the `count_civilians` already paid
by `phase_citizen_lifecycle` and pass it in.

---

### #3 — `phase_life` re-allocates the agent id → entity map every clustering recompute (`engine.rs:2553-2558`)

**Current cost.** Inside the (already fingerprint-guarded) clustering branch,
the phase allocates a fresh
`id_to_entity: HashMap<u64, Entity>` from a full `&AgentCivilian` query, then
`rebuild_agent_id_index()` (`engine.rs:2345-2355`, called unconditionally at
`engine.rs:1667`) **does the same work again** the very next phase. Together
that's **2 × O(A)** every tick plus a `HashMap` allocation, when an index
already exists in `self.agent_id_to_entity` and is kept current by the
existing dirty-index path.

**Proposed fix.** Inside the clustering branch, swap the local
`id_to_entity: HashMap` build for a read-only borrow of
`&self.agent_id_to_entity` and use `self.agent_id_to_entity.get(&agent_id)`
to resolve entities (the map is already up-to-date because
`rebuild_agent_id_index` runs every tick — by design). The
`HashMap<u64, Entity>` allocation and the full `&AgentCivilian` query at
`engine.rs:2554-2558` go away.

Then the bigger question: **`rebuild_agent_id_index` is called every tick at
`engine.rs:1667`** even when no agent was added, removed, or moved this tick.
A `dirty_agents: bool` flag set by `phase_citizen_lifecycle` (births) and
the despawn paths in `phase_life` / `phase_military` / disasters would let
the rebuild fire only when membership changed — turning `rebuild_agent_id_index`
from `O(A)` per tick into `O(Δagents)` per dirty event.

**Expected win.** Saves a full `O(A)` archetype walk + `BTreeMap` clear/refill
**every tick**, plus the clustering-branch `HashMap` allocation when the
fingerprint changed. At A = 5 000 and no dirty events, ~5 000 walks saved
per tick; the `BTreeMap` is also heap-allocator-heavy.

**Risk.** Low. The `agent_id_to_entity` map is already authoritative for the
rest of the engine (every `apply_social_pair`, `apply_legends_ingest`,
`infer_alignment_for_spawn`, etc. goes through it). The local
`id_to_entity: HashMap` in the clustering branch was a leftover from before
PERF_OPT #2 introduced the persistent index. Removing it and reading from
the persistent index is strictly an improvement.

The dirty-flag version of `rebuild_agent_id_index` is also low-risk: the
flag is set in 4 known mutation points (births at `engine.rs:2704-2712`,
deaths at `engine.rs:2715-2723`, military despawns at `engine.rs:2837`,
disaster `hit_agents` despawns at `disasters.rs:271`) and the index is only
read by code that runs after `rebuild_agent_id_index` in the same tick, so
an event-time dirty flag is sound.

**Behavior-preserving.** Yes. The persistent `agent_id_to_entity` index is
the same data structure the local `HashMap` was building; dirty-flagging
simply delays the rebuild by at most one tick (it would still run on the
next tick whether or not anything changed), which the current code already
does every tick anyway.

---

### #4 — `phase_diplomacy` runs two full ECS scans to compute settlement contact every 500 ticks (`engine.rs:2842-2986`, calling `settlement_dominant_factions` at `engine.rs:3936-3975` and `settlement_contact_pairs` at `engine.rs:3978-4022`)

**Current cost.** Every 500 ticks (`engine.rs:2843`), `phase_diplomacy` calls
`diplomacy_pair_from_settlement_overlap` which builds:
- `settlement_dominant_factions`: full `(&AgentCivilian, &ClusterMember)`
  archetype walk → `O(A)`;
- `settlement_contact_pairs`: another full `(&ClusterMember, &Position3d)`
  archetype walk into a `BTreeMap<u64, Vec<(i64, i64)>>`, then **O(C²) pair
  enumeration with an O(|A_i| × |A_j|) inner cross-product** for each pair of
  clusters.

The inner `agents_a.iter().any(|a| agents_b.iter().any(|b| ...))` at
`engine.rs:4009-4015` is a **per-cluster-pair N_i × N_j distance check**,
i.e. up to **O(A²) in the worst case** if everyone is in one cluster (they
aren't, but with 4 factions × 32 civilians = 128 agents and ~8 clusters, the
pairs are 28 and the cross-products are bounded but still thousands of
distance checks per diplomacy tick). This is also the path that runs
`decay_faction_relations` (`engine.rs:4086-4116`), which materialises a full
`matrix.snapshot()` every 500 ticks (cheap when the matrix is sparse, but it
always clones).

**Proposed fix.** Cache the dominant-faction table on
`EmergenceState::settlement_dominant_factions` (it only changes when
`cluster_member_counts` changes — i.e. when the clustering phase recomputed),
and replace the per-pair `iter().any()` cross-product with a **grid bucket**
index keyed on `(floor(x/grid), floor(z/grid))`. Cluster agents bucket by
`(x >> GRID_BITS, z >> GRID_BITS)`; contact between cluster A and cluster B is
then a sweep over the union of their non-empty buckets, replacing
`O(|A| · |B|)` with `O(|A|_in_shared_buckets + |B|_in_shared_buckets)`.

The decay loop is harder to fix without a `DiplomacyMatrix::for_each_pair_mut`
API; the cheap interim fix is to early-out when `matrix.snapshot()` is empty
(it usually is until a few hundred ticks in), and to do the decay on
`tick % 500 == 0` only (already true) and only when `!snapshot.is_empty()`.

**Expected win.** At C = 8 clusters of 16 agents each, the contact pair
phase drops from 8²/2 × 16 × 16 = 8 192 distance checks to ~8²/2 × (one
shared bucket worth per pair) ≈ 100s. The dominant-factions table cache
saves one full A-scan per diplomacy tick (every 500 ticks, but it shows up
in flame graphs next to `cluster_by_colocation`).

**Risk.** Medium. Grid bucketing changes the contact definition from
"any cross-cluster pair within radius" to "any cross-cluster pair within
radius inside a shared grid cell", which is equivalent for any radius ≤ grid
cell size. Choose `GRID_BITS` from `SETTLEMENT_CONTACT_RADIUS_FP` so the cell
size ≥ radius; the per-bucket `O(1)` cell hash keeps determinism
(`BTreeMap` keys are sorted, agents sort by id before bucketing — same
recipe as the existing `cluster_by_colocation`).

**Behavior-preserving.** Yes **iff** the grid cell size ≥ the contact
radius, which is a single constant choice. The dominant-factions cache is
strictly equivalent (same data, computed at the same point in the tick).

---

### #5 — `phase_research`, `phase_unrest`, `phase_cohesion`, `phase_social_mood` each re-scan the same archetype collections for scalar outputs (`engine.rs:2000-2011, 2051-2078, 2119-2131, 2138-2143`)

**Current cost.** Four separate functions, each of which makes a
`world.query::<...>` full walk to compute a **single scalar**:
- `phase_research` → `sentience_research_bonus(&self.world)` scans every
  `&Dna` to count sentient vs total (`engine.rs:3443-3460`);
- `phase_unrest` → `agent_misery_unrest(&self.world)` scans every `&Psyche`
  to compute the mean mood valence (`engine.rs:3373-3384`);
- `phase_cohesion` → `micro_cohesion_delta(&self.world)` scans every `&Psyche`
  again to compute the mean + variance of `psyche.beliefs[0]`
  (`engine.rs:3388-3415`), and `micro_social_trust_permille` scans every
  `&SocialGraph` *and every tie within it* to compute a trust mean
  (`engine.rs:3419-3439`);
- `phase_social_mood` → `world.query_mut::<&mut Psyche>()` walks every psyche
  *again* to clamp its valence (`engine.rs:2140-2142`).

That is **4–5 full walks over largely overlapping entity sets** to produce
scalars that change slowly. The Psyche walks in particular are over the same
entity set (every agent with a Psyche component), three times in a row.

**Proposed fix.** Compute the per-tick micro→macro scalar aggregates
**once per tick** in `phase_emergence` (where the Psyche components are
already iterated for `update_mood` / `update_beliefs` at
`emergence.rs:406-440`), and stash the results on `EmergenceState` as
`last_micro_aggregates: MicroAggregates { sentient, total, mean_misery,
psyche_n, mood_sum, mood_sq_sum, trust_sum, trust_n }`. The four
`phase_*` consumers then become a `&self.emergence.last_micro_aggregates`
field read — `O(1)` per phase.

For `phase_social_mood` specifically, the downward causation (clamp valence
from cohesion) is a write to every `&mut Psyche` — that one **must** stay a
walk, but it can be folded into the `emergence_psyche` loop (which already
holds the same `&mut Psyche` borrow per entity) to remove the third
duplicate scan entirely. The current code touches the entity twice
(`update_mood`/`update_beliefs` in `phase_emergence`, then `phase_social_mood`
clamps the same field) — fold the clamp into `update_mood`.

**Expected win.** Drops 3–4 full A-scans per tick to 1. At A = 5 000 with
Psyche-bearing agents, that's ~15 000–20 000 archetype iterations saved per
tick. The `SocialGraph` walks inside `micro_social_trust_permille` are
O(A · L) with L = average tie count — this is one of the few sub-quadratic
loops in the engine that compounds at large A (L grows with social
activity), so collapsing the walk is the single biggest improvement at
A > 2 000.

**Risk.** Medium. Folding the per-entity Psyche update into
`emergence_psyche` is a small refactor and the call order must stay
deterministic: `phase_emergence` runs before `phase_research` /
`phase_unrest` / `phase_cohesion` / `phase_social_mood` in the existing
`PHASE_ORDER` (`engine.rs:56-72` → `engine.rs:1652-1680`), so the cached
aggregates are read after the writes that produced them — exactly the
order required for the current scalar values to be reproduced.

**Behavior-preserving.** Yes for all scalar consumers (they read the
aggregates, do the same arithmetic, write the same `state.unrest` /
`state.cohesion` / `state.research_progress` numbers). The folded
`phase_social_mood` clamp is observably equivalent: it adds the same
`uplift` to `psyche.mood.valence` and clamps the same way. The only
externally observable change is the **tick** at which a Psyche write
occurs (in `phase_emergence` instead of `phase_social_mood`); nothing
downstream of `phase_emergence` reads `Psyche.mood.valence` per tick
outside of the social-feed (`emergence_psyche` itself, which already
saw the just-written value because it wrote it in the same call).

</response>

## Honoured mentions (not in the top 5, but worth noting)

- **`agent_misery_unrest` (`engine.rs:3373-3384`) and
  `micro_social_trust_permille` (`engine.rs:3419-3439`)** both walk `&Psyche`
  / `&SocialGraph` in the same call frame as the
  `phase_research`/`phase_unrest`/`phase_cohesion` chain — covered by #5.
- **`rebuild_agent_id_index` (`engine.rs:2345-2355`)** is the only
  `O(A)` *unconditional* per-tick rebuild; covered by #3.
- **`tick_trade_routes` (`engine.rs:3066-3154`)** is `O(R)` with R capped at
  64, so in practice it's small, but the per-route `faction_resources.get`
  × 4 hash lookups are a cache miss hotspot. Switching the iteration order
  to read both endpoints into local `Fixed` values before the transfer is a
  one-line win, but R is bounded, so the absolute savings are bounded.
- **`phase_emergence_culture` (`emergence.rs:183-239`)** builds the
  `ContactEdge` list as `O(C²)` every tick, even though the cluster set
  changes at most once per `phase_life` recompute. A `C < 2` early-out is
  already there; for the common case (3 factions, ~3–8 clusters) this is
  already a small constant, but the same dirty-flag-from-`phase_life` idea
  from #3 would let it no-op in the steady state.
- **`snapshot()` (`engine.rs:3186-3211`)** does 3 full count scans per call.
  It is not in the hot path of `tick()` (the server calls it on demand), but
  it is a high-frequency consumer in the JSON-RPC catalog drift path —
  converting it to a `CachedCounts` field on `Simulation` updated at the end
  of `tick()` would be a 3 × O(entities) → 3 × O(1) win at the call site.

## Summary table

| # | Phase                          | Hot file:line                       | Current cost | Expected win          | Risk | Behaviour-preserving |
|---|--------------------------------|-------------------------------------|--------------|-----------------------|------|----------------------|
| 1 | `phase_military` inner loops   | `engine.rs:2775-2787, 2812-2820`    | `O(M²)`      | 10–100×               | Low  | Yes                  |
| 2 | `phase_diffusion` × 4 scans    | `engine.rs:790-834, 836-880`        | `4·O(A)`     | 4× → 1×               | Low  | Yes (1-tick lag on `last_cohort_stats.currently_at_target`) |
| 3 | `phase_life` HashMap + `rebuild_agent_id_index` | `engine.rs:2553-2558, 2345-2355` | `2·O(A)`     | `2·O(A) → O(Δagents)` | Low  | Yes                  |
| 4 | `phase_diplomacy` contact scan | `engine.rs:3978-4022`               | `O(C²·n²)`   | 10–100× at C > 4      | Med  | Yes (grid size ≥ radius) |
| 5 | micro→macro Psyche/SocialGraph re-scans | `engine.rs:2000, 2051, 2119, 2138, 3373-3439` | `4·O(A)` | `4× → 1×` (1× folded) | Med  | Yes                  |

The top 5 together remove **~12 of the 14 full agent-bearing ECS walks per
tick**, and the one quadratic scan (`phase_military`'s O(M²) inner loop).
For a mid-size sim (A = 5 000, M = 100, C = 8) the estimated steady-state
savings are a 3–5× drop in `tick()` wall time before any further work on
the 3D voxel path, the mod-host stubs, or the LOD diffusion propagators.

---

**Audit method.** Read-only static analysis against
`crates/engine/src/engine.rs` (7 022 lines), `crates/engine/src/emergence.rs`
(1 010 lines), `crates/engine/src/emergence_metrics.rs` (1 047 lines),
`crates/engine/src/disasters.rs` (533 lines), `crates/engine/src/lod.rs`
(153 lines), `crates/agents/src/cluster.rs` (310 lines),
`crates/agents/src/daily_path.rs` (426 lines), and
`crates/agents/src/lib.rs` (1 566 lines). All phase functions listed in
`engine.rs:56-72` `PHASE_ORDER` were traced; all loops in those functions
were classified as O(A), O(M), O(R), O(F), O(C), O(W), or O(K) as
appropriate. No code was edited, no commits were made, and no `cargo`
commands were run.

— Forge, 2026-06-16
