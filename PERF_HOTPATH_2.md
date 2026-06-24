# PERF_HOTPATH_2 — Per-tick hot-path audit (post clustering skip)

**Scope:** Static read-only analysis of `crates/engine/src/engine.rs` and its phase callees on `main`, with **PERF_OPT #1** applied: `phase_life` settlement clustering (`cluster_by_colocation`) is skipped when `life_cluster_position_fingerprint` is unchanged since the last full recompute (spawn, despawn, or movement dirty the fingerprint).

**Method:** Line-level complexity review of the `tick_with_emergence_source` phase order. No `cargo`, no source edits, no profiling harness. Costs use **N** = civilian/agent count, **U** = military units, **T̄** = mean social ties per agent, **C** = active cluster count, **B** = buildings, **L** = Bresenham LOS steps per voxel ray.

**Default scale (baseline sim):** N ≈ 128 (4 factions × 32 civilians), U small unless a scenario loads more.

---

## Resolved (no longer in top 4)

| Prior rank | Phase | What changed |
|------------|-------|--------------|
| **#1** | `phase_life` §5 | **O(N²)** `cluster_by_colocation` + `HashMap`/`BTreeMap` membership rebuild ran **every tick**. Now gated by `life_cluster_position_fingerprint`; stationary populations amortize to **O(N log N)** fingerprint + cached `cluster_member_counts` / `ClusterMember` reuse. Movement or population change still triggers full recompute. |

---

## Next 4 most expensive per-tick operations

| # | Phase / function | Dominant cost (steady-state tick) | Optimization |
|---|------------------|-----------------------------------|--------------|
| **1** | `phase_emergence` → `emergence_psyche` (+ `agent_entity`) | **O(N² · T̄)** — each agent walks social ties; every tie exposure calls `agent_entity(other_id)`, which linear-scans `query::<&Civilian>()` (**O(N)** per lookup). `emergence_social` → `apply_social_pair` repeats the same pattern (2× `agent_entity` per accepted pair). Secondary: `emergence_genetics_sentience` **clones every `Dna`** each tick (**O(N · L_dna)**); `emergence_culture` **clones all `CultureProfile` values** into a `Vec` and rebuilds **O(K²)** contact edges. | Maintain a persistent **`agent_id → Entity`** map (updated on spawn/despawn). Store `Entity` or cluster id on `Tie` at write time so psyche never rescans the world. In genetics, **skip agents already in `sentient_agents`** and borrow `&Dna` instead of `clone()`. Culture: drift in-place in `BTreeMap` or mark dirty clusters only. |
| **2** | `phase_life` (steps 1–4, 6; clustering skipped) | **O(N log N + k·N)** every tick even when clustering is skipped: (1) scan for missing `LifeNeeds`; (2) **`build_poi_registry`** full `Building` query; (3) collect all `AgentCivilian` entities; (4) per-agent loop with **`AgentCivilian::clone`**, multiple `get`/`get_mut`, utility pathing; (5) **`life_cluster_position_fingerprint`** — collect + **sort** all positions; (6) **`cluster_stocks`** — allocate fresh `BTreeMap`, **`ClusterStocks::clone`** per cluster, replace map. On movement ticks, §5 still adds **O(N²)** clustering + `id_to_entity` `HashMap` rebuild. | **Cache `PoiRegistry`** with a building-graph dirty flag. **Fuse** ECS passes (one `query::<(&AgentCivilian, …)>` drives needs, pathing, fingerprint). Drop `civ.clone()` — use `&AgentCivilian`. **In-place** `cluster_stocks` add food per `cluster_member_counts` (no full map rebuild). Fingerprint: rolling xor over sorted ids (or update incrementally on move) to avoid sort when only a few agents moved. |
| **3** | `phase_diffusion` → `propagate_cohort_wardrobe_with_lod` + `propagate_cohort_tools_with_lod` | **~7 full-world traversals per tick:** `count_civilians` ×2, pre-count at-target wardrobe ×1, `query_mut` promote ×1, post-count wardrobe ×1, same pattern for tools ×2. Each pass is **O(N)** over civilians / components. | **Single fused pass** per cohort (wardrobe, tools): one `query_mut::<(&mut Wardrobe, &LodTier)>` computes total, at-target count, and promotions together. **Cache `current_fraction`** on the sim until a promotion or spawn changes the ratio. |
| **4** | `phase_citizen_lifecycle` → `attach_citizen_to_agents` | **O(N) clone + allocate every tick:** collects `(Entity, AgentCivilian)` with **`civilian.clone()`** for all agents, then no-ops when `Citizen` already exists — pure overhead after tick 1. Same phase also runs **O(N) `query_mut`** for aging, needs, birth checks. | **`attach_citizen_to_agents` only on spawn/birth** (dirty flag from `spawn_civilian_at` / `spawn_child_near`). When attach is needed, iterate with **`&AgentCivilian`** (no clone). Lifecycle aging loop can share the fused agent iterator with `phase_life` if both stay deterministic. |

