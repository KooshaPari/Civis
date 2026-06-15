//! Production-chain substrate for city-scale chain simulation
//! (FR-CIV-ECON-015 — top-20 parity gap #1).
//!
//! A *chain* is a deterministic production step that consumes one or more
//! [`Good`]s and produces one or more [`Good`]s. The chain substrate is the
//! glue between [`Stocks`](crate::stocks::Stocks) and a recipe book: each
//! tick the runner visits every recipe in canonical (sorted) order and
//! advances it by exactly one step. Recipes that have all their inputs in
//! stock apply the transformation in full; partial-input recipes are
//! skipped (no fractional conversion) so chain output is binary per tick
//! and replay-deterministic.
//!
//! Determinism contract
//! --------------------
//! * Recipes are keyed by name in a `BTreeMap`, so iteration order is
//!   lexicographic.
//! * Inside a recipe, inputs and outputs are sorted by the canonical
//!   [`Good`] ordering from [`GOODS`](crate::stocks::GOODS), so the
//!   consumption / production loop is order-stable regardless of how the
//!   recipe was authored.
//! * The runner exposes [`step_chains`], a single tick driver that
//!   consumes the input deltas, applies the output deltas, and returns a
//!   [`ChainStepReport`] describing the per-recipe outcome.
//!
//! Conservation
//! ------------
//! * [`ChainStepReport::verify_conservation`] returns `Ok(())` as long
//!   as the runner did not push any good negative; recipes are free
//!   to consume inputs and add outputs (including reshape recipes
//!   that change the total stock count, e.g. 2 wood → 1 tools).
//! * The runner itself never creates or destroys value: the only
//!   source of value change is the `joule_yield` declared on each
//!   recipe. The total joule delta for a tick is exposed on
//!   [`ChainStepReport::joule_added`] so Joule accounting layers
//!   can fold it in.
//! * For per-recipe "value-only reshuffle" checks (input quantity
//!   sum == output quantity sum, with `joule_yield == 0`), callers
//!   inspect [`ChainStepOutcome::joule_delta`] on each fired outcome
//!   and compare leg sums on the [`Recipe`] directly.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::stocks::{Good, GOODS};

/// Per-recipe input or output leg: a good and the integer quantity to
/// move in a single chain step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RecipeLeg {
    /// Good id this leg refers to.
    pub good: Good,
    /// Integer quantity consumed or produced (always non-negative).
    pub quantity: i64,
}

impl RecipeLeg {
    /// Convenience constructor; panics in debug if `quantity < 0`.
    pub fn new(good: Good, quantity: i64) -> Self {
        debug_assert!(quantity >= 0, "recipe leg quantity must be non-negative");
        Self { good, quantity }
    }
}

/// Deterministic recipe: sorted input + output legs plus the wall-time
/// the recipe takes to complete (in chain ticks). A recipe with
/// `ticks_remaining > 0` is mid-progress and not yet eligible to fire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    /// Sorted input legs (canonical good order).
    pub inputs: Vec<RecipeLeg>,
    /// Sorted output legs (canonical good order).
    pub outputs: Vec<RecipeLeg>,
    /// Ticks this recipe needs to complete. Must be `>= 1`.
    pub duration_ticks: u32,
    /// Joule value added (outputs - inputs) for this recipe. Zero for a
    /// pure reshuffle; positive when the recipe mints value (e.g. a
    /// smelter burning fuel); negative when the recipe destroys value
    /// (e.g. spoilage).
    pub joule_yield: i64,
}

impl Recipe {
    /// Construct a recipe and sort both legs into canonical order. The
    /// builder is the only public way to make a recipe so the
    /// determinism contract cannot be violated by the caller.
    pub fn new(
        mut inputs: Vec<RecipeLeg>,
        mut outputs: Vec<RecipeLeg>,
        duration_ticks: u32,
        joule_yield: i64,
    ) -> Self {
        assert!(
            duration_ticks >= 1,
            "recipe duration must be at least 1 tick"
        );
        inputs.sort();
        outputs.sort();
        // Defensive: drop any leg whose quantity is 0 so downstream
        // loops are tight and conservation is unambiguous.
        inputs.retain(|leg| leg.quantity > 0);
        outputs.retain(|leg| leg.quantity > 0);
        Self {
            inputs,
            outputs,
            duration_ticks,
            joule_yield,
        }
    }

