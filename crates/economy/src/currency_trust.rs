//! FR-CIV-CURRENCY-TRUST — currency trust rises with stable exchange and
//! collapses with hyperinflation, affecting acceptance.
//!
//! Distinct from [`crate::institution`] / [`crate::tax_policy`] /
//! [`crate::trade_flow`]: those modules track ledger mechanics, production
//! skimming, and per-tick complementary trade formation. FR-CIV-CURRENCY-TRUST
//! is the **acceptance** layer that bridges the underlying trade signal and
//! the issuer's monetary discipline into a single per-currency trust score in
//! `[0, 1]`. Trade volume, prices, and supply growth are inputs; trust is
//! the output that downstream markets, taxation, and acceptance checks
//! consume.
//!
//! # Semantics
//!
//! The module owns a [`CurrencyTrust`] per currency. Each
//! [`step_currency_trust`] pass takes:
//!
//! 1. `trade_volume` — settled exchange in the currency for the tick.
//! 2. `price_level_cents` — clearing price of a unit good this tick, in cents.
//! 3. `previous_price_level_cents` — last tick's clearing price.
//! 4. `supply` — current outstanding quantity of the currency. Used to
//!    compute period-over-period money-supply growth.
//! 5. `previous_supply` — supply at the previous boundary.
//!
//! From these it derives:
//!
//! - **Stable exchange component**: trust rises when trade happens AND
//!   price is stable (small period-over-period change). Specifically, the
//!   stable-exchange contribution is proportional to log-ish volume scaled
//!   by an inverse-volatility factor (1 − |inflation|).
//! - **Hyperinflation penalty**: trust collapses when the money supply
//!   grows fast relative to trade (no real-economy backing) OR when prices
//!   inflate above an acute threshold. The penalty is quadratic in the
//!   excess so a runaway supply or a hyperinflation spike cuts trust fast.
//!
//! All math is integer-saturating with fixed-point trust in basis points
//! (0–10_000). No floats accumulate across calls.
//!
//! The headline acceptance property:
//!
//! > **Stable trade raises trust; runaway supply collapses it.**
//!
//! # Determinism
//!
//! `step_currency_trust` is pure: given identical `(state, inputs)` it
//! returns identical `(state', outcome)`. No hidden state, no RNG, no I/O.
//!
//! # Non-goals
//!
//! - No ledger writes, no institutions, no trade-route computation.
//! - No persistence beyond `serde` derive on the public struct.
//! - No Bevy rendering or any other I/O — pure logic.

use serde::{Deserialize, Serialize};

/// Fixed-point trust scale. 10_000 bp = 100 % trust.
const TRUST_BP_MAX: i64 = 10_000;

/// Below this per-tick price move (in basis points × 10 of the prior price),
/// exchange is considered price-stable.
const STABLE_INFLATION_BP10: i64 = 500; // 5 % per tick

/// At or above this per-tick inflation rate the currency is treated as
/// hyperinflating and trust collapses faster than the stable-exchange gain.
const HYPER_INFLATION_BP10: i64 = 5_000; // 50 % per tick

/// Above this per-tick money-supply growth (basis points × 10) the
/// hyperinflation penalty kicks in even when prices are flat (pure
/// supply-shock runaway).
const HYPER_SUPPLY_GROWTH_BP10: i64 = 3_000; // 30 % per tick

/// Maximum per-pass trust gain (basis points) from stable exchange.
const MAX_PASS_GAIN_BP: i64 = 200; // +2 %/tick

/// Maximum per-pass trust loss (basis points) from hyperinflation.
const MAX_PASS_LOSS_BP: i64 = 600; // −6 %/tick

/// Trade-volume scale at which the stable-exchange gain saturates.
/// `volume >= SATURATION_VOLUME` yields the full per-pass gain.
const SATURATION_VOLUME: i64 = 1_000;

