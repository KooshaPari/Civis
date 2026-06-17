# COVERAGE_GAPS_3 — engine / agents / economy / genetics

**Generated:** 2026-06-16  
**Method:** Read-only static scan (no `cargo`). Compare `pub fn` definitions in `crates/{engine,agents,economy,genetics}/src/` against call sites / `#[test]` bodies in the same crate (`src/**` `#[cfg(test)]` modules + `tests/**`). A function counts as **tested** only when the test corpus contains a direct invocation (`fn(`, `.fn(`, `::fn(`) or a dedicated `fn test_*{fn}*` wrapper.  
**Workspace scale:** ~1453 runnable Rust tests (user baseline); static `#[test]` attribute count across the repo is higher (~3.8k) because it includes client crates, proptest macros, and FR-matrix duplicates.

## Scope summary

| Crate | `pub fn` (src) | Untested `pub fn` (direct) | Notes |
|-------|----------------|----------------------------|-------|
| engine | 179 | 13 | Many `Simulation` accessors; coupling helpers are mostly **private** |
| agents | 56 | 4 | POI need↔kind maps + `newborn_default` / `default_profile` |
| economy | 50 | 3 | `MarketState::prices`, `ProductionProfile::{production,consumption}` getters |
| genetics | 29 | 3 | `SeedLibrary::{from_seed_set,base_dna,retain}` |

## Coupling hotspots — already covered (batch-25)

These were prioritised in the request; most **do** have direct unit or integration tests today:

| Coupling | Symbol(s) | Test evidence |
|----------|-----------|---------------|
| **branching-ratio** | `Simulation::branching_ratio`, `emergence_branching_state`, `phase_emergence_events_close` | `emergence_metrics.rs` — `phase_emergence_events_close_updates_branching_state`, σ̄/regime asserts |
| **building_graph** | `Simulation::building_graph`, `phase_buildings` material debit | `engine.rs` — `phase_buildings_allocates_over_time_when_signals_are_high`, `phase_buildings_gated_by_wood_and_metal_stockpile` |
| **N1** (settlement food → market) | `cluster_stocks_food_lowers_market_price` integration | `engine.rs` — end-to-end tick test with `test_set_cluster_food_stock` |
| **N2** (culture → diplomacy) | `diplomacy_culture_threshold_bias` | `engine.rs` — `diplomacy_culture_threshold_bias_scales_with_similarity` |
| **N3** (settlement contact → diplomacy pair) | `diplomacy_pair_from_settlement_overlap` | `engine.rs` — `diplomacy_pair_from_settlement_overlap_prefers_contact` (+ `seed_n3_settlement_agent`) |
| **M1-A** (beliefs → cohesion) | `micro_cohesion_delta` | `engine.rs` — consensus vs polarized world asserts |
| **M1-C** (tie trust → trade) | `micro_social_trust_permille`, `society_trade_factor` | `engine.rs` — dedicated unit tests |

**Gap pattern:** N3 / building_graph **decomposition helpers** below are only exercised indirectly via parent functions or tick integration — not as isolated units.

---

## Top 10 highest-value gaps

Ranked by emergence-coupling risk, criticality (FC-3 building material gate), and public API surface.  
Entries marked **(private)** are not `pub fn` but are included because the brief explicitly calls out untested coupling helpers.