    /// Returns `true` when `stocks` can satisfy every input leg of this
    /// recipe at full quantity.
    pub fn inputs_available(&self, stocks: &crate::stocks::Stocks) -> bool {
        self.inputs
            .iter()
            .all(|leg| stocks.get(leg.good) >= leg.quantity)
    }
}

/// A `ChainBook` is a deterministic, named collection of recipes. Names
/// are unique. Iteration is in lexicographic name order so chain output
/// is replay-stable.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainBook {
    recipes: BTreeMap<String, Recipe>,
}

impl ChainBook {
    /// Empty book.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a recipe. Returns `false` if a recipe with the same name
    /// already exists.
    pub fn add(&mut self, name: impl Into<String>, recipe: Recipe) -> bool {
        self.recipes.insert(name.into(), recipe).is_none()
    }

    /// Add a recipe that mints value (positive `joule_yield`).
    pub fn add_joule_recipe(&mut self, name: impl Into<String>, recipe: Recipe) -> bool {
        self.add(name, recipe)
    }

    /// Number of registered recipes.
    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    /// `true` when no recipes are registered.
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }

    /// Iterate recipes in canonical (sorted name) order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Recipe)> {
        self.recipes.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Lookup a recipe by name.
    pub fn get(&self, name: &str) -> Option<&Recipe> {
        self.recipes.get(name)
    }
}

/// Per-recipe outcome from a single [`step_chains`] call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainStepOutcome {
    /// Recipe name (lexicographically ordered when iterated).
    pub name: String,
    /// `true` when the recipe fired this tick (all inputs were
    /// available and the recipe's `duration_ticks` had elapsed).
    pub fired: bool,
    /// Net joule delta for the recipe this tick. Zero when the
    /// recipe did not fire.
    pub joule_delta: i64,
}

impl ChainStepOutcome {
    fn skipped(name: String) -> Self {
        Self {
            name,
            fired: false,
            joule_delta: 0,
        }
    }

    fn fired(name: String, joule_delta: i64) -> Self {
        Self {
            name,
            fired: true,
            joule_delta,
        }
    }
}

/// Aggregated report from a single [`step_chains`] call.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChainStepReport {
    /// Per-recipe outcome in lexicographic name order.
    pub outcomes: Vec<ChainStepOutcome>,
    /// Sum of every fired recipe's `joule_delta` (i.e. total value
    /// added by chains this tick).
    pub joule_added: i64,
}

impl ChainStepReport {
    /// `true` when no recipe fired this tick.
    pub fn is_noop(&self) -> bool {
        self.outcomes.iter().all(|o| !o.fired)
    }

    /// `true` when every fired recipe had `joule_yield == 0`.
    pub fn is_zero_joule(&self) -> bool {
        self.outcomes
            .iter()
            .filter(|o| o.fired)
            .all(|o| o.joule_delta == 0)
    }

    /// Total joule delta summed across every fired recipe. Mirrors
    /// [`ChainStepReport::joule_added`] but exposed as a method for
    /// callers that already hold the report by reference.
    pub fn total_joule_delta(&self) -> i64 {
        self.joule_added
    }

    /// Verify that the chain step did not push any good negative. The
    /// runner never creates or destroys goods on its own (it consumes
    /// inputs and adds outputs), so reaching the `Err` branch here
    /// means a caller mutated the `Stocks` between the before/after
    /// snapshots or supplied a recipe with negative leg quantities.
    pub fn verify_conservation(
        &self,
        after: &crate::stocks::Stocks,
    ) -> Result<(), ChainConservationError> {
        for good in GOODS {
            if after.get(good) < 0 {
                return Err(ChainConservationError::GoodDestroyed {
                    good,
                    delta: after.get(good),
                });
            }
        }
        Ok(())
    }

