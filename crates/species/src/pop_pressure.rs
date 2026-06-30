//! FR-CIV-POP-PRESSURE — population pressure / carrying-capacity overshoot.
//!
//! When a population overshoots the environment's carrying capacity, the
//! surplus raises additional per-tick mortality until the population
//! rebalances back to (or below) capacity. This is a pure, deterministic
//! helper that callers (simulation tick, agent layer) compose with their
//! own baseline mortality / birth rates.
//!
//! The model is intentionally simple and substrate-agnostic:
//!
//! - Below or at capacity, `pressure_mortality` returns `0.0` (no
//!   additional deaths from crowding).
//! - Above capacity, the overshoot fraction `overshoot = (population -
//!   capacity) / capacity` is clamped to `[0.0, 1.0]` and used as the
//!   extra mortality rate. At 2× capacity (overshoot = 1.0) every
//!   remaining individual is killed off in a single tick (a worst-case
//!   starvation floor); callers are expected to cap further.
//!
//! The function is total over its inputs and side-effect-free.

/// Per-tick population pressure mortality rate, in `[0.0, 1.0]`.
///
/// Returns `0.0` when `population <= capacity`. When the population
/// overshoots, the additional mortality equals the overshoot fraction
/// `(population - capacity) / capacity`, clamped at `1.0`.
///
/// `population` and `capacity` are `u64`; a `capacity` of `0` is treated
/// as "no environment" and yields `1.0` mortality (population collapses
/// immediately) so callers don't divide by zero.
#[must_use]
pub fn pressure_mortality(population: u64, capacity: u64) -> f32 {
    if population <= capacity {
        return 0.0;
    }
    if capacity == 0 {
        // No environment can support life; the entire population perishes.
        return 1.0;
    }
    let overshoot = (population - capacity) as f64;
    let denom = capacity as f64;
    let rate = (overshoot / denom).clamp(0.0, 1.0);
    // Safe: rate is in [0.0, 1.0] and fits in f32.
    rate as f32
}

/// Apply pressure mortality on top of a baseline mortality rate.
///
/// `baseline_mortality` is the normal per-tick death rate (disease,
/// age, predation, …). Pressure mortality is added on top, then the
/// combined rate is clamped to `[0.0, 1.0]`. Returns the total
/// per-individual death probability for this tick.
///
/// Ordering note: pressure mortality is *additive* on top of the
/// baseline, not multiplicative, so that even when baseline mortality
/// is high the carrying-capacity overshoot still bites. This matches
/// the FR's intent — pressure is an *additional* force on top of
/// existing mortality sources.
#[must_use]
pub fn apply_pressure_loss(population: u64, capacity: u64, baseline_mortality: f32) -> f32 {
    let baseline = baseline_mortality.clamp(0.0, 1.0);
    let pressure = pressure_mortality(population, capacity);
    (baseline + pressure).clamp(0.0, 1.0)
}

/// Result of applying pressure mortality to a population for one tick.
///
/// Useful when callers need to know how many individuals died *from
/// pressure* (vs. baseline) for telemetry / emergence metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PressureLoss {
    /// Number of deaths attributable to carrying-capacity pressure.
    pub pressure_deaths: u64,
    /// Number of deaths attributable to the baseline mortality rate.
    pub baseline_deaths: u64,
    /// Population after the tick.
    pub survivors: u64,
}

