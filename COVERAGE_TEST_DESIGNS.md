# Coverage Test Designs — engine / agents / economy

Source: `COVERAGE_GAPS.txt` (2026-06-16 static scan). Selection: eight highest-value untested `pub fn` entries across the priority **economy** and **agents** crates, favouring pure policy functions with deterministic assertions. The economy crate has exactly six gaps; the remaining two slots are filled from agents pure need↔POI routing.

---

## 1. `economy/src/chains.rs::verify_reserve_reshuffle`

Build a `ChainBook` with two recipes: `"reshape"` (balanced legs, `joule_yield = 0`, e.g. 2 Food → 2 Water) and `"mint"` (positive `joule_yield`, e.g. 1 Wood → 1 Tools with yield 10), seed `Stocks` so both fire in one `step_chains` call, then assert `report.verify_reserve_reshuffle(&book)` returns `Ok(())` when only the reshape recipe fired and returns `Err(ChainConservationError::ValueMinted { .. })` when the mint recipe fired; edge case: a zero-yield recipe whose input quantity sum ≠ output quantity sum (e.g. 2 Wood → 1 Tools) must yield `Err(ChainConservationError::RecipeImbalance { name, in_sum, out_sum })` even though `verify_conservation` on stocks still passes.

## 2. `economy/src/chains.rs::is_noop`

Construct `ChainStepReport` values directly (no `step_chains` required): an empty `outcomes` vec should be a noop; a single skipped outcome (`fired: false`) should be a noop; a mix of skipped and fired outcomes should return `false`; invariant: `is_noop()` iff `outcomes.iter().all(|o| !o.fired)`; edge case: report with `joule_added > 0` but all outcomes marked `fired: false` (malformed/manual construction) still follows the `fired` flag predicate, documenting that `is_noop` inspects outcomes only, not `joule_added`.

## 3. `economy/src/chains.rs::is_zero_joule`

Using reports built from `step_chains` or hand-crafted `ChainStepOutcome` slices, assert `is_zero_joule()` is `true` when no recipe fired (vacuous truth) and when every fired outcome has `joule_delta == 0`; assert `false` when at least one fired outcome has non-zero `joule_delta`; invariant: for reports produced by `step_chains`, `is_zero_joule()` agrees with `joule_added == 0` only when every fired recipe had `joule_yield == 0`; edge case: one fired outcome with `joule_delta = 0` and another with `joule_delta = 5` returns `false` even if skipped outcomes carry arbitrary deltas.

## 4. `economy/src/chains.rs::total_joule_delta`

After `step_chains` with a book containing recipes of yields `0`, `50`, and `-10` (only those with available inputs firing), assert `report.total_joule_delta()` equals the arithmetic sum of `joule_delta` on fired outcomes and equals `report.joule_added` field-for-field; invariant: `total_joule_delta() == joule_added` for every report returned by `step_chains`; edge case: empty book / all-skipped report yields `0` from both the method and the field, guarding against future drift if one accessor is updated without the other.

## 5. `economy/src/allocator.rs::next_order_id`

On a fresh `Allocator::new()`, assert `next_order_id()` returns `0` before any posts; post one bid and one offer (via existing `post_bid` / `post_offer` helpers) and assert the counter advances to `2` without consuming an id (peek-only semantics); invariant: ids assigned by `post_*` are exactly `0..next_id` in insertion order; edge case: rejected posts (negative quantity or price) must not bump `next_order_id`, confirming the counter tracks successful assignments only.

## 6. `economy/src/chains.rs::add_joule_recipe`

Call `add_joule_recipe` on an empty `ChainBook` with a recipe carrying `joule_yield > 0` and assert it returns `true`, `book.len() == 1`, and `book.get(name)` round-trips the recipe; duplicate name must return `false` and leave `len` unchanged; invariant: `add_joule_recipe` is behaviourally identical to `add` (value-minting recipes are not validated beyond `Recipe::new` assertions); edge case: recipe with negative `joule_yield` (value destruction) still registers successfully, since the method name documents intent rather than enforcing sign.

## 7. `agents/src/daily_path.rs::poi_kind_for_need`

For each `NeedKind` variant (`Food`, `Water`, `Rest`, `Safety`, `Social`, `Health`), assert `poi_kind_for_need` maps to the documented `PoiKind` (`FoodSource`, `WaterSource`, `Shelter`, `SafeZone`, `SocialHub`, `Clinic`); invariant: mapping is total and injective on the `NeedKind` domain; edge case: exhaustive `match` coverage test via `NeedKind::ALL` or a compile-time enum iteration so a newly added `NeedKind` without a branch fails the test at compile time or via a catch-all `panic!` arm.

## 8. `agents/src/daily_path.rs::need_for_poi_kind`

Mirror test for every `PoiKind` variant, asserting the inverse mapping back to `NeedKind`; invariant: `need_for_poi_kind(poi_kind_for_need(n)) == n` for all `n: NeedKind` and `poi_kind_for_need(need_for_poi_kind(p)) == p` for all `p: PoiKind` (bijection between the two enums); edge case: if `PoiKind` gains a variant without a corresponding need, the round-trip property fails loudly, preserving routing consistency for daily-path POI selection.