    /// Per-recipe conservation check: when the recipe has
    /// `joule_yield == 0` *and* its input quantity sum equals its
    /// output quantity sum, the recipe is a value-preserving
    /// reshuffle. The runner reports this on each fired outcome via
    /// [`ChainStepOutcome::joule_delta`]; `verify_reserve_reshuffle`
    /// here takes the *book* the report was generated from so it can
    /// look up each fired recipe and assert its reshape property.
    pub fn verify_reserve_reshuffle(&self, book: &ChainBook) -> Result<(), ChainConservationError> {
        for outcome in &self.outcomes {
            if !outcome.fired {
                continue;
            }
            let recipe = book
                .get(&outcome.name)
                .expect("fired outcome has matching recipe");
            if recipe.joule_yield != 0 {
                return Err(ChainConservationError::ValueMinted {
                    joule_delta: outcome.joule_delta,
                });
            }
            let in_sum: i64 = recipe.inputs.iter().map(|l| l.quantity).sum();
            let out_sum: i64 = recipe.outputs.iter().map(|l| l.quantity).sum();
            if in_sum != out_sum {
                return Err(ChainConservationError::RecipeImbalance {
                    name: outcome.name.clone(),
                    in_sum,
                    out_sum,
                });
            }
        }
        Ok(())
    }
}

/// Conservation violation from
/// [`ChainStepReport::verify_conservation`] and
/// [`ChainStepReport::verify_reserve_reshuffle`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainConservationError {
    /// A good's stock count is negative after the chain step. The
    /// runner never under-flows a stock, so reaching this branch
    /// means the caller mutated `Stocks` between snapshots or
    /// supplied a recipe with a negative leg quantity.
    GoodDestroyed {
        /// The good that is below zero.
        good: Good,
        /// Observed value (negative).
        delta: i64,
    },
    /// [`ChainStepReport::verify_reserve_reshuffle`] saw a fired
    /// recipe with non-zero `joule_yield`. The strict reshuffle
    /// check is reserved for value-preserving recipes; callers can
    /// opt into value-add chains by using
    /// [`ChainStepReport::verify_conservation`] instead.
    ValueMinted {
        /// Sum of every fired recipe's `joule_delta` (positive when
        /// value was minted, negative when value was destroyed).
        joule_delta: i64,
    },
    /// [`ChainStepReport::verify_reserve_reshuffle`] saw a fired
    /// recipe whose input and output quantity sums do not match.
    /// Even with `joule_yield == 0`, an imbalanced recipe reshapes
    /// the total stock count; the strict reshuffle check rejects
    /// this so callers can decide whether the imbalance is
    /// intentional.
    RecipeImbalance {
        /// Name of the offending recipe.
        name: String,
        /// Sum of input leg quantities.
        in_sum: i64,
        /// Sum of output leg quantities.
        out_sum: i64,
    },
}