/// Apply pressure loss to a population in-place (functionally) and
/// return the breakdown. `baseline_mortality` is in `[0.0, 1.0]` and
/// is computed against the *pre-tick* population so births / regrowth
/// compose cleanly in the caller.
#[must_use]
pub fn tick_pressure_loss(population: u64, capacity: u64, baseline_mortality: f32) -> PressureLoss {
    let total_rate = apply_pressure_loss(population, capacity, baseline_mortality);
    let baseline = baseline_mortality.clamp(0.0, 1.0);

    let total_deaths = ((population as f64) * (total_rate as f64)).round() as u64;
    let baseline_deaths = ((population as f64) * (baseline as f64)).round() as u64;
    let pressure_deaths = total_deaths.saturating_sub(baseline_deaths);

    PressureLoss {
        pressure_deaths,
        baseline_deaths,
        survivors: population.saturating_sub(total_deaths),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Covers FR-CIV-POP-PRESSURE — at or below carrying capacity the
    /// pressure mortality is zero (no extra deaths from crowding).
    #[test]
    fn at_or_below_capacity_has_zero_pressure_mortality() {
        assert_eq!(pressure_mortality(0, 100), 0.0);
        assert_eq!(pressure_mortality(50, 100), 0.0);
        assert_eq!(pressure_mortality(100, 100), 0.0);
    }

    /// Covers FR-CIV-POP-PRESSURE — overshoot above capacity raises
    /// mortality proportionally to the overshoot fraction, until it
    /// saturates at 1.0 for a fully-stressed population.
    #[test]
    fn overshoot_above_capacity_increases_mortality() {
        // 50% overshoot → 0.5 pressure mortality.
        let pop_50_over = pressure_mortality(150, 100);
        assert!(
            pop_50_over > pressure_mortality(100, 100),
            "overshoot must increase mortality (got {pop_50_over})",
        );
        assert!(
            (pop_50_over - 0.5).abs() < 1e-6,
            "50% overshoot should yield 0.5 mortality, got {pop_50_over}",
        );

        // More overshoot → strictly more mortality (monotone).
        let pop_100_over = pressure_mortality(200, 100);
        assert!(
            pop_100_over > pop_50_over,
            "more overshoot must yield strictly more mortality",
        );

        // Even more overshoot saturates at 1.0.
        let pop_way_over = pressure_mortality(10_000, 100);
        assert!(
            (pop_way_over - 1.0).abs() < 1e-6,
            "saturating overshoot should yield 1.0 mortality, got {pop_way_over}",
        );
    }

    /// Covers FR-CIV-POP-PRESSURE — capacity of zero collapses the
    /// population (no environment can support life).
    #[test]
    fn zero_capacity_collapses_population() {
        assert_eq!(pressure_mortality(0, 0), 0.0);
        assert_eq!(pressure_mortality(42, 0), 1.0);
    }

    /// Covers FR-CIV-POP-PRESSURE — `apply_pressure_loss` is additive
    /// on top of the baseline rate and clamps to `[0.0, 1.0]`.
    #[test]
    fn apply_pressure_loss_is_additive_and_clamped() {
        // Below capacity: total equals baseline.
        let below = apply_pressure_loss(50, 100, 0.1);
        assert!((below - 0.1).abs() < 1e-6, "below capacity no extra");

        // Above capacity: baseline + overshoot fraction.
        let above = apply_pressure_loss(150, 100, 0.1);
        assert!((above - 0.6).abs() < 1e-6, "above capacity adds pressure");

        // Saturates at 1.0 even with a high baseline.
        let saturated = apply_pressure_loss(10_000, 100, 0.5);
        assert!((saturated - 1.0).abs() < 1e-6, "must clamp to 1.0");

        // Baseline above 1.0 is also clamped.
        let over_baseline = apply_pressure_loss(50, 100, 1.5);
        assert!((over_baseline - 1.0).abs() < 1e-6, "baseline must clamp");
    }

    /// Covers FR-CIV-POP-PRESSURE — `tick_pressure_loss` breaks down
    /// deaths into baseline vs. pressure, with survivors = pop - total.
    #[test]
    fn tick_pressure_loss_reports_breakdown() {
        // 50% overshoot + 0% baseline → all deaths are pressure deaths.
        let pop = 750_u64;
        let cap = 500_u64;
        let result = tick_pressure_loss(pop, cap, 0.0);
        assert_eq!(result.baseline_deaths, 0);
        // 0.5 mortality × 750 = 375 deaths, all from pressure.
        assert_eq!(result.pressure_deaths, 375);
        assert_eq!(result.survivors, 375);

        // With baseline: combined rate, baseline deaths first.
        let result2 = tick_pressure_loss(150, 100, 0.1);
        // total_rate = 0.6, total_deaths = 90
        // baseline_deaths = 0.1 × 150 = 15
        // pressure_deaths = 90 - 15 = 75
        assert_eq!(result2.baseline_deaths, 15);
        assert_eq!(result2.pressure_deaths, 75);
        assert_eq!(result2.survivors, 60);
    }
}