/// FR-CIV-CURRENCY-TRUST — per-currency trust state.
///
/// Holds the running trust score and a few diagnostics. The struct is the
/// unit of additive integration: callers create one per currency they
/// issue, then drive it through [`step_currency_trust`] every tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyTrust {
    /// Stable currency id assigned by the caller.
    pub currency_id: u32,
    /// Trust ∈ `[0, 10_000]` bp (divide by 10_000 for the fraction).
    trust_bp: i64,
    /// Cumulative accepted-trade volume since construction.
    pub total_trade_volume: i64,
    /// Cumulative trust gain (positive i64) from stable exchange.
    pub total_trust_gained_bp: i64,
    /// Cumulative trust loss (stored as positive magnitude) from
    /// hyperinflation penalties.
    pub total_trust_lost_bp: i64,
    /// Number of passes applied.
    pub passes: u64,
    /// Number of passes classified as hyperinflationary.
    pub hyperinflation_passes: u64,
    /// Last price level (cents) seen, for downstream consumers that need
    /// the running series without re-deriving it.
    pub last_price_level_cents: i64,
    /// Last supply seen.
    pub last_supply: i64,
}

impl Default for CurrencyTrust {
    fn default() -> Self {
        // Additive default: every new currency starts at full trust. Callers
        // who want a different starting point should construct with
        // [`CurrencyTrust::with_initial_trust`].
        Self {
            currency_id: 0,
            trust_bp: TRUST_BP_MAX,
            total_trade_volume: 0,
            total_trust_gained_bp: 0,
            total_trust_lost_bp: 0,
            passes: 0,
            hyperinflation_passes: 0,
            last_price_level_cents: 0,
            last_supply: 0,
        }
    }
}

impl CurrencyTrust {
    /// Construct with a custom starting trust in `[0.0, 1.0]`. Out-of-range
    /// values are clamped.
    pub fn with_initial_trust(currency_id: u32, initial_trust: f32) -> Self {
        let mut t = Self::default();
        t.currency_id = currency_id;
        t.set_trust(initial_trust);
        t
    }

    /// Construct with explicit integer starting trust in basis points.
    /// Clamped to `[0, 10_000]`.
    pub fn with_initial_trust_bp(currency_id: u32, trust_bp: i64) -> Self {
        let mut t = Self::default();
        t.currency_id = currency_id;
        t.trust_bp = trust_bp.clamp(0, TRUST_BP_MAX);
        t
    }

    /// Current trust as a fraction in `[0, 1]`.
    pub fn trust(&self) -> f32 {
        (self.trust_bp as f32) / 10_000.0
    }

    /// Current trust in basis points (0–10_000).
    pub fn trust_bp(&self) -> i64 {
        self.trust_bp
    }

