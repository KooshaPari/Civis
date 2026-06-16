# Coverage Gaps 2 — engine / agents / economy

**Generated:** 2026-06-16  
**Method:** Static read-only scan — compare `pub fn` names in each crate's `src/` against that crate's test corpus only (`tests/` + `#[cfg(test)]` / `#[test]` blocks in `src/`). A function counts as tested when its name appears as a call (`fn(`, `::fn(`, `.fn(`), test helper (`test_fn`, `fn fn_`), or dedicated test module.  
**Scope:** `crates/engine`, `crates/agents`, `crates/economy` only (no cross-crate attribution).  
**Cargo:** Not run. Suite size ~1424 `#[test]` functions workspace-wide (static count).

---

## Crate summary

| Crate   | Untested `pub fn` | Total `pub fn` | Gap % | Notes |
|---------|-------------------|----------------|-------|-------|
| engine  | 37                | 179            | 20.7% | Dominant gap: emergence seeds, military scenario wiring, inspector accessors |
| agents  | 7                 | 56             | 12.5% | Lifecycle spawn + POI↔need routing + psyche default |
| economy | 2                 | 50             | 4.0%  | Prior six chain/allocator gaps closed by `COVERAGE_TEST_DESIGNS` tests; only `ProductionProfile` getters remain |

**Combined:** 46 untested / 285 public functions (16.1%).

---

## Top 12 highest-value still-untested public functions

Ranked by simulation impact: emergence/seed control, military scenario integration, population lifecycle, emergence observability, homing movement. Economy's former top gaps (`verify_reserve_reshuffle`, `next_order_id`, chain report helpers) are now covered in-crate.

| # | Location | One-line test idea |
|---|----------|-------------------|
| 1 | `engine/src/emergence.rs:register_seed_set` | Register a `SeedSet` with two valid seeds and assert both ids appear in `seed_library()`; re-register a set that replaces one id and assert the old seed is dropped while unrelated library entries survive. |
| 2 | `engine/src/emergence.rs:set_active_seed` | With a known seed in the library, `set_active_seed(Some(id))` updates `active_seed_id()`; an unknown id is rejected, `active_seed_id` stays unchanged, and `emergence_feed()` records a `seed_unknown` event. |
| 3 | `engine/src/emergence.rs:register_seed_file` | Load a fixture `.ron` seed file via `register_seed_file`, assert seeds merge into `seed_library()` and feed contains `seed_loaded`; a missing path emits `seed_load_failed` without panicking. |
| 4 | `engine/src/engine.rs:tick_with_emergence_source` | Fixed-seed sim: run ticks with `Some(minimal CaGrid)` vs `None` and assert `state.tick` advances identically while emergence metric sampling (50-tick boundary) differs when a grid is supplied. |
| 5 | `engine/src/engine.rs:apply_scenario_military` | Apply `ScenarioMilitary` overrides (`movement_cadence_ticks`, `engage_range_grid`, etc.) and assert `military_phase_config()` reflects each field, with `engage_range_grid` clamped to ≥ 1. |
| 6 | `agents/src/lib.rs:spawn_child_near` | Fixed `ChaCha8Rng` seed: spawn at `(0.5, 0.5)` and assert spawned entity has `age == 0`, `LodTier::Hot`, and normalized position within ±0.015 of parent, clamped to `[0.01, 0.99]`. |
| 7 | `engine/src/emergence.rs:agent_social_graph` | Spawn a civilian with a `SocialGraph` component, call `agent_social_graph(civilian.id)` and assert `Some` graph with expected tie count; unknown agent id returns `None`. |
| 8 | `engine/src/engine.rs:configure_military_fog` | Call with `vision_radius = Some(8)` and `grid_size = 12`; assert `military_phase_config().war.fog_vision_radius == Some(8)` and `fog_grid_size >= 16`; `None` radius leaves existing fog settings unchanged. |
| 9 | `engine/src/emergence.rs:civ_ai_decisions` | After tick(s) that populate civ-ai decisions under a seeded scenario, assert `civ_ai_decisions()` length matches internal state and each entry has a non-empty action/target field. |
| 10 | `engine/src/emergence.rs:sentience_events` | Drive a psyche/sentience threshold crossing in a minimal sim and assert `sentience_events()` records the crossing with the expected `agent_id` and tick. |
| 11 | `agents/src/lib.rs:child_bundle_from_parent` | Same RNG seed yields identical bundles twice; velocity is unit-length (`dx² + dy² ≈ 1`), all needs are `0.25`, wardrobe/tools era `0`, `LodTier::Hot`. |
| 12 | `agents/src/lib.rs:drift_toward_home` | With `shelter_need > 0.5`, returned `Velocity` is unit-length and points from civilian toward home; `shelter_need <= 0.5` or zero distance returns `current_velocity` unchanged. |

---

## Runners-up (same scope, not in top 12)

| Location | Why deferred |
|----------|--------------|
| `engine/src/engine.rs:faction_alignment` | Tested indirectly in `clients/bevy-ref/tests` but not in-crate |
| `engine/src/engine.rs:push_voxel_write` | Exercised via disasters/replay paths; no direct unit test |
| `agents/src/daily_path.rs:poi_kind_for_need` | Pure bijection; cheap next batch after lifecycle tests |
| `agents/src/daily_path.rs:need_for_poi_kind` | Mirror of above |
| `economy/src/stocks.rs:production` | Low-risk getter on `ProductionProfile` |
| `economy/src/stocks.rs:consumption` | Low-risk getter on `ProductionProfile` |

---

## Delta vs `COVERAGE_GAPS.txt` (2026-06-16)

- **economy:** All six previously listed gaps (`verify_reserve_reshuffle`, `add_joule_recipe`, `is_noop`, `is_zero_joule`, `total_joule_delta`, `next_order_id`) now have dedicated tests in `chains.rs` / `allocator.rs` (see `COVERAGE_TEST_DESIGNS.md`).
- **engine / agents:** Gap list unchanged at the high-value tier; emergence seed API and military scenario hooks remain the primary untested surface.