---

## Cadence spikes (not steady per-tick, but next after the table)

| Phase | When | Cost | Note |
|-------|------|------|------|
| `phase_military` | `tick % war_cadence == 0` (default 16) | **O(U² · L)** `tick_war_bridge` (pairwise units × Bresenham LOS) + **O(F · U · r² · L)** `FogOfWar::update` rebuild + nested **`query_mut::<MilitaryUnit>`** scans for each grid move / HP apply | Dominates combat-heavy scenarios; usually below emergence/life at default N≈128, U≪N. Spatial buckets + per-tick LOS cache + direct `entities[unit_index]` updates. |
| `phase_unrest` / `phase_research` | Every tick | **O(N)** each: `agent_misery_unrest` (`Psyche` scan), `sentience_research_bonus` (`Dna` scan + `cognition_score`) | Fold into a single upward-causation pass or cache on psyche/Dna dirty flags. |
| `phase_cohesion` (stacked PR) | Every tick | **+2 O(N)** scans: `micro_cohesion_delta`, `micro_social_trust_permille` over `Psyche` / `SocialGraph` | Merge with `phase_social_mood` or cache permille when social graph unchanged. |
| `sample_emergence` | Every 50 ticks | Voxel / dashboard histogram work | Amortized spike, not steady hot path. |
| `phase_compact` | Every 64 ticks | `voxel.compact()` | Amortized allocator pass. |

---

## Tick order reference (`tick_with_emergence_source`)

Phases run in this order each tick (clustering skip affects only `phase_life` §5):

`phase_production` → `phase_citizen_lifecycle` → `phase_military` → `phase_economy` → `phase_planet` → `phase_diplomacy` → `phase_tactics` → `phase_voxel` → `phase_compact` → `phase_buildings` → `phase_diffusion` → `phase_disasters` → **`phase_life`** → `phase_settlement_consumption` → **`phase_emergence`** → `phase_research` → `phase_tech` → `phase_belief` → `phase_unrest` → `phase_faction_unrest` → `phase_cohesion` → `phase_social_mood` → … → `sample_emergence` (50-tick boundary).

---

## Suggested optimization wave (PERF_OPT #2–#5)

| Wave | Target | Expected win |
|------|--------|--------------|
| **#2** | `agent_id → Entity` index + psyche tie fix | Largest at scale; removes **O(N²·T̄)** |
| **#3** | `phase_life` scan fusion + POI cache + in-place `cluster_stocks` | Cuts constant factors on every tick |
| **#4** | Fused `phase_diffusion` pass | ~6× fewer ECS traversals |
| **#5** | Dirty-gated `attach_citizen_to_agents` | Removes N clones/tick after warmup |

---

*Generated by static audit. Re-validate with `perf` / `cargo flamegraph` on a representative scenario before implementing.*