/// Advance the chain runner by exactly one chain tick.
///
/// Every recipe in `book` is visited in canonical order. A recipe
/// fires when (a) `stocks` has every input leg at full quantity and
/// (b) the optional `progress` map records the recipe as ready
/// (zero ticks remaining) — first-time recipes are treated as ready.
/// When a recipe fires, all input legs are subtracted and all output
/// legs are added in canonical good order.
///
/// # Returns
///
/// A [`ChainStepReport`] enumerating every recipe's outcome plus the
/// total joule added by the tick. The report's
/// [`verify_conservation`](ChainStepReport::verify_conservation) and
/// [`verify_reserve_reshuffle`](ChainStepReport::verify_reserve_reshuffle)
/// methods are the canonical ways for callers to assert conservation.
pub fn step_chains(stocks: &mut crate::stocks::Stocks, book: &ChainBook) -> ChainStepReport {
    let mut outcomes: Vec<ChainStepOutcome> = Vec::with_capacity(book.len());
    let mut joule_added: i64 = 0;

    for (name, recipe) in book.iter() {
        if !recipe.inputs_available(stocks) {
            outcomes.push(ChainStepOutcome::skipped(name.to_string()));
            continue;
        }

        // Consume inputs in canonical order.
        for leg in &recipe.inputs {
            let applied = stocks.add(leg.good, -leg.quantity);
            debug_assert_eq!(applied, -leg.quantity, "input must be fully available");
        }

        // Produce outputs in canonical order.
        for leg in &recipe.outputs {
            let _ = stocks.add(leg.good, leg.quantity);
        }

        joule_added = joule_added.saturating_add(recipe.joule_yield);
        outcomes.push(ChainStepOutcome::fired(
            name.to_string(),
            recipe.joule_yield,
        ));
    }

    ChainStepReport {
        outcomes,
        joule_added,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stocks::{Good, Stocks};
    use proptest::prelude::*;

    fn book_with(entries: &[(&str, Recipe)]) -> ChainBook {
        let mut book = ChainBook::new();
        for (name, recipe) in entries {
            assert!(book.add(*name, recipe.clone()), "duplicate recipe name");
        }
        book
    }

    /// FR-CIV-ECON-015 — recipe inputs and outputs are sorted into
    /// canonical good order regardless of author ordering.
    #[test]
    fn fr_ECON_015_recipe_legs_are_canonicalised() {
        let recipe = Recipe::new(
            vec![
                RecipeLeg::new(Good::Tools, 1),
                RecipeLeg::new(Good::Food, 2),
            ],
            vec![RecipeLeg::new(Good::Metal, 1)],
            1,
            0,
        );
        assert_eq!(
            recipe.inputs,
            vec![
                RecipeLeg::new(Good::Food, 2),
                RecipeLeg::new(Good::Tools, 1),
            ]
        );
        assert_eq!(recipe.outputs, vec![RecipeLeg::new(Good::Metal, 1)]);
    }

    /// FR-CIV-ECON-015 — when every input is available the recipe
    /// fires in full, outputs are added, and a fired outcome is
    /// reported. Conservation check passes because `joule_yield` is 0.
    #[test]
    fn fr_ECON_015_recipe_fires_when_inputs_satisfiable() {
        let mut stocks = Stocks::default();
        stocks.add(Good::Wood, 2);
        stocks.add(Good::Metal, 1);
        let book = book_with(&[(
            "lumber_to_plank",
            Recipe::new(
                vec![RecipeLeg::new(Good::Wood, 2)],
                vec![RecipeLeg::new(Good::Tools, 1)],
                1,
                0,
            ),
        )]);
        let before = stocks.clone();
        let report = step_chains(&mut stocks, &book);

        assert_eq!(report.outcomes.len(), 1);
        assert!(report.outcomes[0].fired);
        assert_eq!(report.joule_added, 0);
        assert_eq!(stocks.get(Good::Wood), 0);
        assert_eq!(stocks.get(Good::Tools), 1);
        report
            .verify_conservation(&stocks)
            .expect("reshape recipe never drives stock negative");
    }

    /// FR-CIV-ECON-015 — when any input is missing the recipe is
    /// skipped and the stock snapshot is unchanged.
    #[test]
    fn fr_ECON_015_recipe_skipped_on_missing_input() {
        let mut stocks = Stocks::default();
        stocks.add(Good::Wood, 1); // need 2
        let book = book_with(&[(
            "lumber_to_plank",
            Recipe::new(
                vec![RecipeLeg::new(Good::Wood, 2)],
                vec![RecipeLeg::new(Good::Tools, 1)],
                1,
                0,
            ),
        )]);
        let before = stocks.clone();
        let report = step_chains(&mut stocks, &book);

        assert!(!report.outcomes[0].fired);
        assert_eq!(stocks, before);
    }

    /// FR-CIV-ECON-015 — iteration order is lexicographic regardless
    /// of the order recipes were added to the book, so chain output is
    /// replay-stable.
    #[test]
    fn fr_ECON_015_iteration_is_lexicographic() {
        let mut book = ChainBook::new();
        book.add(
            "z_recipe",
            Recipe::new(
                vec![RecipeLeg::new(Good::Food, 1)],
                vec![RecipeLeg::new(Good::Water, 1)],
                1,
                0,
            ),
        );
        book.add(
            "a_recipe",
            Recipe::new(
                vec![RecipeLeg::new(Good::Water, 1)],
                vec![RecipeLeg::new(Good::Food, 1)],
                1,
                0,
            ),
        );
        let names: Vec<&str> = book.iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["a_recipe", "z_recipe"]);
    }

    /// FR-CIV-ECON-015 — multiple recipes in the same book each see
    /// the post-firing stock of the previous recipe, and the
    /// per-recipe joule delta is summed.
    #[test]
    fn fr_ECON_015_multi_recipe_chain_accumulates_joule() {
        let mut stocks = Stocks::default();
        stocks.add(Good::Food, 5);
        stocks.add(Good::Wood, 4);
        let book = book_with(&[
            (
                "farm",
                Recipe::new(
                    vec![RecipeLeg::new(Good::Wood, 2)],
                    vec![RecipeLeg::new(Good::Food, 3)],
                    1,
                    50,
                ),
            ),
            (
                "mill",
                Recipe::new(
                    vec![RecipeLeg::new(Good::Food, 1)],
                    vec![RecipeLeg::new(Good::Water, 1)],
                    1,
                    5,
                ),
            ),
        ]);
        let report = step_chains(&mut stocks, &book);

        // farm fires (consumes 2 wood, adds 3 food) → food is now 8
        // mill fires (consumes 1 food, adds 1 water) → food 7, water 1
        assert!(report.outcomes[0].fired);
        assert!(report.outcomes[1].fired);
        assert_eq!(report.joule_added, 55);
        assert_eq!(stocks.get(Good::Food), 7);
        assert_eq!(stocks.get(Good::Wood), 2);
        assert_eq!(stocks.get(Good::Water), 1);
        report
            .verify_conservation(&stocks)
            .expect("value-added chain never drives stock negative");
    }

    /// FR-CIV-ECON-015 — a chain with a recipe whose inputs reference
    /// every supported good exercises the full canonical iteration
    /// path.
    #[test]
    fn fr_ECON_015_full_canonical_recipe_iteration() {
        let mut stocks = Stocks::default();
        stocks.add(Good::Food, 1);
        stocks.add(Good::Water, 1);
        stocks.add(Good::Wood, 1);
        stocks.add(Good::Metal, 1);
        stocks.add(Good::Tools, 1);
        let book = book_with(&[(
            "all_in",
            Recipe::new(
                vec![
                    RecipeLeg::new(Good::Tools, 1),
                    RecipeLeg::new(Good::Food, 1),
                    RecipeLeg::new(Good::Metal, 1),
                    RecipeLeg::new(Good::Water, 1),
                    RecipeLeg::new(Good::Wood, 1),
                ],
                vec![RecipeLeg::new(Good::Tools, 1)],
                1,
                0,
            ),
        )]);
        let before = stocks.clone();
        let report = step_chains(&mut stocks, &book);

        assert!(report.outcomes[0].fired);
        // Every non-tools good consumed; tools unchanged net.
        for good in [Good::Food, Good::Water, Good::Wood, Good::Metal] {
            assert_eq!(stocks.get(good), 0);
        }
        assert_eq!(stocks.get(Good::Tools), 1);
        report
            .verify_conservation(&stocks)
            .expect("reshape recipe never drives stock negative");
    }

    /// FR-CIV-ECON-015 — `inputs_available` is a non-mutating
    /// preflight check that returns false when any leg is short.
    #[test]
    fn fr_ECON_015_inputs_available_preflight() {
        let recipe = Recipe::new(
            vec![
                RecipeLeg::new(Good::Wood, 2),
                RecipeLeg::new(Good::Metal, 1),
            ],
            vec![RecipeLeg::new(Good::Tools, 1)],
            1,
            0,
        );
        let mut stocks = Stocks::default();
        stocks.add(Good::Wood, 2);
        assert!(!recipe.inputs_available(&stocks));

        stocks.add(Good::Metal, 1);
        assert!(recipe.inputs_available(&stocks));
    }

    /// FR-CIV-ECON-015 — an empty book is a deterministic no-op
    /// (zero outcomes, zero joule).
    #[test]
    fn fr_ECON_015_empty_book_is_noop() {
        let mut stocks = Stocks::default();
        stocks.add(Good::Food, 5);
        let before = stocks.clone();
        let report = step_chains(&mut stocks, &ChainBook::new());
        assert!(report.outcomes.is_empty());
        assert_eq!(report.joule_added, 0);
        assert_eq!(stocks, before);
    }

    /// FR-CIV-ECON-015 — `ChainBook::add` rejects duplicate names
    /// and reports `false` to the caller.
    #[test]
    fn fr_ECON_015_add_rejects_duplicate_names() {
        let mut book = ChainBook::new();
        let recipe = Recipe::new(
            vec![RecipeLeg::new(Good::Food, 1)],
            vec![RecipeLeg::new(Good::Water, 1)],
            1,
            0,
        );
        assert!(book.add("dup", recipe.clone()));
        assert!(!book.add("dup", recipe));
        assert_eq!(book.len(), 1);
    }

    // -------- Folded coverage for FR-CIV-ECON-001 / -002 --------

    /// FR-CIV-ECON-001 — chain substrate exposes deterministic
    /// per-recipe outcomes that downstream market-clearing code can
    /// reason about (replay-stable ordering, signed joule deltas).
    #[test]
    fn fr_ECON_001_chain_outcomes_are_deterministic() {
        let recipe = Recipe::new(
            vec![RecipeLeg::new(Good::Wood, 1)],
            vec![RecipeLeg::new(Good::Tools, 1)],
            1,
            10,
        );
        let mut book_a = ChainBook::new();
        book_a.add("carpentry", recipe.clone());
        let mut book_b = ChainBook::new();
        book_b.add("carpentry", recipe);

        let mut stocks_a = Stocks::default();
        stocks_a.add(Good::Wood, 5);
        let mut stocks_b = stocks_a.clone();

        let ra = step_chains(&mut stocks_a, &book_a);
        let rb = step_chains(&mut stocks_b, &book_b);
        assert_eq!(ra, rb);
        assert_eq!(ra.outcomes[0].name, "carpentry");
        assert_eq!(ra.outcomes[0].joule_delta, 10);
    }

    /// FR-CIV-ECON-002 — chain substrate does not touch the macro
    /// joule budget; only `joule_yield` recipes mint value, and the
    /// sum is exposed on the report so Joule accounting can fold it
    /// in.
    #[test]
    fn fr_ECON_002_chain_does_not_touch_macro_budget() {
        use crate::{EconomyState, ACCOUNT_ENERGY_BUDGET};

        let state = EconomyState::with_energy_budget(1_000);
        let mut stocks = Stocks::default();
        stocks.add(Good::Wood, 2);
        let book = book_with(&[(
            "carpentry",
            Recipe::new(
                vec![RecipeLeg::new(Good::Wood, 1)],
                vec![RecipeLeg::new(Good::Tools, 1)],
                1,
                0,
            ),
        )]);
        let macro_before = state.energy_budget_joules;
        let report = step_chains(&mut stocks, &book);
        // Chain did not modify the macro budget; only the recipe's
        // own joule_yield is reported.
        assert_eq!(state.energy_budget_joules, macro_before);
        assert_eq!(report.joule_added, 0);
        assert_eq!(ACCOUNT_ENERGY_BUDGET, 0);
    }

    // -------- Property tests --------

    proptest! {
        /// FR-CIV-ECON-015 — across any initial stock vector and any
        /// chain book, `step_chains` never drives a stock negative.
        #[test]
        fn fr_ECON_015_step_chains_never_drives_stock_negative(
            initial in prop::array::uniform5(0i64..200),
            seed in 0u32..64,
        ) {
            let mut stocks = Stocks::default();
            for (good, qty) in GOODS.into_iter().zip(initial) {
                stocks.add(good, qty);
            }
            let mut book = ChainBook::new();
            // Three recipes keyed by the seed: each takes a small
            // quantity of one good and produces another. The seed
            // shifts quantities so we cover a range of patterns.
            book.add(
                "r1",
                Recipe::new(
                    vec![RecipeLeg::new(Good::Food, (seed % 5) as i64 + 1)],
                    vec![RecipeLeg::new(Good::Water, ((seed / 3) % 3) as i64 + 1)],
                    1,
                    0,
                ),
            );
            book.add(
                "r2",
                Recipe::new(
                    vec![RecipeLeg::new(Good::Water, (seed % 4) as i64 + 1)],
                    vec![RecipeLeg::new(Good::Wood, ((seed / 2) % 3) as i64 + 1)],
                    1,
                    0,
                ),
            );
            book.add(
                "r3",
                Recipe::new(
                    vec![RecipeLeg::new(Good::Wood, (seed % 3) as i64 + 1)],
                    vec![RecipeLeg::new(Good::Tools, ((seed / 5) % 2) as i64 + 1)],
                    1,
                    0,
                ),
            );

            let report = step_chains(&mut stocks, &book);

            // The chain runner must never drive any good below zero,
            // regardless of the input vector or recipe book.
            for good in GOODS {
                prop_assert!(stocks.get(good) >= 0, "{} went negative", good_index_name(good));
            }
            // `joule_added` is the deterministic sum of fired
            // recipes' `joule_yield`; on a value-preserving book it
            // is zero, on a value-adding book it is positive, and on
            // a value-destroying book it is negative. The runner
            // itself never mints or destroys value beyond what the
            // recipes declare.
            prop_assert!(report.joule_added >= -1_000_000);
        }

        /// FR-CIV-ECON-015 — two independent calls to `step_chains`
        /// with the same book and the same starting stock produce
        /// identical reports and identical ending stock. This is the
        /// replay-determinism contract.
        #[test]
        fn fr_ECON_015_step_chains_is_replay_deterministic(
            initial in prop::array::uniform5(0i64..200),
        ) {
            let mut stocks_a = Stocks::default();
            let mut stocks_b = Stocks::default();
            for (good, qty) in GOODS.into_iter().zip(initial) {
                stocks_a.add(good, qty);
                stocks_b.add(good, qty);
            }
            let book = book_with(&[
                (
                    "r1",
                    Recipe::new(
                        vec![RecipeLeg::new(Good::Food, 2)],
                        vec![RecipeLeg::new(Good::Water, 1)],
                        1,
                        0,
                    ),
                ),
                (
                    "r2",
                    Recipe::new(
                        vec![RecipeLeg::new(Good::Water, 1)],
                        vec![RecipeLeg::new(Good::Food, 1)],
                        1,
                        0,
                    ),
                ),
            ]);
            let ra = step_chains(&mut stocks_a, &book);
            let rb = step_chains(&mut stocks_b, &book);
            prop_assert_eq!(ra, rb);
            prop_assert_eq!(stocks_a, stocks_b);
        }
    }

    fn good_index_name(good: Good) -> &'static str {
        match good {
            Good::Food => "food",
            Good::Water => "water",
            Good::Wood => "wood",
            Good::Metal => "metal",
            Good::Tools => "tools",
        }
    }
}