    /// Replace the trust level directly. Clamps to `[0, 1]`.
    pub fn set_trust(&mut self, trust: f32) {
        let clamped = if trust.is_finite() {
            trust.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let scaled = (clamped * 10_000.0).round();
        self.trust_bp = scaled.max(0.0).min(TRUST_BP_MAX as f32) as i64;
    }

    /// Acceptance factor — trust projected onto a multiplier in `[0, 1]`.
    /// Convenience for downstream callers that want a single scalar to
    /// scale acceptance probability or fee revenue by. Equivalent to
    /// [`Self::trust`].
    pub fn acceptance(&self) -> f32 {
        self.trust()
    }
}

/// Per-pass summary returned by [`step_currency_trust`].
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CurrencyTrustOutcome {
    /// Currency id this outcome applies to.
    pub currency_id: u32,
    /// Trust in basis points **after** this pass's adjustment.
    pub trust_bp: i64,
    /// Stable-exchange contribution to trust this pass (signed bp; positive
    /// ⇒ gain, negative ⇒ loss when the gain factor itself is negative —
    /// i.e. hyperinflating with no trade).
    pub stable_exchange_delta_bp: i64,
    /// Hyperinflation penalty applied this pass (positive magnitude;
    /// reported as a positive number for symmetry with the gain field).
    pub hyperinflation_penalty_bp: i64,
    /// Observed period-over-period price inflation in basis points × 10.
    /// Sign indicates direction (positive ⇒ prices rose).
    pub observed_inflation_bp10: i64,
    /// Observed period-over-period money-supply growth in basis points × 10.
    pub observed_supply_growth_bp10: i64,
    /// `true` when this pass was classified as hyperinflationary
    /// (price move ≥ `HYPER_INFLATION_BP10` or supply growth ≥
    /// `HYPER_SUPPLY_GROWTH_BP10`).
    pub hyperinflation: bool,
}

/// Compute period-over-period change in basis points × 10.
///
/// `previous` is the baseline; `current` is the new value. Returns the
/// signed `(current - previous) * 10_000 / previous` clamped so a zero
/// (or negative) previous value does not blow up. Negative `previous` is
/// treated as zero (we cannot compute a percentage off a non-positive
/// base).
fn period_change_bp10(current: i64, previous: i64) -> i64 {
    let prev = previous.max(0);
    if prev == 0 {
        return 0;
    }
    let delta = current.saturating_sub(prev);
    // numerator = delta * 10_000 ; integer division. Saturating cast to
    // i128 to avoid overflow on huge inputs.
    let numerator = (delta as i128).saturating_mul(10_000);
    let value = (numerator / prev as i128).min(i64::MAX as i128) as i64;
    // Direction: negative when current < previous.
    if current < prev {
        -value
    } else {
        value
    }
}

/// Volume-scale factor in `[0, 1]` for the stable-exchange contribution.
/// Saturates at [`SATURATION_VOLUME`] traded units per tick; below that it
/// grows linearly. Negative volume is clamped to zero.
fn volume_factor(volume: i64) -> i64 {
    let v = volume.max(0);
    if v >= SATURATION_VOLUME {
        return 10_000;
    }
    // Numerator: v * 10_000 ; denominator: SATURATION_VOLUME. Saturating.
    let numerator = (v as i128).saturating_mul(10_000);
    (numerator / SATURATION_VOLUME as i128).min(10_000) as i64
}

/// Apply one currency-trust pass.
///
/// Steps:
///
/// 1. Compute the period-over-period price inflation and money-supply
///    growth (both in bp10).
/// 2. **Stable exchange component** (gain):
///    `gain = MAX_PASS_GAIN_BP * volume_factor * (1 − min(inflation, 1))`.
///    i.e. traded volume raises trust, but only when prices are stable.
///    At/above 100 % inflation the gain factor collapses to zero — pure
///    volume with runaway prices does not build trust.
/// 3. **Hyperinflation penalty** (loss):
///    applied when `|inflation| ≥ HYPER_INFLATION_BP10` (price side) OR
///    `supply_growth ≥ HYPER_SUPPLY_GROWTH_BP10` (supply side). The penalty
///    is quadratic in the excess: a runaway above the threshold erodes
///    trust fast, a glancing above-the-line barely dents it. The maximum
///    penalty per pass is [`MAX_PASS_LOSS_BP`].
/// 4. Update `trust_bp`, accumulate diagnostics, bump `passes`, and return
///    a [`CurrencyTrustOutcome`].
///
/// All inputs are clamped to non-negative (negative `previous_price` or
/// `previous_supply` is treated as zero). Volume and supply are integer
/// units; price is integer cents.
pub fn step_currency_trust(
    state: &mut CurrencyTrust,
    trade_volume: i64,
    price_level_cents: i64,
    previous_price_level_cents: i64,
    supply: i64,
    previous_supply: i64,
) -> CurrencyTrustOutcome {
    // Clamp all observable inputs to non-negative for the percent math.
    let price = price_level_cents.max(0);
    let prev_price = previous_price_level_cents.max(0);
    let sup = supply.max(0);
    let prev_sup = previous_supply.max(0);

    // 1. Period-over-period moves.
    let inflation_bp10 = period_change_bp10(price, prev_price);
    let supply_growth_bp10 = period_change_bp10(sup, prev_sup);

    // 2. Stable-exchange contribution (signed delta in bp).
    //
    // |inflation| is the magnitude in bp10. Scale it down to [0, 1] by
    // dividing by 10_000 and clamping. volume_factor ∈ [0, 10_000].
    let abs_inflation_bp10 = inflation_bp10.abs();
    // inf_factor ∈ [0, 10_000] : 0 ⇒ perfectly stable, 10_000 ⇒ 100 %+ inflation.
    let inf_factor_bp = abs_inflation_bp10.min(10_000);
    // Stability = 10_000 − inf_factor_bp (clamped at 0).
    let stability_bp = (10_000 - inf_factor_bp).max(0);
    let vol_bp = volume_factor(trade_volume);

    // gain_bp = MAX_PASS_GAIN_BP * (stability_bp/10_000) * (vol_bp/10_000)
    //         = MAX_PASS_GAIN_BP * stability_bp * vol_bp / 100_000_000
    let gain_num = (MAX_PASS_GAIN_BP as i128)
        .saturating_mul(stability_bp as i128)
        .saturating_mul(vol_bp as i128);
    let gain_bp = (gain_num / 100_000_000).min(i64::MAX as i128) as i64;

    // 3. Hyperinflation penalty.
    //
    // Classify as hyperinflating when EITHER the price move crosses the
    // acute threshold OR the money supply is exploding (pure supply-shock
    // runaway, where prices may not yet have caught up).
    let price_hyper = abs_inflation_bp10 >= HYPER_INFLATION_BP10;
    let supply_hyper = supply_growth_bp10 >= HYPER_SUPPLY_GROWTH_BP10;
    let hyperinflation = price_hyper || supply_hyper;

    let penalty_bp = if hyperinflation {
        // Use the larger excess (price or supply) so a runaway on EITHER
        // side bites. Excess_bp10 = max(0, observed − threshold).
        let price_excess_bp10 =
            (abs_inflation_bp10 - HYPER_INFLATION_BP10).max(0);
        let supply_excess_bp10 =
            (supply_growth_bp10 - HYPER_SUPPLY_GROWTH_BP10).max(0);
        let excess_bp10 = price_excess_bp10.max(supply_excess_bp10);

        // Quadratic ramp in (excess / threshold). Saturating.
        // penalty = MAX_PASS_LOSS_BP * (excess/threshold)^2, capped at MAX_PASS_LOSS_BP.
        // Use integer arithmetic: penalty_num = MAX_PASS_LOSS_BP * excess^2 / threshold^2.
        let excess = excess_bp10.min(i64::MAX as i128) as i64;
        let threshold = if price_hyper {
            HYPER_INFLATION_BP10
        } else {
            HYPER_SUPPLY_GROWTH_BP10
        };
        let ratio_num = (excess as i128).saturating_mul(10_000);
        let ratio = ratio_num / threshold.max(1) as i128; // [0, ~very large]
        // ratio^2, saturating.
        let ratio_sq = ratio.saturating_mul(ratio); // i128
        // penalty_bp = MAX_PASS_LOSS_BP * ratio_sq / (10_000 * 10_000)
        //            = MAX_PASS_LOSS_BP * ratio_sq / 100_000_000
        let pen_num = (MAX_PASS_LOSS_BP as i128).saturating_mul(ratio_sq);
        let pen = pen_num / 100_000_000;
        // Cap at MAX_PASS_LOSS_BP so a single pass can never wipe more
        // than that off trust.
        pen.min(MAX_PASS_LOSS_BP as i128).min(i64::MAX as i128) as i64
    } else {
        0
    };

    // 4. Apply: trust + gain − penalty, clamped to [0, 10_000].
    let before = state.trust_bp;
    let after_gain = before.saturating_add(gain_bp);
    let after_loss = after_gain.saturating_sub(penalty_bp);
    state.trust_bp = after_loss.clamp(0, TRUST_BP_MAX);

    // Diagnostics.
    state.total_trade_volume = state.total_trade_volume.saturating_add(trade_volume.max(0));
    state.total_trust_gained_bp = state.total_trust_gained_bp.saturating_add(gain_bp);
    state.total_trust_lost_bp = state.total_trust_lost_bp.saturating_add(penalty_bp);
    state.passes = state.passes.saturating_add(1);
    if hyperinflation {
        state.hyperinflation_passes = state.hyperinflation_passes.saturating_add(1);
    }
    state.last_price_level_cents = price;
    state.last_supply = sup;

    CurrencyTrustOutcome {
        currency_id: state.currency_id,
        trust_bp: state.trust_bp,
        stable_exchange_delta_bp: gain_bp,
        hyperinflation_penalty_bp: penalty_bp,
        observed_inflation_bp10: inflation_bp10,
        observed_supply_growth_bp10: supply_growth_bp10,
        hyperinflation,
    }
}

/// Convenience: compute the acceptance factor (`trust()`) for an immutable
/// snapshot. Useful for downstream callers that want to gate trades on
/// trust without taking a mutable borrow.
pub fn acceptance(state: &CurrencyTrust) -> f32 {
    state.trust()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_passes(
        state: &mut CurrencyTrust,
        n: u64,
        volume: i64,
        price: i64,
        supply: i64,
    ) -> Vec<CurrencyTrustOutcome> {
        let mut prev_price = state.last_price_level_cents;
        let mut prev_supply = state.last_supply;
        let mut outcomes = Vec::new();
        for _ in 0..n {
            let out = step_currency_trust(state, volume, price, prev_price, supply, prev_supply);
            prev_price = price;
            prev_supply = supply;
            outcomes.push(out);
        }
        outcomes
    }

    /// FR-CIV-CURRENCY-TRUST acceptance test (canonical, part A):
    /// stable trade raises trust.
    #[test]
    fn fr_civ_currency_trust_stable_trade_raises_trust() {
        let mut c = CurrencyTrust::with_initial_trust_bp(1, 5_000); // start at 50 %

        // Many passes of stable exchange: volume at saturation, price
        // perfectly flat. Supply also flat so the supply-growth term is
        // zero.
        let outcomes = run_passes(&mut c, 50, SATURATION_VOLUME, 100, 10_000);

        assert!(
            c.trust_bp() > 5_000,
            "stable trade must raise trust above the 50 % start (got {} bp)",
            c.trust_bp()
        );
        // Each pass should classify as non-hyperinflationary and apply a
        // positive gain.
        for o in &outcomes {
            assert!(!o.hyperinflation, "flat prices ⇒ no hyperinflation");
            assert!(
                o.stable_exchange_delta_bp > 0,
                "stable exchange with full volume must yield a positive gain"
            );
            assert_eq!(o.hyperinflation_penalty_bp, 0);
        }
        // Gain-per-pass saturates at MAX_PASS_GAIN_BP when volume is full
        // and prices are perfectly stable.
        assert_eq!(outcomes[0].stable_exchange_delta_bp, MAX_PASS_GAIN_BP);
        // Trust is capped at TRUST_BP_MAX; running enough passes saturates
        // trust to 100 %.
        assert!(
            c.trust_bp() <= TRUST_BP_MAX,
            "trust must not exceed 100 %"
        );
        assert_eq!(c.hyperinflation_passes, 0);
    }

    /// FR-CIV-CURRENCY-TRUST acceptance test (canonical, part B):
    /// runaway supply collapses trust.
    #[test]
    fn fr_civ_currency_trust_runaway_supply_collapses_trust() {
        // Start at full trust so the collapse is unambiguous.
        let mut c = CurrencyTrust::with_initial_trust_bp(7, TRUST_BP_MAX);
        assert_eq!(c.trust_bp(), TRUST_BP_MAX);

        // Even with zero trade and flat prices, a single pass that more
        // than doubles the supply (≥ 100 % growth ⇒ supply_growth_bp10 ≥
        // 10_000 ≫ HYPER_SUPPLY_GROWTH_BP10 = 3_000) trips the supply-side
        // hyperinflation rule.
        let prev_price = 100;
        let prev_supply = 1_000;
        let out = step_currency_trust(&mut c, 0, prev_price, prev_price, 3_000, prev_supply);

        assert!(
            out.hyperinflation,
            "100 % money-supply growth must trip hyperinflation even at zero volume"
        );
        assert!(
            out.hyperinflation_penalty_bp > 0,
            "hyperinflation pass must apply a non-zero penalty"
        );
        assert!(
            c.trust_bp() < TRUST_BP_MAX,
            "trust must collapse from full after a runaway-supply pass"
        );

        // Run many runaway passes and assert trust collapses towards 0.
        let prev_price = 100_i64;
        let mut prev_supply = prev_supply;
        for _ in 0..200 {
            // Each pass: double the supply. Pure runaway.
            prev_supply = prev_supply.saturating_mul(2).max(1);
            let _ = step_currency_trust(&mut c, 0, prev_price, prev_price, prev_supply, 0);
        }
        assert!(
            c.trust_bp() <= 1_000,
            "200 runaway-supply passes must collapse trust to ≤ 10 % (got {} bp)",
            c.trust_bp()
        );
        // The hyperinflation pass counter should dominate.
        assert!(
            c.hyperinflation_passes >= 100,
            "every runaway pass must be classified as hyperinflationary (got {})",
            c.hyperinflation_passes
        );
        // Acceptance factor is the same scalar as trust; a near-zero
        // trust means near-zero acceptance.
        assert!(c.acceptance() < 0.10);
    }

    /// FR-CIV-CURRENCY-TRUST — the contrast case: stable trade without
    /// supply growth builds trust; identical volume WITH supply growth
    /// does not, and the supply-growth path itself costs trust.
    #[test]
    fn fr_civ_currency_trust_supply_growth_blocks_gain() {
        // Both currencies start at 50 % trust.
        let mut stable = CurrencyTrust::with_initial_trust_bp(1, 5_000);
        let mut runaway = CurrencyTrust::with_initial_trust_bp(2, 5_000);

        // 10 passes. Stable currency: full volume, flat price, flat supply.
        let _ = run_passes(&mut stable, 10, SATURATION_VOLUME, 100, 10_000);
        // Runaway currency: identical volume and price BUT supply grows
        // 50 % every tick (above HYPER_SUPPLY_GROWTH_BP10).
        let mut prev_supply = 10_000_i64;
        let prev_price = 100_i64;
        for _ in 0..10 {
            let next_supply = prev_supply + prev_supply / 2; // +50 %
            let _ = step_currency_trust(
                &mut runaway,
                SATURATION_VOLUME,
                prev_price,
                prev_price,
                next_supply,
                prev_supply,
            );
            prev_supply = next_supply;
        }

        assert!(
            stable.trust_bp() > runaway.trust_bp(),
            "stable trade (trust={}) must outpace trade with runaway supply (trust={})",
            stable.trust_bp(),
            runaway.trust_bp()
        );
        assert!(
            runaway.hyperinflation_passes > 0,
            "supply-growth above threshold must trigger hyperinflation"
        );
    }

    /// Trust is bounded: it cannot exceed 10_000 bp or fall below 0.
    #[test]
    fn trust_is_bounded_in_unit_interval() {
        let mut c = CurrencyTrust::with_initial_trust_bp(1, 9_900);
        // Lots of stable, saturated-volume passes — must cap at 10_000.
        let _ = run_passes(&mut c, 100, SATURATION_VOLUME, 100, 10_000);
        assert_eq!(c.trust_bp(), TRUST_BP_MAX);

        // Reset and drive many runaway-supply passes — must floor at 0.
        let mut d = CurrencyTrust::with_initial_trust_bp(2, 5_000);
        let prev_price = 100;
        let mut prev_supply = 1_000_i64;
        for _ in 0..500 {
            prev_supply = prev_supply.saturating_mul(2).max(1);
            let _ = step_currency_trust(&mut d, 0, prev_price, prev_price, prev_supply, 0);
        }
        assert_eq!(d.trust_bp(), 0);
    }

    /// Volume below saturation scales the gain linearly; volume zero
    /// produces no gain.
    #[test]
    fn gain_scales_with_volume_below_saturation() {
        let mut full = CurrencyTrust::with_initial_trust_bp(1, 5_000);
        let mut half = CurrencyTrust::with_initial_trust_bp(2, 5_000);
        let mut none = CurrencyTrust::with_initial_trust_bp(3, 5_000);

        // Pass 1 each: full saturation, half saturation, zero volume.
        let out_full =
            step_currency_trust(&mut full, SATURATION_VOLUME, 100, 100, 10_000, 10_000);
        let out_half =
            step_currency_trust(&mut half, SATURATION_VOLUME / 2, 100, 100, 10_000, 10_000);
        let out_none = step_currency_trust(&mut none, 0, 100, 100, 10_000, 10_000);

        assert_eq!(out_full.stable_exchange_delta_bp, MAX_PASS_GAIN_BP);
        assert_eq!(out_none.stable_exchange_delta_bp, 0);
        // Half-volume gain is half of full-volume (integer arithmetic may
        // round down by 1; allow either).
        let expected_half = out_full.stable_exchange_delta_bp / 2;
        assert!(
            (out_half.stable_exchange_delta_bp - expected_half).abs() <= 1,
            "half-volume gain ({}) must be ~half of full-volume gain ({})",
            out_half.stable_exchange_delta_bp,
            out_full.stable_exchange_delta_bp
        );
    }

    /// Inflation at-or-above the acute threshold collapses the gain to
    /// zero AND applies the hyperinflation penalty, even at full volume.
    #[test]
    fn acute_inflation_zeroes_gain_and_applies_penalty() {
        let mut c = CurrencyTrust::with_initial_trust_bp(1, 5_000);
        // Price jumps from 100 → 200 (100 % inflation ⇒ inflation_bp10 =
        // 10_000). Supply flat.
        let out = step_currency_trust(
            &mut c,
            SATURATION_VOLUME,
            200,
            100,
            10_000,
            10_000,
        );
        assert_eq!(out.stable_exchange_delta_bp, 0, "100 % inflation ⇒ gain = 0");
        assert!(out.hyperinflation, "100 % price jump is hyperinflationary");
        assert!(
            out.hyperinflation_penalty_bp > 0,
            "hyperinflation pass must apply a non-zero penalty"
        );
        // Trust must drop because the penalty outweighs the (zero) gain.
        assert!(c.trust_bp() < 5_000);
    }

    /// Mild inflation still allows a gain — the stability window is
    /// STABLE_INFLATION_BP10 (5 %). At 5 % inflation the gain is roughly
    /// half of the perfect-stability gain.
    #[test]
    fn mild_inflation_reduces_but_does_not_zero_gain() {
        let mut stable = CurrencyTrust::with_initial_trust_bp(1, 5_000);
        let mut mild = CurrencyTrust::with_initial_trust_bp(2, 5_000);

        // Price flat (perfectly stable).
        let out_stable =
            step_currency_trust(&mut stable, SATURATION_VOLUME, 100, 100, 10_000, 10_000);
        // Price rises from 100 → 105 (+5 % ⇒ inflation_bp10 = 500).
        let out_mild = step_currency_trust(
            &mut mild,
            SATURATION_VOLUME,
            105,
            100,
            10_000,
            10_000,
        );

        assert_eq!(out_stable.stable_exchange_delta_bp, MAX_PASS_GAIN_BP);
        assert!(
            out_mild.stable_exchange_delta_bp < out_stable.stable_exchange_delta_bp,
            "mild inflation must reduce the gain"
        );
        assert!(
            out_mild.stable_exchange_delta_bp > 0,
            "5 % inflation is below the hyperinflation threshold, gain should remain positive"
        );
        assert!(!out_mild.hyperinflation);
    }

    /// Determinism: identical inputs produce byte-identical outcomes.
    #[test]
    fn fr_civ_currency_trust_is_deterministic() {
        let mut a = CurrencyTrust::with_initial_trust_bp(1, 5_000);
        let mut b = CurrencyTrust::with_initial_trust_bp(1, 5_000);
        let mut prev_price = 100;
        let mut prev_supply = 10_000;
        for tick in 0..30 {
            let volume = 250 + tick * 7;
            let price = 100 + tick;
            let supply = 10_000 + tick * 50;
            let out_a = step_currency_trust(&mut a, volume, price, prev_price, supply, prev_supply);
            let out_b = step_currency_trust(&mut b, volume, price, prev_price, supply, prev_supply);
            assert_eq!(out_a, out_b, "outcomes must be identical for identical inputs");
            prev_price = price;
            prev_supply = supply;
        }
        assert_eq!(a, b);
    }

    /// set_trust clamps out-of-range inputs (and NaN).
    #[test]
    fn set_trust_clamps_to_unit_interval() {
        let mut c = CurrencyTrust::default();
        c.set_trust(-0.5);
        assert_eq!(c.trust_bp(), 0);
        c.set_trust(1.5);
        assert_eq!(c.trust_bp(), TRUST_BP_MAX);
        c.set_trust(f32::NAN);
        assert_eq!(c.trust_bp(), 0);
    }

    /// Negative inputs are treated as zero in the percent math (no panic).
    #[test]
    fn negative_inputs_are_clamped_to_zero() {
        let mut c = CurrencyTrust::with_initial_trust_bp(1, 5_000);
        // Negative price / supply / previous values must not blow up.
        let out = step_currency_trust(&mut c, -10, -5, -5, -7, -7);
        assert_eq!(out.observed_inflation_bp10, 0);
        assert_eq!(out.observed_supply_growth_bp10, 0);
        // No hyperinflation with all-zero inputs.
        assert!(!out.hyperinflation);
    }

    /// Acceptance factor mirrors trust.
    #[test]
    fn acceptance_mirrors_trust() {
        let c = CurrencyTrust::with_initial_trust_bp(1, 7_500);
        assert_eq!(c.acceptance(), c.trust());
        assert!((c.acceptance() - 0.75).abs() < 1e-6);

        let d = CurrencyTrust::with_initial_trust_bp(2, 0);
        assert_eq!(d.acceptance(), 0.0);
        assert_eq!(acceptance(&d), 0.0);
    }

    /// Passes counter increments monotonically.
    #[test]
    fn passes_counter_increments() {
        let mut c = CurrencyTrust::default();
        assert_eq!(c.passes, 0);
        let _ = step_currency_trust(&mut c, 0, 100, 100, 10_000, 10_000);
        let _ = step_currency_trust(&mut c, 0, 100, 100, 10_000, 10_000);
        let _ = step_currency_trust(&mut c, 0, 100, 100, 10_000, 10_000);
        assert_eq!(c.passes, 3);
    }
}