| # | File:fn | Visibility | Coupling | One-line test idea |
|---|---------|------------|----------|-------------------|
| 1 | `engine/src/engine.rs:building_materials_affordable` | private | building_graph / FC-3 | Assert `false` when wood/metal below `building_material_cost(n)` and `true` at exact threshold for `n ∈ {1,4}`. |
| 2 | `engine/src/engine.rs:settlement_contact_pairs` | private | N3 | Two 2-member clusters with agents at known `Position3d` offsets: in-contact at `2×` cluster radius, out-of-contact beyond; expect canonical `(min,max)` edge set. |
| 3 | `engine/src/engine.rs:settlement_dominant_factions` | private | N3 | World with mixed `Alignment::Faction` counts per cluster: plurality wins; tie-break on lower `faction_id`; sub-threshold clusters omitted. |
| 4 | `engine/src/engine.rs:diplomacy_faction_pairs_from_settlement_contact` | private | N3 | Given dominant map + contact edges, emit sorted unique faction pairs only when dominants differ; ignore same-faction contacts. |
| 5 | `engine/src/engine.rs:faction_wealth_scarcity_shadow` | private | N1 (faction unrest) | Treasury/food at comfort → `FOOD_SCARCITY_BASELINE`; drain food/treasury → shadow price rises monotonically and floors at baseline on surplus. |
| 6 | `engine/src/engine.rs:faction_alignment` | **pub** | N3 surface | Seed sim with known faction-aligned civilians; `faction_alignment(id)` returns `Alignment::Faction(id)` for populated ids and `None`/default for empty registry slots. |
| 7 | `engine/src/engine.rs:install_mod_path` | **pub** | mod ↔ sim | Install signed example `.civmod` from fixture path; assert `mod_host()` entry count, browser entry, and deterministic replay bus JSON on next tick. |
| 8 | `agents/src/daily_path.rs:poi_kind_for_need` | **pub** | daily-path / needs | Table-driven: each `NeedKind` maps to expected `PoiKind`; unknown/edge kinds panic or hit documented fallback. |
| 9 | `agents/src/daily_path.rs:need_for_poi_kind` | **pub** | daily-path / needs | Round-trip with `poi_kind_for_need` on bijective kinds; verify `score_poi` uses matching need pressure for a synthetic `Poi`. |
| 10 | `genetics/src/seeds.rs:from_seed_set` | **pub** | emergence spawn | Load `example_seed_set()`; assert id count, `validate()` ok, and `get("raw_organism")` matches embedded trait vector / divergence. |

### Runners-up (pub, same scan)

- `engine/src/engine.rs:faction_count` — expect count matches sorted `state.factions` keys after scenario load.
- `engine/src/engine.rs:research_cache` — after `phase_research` ticks, cache snapshot matches progress/tier getters.
- `engine/src/engine.rs:diplomacy_relation_threshold_bias` (private, N2 adjunct) — `relation=±1` → `±FACTION_RELATION_THRESHOLD_SPAN` bias.
- `engine/src/engine.rs:building_parcel_count` / `building_material_cost` (private) — saturated `DemandSignals` → parcel count 4 and cost `(10n, 5n)` wood/metal.
- `agents/src/lib.rs:newborn_default` — hot LOD, `is_newborn`, default needs/velocity invariants.
- `economy/src/market.rs:prices` — immutability accessor returns same `BTreeMap` ref across `step`/`apply_pressure`.

---

## Per-crate untested `pub fn` inventory

### engine (13)

`replay.rs`: `record_climate`, `record_mod_permission_violation`, `mod_loaded_bus_at_tick`  
`engine.rs`: `faction_count`, `faction_alignment`, `install_mod_path`, `unload_mod_by_id`, `reload_mod_by_id`, `research_cache` (+ others only used from production: `last_births`, `push_voxel_write`, …)  
`scenario.rs`: `policy_input`, `apply_world_state`  
`save_bundle.rs`: `read_metadata`, `is_save_dir`  
`spawn.rs`: `grid_to_norm`, `military_pin_id`, `unit_type_label`  
`lib.rs`: `create_rng`, `saturating_add`  
`io.rs`: `write_text`

### agents (4)

`daily_path.rs`: `poi_kind_for_need`, `need_for_poi_kind`  
`lib.rs`: `newborn_default`  
`psyche.rs`: `default_profile`

### economy (3)

`market.rs`: `prices`  
`stocks.rs`: `production`, `consumption`

### genetics (3)

`seeds.rs`: `from_seed_set`, `base_dna`, `retain`

---

## Method limits

- **Per-crate attribution only** — e.g. `faction_count` is exercised in `clients/bevy-ref` but not in `crates/engine` tests; counted untested here.
- **Indirect coverage ≠ direct** — `building_materials_affordable` is implied by `phase_buildings_gated_by_wood_and_metal_stockpile` but has no isolated predicate test (FC-3 re-verify target).
- **Name collision** — heuristic may mark a `pub fn` tested when only a same-named test helper exists; manual review applied for the top-10 list above.
- **No line coverage** — a listed gap may still be reached by integration ticks without a named unit test.

---

*Read-only artifact. Does not modify source or run `cargo test`.*
